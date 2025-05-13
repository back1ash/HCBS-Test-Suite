use crate::prelude::*;

pub struct MyArgs {
    pub cgroup: String,
    pub runtime_ms: u64,
    pub period_ms: u64,
    pub num_tasks: u64,
    pub max_time: Option<u64>,
}

pub fn my_test(args: MyArgs, ctrlc_flag: Option<CtrlFlag>) -> Result<f32, Box<dyn std::error::Error>> {
    let cgroup = MyCgroup::new(&args.cgroup, args.runtime_ms * 1000, args.period_ms * 1000, true)?;
    migrate_task_to_cgroup(&args.cgroup, std::process::id())?;
    chrt(std::process::id(), MySchedPolicy::RR(99))?;

    let procs: Vec<_> = (0..args.num_tasks)
        .map(|_| run_yes()).try_collect()?;

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