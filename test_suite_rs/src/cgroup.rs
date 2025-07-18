use std::io::Write;

use crate::{process::{get_cgroup_pids, is_pid_in_cgroup, kill, migrate_task_to_cgroup}, utils::{__println_debug, __shell}};

pub mod prelude {
    pub use super::{
        mount_cgroup_fs,
        create_cgroup,
        delete_cgroup,
        cgroup_setup,
        MyCgroup,
        get_system_rt_period,
        get_system_rt_runtime,
        set_system_rt_period,
        set_system_rt_runtime
    };
}

const CGROUP_ROOT: &'static str = "/sys/fs/cgroup";

#[cfg(not(feature = "cgroup_v2"))]
pub fn __cgroup_path(name: &str) -> String {
    format!("{CGROUP_ROOT}/cpu/{name}")
}

#[cfg(feature = "cgroup_v2")]
pub fn __cgroup_path(name: &str) -> String {
    format!("{CGROUP_ROOT}/{name}")
}

pub fn __cgroup_exists(name: &str) -> bool {
    let path = __cgroup_path(name);
    let path = std::path::Path::new(&path);
    path.exists() && path.is_dir()
}

pub fn __cgroup_num_procs(name: &str) -> Result<i32, Box<dyn std::error::Error>> {
    let path = __cgroup_path(name);
    Ok(std::fs::read_to_string(format!("{path}/cgroup.procs"))
        .map_err(|err| format!("Error in reading {path}/cgroup.procs: {err}"))?
        .lines().count() as i32)
}

pub fn mount_cgroup_fs() -> Result<(), Box<dyn std::error::Error>> {
    __mount_cgroup_fs()?;
    __mount_cpu_fs()?;
    
    Ok(())
}

#[cfg(not(feature = "cgroup_v2"))]
pub fn __mount_cgroup_fs() -> Result<(), Box<dyn std::error::Error>> {
    if __shell(&format!("mount | grep cgroup"))?.stdout.len() > 0 {
        __println_debug(|| format!("Cgroup v1 FS already mounted"));
        return Ok(());
    }

    if !__shell(&format!("mount -t tmpfs tmpfs {CGROUP_ROOT}"))?.status.success() {
        __println_debug(|| format!("Error in mounting Cgroup FS"));
        return Err(format!("Error in mounting Cgroup v1 FS"))?;
    }

    __println_debug(|| format!("Mounted Cgroup v1 FS"));

    Ok(())
}

#[cfg(not(feature = "cgroup_v2"))]
pub fn __mount_cpu_fs() -> Result<(), Box<dyn std::error::Error>> {
    let cpu_path = format!("{CGROUP_ROOT}/cpu");
    let cpu_path = std::path::Path::new(&cpu_path);
    if cpu_path.exists() && cpu_path.is_dir() {
        __println_debug(|| format!("Cgroup CPU FS already mounted"));
        return Ok(());
    }

    if !__shell(&format!("mkdir {CGROUP_ROOT}/cpu"))?.status.success() ||
        !__shell(&format!("mount -t cgroup -o cpu cpu-cgroup {CGROUP_ROOT}/cpu"))?.status.success()
    {
        __println_debug(|| format!("Error in mounting Cgroup v1 CPU FS"));
        return Err(format!("Error in mounting Cgroup v1 CPU FS"))?;
    }

    __println_debug(|| format!("Mounted Cgroup v1 CPU FS"));

    Ok(())
}

#[cfg(feature = "cgroup_v2")]
pub fn __mount_cgroup_fs() -> Result<(), Box<dyn std::error::Error>> {
    if __shell(&format!("mount | grep cgroup2"))?.stdout.len() > 0 {
        __println_debug(|| format!("Cgroup v2 FS already mounted"));
        return Ok(());
    }

    if !__shell(&format!("mount -t cgroup2 none {CGROUP_ROOT}"))?.status.success() {
        __println_debug(|| format!("Error in mounting Cgroup v2 FS"));
        return Err(format!("Error in mounting Cgroup v2 FS"))?;
    }

    __println_debug(|| format!("Mounted Cgroup v2 FS"));

    Ok(())
}

#[cfg(feature = "cgroup_v2")]
pub fn __mount_cpu_fs() -> Result<(), Box<dyn std::error::Error>> {
    __enable_cpu_contoller_v2(".")
}

#[cfg(feature = "cgroup_v2")]
pub fn __is_cpu_contoller_v2_enabled(name: &str) -> Result<bool, Box<dyn std::error::Error>> {
    if !__cgroup_exists(name) {
        return Err(format!("Cgroup {name} does not exist"))?;
    }

    let controllers_path = format!("{CGROUP_ROOT}/{name}/cgroup.subtree_control");
    let controllers_path = std::path::Path::new(&controllers_path);
    if !controllers_path.exists() || !controllers_path.is_file() {
        return Err(format!("Unexpected! Controllers file for cgroup {name} does not exist"))?;
    }

    Ok(
        std::fs::read_to_string(controllers_path)
        .map_err(|err| format!("Error in reading controllers for cgroup {name}: {err}") )?
        .contains("cpu")
    )
}

