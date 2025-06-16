pub use hcbs_test_suite::prelude::*;
use rand::*;

#[derive(clap::Parser, Debug)]
pub struct MyArgs {
    /// cgroup's name
    #[arg(short = 'c', long = "cgroup", default_value = "g0", value_name = "name")]
    pub cgroup: String,

    /// number of tests to run
    #[arg(short = 'n', long = "num-tests", value_name = "u64", default_value = "60")]
    pub num_tests: u64,

    /// max running time per test
    #[arg(short = 't', long = "max-time", value_name = "sec: u64", default_value = "60")]
    pub max_time_per_test: u64,

    /// RNG's seed
    #[arg(long = "seed", value_name = "u64", default_value = "42")]
    pub seed: u64,
}

#[derive(Debug)]
enum TestType {
    FairServer,
    SchedDeadline,
    SchedFifo,
}

impl rand::distr::Distribution<TestType> for rand::distr::StandardUniform {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> TestType {
        match rng.random_range(0..=2) {
            0 => TestType::FairServer,
            1 => TestType::SchedDeadline,
            2 => TestType::SchedFifo,
            _ => panic!("unexpected"),
        }
    }
}

pub fn main(args: MyArgs, ctrlc_flag: Option<CtrlFlag>) -> Result<(), Box<dyn std::error::Error>> {
    unsafe { set_batch_test(); }

    let ctrlc_flag = match ctrlc_flag {
        Some(exit) => exit,
        None => create_ctrlc_handler()?,
    };

    let mut rand = rand::rngs::StdRng::seed_from_u64(args.seed);
    for i in 0..args.num_tests {
        if ctrlc_flag.load(std::sync::atomic::Ordering::Relaxed) {
            break;
        }

        let test_type: TestType = rand.random();
        let period_ms = rand.random_range(10..=20) * 10;
        let runtime_max_ms = period_ms * 90 / 100;
        let runtime_min_ms = 20;

        println!("Running test {i}/{0}: {test_type:?}", args.num_tests);
        match test_type {
            TestType::FairServer => {
                crate::fair_server::main(crate::fair_server::MyArgs {
                    max_time: Some(args.max_time_per_test),
                },  Some(ctrlc_flag.clone()),
                )?;
            },
            TestType::SchedFifo => {
                let runtime_ms = rand.random_range(runtime_min_ms..=runtime_max_ms);

                crate::sched_fifo::main(crate::sched_fifo::MyArgs {
                    cgroup: args.cgroup.clone(),
                    runtime_ms,
                    period_ms,
                    max_time: Some(args.max_time_per_test),
                },  Some(ctrlc_flag.clone()),
                )?;
            },
            TestType::SchedDeadline => {
                let runtime_ms = rand.random_range(runtime_min_ms..=runtime_max_ms);

                crate::sched_deadline::main(crate::sched_deadline::MyArgs {
                    cgroup: args.cgroup.clone(),
                    runtime_ms,
                    period_ms,
                    max_time: Some(args.max_time_per_test),
                },  Some(ctrlc_flag.clone()),
                )?;
            },
        }
    }

    Ok(())
}