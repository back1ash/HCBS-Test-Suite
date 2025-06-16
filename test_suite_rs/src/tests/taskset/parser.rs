use nom::Parser;
use nom::multi::*;
use nom::character::complete::*;
use nom::combinator::*;
use nom::sequence::*;

use super::*;

pub fn parse_taskset_file(taskset_file: &str) -> Result<Taskset, Box<dyn std::error::Error>> {
    let data = std::fs::read_to_string(taskset_file)
        .map_err(|err| format!("Failed to read taskset file {}: {}", taskset_file, err))?;
    let taskset_name = std::path::Path::new(taskset_file)
        .parent().ok_or_else(|| format!("Unknown parent"))?
        .file_name().ok_or_else(|| format!("Unknown directory"))?;
    let taskset_name = __os_str_to_str(taskset_name)?;

    let u64_parser = || map_res(digit1::<&str, ()>, |num: &str| num.parse::<u64>());
    let line_parser = map_res(
        (u64_parser(), space1, u64_parser(), space1, u64_parser()),
        |(runtime_us, _, deadline_us, _, period_us)| {
            if deadline_us == period_us {
                Ok(PeriodicTaskData { runtime_ms: runtime_us, period_ms: period_us })
            } else {
                Err(format!("Expected deadline to be equal to period"))
            }
        }
    );

    let mut parser = separated_list1(newline, line_parser);

    Ok(Taskset {
        name: taskset_name,
        data: parser.parse(&data).map_err(|err| format!("Taskset parser error: {err}"))?.1,
    })
}

pub fn parse_config_file(config_file: &str) -> Result<TasksetConfig, Box<dyn std::error::Error>> {
    let data = std::fs::read_to_string(config_file)
        .map_err(|err| format!("Failed to read config file {}: {}", config_file, err))?;
    let config_name = std::path::Path::new(config_file)
        .file_name().ok_or_else(|| format!("Unknown filename"))?;
    let config_name = __os_str_to_str(&config_name)?;


    let u64_parser = || map_res(digit1::<&str, ()>, |num: &str| num.parse::<u64>());
    let mut parser = map(
        (u64_parser(), space1, u64_parser(), space1, u64_parser()),
        |(num_cpus, _, runtime_us, _, period_us)|
            TasksetConfig {
                name: String::new(),
                num_cpus,
                runtime_ms: runtime_us,
                period_ms: period_us,
            }
    );

    let mut config = parser.parse(&data).map_err(|err| format!("Taskset config parser error: {err}"))?.1;
    config.name = config_name;

    Ok(config)
}

pub fn parse_taskset_results(out_file: &str) -> Result<Vec<TasksetRunResultInstance>, Box<dyn std::error::Error>> {
    let data = std::fs::read_to_string(out_file)
        .map_err(|err| format!("Failed to read output file {}: {}", out_file, err))?;

    let u64_parser = map_res(digit1::<&str, ()>, |num: &str| num.parse::<u64>());
    let f64_parser = map_res(recognize((
            opt(char('-')),
            digit1,
            char('.'),
            digit1
        )), |num: &str| num.parse::<f64>());
    let mut line_parser = 
        map_res(
            (count(terminated(u64_parser, space1), 5), f64_parser),
            |(fields, offset)| {
                Ok::<_, ()>(TasksetRunResultInstance {
                    task: fields[0],
                    instance: fields[1],
                    abs_activation_time_us: fields[2],
                    rel_start_time_us: fields[3],
                    rel_finishing_time_us: fields[4],
                    deadline_offset: offset,
                })
            }
        );

    let data: Vec<_> = data.trim_ascii().lines()
        .filter_map(|line| {
            let line = line.trim_ascii();
            if line.starts_with("#") {
                None
            } else {
                Some(line_parser.parse(&line).map(|(_, res)| res))
            }
        })
        .try_collect()
        .map_err(|err| format!("Taskset result parser error: {err}"))?;

    Ok(data)
}