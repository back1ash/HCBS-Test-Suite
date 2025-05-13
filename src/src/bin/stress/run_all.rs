use hcbs_test_suite::prelude::*;
use hcbs_test_suite::tests::stress::*;
use rand::*;

#[derive(Debug)]
enum TestType {
    CgroupMakeDestroy,
    ChangePinning,
    ChangePriority,
}

impl rand::distr::Distribution<TestType> for rand::distr::StandardUniform {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> TestType {
        match rng.random_range(0..5) {
            0 => TestType::CgroupMakeDestroy,
            1..=2 => TestType::ChangePinning,
            3..=4 => TestType::ChangePriority,
            _ => panic!("unexpected"),
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    unsafe { set_batch_test(); }

    let mut rand = rand::rngs::StdRng::seed_from_u64(42);

    let cgroup = format!("g0");
    let num_tests = 60;
    let max_time = Some(10);

    let ctrlc_flag = create_ctrlc_handler()?;

    // one hour of tests
    for i in 0..num_tests {
        if ctrlc_flag.load(std::sync::atomic::Ordering::Relaxed) {
            break;
        }

        let test_type: TestType = rand.random();
        let period_ms = rand.random_range(10..=20) * 10;

        println!("Running test {i}/{num_tests}: {test_type:?}");
        match test_type {
            TestType::CgroupMakeDestroy => {
                let runtime_min_ms = rand.random_range(5..=period_ms/20) * 10;
                let runtime_max_ms = rand.random_range(period_ms/20..period_ms/11) * 10;

                cgroup_make_destroy::my_test(cgroup_make_destroy::MyArgs {
                    cgroup: cgroup.clone(),
                    runtime_min_ms,
                    runtime_max_ms,
                    period_ms,
                    max_time,
                },  Some(&mut rand),
                    Some(ctrlc_flag.clone()),
                )?
            },
            
            TestType::ChangePinning => {
                let runtime_ms = rand.random_range(5..period_ms/11) * 10;
                let change_period = rand.random_range(0.5f32..=3f32);

                change_pinning::my_test(change_pinning::MyArgs {
                    cgroup: cgroup.clone(),
                    runtime_ms,
                    period_ms,
                    change_period,
                    cpu_set1: "0,2".parse()?,
                    cpu_set2: "1,3".parse()?,
                    max_time,
                },  Some(ctrlc_flag.clone()),
                )?
            },

            TestType::ChangePriority => {
                let runtime_ms = rand.random_range(5..period_ms/11) * 10;
                let change_period = rand.random_range(0.5f32..=3f32);

                change_priority::my_test(change_priority::MyArgs {
                    cgroup: cgroup.clone(),
                    runtime_ms,
                    period_ms,
                    change_period,
                    max_time,
                },  Some(ctrlc_flag.clone()),
                )?
            },
        }
    }

    Ok(())
}