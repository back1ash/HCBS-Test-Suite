use hcbs_test_suite::tests::stress::change_pinning::*;

fn print_usage() {
    let arg0 = std::env::args().nth(0).unwrap();
    println!("Usage: {arg0} <cgroup> <runtime ms> <period ms> <change period sec:f32> <cpu set1> <cpu set2> [maxtime]");
    println!("Constraints: runtime <= period");
}

fn parse_args() -> Result<MyArgs, Box<dyn std::error::Error>> {
    if std::env::args().len() < 7 {
        print_usage();
        return Err(format!("Invalid arguments..."))?;
    }

    let args: Vec<_> = std::env::args().collect();

    let myargs = MyArgs {
        cgroup: args[1].clone(),
        runtime_ms: args[2].parse()?,
        period_ms: args[3].parse()?,
        change_period: args[4].parse()?,
        cpu_set1: args[5].parse()?,
        cpu_set2: args[6].parse()?,
        max_time: args.get(7).map(|x| x.parse()).transpose()?,
    };

    Ok(myargs)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let my_args = parse_args()?;

    my_test(my_args)
}