#[cfg(feature = "cgroup_v2")]
pub fn __enable_cpu_contoller_v2(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    if __is_cpu_contoller_v2_enabled(name)? { return Ok(()); }

    let controllers_path = format!("{CGROUP_ROOT}/{name}/cgroup.subtree_control");
    std::fs::write(controllers_path, "+cpu")
        .map_err(|err| format!("Error in enabling CPU controller for cgroup {name}: {err}") )?;

    __println_debug(|| format!("Enabled CPU controller for cgroup {name}"));

    Ok(())
}

#[cfg(feature = "cgroup_v2")]
pub fn __enable_cpu_contoller_v2_recursive(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let path = std::path::Path::new(name);
    let ancestors: Vec<_> = path.ancestors()
        .filter(|ancestor| ancestor.as_os_str().is_empty())
        .collect();
    
    ancestors.into_iter().rev()
        .try_for_each(|ancestror| __enable_cpu_contoller_v2(ancestror.to_str().unwrap()))?;

    Ok(())
}

pub fn get_system_rt_period() -> Result<u64, Box<dyn std::error::Error>> {
    Ok(
        std::fs::read_to_string("/proc/sys/kernel/sched_rt_period_us")
            .map_err(|err| format!("Error in reading from /proc/sys/kernel/sched_rt_period_us: {err}"))
        .and_then(|s| s.trim().parse::<u64>()
            .map_err(|err| format!("Error in parsing /proc/sys/kernel/sched_rt_period_us: {err}")))?
    )
}

pub fn get_system_rt_runtime() -> Result<u64, Box<dyn std::error::Error>> {
    Ok(
        std::fs::read_to_string("/proc/sys/kernel/sched_rt_runtime_us")
            .map_err(|err| format!("Error in reading from /proc/sys/kernel/sched_rt_runtime_us: {err}"))
        .and_then(|s| s.trim().parse::<u64>()
            .map_err(|err| format!("Error in parsing /proc/sys/kernel/sched_rt_runtime_us: {err}")))?
    )
}

pub fn set_system_rt_period(period_us: u64) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::write("/proc/sys/kernel/sched_rt_period_us", format!("{period_us}"))
        .map_err(|err| format!("Error in writing period {period_us} us to /proc/sys/kernel/sched_rt_runtime_us: {err}"))?;

    __println_debug(|| format!("Set period {period_us} us to /proc/sys/kernel/sched_rt_runtime_us"));

    Ok(())
}

pub fn set_system_rt_runtime(runtime_us: u64) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::write("/proc/sys/kernel/sched_rt_runtime_us", format!("{runtime_us}"))
        .map_err(|err| format!("Error in writing runtime {runtime_us} us to /proc/sys/kernel/sched_rt_runtime_us: {err}"))?;
    
    __println_debug(|| format!("Set runtime {runtime_us} us to /proc/sys/kernel/sched_rt_runtime_us"));

    Ok(())
}

pub fn create_cgroup(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    mount_cgroup_fs()?;

    if name == "." { return Ok(()); }

    if __cgroup_exists(name) {
        __println_debug(|| format!("Cgroup {name} already exists"));
        return Ok(());
    }

    let path = __cgroup_path(name);
    std::fs::create_dir_all(&path)
        .map_err(|err| format!("Error in creating directory {path}: {err}"))?;

    #[cfg(feature = "cgroup_v2")]
    __enable_cpu_contoller_v2_recursive(name)?;

    __println_debug(|| format!("Created Cgroup {name}"));

    Ok(())
}

pub fn delete_cgroup(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    if name == "." { return Ok(()); }

    if !__cgroup_exists(name) {
        __println_debug(|| format!("Cgroup {name} does not already exist"));
        return Ok(());
    }

    if __cgroup_num_procs(name)? > 0 {
        let procs = get_cgroup_pids(name)?;
        println!("Cgroup {name} has active processes: {procs:?}");
        return Err(format!("Cgroup {name} has active processes"))?;
    }

    let path = __cgroup_path(name);
    std::fs::remove_dir(&path)
        .map_err(|err| format!("Error in destroying directory {path}: {err}"))?;

    __println_debug(|| format!("Deleted Cgroup {name}"));

    Ok(())
}

