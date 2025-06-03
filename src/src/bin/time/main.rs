#![feature(iterator_try_collect)]

mod one_task;
mod pin_task;
mod many_tasks;

#[derive(clap::Parser, Debug)]
#[command(about, long_about = None)]
pub enum Command {
    /// Run a single yes task in a RT cgroup
    /// 
    /// This command executes a single yes task in a RT cgroup with user
    /// specified parameters, and reports the total used bandwidth of the
    /// process at the end of execution. The test is successful if the task
    /// consumes no more bandwdith than the one allocated to the cgroup.
    /// 
    /// Constraints: runtime <= period
    #[command(name = "one", verbatim_doc_comment)]
    OneTask(one_task::MyArgs),

    /// Run a single yes task in a RT cgroup, pinning it to a specified cpu set
    /// 
    /// This command executes a single yes task in a RT cgroup with user
    /// specified parameters, pinning it to the specified CPU set, and reports
    /// the total used bandwidth of the process at the end of execution. The
    /// test is successful if the task consumes no more bandwdith than the one
    /// allocated to the cgroup.
    /// 
    /// Constraints: runtime <= period
    #[command(name = "pin", verbatim_doc_comment)]
    PinTask(pin_task::MyArgs),

    /// Run multiple yes tasks in a RT cgroup
    /// 
    /// This command executes a user specified number of yes task in a RT cgroup
    /// with user specified parameters, and reports the cumulative total used
    /// bandwidth of the processes at the end of execution. The test is
    /// successful if the tasks consume no more bandwdith than the one allocated
    /// to the cgroup.
    /// 
    /// Constraints: runtime <= period
    ManyTasks(many_tasks::MyArgs),
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = <Command as clap::Parser>::parse();
    
    use Command::*;

    match args {
        OneTask(args) => one_task::main(args, None)?,
        PinTask(args) => pin_task::main(args, None)?,
        ManyTasks(args) => many_tasks::main(args, None)?,
    };

    Ok(())
}