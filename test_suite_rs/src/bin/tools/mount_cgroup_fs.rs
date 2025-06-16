use hcbs_test_suite::cgroup::mount_cgroup_fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    mount_cgroup_fs()
}