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

    /// number of processes to spawn
    #[arg(short = 'n', long = "num-tasks", value_name = "N")]
    pub num_tasks: u64,

    /// max running time
    #[arg(short = 't', long = "max-time", value_name = "sec: u64")]
    pub max_time: Option<u64>,
}

pub fn main(args: MyArgs, ctrlc_flag: Option<ExitFlag>) -> Result<f32, Box<dyn std::error::Error>> {
    let cgroup = MyCgroup::new(&args.cgroup, args.runtime_ms * 1000, args.period_ms * 1000, true)?;

    migrate_task_to_cgroup(&args.cgroup, std::process::id())?;

    let procs: Vec<_> = (0..args.num_tasks)
        .map(|_| run_yes()).try_collect()?;
    
    chrt(std::process::id(), MySchedPolicy::RR(99))?;
    procs.iter()
        .try_for_each(|proc| {
            migrate_task_to_cgroup(&args.cgroup, proc.id())?;
            chrt(proc.id(), MySchedPolicy::RR(50))?;

            Ok::<_, Box<dyn std::error::Error>>(())
        })?;

    if !is_batch_test() {
        println!("Started Yes processes\nPress Ctrl+C to stop");
    }

    wait_loop(args.max_time, ctrlc_flag)?;

    let total_usage = 
        procs.iter()
            .try_fold(0f32, |sum, proc| Ok::<f32, String>(sum + get_process_total_cpu_usage(proc.id())?))?;

    if !is_batch_test() {
        println!("Yes processes used an average of {total_usage} units of CPU bandwidth.");
    }

    procs.into_iter()
        .try_for_each(|mut proc| proc.kill())?;

    chrt(std::process::id(), MySchedPolicy::OTHER)?;
    migrate_task_to_cgroup(".", std::process::id())?;
    cgroup.destroy()?;

    if is_batch_test() {
        println!("Total usage: {total_usage}");
    }

    Ok(total_usage)
}