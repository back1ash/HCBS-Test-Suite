#![feature(iterator_try_collect)]

use hcbs_test_suite::tests::stress::cgroup_make_destroy::*;

fn print_usage() {
    let arg0 = std::env::args().nth(0).unwrap();
    println!("Usage: {arg0} <cgroup> <runtime min ms> <runtime max ms> <period ms> [maxtime]");
    println!("Constraints: runtime max <= period; runtime min <= runtime max");
}

fn parse_args() -> Result<MyArgs, Box<dyn std::error::Error>> {
    if std::env::args().len() < 5 {
        print_usage();
        return Err(format!("Invalid arguments..."))?;
    }

    let args: Vec<_> = std::env::args().collect();

    let myargs = MyArgs {
        cgroup: args[1].clone(),
        runtime_min_ms: args[2].parse()?,
        runtime_max_ms: args[3].parse()?,
        period_ms: args[4].parse()?,
        max_time: args.get(5).map(|x| x.parse()).transpose()?,
    };

    Ok(myargs)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let my_args = parse_args()?;

    my_test(my_args, None)
}