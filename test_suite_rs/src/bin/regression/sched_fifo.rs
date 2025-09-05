use hcbs_test_suite::prelude::*;

#[derive(clap::Parser, Debug)]
pub struct MyArgs {
    /// cgroup's name
    #[arg(short = 'c', long = "cgroup", default_value = "g0", value_name = "name")]
    pub cgroup: String,

    /// cgroup's runtime
    #[arg(short = 'r', long = "runtime", value_name = "ms: u64")]
    pub runtime_ms: u64,

    /// cgroup's period
    #[arg(short = 'p', long = "period", value_name = "ms: u64")]
    pub period_ms: u64,

    /// max running time
    #[arg(short = 't', long = "max-time", value_name = "sec: u64")]
    pub max_time: Option<u64>
}

pub fn main(args: MyArgs, ctrlc_flag: Option<ExitFlag>) -> Result<f64, Box<dyn std::error::Error>> {
    let cpus = num_cpus::get();
    let cgroup = MyCgroup::new(&args.cgroup, args.runtime_ms * 1000, args.period_ms * 1000, false)?;
    let bandwidth = args.runtime_ms as f64 / args.period_ms as f64;

    migrate_task_to_cgroup(".", std::process::id())?;
    let fifo_processes: Vec<_> = (0..cpus).map(|_| run_yes()).try_collect()?;
    let cgroup_processes: Vec<_> = (0..cpus).map(|_| run_yes()).try_collect()?;

    chrt(std::process::id(), MySchedPolicy::RR(99))?;
    fifo_processes.iter()
        .try_for_each(|proc| chrt(proc.id(), MySchedPolicy::RR(50)))?;

    cgroup_processes.iter()
        .try_for_each(|proc| {
            migrate_task_to_cgroup(&args.cgroup, proc.id())?;
            chrt(proc.id(), MySchedPolicy::RR(50))
                .map_err(|err| Into::<Box<dyn std::error::Error>>::into(err))
        })?;

    if !is_batch_test() {
        println!("Press Ctrl+C to stop");
    }

    wait_loop(args.max_time, ctrlc_flag)?;

    let mut total_usage = 0f64;
    for proc in cgroup_processes.iter() {
        total_usage += get_process_total_cpu_usage(proc.id())?;
    }

    if !is_batch_test() {
        println!("Cgroup processes used an average of {total_usage} units of CPU bandwidth.");
    }

    fifo_processes.into_iter()
        .try_for_each(|mut proc| proc.kill())?;

    cgroup_processes.into_iter()
        .try_for_each(|mut proc| proc.kill())?;

    cgroup.destroy()?;

    let per_cpu_usage = total_usage / cpus as f64;
    if per_cpu_usage < bandwidth {
        Err(format!("Expected a consumption of at least {bandwidth:.2} per-CPU for cgroup's tasks, but got {per_cpu_usage:.2} per-CPU"))?;
    }

    Ok(total_usage)
}