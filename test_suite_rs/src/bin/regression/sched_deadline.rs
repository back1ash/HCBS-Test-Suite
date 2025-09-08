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

fn reduce_cgroups_runtime() -> Result<(), Box<dyn std::error::Error>> {
    use hcbs_test_suite::cgroup::*;

    let rt_period = get_system_rt_period_us()?;
    let rt_runtime = rt_period * 5 / 10;
    __set_cgroup_runtime_us(".", rt_runtime)?;
    Ok(())
}

pub fn main(args: MyArgs, ctrlc_flag: Option<ExitFlag>) -> Result<f64, Box<dyn std::error::Error>> {
    let cpus = num_cpus::get();
    let cgroup = MyCgroup::new(&args.cgroup, args.runtime_ms * 1000, args.period_ms * 1000, false)?;
    reduce_cgroups_runtime()?;
    let bandwidth = args.runtime_ms as f64 / args.period_ms as f64;
    let dl_runtime_ms = args.period_ms * 40 / 100;

    migrate_task_to_cgroup(".", std::process::id())?;
    let dl_processes: Vec<_> = (0..cpus).map(|_| run_yes()).try_collect()?;
    let cgroup_processes: Vec<_> = (0..cpus).map(|_| run_yes()).try_collect()?;

    set_scheduler(std::process::id(), SchedPolicy::RR(99))?;
    dl_processes.iter()
        .try_for_each(|proc| {
            set_scheduler(proc.id(), SchedPolicy::DEADLINE {
                runtime_ms: dl_runtime_ms,
                deadline_ms: args.period_ms,
                period_ms: args.period_ms,
            })
        })?;

    cgroup_processes.iter()
        .try_for_each(|proc| {
            migrate_task_to_cgroup(&args.cgroup, proc.id())?;
            set_scheduler(proc.id(), SchedPolicy::RR(50))
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

    dl_processes.into_iter()
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