use hcbs_test_suite::*;
use hcbs_test_suite::prelude::*;

fn cgroup_time_tests(cgroup_name: &str, runtime_us: u64, period_us: u64) -> Result<(), Box<dyn std::error::Error>> {
    use hcbs_test_suite::cgroup::{__set_cgroup_period_us, __set_cgroup_runtime_us};

    if !is_batch_test() {
        println!("Cgroup \'{cgroup_name}\' setup with {runtime_us}/{period_us} runtime/period should fail.");
    }

    create_cgroup(cgroup_name)?;

    let failure: Result<(), _> = 
        __set_cgroup_period_us(cgroup_name, period_us)
            .and_then(|_| __set_cgroup_runtime_us(cgroup_name, runtime_us));

    delete_cgroup(cgroup_name)?;

    if failure.is_ok() {
        Err(format!("Cgroup \'{cgroup_name}\' creation with {runtime_us}/{period_us} did not fail"))?
    } else {
        if !is_batch_test() { println!("Ok!"); }
        Ok(())
    }
}

fn add_task_to_runtime_zero(cgroup_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    if !is_batch_test() {
        println!("Task migration to cgroup \'{cgroup_name}\' with runtime 0 should fail.");
    }

    cgroup_setup(cgroup_name, 0, 100_000)?;
    let mut yes = run_yes()?;

    let failure: Result<(), Box<dyn std::error::Error>> =
        set_scheduler(yes.id(), SchedPolicy::RR(50)).map_err(|err| err.into())
            .and_then(|_| migrate_task_to_cgroup(cgroup_name, yes.id()));

    yes.kill()?;
    delete_cgroup(cgroup_name)?;

    if failure.is_ok() {
        Err(format!("Cgroup with 0 runtime must not allow to run tasks"))?
    } else {
        if !is_batch_test() { println!("Ok!"); }
        Ok(())
    }
}

fn set_runtime_zero_to_active(cgroup_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    if !is_batch_test() {
        println!("Zeroing runtime to cgroup \'{cgroup_name}\' with active task should fail.");
    }

    use hcbs_test_suite::cgroup::__set_cgroup_runtime_us;

    cgroup_setup(cgroup_name, 10_000, 100_000)?;
    let mut yes = run_yes()?;
    set_scheduler(yes.id(), SchedPolicy::RR(50))?;
    migrate_task_to_cgroup(cgroup_name, yes.id())?;

    let failed = __set_cgroup_runtime_us(cgroup_name, 0);

    yes.kill()?;
    migrate_task_to_cgroup(".", yes.id())?;
    delete_cgroup(cgroup_name)?;

    if failed.is_ok() {
        Err(format!("Cannot set runtime zero to cgroup with active tasks"))?
    } else {
        if !is_batch_test() { println!("Ok!"); }
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    mount_cgroup_fs()?;

    migrate_task_to_cgroup(".", std::process::id())?;
    set_scheduler(std::process::id(), SchedPolicy::RR(99))?;

    // batch test utils
    let test_category = "constraints";

    // cannot set period to zero
    batch_test_header("runtime_0_period_0", test_category);
    batch_test_result(cgroup_time_tests("g0", 0, 0))?;

    // given DL_SCALE = 10, runtime must be at least 1024ns, i.e. > 1us
    batch_test_header("runtime_too_small", test_category);
    batch_test_result(cgroup_time_tests("g0", 1, 100_000))?;

    // cannot set runtime greater than period
    batch_test_header("runtime_gt_period", test_category);
    batch_test_result(cgroup_time_tests("g0", 110_000, 100_000))?;

    // period cannot be greater than ~2^53us (i.e. >=2^63ns, which is a negative integer in signed 64-bit)
    batch_test_header("period_too_big", test_category);
    batch_test_result(cgroup_time_tests("g0", 110_000, (2<<63) / 1000 + 1))?;

    // adding task to cgroup with runtime zero
    batch_test_header("runtime_0_add_task", test_category);
    batch_test_result(add_task_to_runtime_zero("g0"))?;

    // set runtime to zero of running cgroup
    batch_test_header("runtime_0_while_running", test_category);
    batch_test_result(set_runtime_zero_to_active("g0"))?;

    // change runtime/period of parent with child with active tasks

    if !is_batch_test() { println!("All tests passed!"); }

    Ok(())
}