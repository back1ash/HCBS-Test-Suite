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

    /// number of processes to spawn
    #[arg(short = 'n', long = "num-tasks", default_value= "1", value_name = "#num")]
    pub num_tasks: u64,

    /// task's allowed cpus
    #[arg(long = "cpu-set", value_parser = <CpuSet as std::str::FromStr>::from_str)]
    pub cpu_set: Option<CpuSet>,

    /// max running time
    #[arg(short = 't', long = "max-time", value_name = "sec: u64")]
    pub max_time: Option<u64>,
}

pub fn batch_runner(args: MyArgs, ctrlc_flag: Option<ExitFlag>) -> Result<(), Box<dyn std::error::Error>> {
    if is_batch_test() && args.max_time.is_none() {
        Err(format!("Batch testing requires a maximum running time"))?;
    }

    let single_bw = args.runtime_ms as f64 / args.period_ms as f64;
    let num_cpus = args.cpu_set.as_ref()
        .map_or(CpuSet::all()?.num_cpus(), |cpu_set| cpu_set.num_cpus());

    let total_cgroup_bw = single_bw * num_cpus as f64;
    let max_expected_bw = f64::min(total_cgroup_bw, args.num_tasks as f64);
    let error = 0.01f64; // 1% error

    batch_test_header(&format!("time c{} n{} r{} p{} set{:?}", args.cgroup, args.num_tasks, args.runtime_ms, args.period_ms, args.cpu_set), "time");
    batch_test_result(
        main(args, ctrlc_flag)
        .and_then(|used_bw| {
            if f64::abs(used_bw - max_expected_bw) < error {
                Ok(())
            } else {
                Err(format!("Expected cgroup's task to use {:.2} % of total runtime, but used {:.2} %", max_expected_bw, used_bw).into())
            }
        })
    )?;

    Ok(())
}

pub fn main(args: MyArgs, ctrlc_flag: Option<ExitFlag>) -> Result<f64, Box<dyn std::error::Error>> {
    let cgroup = MyCgroup::new(&args.cgroup, args.runtime_ms * 1000, args.period_ms * 1000, true)?;

    migrate_task_to_cgroup(&args.cgroup, std::process::id())?;

    let procs: Vec<_> = (0..args.num_tasks)
        .map(|_| run_yes()).try_collect()?;
    
    chrt(std::process::id(), MySchedPolicy::RR(99))?;
    procs.iter()
        .try_for_each(|proc| {
            migrate_task_to_cgroup(&args.cgroup, proc.id())?;
            chrt(proc.id(), MySchedPolicy::RR(50))?;
            if args.cpu_set.is_some() {
                set_cpuset_to_pid(proc.id(), args.cpu_set.as_ref().unwrap())?;
            }

            Ok::<_, Box<dyn std::error::Error>>(())
        })?;

    if !is_batch_test() {
        println!("Started Yes processes\nPress Ctrl+C to stop");
    }

    wait_loop(args.max_time, ctrlc_flag)?;

    let total_usage = 
        procs.iter()
            .try_fold(0f64, |sum, proc| Ok::<f64, String>(sum + get_process_total_cpu_usage(proc.id())?))?;

    if !is_batch_test() {
        println!("Yes processes used an average of {total_usage:.5} units of CPU bandwidth.");
    }

    procs.into_iter()
        .try_for_each(|mut proc| proc.kill())?;

    chrt(std::process::id(), MySchedPolicy::OTHER)?;
    migrate_task_to_cgroup(".", std::process::id())?;
    cgroup.destroy()?;

    Ok(total_usage)
}