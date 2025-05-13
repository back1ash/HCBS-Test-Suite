use crate::prelude::*;

pub struct MyArgs {
    pub max_time: Option<u64>
}

pub fn my_test(args: MyArgs) -> Result<f32, Box<dyn std::error::Error>> {
    let cpus = num_cpus::get();

    let fifo_processes: Vec<_> = (0..cpus).map(|_| run_yes()).try_collect()?;
    let non_fifo_processes: Vec<_> = (0..cpus).map(|_| run_yes()).try_collect()?;

    chrt(std::process::id(), MySchedPolicy::RR(99))?;
    fifo_processes.iter()
        .try_for_each(|proc| chrt(proc.id(), MySchedPolicy::RR(50)))?;

    if !is_batch_test() {
        println!("Press Ctrl+C to stop");
    }

    wait_loop(args.max_time, None)?;

    let mut total_usage = 0f32;
    for proc in non_fifo_processes.iter() {
        total_usage += get_process_total_cpu_usage(proc.id())?;
    }

    if !is_batch_test() {
        println!("SCHED_OTHER processes used an average of {total_usage} units of CPU bandwidth.");
    }

    fifo_processes.into_iter()
        .try_for_each(|mut proc| proc.kill())?;

    non_fifo_processes.into_iter()
        .try_for_each(|mut proc| proc.kill())?;

    Ok(total_usage)
}