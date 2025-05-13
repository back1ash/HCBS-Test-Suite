use hcbs_test_suite::tests::time::one_task::*;

fn print_usage() {
    let arg0 = std::env::args().nth(0).unwrap();
    println!("Usage: {arg0} <cgroup> <runtime ms> <period ms> [maxtime]");
    println!("Constraints: runtime <= period");
}

fn parse_args() -> Result<MyArgs, Box<dyn std::error::Error>> {
    if std::env::args().len() < 4 {
        print_usage();
        return Err(format!("Invalid arguments..."))?;
    }

    let args: Vec<_> = std::env::args().collect();

    let myargs = MyArgs {
        cgroup: args[1].clone(),
        runtime_ms: args[2].parse()?,
        period_ms: args[3].parse()?,
        max_time: args.get(4).map(|x| x.parse()).transpose()?,
    };

    Ok(myargs)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = parse_args()?;
    
    my_test(args, None)?;
    Ok(())
}