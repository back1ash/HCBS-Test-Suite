use hcbs_test_suite::tests::taskset::*;

fn print_usage() {
    let arg0 = std::env::args().nth(0).unwrap();
    println!("Usage: {arg0} <cgroup> <max cpus> <max bw> <num instances per job> <taskset dir> <output dir>");
}

fn parse_args() -> Result<MyArgs, Box<dyn std::error::Error>> {
    if std::env::args().len() < 7 {
        print_usage();
        return Err(format!("Invalid arguments..."))?;
    }

    let args: Vec<_> = std::env::args().collect();

    let myargs = MyArgs {
        cgroup: args[1].clone(),
        max_num_cpus: args[2].parse()?,
        max_allocatable_bw: args[3].parse()?,
        num_instances_per_job: args[4].parse()?,
        tasksets_dir: args[5].clone(),
        output_dir: args[6].clone(),
    };

    Ok(myargs)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = parse_args()?;
    
    run_taskset_array(args)?;
    Ok(())
}