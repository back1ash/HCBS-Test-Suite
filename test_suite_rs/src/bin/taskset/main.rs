use hcbs_test_suite::tests::taskset::*;

#[derive(clap::Parser, Debug)]
#[command(about, long_about = None)]
pub enum Command {
    /// Run all taskset tests
    /// 
    /// Run all the taskset tests found in the given input folder.
    #[command(name = "all", verbatim_doc_comment)]
    All(MyArgs),
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let Command::All(args) = <Command as clap::Parser>::parse();
    
    run_taskset_array(args)?;
    Ok(())
}