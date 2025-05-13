#![feature(iterator_try_collect)]

use hcbs_test_suite::prelude::*;

struct MyArgs {
    cgroup: String,
    runtime_ms: u64,
    period_ms: u64,
    num_tasks: u64,
    max_time: Option<u64>,
}

fn print_usage() {
    let arg0 = std::env::args().nth(0).unwrap();
    println!("Usage: {arg0} <cgroup> <runtime ms> <period ms> <num tasks> [maxtime]");
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
        num_tasks: args[4].parse()?,
        max_time: args.get(5).map(|x| x.parse()).transpose()?,
    };

    Ok(myargs)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let myargs = parse_args()?;

    let cgroup = MyCgroup::new(&myargs.cgroup, myargs.runtime_ms * 1000, myargs.period_ms * 1000, true)?;
    migrate_task_to_cgroup(&myargs.cgroup, std::process::id())?;
    chrt(std::process::id(), MySchedPolicy::RR(99))?;

    let procs: Vec<_> = (0..myargs.num_tasks)
        .map(|_| run_yes()).try_collect()?;

    if !is_batch_test() {
        println!("Started Yes processes\nPress Ctrl+C to stop");
    }

    wait_loop(myargs.max_time, None)?;

    let total_usage = 
        procs.iter()
            .try_fold(0f32, |sum, proc| Ok::<f32, String>(sum + get_process_total_cpu_usage(proc.id())?))?;

    if !is_batch_test() {
        println!("Yes processes used an average of {total_usage} units of CPU bandwidth.");
    }

    procs.into_iter()
        .try_for_each(|mut proc| proc.kill())?;

    chrt(std::process::id(), MySchedPolicy::OTHER)?;
    migrate_task_to_cgroup(".", std::process::id())?;
    cgroup.destroy()?;

    if is_batch_test() {
        println!("Total usage: {total_usage}");
    }

    Ok(())
}