use std::time::Duration;

pub mod prelude {
    pub use super::{
        __shell,
        __println_debug,
        set_batch_test,
        is_batch_test,
        wait_loop,
        wait_loop_periodic_fn,
        create_ctrlc_handler,
        ExitFlag,
        mount_debug_fs,
    };
}

pub fn __println_debug<'a, F: FnOnce() -> String>(fun: F) {
    use std::env;

    match env::var("DEBUG") {
        Ok(v) if v != "" => {
            let str = fun();
            println!("{str}");
        },
        _ => (),
    };
}

pub unsafe fn set_batch_test() {
    use std::env;

    unsafe { env::set_var("BATCH_TEST", "1") };
}

pub fn is_batch_test() -> bool {
    use std::env;

    match env::var("BATCH_TEST") {
        Ok(v) if v != "" => true,
        _ => false,
    }
}

#[derive(Clone)]
pub struct ExitFlag {
    ch: crossbeam::channel::Receiver<()>,
}

impl ExitFlag {
    pub fn is_exit(&self) -> bool {
        use crossbeam::channel::TryRecvError::*;

        match self.ch.try_recv() {
            Ok(()) => true,
            Err(Empty) => false,
            _ => panic!("unexpected"),
        }
    }
}

pub fn create_ctrlc_handler() -> Result<ExitFlag, Box<dyn std::error::Error>> {
    let (send, recv) = crossbeam::channel::bounded(1);

    ctrlc::set_handler(move || { send.send(()).unwrap(); })?;
    Ok(ExitFlag { ch: recv })
}

pub fn wait_loop(max_time: Option<u64>, ctrlc_flag: Option<ExitFlag>) -> Result<(), Box<dyn std::error::Error>> {
    let exit = match ctrlc_flag {
        Some(exit) => exit,
        None => create_ctrlc_handler()?,
    };

    let max_time_ch =
        match max_time {
            Some(max_time) => crossbeam::channel::after(Duration::from_secs(max_time)),
            None => crossbeam::channel::never(),
        };

    crossbeam::channel::select! {
        recv(exit.ch) -> _ => (),
        recv(max_time_ch) -> _ => (),
    };

    Ok(())
}

pub fn wait_loop_periodic_fn<F>(period_secs: f32, max_time: Option<u64>, ctrlc_flag: Option<ExitFlag>, mut fun: F) -> Result<(), Box<dyn std::error::Error>>
    where F: FnMut() -> Result<(), Box<dyn std::error::Error>>
{
    let exit = match ctrlc_flag {
        Some(exit) => exit,
        None => create_ctrlc_handler()?,
    };

    let max_time_ch =
        match max_time {
            Some(max_time) => crossbeam::channel::after(Duration::from_secs(max_time)),
            None => crossbeam::channel::never(),
        };

    let periodic_ch = crossbeam::channel::tick(Duration::from_secs_f32(period_secs));

    loop {
        crossbeam::channel::select! {
            recv(exit.ch) -> _ => { break; },
            recv(periodic_ch) -> _ => { fun()?; },
            recv(max_time_ch) -> _ => { break; },
        }
    }

    Ok(())
}

pub fn __shell(cmd: &str) -> Result<std::process::Output, Box<dyn std::error::Error>> {
    use std::process::Command;

    Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .map_err(|err| format!("Error in executing \"sh -c {cmd}\": {err}").into())
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