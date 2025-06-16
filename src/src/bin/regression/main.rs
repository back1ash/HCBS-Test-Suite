#![feature(iterator_try_collect)]

mod fair_server;
mod run_all;
mod sched_deadline;
mod sched_fifo;

#[derive(clap::Parser, Debug)]
#[command(about, long_about = None)]
pub enum Command {
    /// Run all tests
    ///
    /// This command runs all the listed tests. It generates pseudo-random
    /// parameters for each of these tests and chooses which to run at random,
    /// totalling to a user specified amount of tests.
    #[command(name = "all", verbatim_doc_comment)]
    All(run_all::MyArgs),

    /// Stress test the fair deadline servers
    ///
    /// This test creates a set of SCHED_FIFO and SCHED_OTHER cpu-bound tasks,
    /// and asserts that the SCHED_OTHER process get at least the minimum amount
    /// of bandwidth that is reserved to non-real-time tasks.
    #[command(name = "fair-server", verbatim_doc_comment)]
    FairServer(fair_server::MyArgs),

    /// Stress test on cgroups vs SCHED_DEADLINE
    ///
    /// This test creates a number of SCHED_DEADLINE tasks to run on the global
    /// runqueue and another batch of FIFO tasks that is run inside a cgroup.
    /// The test expects that the cgroup's tasks consume at least the amount of
    /// requested bandwidth.
    ///
    /// Constraints: runtime <= 0.45 * period
    #[command(name = "deadline", verbatim_doc_comment)]
    SchedDeadline(sched_deadline::MyArgs),

    /// Stress test on cgroups vs SCHED_FIFO
    ///
    /// This test creates a number of SCHED_FIFO tasks to run on the global
    /// runqueue and another batch of FIFO tasks that is run inside a cgroup.
    /// The test expects that the cgroup's tasks consume at least the amount of
    /// requested bandwidth.
    ///
    /// Constraints: runtime <= period
    #[command(name = "fifo", verbatim_doc_comment)]
    SchedFifo(sched_fifo::MyArgs),
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = <Command as clap::Parser>::parse();

    use Command::*;

    match args {
        All(args) => run_all::main(args, None),
        FairServer(args) => fair_server::main(args, None).map(|_| ()),
        SchedDeadline(args) => sched_deadline::main(args, None).map(|_| ()),
        SchedFifo(args) => sched_fifo::main(args, None).map(|_| ()),
    }
}