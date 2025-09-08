use hcbs_test_suite::utils::is_batch_test;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    // check if cgroup filesystem is mounted
    if !hcbs_test_suite::cgroup::__cgroup_exists(".") {
        return Ok(());
    }

    let system = sysinfo::System::new_all();

    for (pid, _) in system.processes() {
        use hcbs_test_suite::prelude::SchedPolicy::*;

        match hcbs_test_suite::prelude::get_scheduler(pid.as_u32()) {
            Ok(OTHER {..}) | Ok(BATCH {..}) | Ok(IDLE) => { continue; },
            Ok(_) => (),
            Err(err) => {
                println!("Error getting policy for pid {pid}: {err}");
                continue;
            }
        };

        let cgroup = hcbs_test_suite::prelude::get_cgroup_of_pid(pid.as_u32())?;
        if cgroup == "." { continue; };

        hcbs_test_suite::prelude::migrate_task_to_cgroup(".", pid.as_u32())?;
        if !is_batch_test() {
            println!("Migrated task {} to root cgroup", pid.as_u32());
        }
    }

    Ok(())
}