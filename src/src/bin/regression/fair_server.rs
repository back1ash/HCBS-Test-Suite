#![feature(iterator_try_collect)]
use hcbs_test_suite::tests::regression::fair_server::*;

fn parse_args() -> Result<MyArgs, String> {
    use std::env;

    let mut args = MyArgs {
        max_time: None,
    };

    if env::args().len() >= 2 {
        args.max_time = Some(env::args().nth(1).unwrap()
            .parse::<u64>().map_err(|err| err.to_string())?);
    }

    Ok(args)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = parse_args()?;

    my_test(args)?;
    Ok(())
}
