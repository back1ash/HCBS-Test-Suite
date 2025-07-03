pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let system = sysinfo::System::new_all();

    for (pid, _) in system.processes() {
        use hcbs_test_suite::prelude::MySchedPolicy::*;

        let policy = hcbs_test_suite::prelude::get_policy(pid.as_u32())?;
        if policy == OTHER || policy == BATCH || policy == IDLE { continue; };

        let cgroup = hcbs_test_suite::prelude::get_cgroup_of_pid(pid.as_u32())?;
        if cgroup == "." { continue; };

        hcbs_test_suite::prelude::migrate_task_to_cgroup(&cgroup, pid.as_u32())?;
    }

    Ok(())
}