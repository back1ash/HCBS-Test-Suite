use std::{sync::{atomic::{AtomicBool, Ordering}, Arc}, thread, time::{self, Duration}};

pub mod prelude {
    pub use super::{
        __println_debug,
        set_batch_test,
        is_batch_test,
        wait_loop,
        wait_loop_periodic_fn,
        create_ctrlc_handler,
        CtrlFlag,
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

pub type CtrlFlag = Arc<AtomicBool>;

pub fn create_ctrlc_handler() -> Result<CtrlFlag, Box<dyn std::error::Error>> {
    let exit = Arc::new(AtomicBool::new(false));
    let exit_clone = Arc::clone(&exit);

    ctrlc::set_handler(move || { exit_clone.store(true, Ordering::Relaxed); })?;
    Ok(exit)
}

pub fn wait_loop(max_time: Option<u64>, ctrlc_flag: Option<CtrlFlag>) -> Result<(), Box<dyn std::error::Error>> {
    let exit = match ctrlc_flag {
        Some(exit) => exit,
        None => create_ctrlc_handler()?,
    };

    match max_time {
        Some(max_time) => {
            let start_time = time::Instant::now();
            while !exit.load(Ordering::Relaxed) {
                if (time::Instant::now() - start_time).as_secs() >= max_time {
                    break;
                }
        
                thread::sleep(Duration::from_secs_f32(0.1));
            }
        },
        None => {
            while !exit.load(Ordering::Relaxed) {
                thread::sleep(Duration::from_secs_f32(0.1));
            }
        },
    }

    Ok(())
}

pub fn wait_loop_periodic_fn<F>(period_secs: f32, max_time: Option<u64>, ctrlc_flag: Option<CtrlFlag>, mut fun: F) -> Result<(), Box<dyn std::error::Error>>
    where F: FnMut() -> Result<(), Box<dyn std::error::Error>>
{
    let exit = match ctrlc_flag {
        Some(exit) => exit,
        None => create_ctrlc_handler()?,
    };

    let mut last_time = time::Instant::now();
    match max_time {
        Some(max_time) => {
            let start_time = time::Instant::now();
            while !exit.load(Ordering::Relaxed) {
                let now = time::Instant::now();
                if (now - start_time).as_secs() >= max_time {
                    break;
                }

                if (now - last_time).as_secs_f32() >= period_secs {
                    fun()?;
                    last_time = now;
                }
        
                thread::sleep(Duration::from_secs_f32(
                    f32::min(0.1, period_secs - (now - last_time).as_secs_f32())
                ));
            }
        },
        None => {
            while !exit.load(Ordering::Relaxed) {
                let now = time::Instant::now();
                if (now - last_time).as_secs_f32() >= period_secs {
                    fun()?;
                    last_time = now;
                }
        
                thread::sleep(Duration::from_secs_f32(
                    f32::min(0.1, period_secs - (now - last_time).as_secs_f32())
                ));
            }
        },
    }

    Ok(())
}