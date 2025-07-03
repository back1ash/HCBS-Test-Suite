use hcbs_test_suite::prelude::*;

#[derive(clap::Parser, Debug)]
pub struct MyArgs {
    /// bandwidth
    #[arg(short = 'b', long = "bandwidth", value_name = "[0,1]: f32")]
    bw: f32,
}

pub fn main(args: MyArgs) -> Result<(), Box<dyn std::error::Error>> {
    let period = get_system_rt_period()? as f32;
    let runtime = (args.bw * period) as u64;
    set_system_rt_runtime(runtime)?;

    Ok(())
}