use crate::prelude::*;

pub struct MyArgs {
    pub cgroup: String,
    pub runtime_ms: u64,
    pub period_ms: u64,
    pub change_period: f32,
    pub max_time: Option<u64>,
}

pub fn my_test(args: MyArgs, ctrlc_flag: Option<CtrlFlag>) -> Result<(), Box<dyn std::error::Error>> {
    let cgroup = MyCgroup::new(&args.cgroup, args.runtime_ms * 1000, args.period_ms * 1000, true)?;
    migrate_task_to_cgroup(&args.cgroup, std::process::id())?;
    chrt(std::process::id(), MySchedPolicy::RR(99))?;

    let mut proc = run_yes()?;
    let mut state = MySchedPolicy::RR(50);

    let update_fn = || {
        if state == MySchedPolicy::RR(50) {
            state = MySchedPolicy::OTHER;
        } else {
            state = MySchedPolicy::RR(50);
        }

        chrt(proc.id(), state)?;
        Ok(())
    };

    if !is_batch_test() {
        println!("Started Yes process\nPress Ctrl+C to stop");
    }

    wait_loop_periodic_fn(args.change_period, args.max_time, ctrlc_flag, update_fn)?;

    proc.kill()?;
    chrt(std::process::id(), MySchedPolicy::OTHER)?;
    migrate_task_to_cgroup(".", std::process::id())?;
    cgroup.destroy()?;

    Ok(())
}