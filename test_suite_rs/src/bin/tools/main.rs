mod realtime_bw_change;
mod move_rt_to_root_cgroup;
mod cgroup_setup;
mod hrtick;

#[derive(clap::Parser, Debug)]
#[command(about, long_about = None)]
pub enum Command {
    /// CPU hog
    #[command(name = "hog", verbatim_doc_comment)]
    Hog,

    /// Mount CGRoup filesystem and CPU controller
    #[command(name = "mount-cgroup-fs", verbatim_doc_comment)]
    MountCgroupFS,

    /// Mount DebugFS
    #[command(name = "mount-debug-fs", verbatim_doc_comment)]
    MountDebugFS,

    /// Change the badwidth allocated to real-time tasks (both FIFO/RR and DEADLINE)
    #[command(name = "rt-bw-change", verbatim_doc_comment)]
    RealtimeBwChange(realtime_bw_change::MyArgs),

    /// Move all real-time tasks to the root control group
    #[command(name = "move-to-root", verbatim_doc_comment)]
    MoveRtTasksToRootCgroup,

    /// Change the runtime and period to the given control group
    #[command(name = "cgroup-setup", verbatim_doc_comment)]
    CgroupBwChange(cgroup_setup::MyArgs),

    /// Enable/Disable the HRTICK_DL scheduler feature
    #[command(name = "hrtick", verbatim_doc_comment)]
    HRTick(hrtick::MyArgs),
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = <Command as clap::Parser>::parse();
    
    use Command::*;

    match args {
        Hog => {
            let mut i = 0;
            loop { unsafe {
                let i_val = core::ptr::read_volatile(&i);
                core::ptr::write_volatile(&mut i, i_val + 1);
            } }
        }
        MountCgroupFS => hcbs_test_suite::cgroup::mount_cgroup_fs()?,
        MountDebugFS => hcbs_test_suite::utils::mount_debug_fs()?,
        RealtimeBwChange(args) => realtime_bw_change::main(args)?,
        MoveRtTasksToRootCgroup => move_rt_to_root_cgroup::main()?,
        CgroupBwChange(args) => cgroup_setup::main(args)?,
        HRTick(args) => hrtick::main(args)?,
    };

    Ok(())
}