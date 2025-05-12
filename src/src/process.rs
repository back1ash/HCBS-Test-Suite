use crate::{cgroup::{__cgroup_exists, __cgroup_path}, utils::__println_debug};

pub mod prelude {
    pub use super::{
        MySchedPolicy,
        is_pid_in_cgroup,
        get_cgroup_pids,
        migrate_task_to_cgroup,
        chrt,
        kill,
        get_process_total_cpu_usage,
    };
}

#[derive(Debug)]
pub enum MySchedPolicy {
    OTHER,
    FIFO(i32),
    RR(i32)
}

pub fn is_pid_in_cgroup(name: &str, pid: u32) -> Result<bool, Box<dyn std::error::Error>> {
    if !__cgroup_exists(name) {
        return Err(format!("Cgroup {name} does not exist"))?;
    }

    let pid = format!("{pid}");
    let path = __cgroup_path(name);
    Ok(std::fs::read_to_string(format!("{path}/cgroup.procs"))?.lines()
        .find(|line| line == &pid).is_some())
}

pub fn get_cgroup_pids(name: &str) -> Result<Vec<u32>, Box<dyn std::error::Error>> {
    if !__cgroup_exists(name) {
        return Err(format!("Cgroup {name} does not exist"))?;
    }

    let path = __cgroup_path(name);
    Ok(std::fs::read_to_string(format!("{path}/cgroup.procs"))?.lines()
        .map(|line| line.parse::<u32>()).try_collect::<Vec<u32>>()?)
}

pub fn migrate_task_to_cgroup(name: &str, pid: u32) -> Result<(), Box<dyn std::error::Error>> {
    if !__cgroup_exists(name) {
        return Err(format!("Cgroup {name} does not exist"))?;
    }

    let path = __cgroup_path(name);
    std::fs::write(format!("{path}/cgroup.procs"), pid.to_string())
        .map_err(|err| format!("Error in migrating task {pid} to cgroup {name}: {err}"))?;

    __println_debug(|| format!("Migrated task {pid} to Cgroup {name}"));

    Ok(())
}

pub fn kill(pid: u32) {
    let system = sysinfo::System::new();
    let pid = sysinfo::Pid::from_u32(pid);

    system.process(pid).iter()
        .for_each(|proc| { proc.kill(); proc.wait(); });

    __println_debug(|| format!("Killed pid {pid}"));
}

pub fn chrt(pid: u32, policy: MySchedPolicy) -> Result<(), String> {
    use scheduler::set_policy;

    let pid = pid as i32;

    match policy {
        MySchedPolicy::OTHER => set_policy(pid, scheduler::Policy::Other, 0),
        MySchedPolicy::FIFO(prio) => set_policy(pid, scheduler::Policy::Fifo, prio),
        MySchedPolicy::RR(prio) => set_policy(pid, scheduler::Policy::RoundRobin, prio),
    }.map_err(|_| format!("Error in changing policy to {policy:?} for pid {pid}"))?;

    __println_debug(|| format!("Changed policy to pid {pid} to {policy:?}"));

    Ok(())
}

pub fn get_process_total_cpu_usage(pid: u32) -> Result<f32, String> {
    let uptime: f32 = 
        std::fs::read_to_string("/proc/uptime")
                .map_err(|err| format!("{err:?}"))?
            .split_whitespace().nth(0).ok_or("Error in reading /proc/uptime".to_owned())?
            .parse()
                .map_err(|err| format!("{err:?}"))?;

    let stats = std::fs::read_to_string(format!("/proc/{pid}/stat"))
        .map_err(|err| format!("{err:?}"))?;
    let stats: Vec<_> = stats.split_whitespace().collect();

    let ticks_per_second = sysconf::sysconf(sysconf::SysconfVariable::ScClkTck)
        .map_err(|err| format!("{err:?}"))? as f32;

    let utime = stats.get(13).ok_or("Error in reading /proc/<pid>/stat".to_owned())?
        .parse::<isize>().map_err(|err| format!("{err:?}"))? as f32 / ticks_per_second;
    let stime = stats.get(14).ok_or("Error in reading /proc/<pid>/stat".to_owned())?
        .parse::<isize>().map_err(|err| format!("{err:?}"))? as f32 / ticks_per_second;
    let start_time = stats.get(21).ok_or("Error in reading /proc/<pid>/stat".to_owned())?
        .parse::<isize>().map_err(|err| format!("{err:?}"))? as f32 / ticks_per_second;

    let elapsed = uptime - start_time;
    Ok((utime + stime)/ elapsed)
}