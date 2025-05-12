use hcbs_test_suite::*;
use hcbs_test_suite::prelude::*;

fn cgroup_time_tests(cgroup_name: &str, runtime_us: u64, period_us: u64) -> Result<(), Box<dyn std::error::Error>> {
    use hcbs_test_suite::cgroup::{__set_cgroup_period, __set_cgroup_runtime};

    create_cgroup(cgroup_name)?;

    let failure: Result<(), _> = 
        __set_cgroup_period(cgroup_name, period_us)
            .and_then(|_| __set_cgroup_runtime(cgroup_name, runtime_us));

    delete_cgroup(cgroup_name)?;

    if failure.is_ok() {
        Err(format!("Cgroup creation with {runtime_us}/{period_us} should fail"))?
    } else {
        Ok(())
    }
}

fn add_task_to_runtime_zero(cgroup_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    cgroup_setup(cgroup_name, 0, 100_000)?;
    let mut yes = run_yes()?;

    let failure: Result<(), Box<dyn std::error::Error>> =
        chrt(yes.id(), MySchedPolicy::RR(50)).map_err(|err| err.into())
            .and_then(|_| migrate_task_to_cgroup(cgroup_name, yes.id()));

    yes.kill()?;
    delete_cgroup(cgroup_name)?;

    if failure.is_ok() {
        Err(format!("Cgroup with 0 runtime should not allow to run tasks"))?
    } else {
        Ok(())
    }
}

fn set_runtime_zero_to_active(cgroup_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    use hcbs_test_suite::cgroup::__set_cgroup_runtime;

    cgroup_setup(cgroup_name, 10_000, 100_000)?;
    let mut yes = run_yes()?;
    chrt(yes.id(), MySchedPolicy::RR(50))?;
    migrate_task_to_cgroup(cgroup_name, yes.id())?;

    let failed = __set_cgroup_runtime(cgroup_name, 0);

    yes.kill()?;
    delete_cgroup(cgroup_name)?;

    if failed.is_ok() {
        Err(format!("Cannot set runtime zero to cgroup with active tasks"))?
    } else {
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    chrt(std::process::id(), MySchedPolicy::RR(99))?;

    mount_cgroup_fs()?;

    // cannot disable root cgroup
    cgroup_time_tests(".", 0, cgroup::__get_cgroup_period(".")?)?;

    // cannot set period to zero
    cgroup_time_tests("g0", 0, 0)?;

    // given DL_SCALE = 10, runtime must be at least 1024ns, i.e. 1us
    cgroup_time_tests("g0", 1, 100_000)?;

    // cannot set runtime greater than period
    // should fail but a weird bug appears
    // cgroup_time_tests("g0", 110_000, 100_000)?;

    // period cannot be greater than ~2^53us (i.e. >=2^63ns, which is a negative integer in signed 64-bit)
    // same as above
    // cgroup_time_tests("g0", 110_000, (2<<63) / 1000 + 1)?;

    // adding task to cgroup with runtime zero
    add_task_to_runtime_zero("g0")?;

    // set runtime to zero of running cgroup
    set_runtime_zero_to_active("g0")?;

    // change runtime/period of parent with child with active tasks

    Ok(())
}