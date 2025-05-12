use hcbs_test_suite::prelude::*;

struct MyArgs {
    bw: f32,
}

fn print_usage() {
    let arg0 = std::env::args().nth(0).unwrap();
    println!("Usage: {arg0} <new bw>");
    println!("Constraints: 0 <= new bw <= 1");
}

fn parse_args() -> Result<MyArgs, Box<dyn std::error::Error>> {
    if std::env::args().len() < 2 {
        print_usage();
        return Err(format!("Invalid arguments..."))?;
    }

    let args: Vec<_> = std::env::args().collect();

    let myargs = MyArgs {
        bw: Ok(args[1].parse::<f32>()?)
            .and_then(|bw| if 0f32 <= bw && bw <= 1f32 { Ok(bw) } else {
                Err(format!("Bandwidth must be in range [0,1]"))
            })?,
    };

    Ok(myargs)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let myargs = parse_args()?;

    let period = get_system_rt_period()? as f32;
    let runtime = (myargs.bw * period) as u64;
    set_system_rt_runtime(runtime)?;

    Ok(())
}