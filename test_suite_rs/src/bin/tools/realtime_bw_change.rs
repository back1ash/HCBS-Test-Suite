use hcbs_test_suite::{cgroup::__shell, prelude::*};

#[derive(clap::Parser, Debug)]
pub struct MyArgs {
    /// bandwidth
    #[arg(short = 'b', long = "bandwidth", value_name = "[0,1]: f32")]
    bw: f32,
}

// mount -t debugfs none /sys/kernel/debug
pub fn mount_debug_fs() -> Result<(), Box<dyn std::error::Error>> {
    if __shell(&format!("mount | grep debugfs"))?.stdout.len() > 0 {
        __println_debug(|| format!("DebugFS already mounted"));
        return Ok(());
    }

    if !__shell(&format!("mount -t debugfs none /sys/kernel/debug"))?.status.success() {
        __println_debug(|| format!("Error in mounting DebugFS"));
        return Err(format!("Error in mounting DebugFS"))?;
    }

    __println_debug(|| format!("Mounted DebugFS"));

    Ok(())
}

pub fn set_fair_server_bw(bw: f32) -> Result<(), Box<dyn std::error::Error>> {
    let runtime_ns = (bw * 1000_000_000f32) as u64;

    for entry in std::fs::read_dir("/sys/kernel/debug/sched/fair_server")? {
        let entry = entry?.path();
        if entry.is_dir() {
            let entry = entry.into_os_string().into_string().unwrap();
            std::fs::write(format!("{entry}/runtime"), format!("{runtime_ns}"))
                .map_err(|err| format!("Error in writing runtime {runtime_ns} ns to {entry}/runtime: {err}"))?;
        }
    }
    
    Ok(())
}

pub fn main(args: MyArgs) -> Result<(), Box<dyn std::error::Error>> {
    mount_debug_fs()?;
    mount_cgroup_fs()?;

    migrate_task_to_cgroup(".", std::process::id())?;
    chrt(std::process::id(), MySchedPolicy::RR(99))?;

    set_fair_server_bw(0f32)?;

    let period = get_system_rt_period()? as f32;
    let runtime = (args.bw * period) as u64;
    set_system_rt_runtime(runtime)?;

    set_fair_server_bw(1f32 - args.bw)?;

    Ok(())
}