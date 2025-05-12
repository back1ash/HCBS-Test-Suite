use crate::prelude::*;
use std::thread;
use rand::Rng;

pub struct MyArgs {
    pub cgroup: String,
    pub runtime_min_ms: u64,
    pub runtime_max_ms: u64,
    pub period_ms: u64,
    pub max_time: Option<u64>,
}

pub fn my_test(args: MyArgs, rng: Option<&mut dyn rand::RngCore>) -> Result<(), Box<dyn std::error::Error>> {
    let mut thread_rng = rand::rng();
    let rng = rng.unwrap_or_else(|| &mut thread_rng);

    wait_loop_periodic_fn(0f32, args.max_time,
        || {
            let runtime_ms = rng.random_range(args.runtime_min_ms ..= args.runtime_max_ms);
            let cgroup = MyCgroup::new(&args.cgroup, runtime_ms * 1000, args.period_ms * 1000, true)?;
            migrate_task_to_cgroup(&args.cgroup, std::process::id())?;
            chrt(std::process::id(), MySchedPolicy::RR(99))?;

            let num_procs = rng.random_range(1..=5);
            let procs: Vec<_> = (0..num_procs)
                .map(|_| run_yes()).try_collect()?;

            thread::sleep(std::time::Duration::from_secs_f32(rng.random_range(0.5f32..=2f32)));

            procs.into_iter()
                .try_for_each(|mut proc| proc.kill())?;

            chrt(std::process::id(), MySchedPolicy::OTHER)?;
            migrate_task_to_cgroup(".", std::process::id())?;
            cgroup.destroy()?;

            Ok(())
        }
    )?;

    Ok(())
}