use hcbs_test_suite::{tests::stress::{my_test, MyArgs}, utils::create_ctrlc_handler};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = MyArgs {
        cgroup: format!("g0"),
        num_tests: 60,
        max_time_per_test: 60,
        seed: 74,
    };

    my_test(args, Some(create_ctrlc_handler()?))
}