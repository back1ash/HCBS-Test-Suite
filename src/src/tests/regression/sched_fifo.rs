use crate::prelude::*;

pub struct MyArgs {
    pub cgroup: String,
    pub runtime_us: u64,
    pub period_us: u64,
    pub max_time: Option<u64>
}

pub fn my_test(args: MyArgs) -> Result<f32, Box<dyn std::error::Error>> {
    let cpus = num_cpus::get();
    let cgroup = MyCgroup::new(&args.cgroup, args.runtime_us, args.period_us, false)?;

    let fifo_processes: Vec<_> = (0..cpus).map(|_| run_yes()).try_collect()?;
    let cgroup_processes: Vec<_> = (0..cpus).map(|_| run_yes()).try_collect()?;

    chrt(std::process::id(), MySchedPolicy::RR(99))?;
    fifo_processes.iter()
        .try_for_each(|proc| chrt(proc.id(), MySchedPolicy::RR(50)))?;

    cgroup_processes.iter()
        .try_for_each(|proc| {
            migrate_task_to_cgroup(&args.cgroup, proc.id())
            .and_then(|_|
                chrt(proc.id(), MySchedPolicy::RR(50))
                .map_err(|err| Into::<Box<dyn std::error::Error>>::into(err))
            )
        })?;

    if !is_batch_test() {
        println!("Press Ctrl+C to stop");
    }

    wait_loop(args.max_time, None)?;

    let mut total_usage = 0f32;
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

    Ok(total_usage)
}