use hcbs_test_suite::prelude::*;

#[derive(clap::Parser, Debug)]
pub struct MyArgs {
    /// max running time
    #[arg(short = 't', long = "max-time", value_name = "sec: u64")]
    pub max_time: Option<u64>
}

pub fn batch_runner(args: MyArgs, ctrlc_flag: Option<ExitFlag>) -> Result<(), Box<dyn std::error::Error>> {
    if is_batch_test() && args.max_time.is_none() {
        Err(format!("Batch testing requires a maximum running time"))?;
    }

    let fair_server_bw = get_fair_server_avg_bw()?;
    let error = 0.01f64; // 1% error

    batch_test_header("fair_server", "regression");
    batch_test_result(
        main(args, ctrlc_flag)
        .and_then(|used_bw| {
            if f64::abs(used_bw - fair_server_bw) < error {
                Ok(())
            } else {
                Err(format!("Expected SCHED_OTHER tasks to use {:.2} % of total runtime, but used {:.2} %", used_bw * 100.0, fair_server_bw * 100.0).into())
            }
        })
    )?;

    Ok(())
}

pub fn main(args: MyArgs, ctrlc_flag: Option<ExitFlag>) -> Result<f64, Box<dyn std::error::Error>> {
    let cpus = num_cpus::get();

    migrate_task_to_cgroup(".", std::process::id())?;
    let fifo_processes: Vec<_> = (0..cpus).map(|_| cpu_hog()).try_collect()?;
    let non_fifo_processes: Vec<_> = (0..cpus).map(|_| cpu_hog()).try_collect()?;

    chrt(std::process::id(), MySchedPolicy::RR(99))?;
    non_fifo_processes.iter()
        .enumerate()
        .try_for_each(|(i, proc)| {
            set_cpuset_to_pid(proc.id(), &CpuSet::single(i as u32)?)
        })?;

    fifo_processes.iter()
        .enumerate()
        .try_for_each::<_, Result<_, Box<dyn std::error::Error>>>(|(i, proc)| {
            set_cpuset_to_pid(proc.id(), &CpuSet::single(i as u32)?)?;
            chrt(proc.id(), MySchedPolicy::RR(50))?;

            Ok(())
        })?;

    if !is_batch_test() {
        println!("Press Ctrl+C to stop");
    }

    wait_loop(args.max_time, ctrlc_flag)?;

    let fifo_total_usage = 
        fifo_processes.iter()
        .map(|proc| get_process_total_runtime_usage(proc.id()))
        .sum::<Result<f64, _>>()?;

    let non_fifo_total_usage = 
        non_fifo_processes.iter()
        .map(|proc| get_process_total_runtime_usage(proc.id()))
        .sum::<Result<f64, _>>()?;

    let non_fifo_ratio =
        non_fifo_total_usage / (non_fifo_total_usage + fifo_total_usage);

    if !is_batch_test() {
        println!("SCHED_OTHER processes got {:.2} % of total runtime.", non_fifo_ratio * 100f64);
    }

    fifo_processes.into_iter()
        .try_for_each(|mut proc| proc.kill())?;

    non_fifo_processes.into_iter()
        .try_for_each(|mut proc| proc.kill())?;

    Ok(non_fifo_ratio)
}