pub fn __set_cgroup_period_us(name: &str, period_us: u64) -> Result<(), Box<dyn std::error::Error>> {
    let path = __cgroup_path(name);

    let old_period_us = __get_cgroup_period_us(name)?;
    if old_period_us != period_us {
        std::fs::OpenOptions::new().write(true)
            .open(format!("{path}/cpu.rt_period_us"))
            .map_err(|err| format!("Error in opening file {path}/cpu.rt_period_us: {err}"))?
            .write_all(format!("{period_us}").as_bytes())
            .map_err(|err| format!("Error in writing period {period_us} us to {path}/cpu.rt_period_us: {err}"))?;
    }

    __println_debug(|| format!("Set period {period_us} us to {path}/cpu.rt_period_us"));

    Ok(())
}

pub fn __set_cgroup_runtime_us(name: &str, runtime_us: u64) -> Result<(), Box<dyn std::error::Error>> {
    let path = __cgroup_path(name);

    let old_runtime_us = __get_cgroup_runtime_us(name)?;
    if old_runtime_us != runtime_us {
        std::fs::OpenOptions::new().write(true)
            .open(format!("{path}/cpu.rt_runtime_us"))
            .map_err(|err| format!("Error in opening file {path}/cpu.rt_runtime_us: {err}"))?
            .write_all(format!("{runtime_us}").as_bytes())
            .map_err(|err| format!("Error in writing runtime {runtime_us} us to {path}/cpu.rt_runtime_us: {err}"))?;
    }
    
    __println_debug(|| format!("Set runtime {runtime_us} us to {path}/cpu.rt_runtime_us"));

    Ok(())
}

pub fn __get_cgroup_period_us(name: &str) -> Result<u64, Box<dyn std::error::Error>> {
    let path = __cgroup_path(name);

    Ok(
        std::fs::read_to_string(format!("{path}/cpu.rt_period_us"))
            .map_err(|err| format!("Error in reading from {path}/cpu.rt_period_us: {err}"))
        .and_then(|s| s.trim().parse::<u64>()
            .map_err(|err| format!("Error in parsing {path}/cpu.rt_period_us: {err}")))?
    )
}

pub fn __get_cgroup_runtime_us(name: &str) -> Result<u64, Box<dyn std::error::Error>> {
    let path = __cgroup_path(name);

    Ok(
        std::fs::read_to_string(format!("{path}/cpu.rt_runtime_us"))
            .map_err(|err| format!("Error in reading from {path}/cpu.rt_runtime_us: {err}"))
        .and_then(|s| s.trim().parse::<u64>()
            .map_err(|err| format!("Error in parsing {path}/cpu.rt_runtime_us: {err}")))?
    )
}

pub fn cgroup_setup(name: &str, runtime_us: u64, period_us: u64) -> Result<(), Box<dyn std::error::Error>> {
    if runtime_us > period_us {
        return Err(format!("Requested runtime {runtime_us} is greater than the period {period_us}"))?;
    }

    create_cgroup(name)?;

    let old_runtime_us = __get_cgroup_runtime_us(name)?;

    if runtime_us > old_runtime_us {
        __set_cgroup_period_us(name, period_us)?;
        __set_cgroup_runtime_us(name, runtime_us)?;
    } else {
        __set_cgroup_runtime_us(name, runtime_us)?;
        __set_cgroup_period_us(name, period_us)?;
    }

    __println_debug(|| format!("Cgroup {name} setup to {runtime_us}/{period_us} reservation"));

    Ok(())
}

pub struct MyCgroup {
    name: String,
    force_kill: bool,
}

impl MyCgroup {
    pub fn new(name: &str, runtime_us: u64, period_us: u64, force_kill: bool) -> Result<MyCgroup, Box<dyn std::error::Error>> {
        if name == "." {
            return Err(format!("Cannot handle root cgroup"))?;
        }

        cgroup_setup(name, runtime_us, period_us)?;
        Ok(MyCgroup {
            name: name.to_owned(),
            force_kill
        })
    }

    pub fn update_runtime(&mut self, runtime_us: u64) -> Result<(), Box<dyn std::error::Error>> {
        __set_cgroup_runtime_us(&self.name, runtime_us)
    }

    pub fn destroy(mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.__destroy()
    }

    fn __destroy(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !__cgroup_exists(&self.name) { return Ok(()); }

        if self.force_kill {
            if is_pid_in_cgroup(&self.name, std::process::id())? {
                migrate_task_to_cgroup(".", std::process::id())?;
            }

            get_cgroup_pids(&self.name)?.iter()
                .try_for_each(|pid| {
                    kill(*pid);
                    migrate_task_to_cgroup(".", *pid)
                })?;
        }

        __set_cgroup_runtime_us(&self.name, 0)?;
        delete_cgroup(&self.name)
    }
}

impl Drop for MyCgroup {
    fn drop(&mut self) {
        let _ = self.__destroy();
    }
}