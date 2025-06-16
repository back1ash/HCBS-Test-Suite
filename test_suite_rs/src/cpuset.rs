use std::str::FromStr;

use crate::utils::__println_debug;

pub mod prelude {
    pub use super::{
        CpuSet,
        set_cpuset_to_pid
    };
}

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
pub struct CpuSet {
    cpus: Vec<u32>,
}

impl CpuSet {
    pub fn empty() -> CpuSet {
        CpuSet { cpus: Vec::with_capacity(0) }
    }

    pub fn all() -> Result<CpuSet, Box<dyn std::error::Error>> {
        let online_cpus = std::fs::read_to_string("/sys/devices/system/cpu/online")?;
        let cpuset = CpuSet::from_str(&online_cpus)?;

        Ok(cpuset)
    }

    pub fn any_subset(num_cpus: u64) -> Result<CpuSet, Box<dyn std::error::Error>> {
        let all = CpuSet::all()?;

        if num_cpus as usize > all.cpus.len() {
            Err(format!("Requesting more CPUs ({num_cpus}) than available ones ({0})", all.cpus.len()))?;
        }

        Ok(CpuSet {
            cpus: all.cpus.into_iter().take(num_cpus as usize).collect()
        })
    }
}

impl FromStr for CpuSet {
    type Err = String;

    fn from_str<'a>(s: &'a str) -> Result<Self, Self::Err> {
        use nom::Parser;
        use nom::bytes::complete::*;
        use nom::branch::*;
        use nom::multi::*;
        use nom::character::complete::*;
        use nom::combinator::*;

        let single_parser = || map_res(digit1::<&str, ()>, |s: &str| s.parse::<u32>());
        let single_parser_pair = map(single_parser(), |cpu| (cpu, cpu) );
        let range_parser = map_res(
            (
                single_parser(),
                tag("-"),
                single_parser()
            ),
            |(min, _, max)| {
                if min > max {
                    Err(format!("Range error"))
                } else {
                    Ok((min, max))
                }
            }
        );

        let separator_parser = map((tag(","), multispace0), |_| ());
        let mut parser = map(
            separated_list1(
                separator_parser,
                alt((range_parser, single_parser_pair))
            ),
            |pairs: Vec<(u32, u32)>| {
                let mut out: Vec<u32> = Vec::new();
                for pair in pairs.into_iter() {
                    for cpu in pair.0 ..= pair.1 {
                        out.push(cpu);
                    }
                }

                out
            }
        );

        Ok(CpuSet {
            cpus: parser.parse(s).map_err(|err| format!("{err}"))?.1,
        })
    }
}

impl From<&CpuSet> for scheduler::CpuSet {
    fn from(cpuset: &CpuSet) -> Self {
        let mut out = scheduler::CpuSet::new(0);
        cpuset.cpus.iter()
            .for_each(|cpu| out.set(*cpu as usize));

        out
    }
}

pub fn set_cpuset_to_pid(pid: u32, cpu_set: &CpuSet) -> Result<(), Box<dyn std::error::Error>> {
    scheduler::set_affinity(pid as i32, cpu_set.into())
        .map_err(|_| format!("Error in setting affinity for pid {pid}"))?;

    __println_debug(|| format!("Changed CPU affinity of pid {pid} to {cpu_set:?}"));

    Ok(())
}