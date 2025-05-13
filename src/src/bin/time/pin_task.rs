use hcbs_test_suite::prelude::*;

struct MyArgs {
    cgroup: String,
    runtime_ms: u64,
    period_ms: u64,
    cpu_set: CpuSet,
    max_time: Option<u64>,
}

fn print_usage() {
    let arg0 = std::env::args().nth(0).unwrap();
    println!("Usage: {arg0} <cgroup> <runtime ms> <period ms> <cpuset> [maxtime]");
    println!("Constraints: runtime <= period");
}

fn parse_args() -> Result<MyArgs, Box<dyn std::error::Error>> {
    if std::env::args().len() < 5 {
        print_usage();
        return Err(format!("Invalid arguments..."))?;
    }

    let args: Vec<_> = std::env::args().collect();

    let myargs = MyArgs {
        cgroup: args[1].clone(),
        runtime_ms: args[2].parse()?,
        period_ms: args[3].parse()?,
        cpu_set: args[4].parse()?,
        max_time: args.get(5).map(|x| x.parse()).transpose()?,
    };

    Ok(myargs)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let myargs = parse_args()?;

    let cgroup = MyCgroup::new(&myargs.cgroup, myargs.runtime_ms * 1000, myargs.period_ms * 1000, true)?;
    migrate_task_to_cgroup(&myargs.cgroup, std::process::id())?;
    chrt(std::process::id(), MySchedPolicy::RR(99))?;

    let mut proc = run_yes()?;
    set_cpuset_to_pid(proc.id(), &myargs.cpu_set)?;
    if !is_batch_test() {
        println!("Started Yes process on PID {}\nPress Ctrl+C to stop", proc.id());
    }

    wait_loop(myargs.max_time, None)?;

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

    Ok(())
}