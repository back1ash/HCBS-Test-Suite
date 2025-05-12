#![feature(iterator_try_collect)]

use std::ops::{Deref, DerefMut};

pub mod cgroup;
pub mod process;
pub mod utils;
pub mod cpuset;
pub mod tests;

pub mod prelude {
    pub use super::cgroup::prelude::*;
    pub use super::process::prelude::*;
    pub use super::utils::prelude::*;
    pub use super::cpuset::prelude::*;

    pub use super::{
        MyProcess,
        run_yes,
    };
}

pub struct MyProcess {
    process: std::process::Child,
}

impl Drop for MyProcess {
    fn drop(&mut self) {
        self.kill().unwrap()
    }
}

impl Deref for MyProcess {
    type Target = std::process::Child;
    
    fn deref(&self) -> &Self::Target {
        &self.process
    }
}

impl DerefMut for MyProcess {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.process
    }
}

pub fn run_yes() -> Result<MyProcess, std::io::Error> {
    use std::process::*;

    let proc = Command::new("yes")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    Ok(MyProcess { process: proc })
}