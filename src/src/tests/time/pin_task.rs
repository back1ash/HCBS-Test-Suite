use crate::prelude::*;

pub struct MyArgs {
    pub cgroup: String,
    pub runtime_ms: u64,
    pub period_ms: u64,
    pub cpu_set: CpuSet,
    pub max_time: Option<u64>,
}

pub fn my_test(args: MyArgs, ctrlc_flag: Option<CtrlFlag>) -> Result<f32, Box<dyn std::error::Error>> {
    let cgroup = MyCgroup::new(&args.cgroup, args.runtime_ms * 1000, args.period_ms * 1000, true)?;
    migrate_task_to_cgroup(&args.cgroup, std::process::id())?;
    chrt(std::process::id(), MySchedPolicy::RR(99))?;

    let mut proc = run_yes()?;
    set_cpuset_to_pid(proc.id(), &args.cpu_set)?;
    if !is_batch_test() {
        println!("Started Yes process on PID {}\nPress Ctrl+C to stop", proc.id());
    }

    wait_loop(args.max_time, ctrlc_flag)?;

    let total_usage = get_process_total_cpu_usage(proc.id())?;
    if !is_batch_test() {
        println!("Yes process used an average of {total_usage} units of CPU bandwidth.");
    }

    proc.kill()?;
    chrt(std::process::id(), MySchedPolicy::OTHER)?;
    migrate_task_to_cgroup(".", std::process::id())?;
    cgroup.destroy()?;

    if is_batch_test() {
        println!("Total usage: {total_usage}");
    }

    Ok(total_usage)
}