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

    /// max running time
    #[arg(short = 't', long = "max-time", value_name = "sec: u64")]
    pub max_time: Option<u64>
}

pub fn batch_runner(args: MyArgs, ctrlc_flag: Option<ExitFlag>) -> Result<(), Box<dyn std::error::Error>> {
    if is_batch_test() && args.max_time.is_none() {
        Err(format!("Batch testing requires a maximum running time"))?;
    }

    let cpus = num_cpus::get();
    let cgroup_expected_bw = cpus as f64 * args.runtime_ms as f64 / args.period_ms as f64;
    let fifo_expected_bw = cpus as f64 - cgroup_expected_bw;
    let error = 0.01f64; // 1% error

    let test_header =
        if is_batch_test() {
            "sched_fifo"
        } else {
            "sched_fifo (Ctrl+C to stop)"
        };

    batch_test_header(test_header, "regression");

    let result = main(args, ctrlc_flag)
        .and_then(|(fifo_bw, cgroup_bw)| {
            if f64::abs(cgroup_bw - cgroup_expected_bw) >= error {
                return Err(format!("Expected cgroup tasks to use {:.2} units of total runtime, but used {:.2} units", cgroup_expected_bw, cgroup_bw).into());
            }

            if f64::abs(fifo_bw - fifo_expected_bw) >= error {
                return Err(format!("Expected SCHED_FIFO tasks to use {:.2} units of total runtime, but used {:.2} units", fifo_expected_bw, fifo_bw).into());
            }

            Ok(format!("Cgroup processes got {:.2} units of total runtime, while SCHED_FIFO processes got {:.2} units of total runtime ", cgroup_bw, fifo_bw))
        });

    if is_batch_test() {
        batch_test_result(result)
    } else {
        batch_test_result_details(result)
    }
}

pub fn main(args: MyArgs, ctrlc_flag: Option<ExitFlag>) -> Result<(f64, f64), Box<dyn std::error::Error>> {
    let cpus = num_cpus::get();
    let cgroup = MyCgroup::new(&args.cgroup, args.runtime_ms * 1000, args.period_ms * 1000, false)?;

    migrate_task_to_cgroup(".", std::process::id())?;
    let fifo_processes: Vec<_> = (0..cpus).map(|_| run_yes()).try_collect()?;
    let cgroup_processes: Vec<_> = (0..cpus).map(|_| run_yes()).try_collect()?;

    set_scheduler(std::process::id(), SchedPolicy::RR(99))?;
    fifo_processes.iter()
        .try_for_each(|proc| set_scheduler(proc.id(), SchedPolicy::RR(50)))?;

    cgroup_processes.iter()
        .try_for_each(|proc| {
            migrate_task_to_cgroup(&args.cgroup, proc.id())?;
            set_scheduler(proc.id(), SchedPolicy::RR(50))
                .map_err(|err| Into::<Box<dyn std::error::Error>>::into(err))
        })?;

    wait_loop(args.max_time, ctrlc_flag)?;

    let mut fifo_total_usage = 0f64;
    for proc in fifo_processes.iter() {
        fifo_total_usage += get_process_total_cpu_usage(proc.id())?;
    }

    let mut cgroup_total_usage = 0f64;
    for proc in cgroup_processes.iter() {
        cgroup_total_usage += get_process_total_cpu_usage(proc.id())?;
    }

    fifo_processes.into_iter()
        .try_for_each(|mut proc| proc.kill())?;

    cgroup_processes.into_iter()
        .try_for_each(|mut proc| proc.kill())?;

    cgroup.destroy()?;

    Ok((fifo_total_usage, cgroup_total_usage))
}