use hcbs_test_suite::prelude::*;

#[derive(clap::Parser, Debug)]
pub struct MyArgs {
    /// max running time
    #[arg(short = 't', long = "max-time", value_name = "sec: u64")]
    pub max_time: Option<u64>
}

pub fn main(args: MyArgs, ctrlc_flag: Option<CtrlFlag>) -> Result<f32, Box<dyn std::error::Error>> {
    let cpus = num_cpus::get();

    migrate_task_to_cgroup(".", std::process::id())?;
    let fifo_processes: Vec<_> = (0..cpus).map(|_| run_yes()).try_collect()?;
    let non_fifo_processes: Vec<_> = (0..cpus).map(|_| run_yes()).try_collect()?;

    chrt(std::process::id(), MySchedPolicy::RR(99))?;
    fifo_processes.iter()
        .try_for_each(|proc| chrt(proc.id(), MySchedPolicy::RR(50)))?;

    if !is_batch_test() {
        println!("Press Ctrl+C to stop");
    }

    wait_loop(args.max_time, ctrlc_flag)?;

    let mut total_usage = 0f32;
    for proc in non_fifo_processes.iter() {
        total_usage += get_process_total_cpu_usage(proc.id())?;
    }

    if !is_batch_test() {
        println!("SCHED_OTHER processes used an average of {total_usage:.2} units of CPU bandwidth.");
    }

    fifo_processes.into_iter()
        .try_for_each(|mut proc| proc.kill())?;

    non_fifo_processes.into_iter()
        .try_for_each(|mut proc| proc.kill())?;

    if total_usage < 0.05 {
        Err(format!("Expected a consumption of at least 0.05 CPUs for SCHED_OTHER tasks, but got {total_usage:.2}"))?;
    }

    Ok(total_usage)
}