use crate::prelude::*;

pub struct MyArgs {
    pub cgroup: String,
    pub runtime_ms: u64,
    pub period_ms: u64,
    pub change_period: f32,
    pub cpu_set1: CpuSet,
    pub cpu_set2: CpuSet,
    pub max_time: Option<u64>,
}

pub fn my_test(args: MyArgs) -> Result<(), Box<dyn std::error::Error>> {
    let cgroup = MyCgroup::new(&args.cgroup, args.runtime_ms * 1000, args.period_ms * 1000, true)?;
    migrate_task_to_cgroup(&args.cgroup, std::process::id())?;
    chrt(std::process::id(), MySchedPolicy::RR(99))?;

    let mut proc = run_yes()?;
    let mut state = &args.cpu_set1;
    set_cpuset_to_pid(proc.id(), state)?;

    let update_fn = || {
        if state == &args.cpu_set1 {
            state = &args.cpu_set2;
        } else {
            state = &args.cpu_set1;
        }

        set_cpuset_to_pid(proc.id(), state)?;
        Ok(())
    };

    if !is_batch_test() {
        println!("Started Yes process\nPress Ctrl+C to stop");
    }

    wait_loop_periodic_fn(args.change_period, args.max_time, update_fn)?;

    proc.kill()?;
    chrt(std::process::id(), MySchedPolicy::OTHER)?;
    migrate_task_to_cgroup(".", std::process::id())?;
    cgroup.destroy()?;

    Ok(())
}