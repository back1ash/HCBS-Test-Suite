mod realtime_bw_change;
mod move_rt_to_root_cgroup;

#[derive(clap::Parser, Debug)]
#[command(about, long_about = None)]
pub enum Command {
    /// Mount CGRoup filesystem and CPU controller
    #[command(name = "mount-cgroup-fs", verbatim_doc_comment)]
    MountCgroupFS,

    /// Change the badwidth allocated to real-time tasks (both FIFO/RR and DEADLINE)
    #[command(name = "realtime-bw-change", verbatim_doc_comment)]
    RealtimeBwChange(realtime_bw_change::MyArgs),

    /// Move all real-time tasks to the root control group
    #[command(name = "move-to-root", verbatim_doc_comment)]
    MoveRtTasksToRootCgroup,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = <Command as clap::Parser>::parse();
    
    use Command::*;

    match args {
        MountCgroupFS => hcbs_test_suite::cgroup::mount_cgroup_fs()?,
        RealtimeBwChange(args) => realtime_bw_change::main(args)?,
        MoveRtTasksToRootCgroup => move_rt_to_root_cgroup::main()?,
    };

    Ok(())
}