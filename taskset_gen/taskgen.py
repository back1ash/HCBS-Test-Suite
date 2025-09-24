import sys, os
from typing import Any
import numpy

class OutputOptions:
    def __init__(self, out_folder: str):
        self.out_folder = out_folder

class ConfigGenOptions:
    def __init__(self, period_minmax: tuple[int, int], period_step: int | None, max_bw: float | None):
        self.permin, self.permax = period_minmax
        self.max_bw = max_bw

        if self.permin <= 0:
            print("Period minimum must be greater than 0", file=sys.stderr)
            exit(1)

        #permax = None is default.  Set to permin in this case
        if self.permax == None:
            self.permax = self.permin

        if self.permin > self.permax:
            print("Period maximum must be greater than or equal to minimum", file=sys.stderr)
            exit(1)

        #pergran = None is default.  Set to permin in this case
        if period_step is not None:
            self.pergran = period_step
        else:
            self.pergran = self.permin

        if self.pergran < 1:
            print("Period granularity must be an integer greater than equal to 1", file=sys.stderr)
            exit(1)
class TaskgenOptions:
    def __init__(self, num_tasksets_per_utilization: int, num_tasks_minmax: tuple[int, int], utilizations: list[float], period_minmax: tuple[int, int], period_step: int | None = None, seed: int = 0):
        self.num_tasksets_per_utilization = num_tasksets_per_utilization
        self.num_tasks_minmax = num_tasks_minmax
        self.utilizations = utilizations
        self.period_minmax = period_minmax
        self.period_step = period_step
        self.seed = seed

        if self.seed > 0:
            numpy.random.seed(self.seed)

class TaskgenLibOptions:
    def __init__(self, id: int, num_tasksets: int, num_tasks: int, utilization: float, period_minmax: tuple[int, int], period_step: int | None = None, distribution: str = 'logunif', round: bool = True):
        self.id = id
        self.n = num_tasks
        self.util = utilization
        self.nsets = num_tasksets
        self.perdist = distribution
        self.permin, self.permax = period_minmax
        self.pergran = period_step
        self.round_C = round
        self.format = '%(C)d %(T)d %(T)d\n'

        if self.n < 1:
            print("Minimum number of tasks is 1", file=sys.stderr)
            exit(1)

        if self.util > self.n:
            print("Taskset utilisation must be less than or equal to number of tasks", file=sys.stderr)
            exit(1)

        if self.nsets < 1:
            print("Minimum number of tasksets is 1", file=sys.stderr)
            exit(1)

        known_perdists = ["unif", "logunif"]
        if self.perdist not in known_perdists:
            print("Period distribution must be one of " + str(known_perdists), file=sys.stderr)
            exit(1)

        if self.permin <= 0:
            print("Period minimum must be greater than 0", file=sys.stderr)
            exit(1)

        #permax = None is default.  Set to permin in this case
        if self.permax == None:
            self.permax = self.permin

        if self.permin > self.permax:
            print("Period maximum must be greater than or equal to minimum", file=sys.stderr)
            exit(1)

        #pergran = None is default.  Set to permin in this case
        if self.pergran == None:
            self.pergran = self.permin

        if self.pergran < 1:
            print("Period granularity must be an integer greater than equal to 1", file=sys.stderr)
            exit(1)

        if (self.permax % self.pergran) != 0:
            print("Period maximum must be a integer multiple of period granularity", file=sys.stderr)
            exit(1)

        if (self.permin % self.pergran) != 0:
            print("Period minimum must be a integer multiple of period granularity", file=sys.stderr)
            exit(1)

class Task:
    def __init__(self, runtime: int, period: int, utilization: float):
        self.runtime = runtime
        self.period = period
        self.deadline = self.period
        self.utilization = utilization

    def __repr__(self):
        return f"<Task {self.runtime:.0f}/{self.deadline:.0f}/{self.period:.0f} (utilization {self.runtime/self.period:.2f})>"

    def format_out(self):
        return f"{self.runtime:.0f} {self.deadline:.0f} {self.period:.0f}"

class Taskset:
    def __init__(self, tasks: list[Task], utilization: float, id: int):
        self.tasks = tasks
        self.utilization = utilization
        self.id = id

    def __repr__(self):
        return f"<Taskset {self.tasks}>"

    def name(self):
        return f"taskset_U{self.utilization:3.1f}_N{len(self.tasks):02d}_{self.id:03d}"

    def format_out(self):
        out = ''
        for task in self.tasks:
            out += f"{task.format_out()}\n"

        return out

class Config:
    def __init__(self, num_cpus: int, runtime: int, period: int):
        self.num_cpus = num_cpus
        self.runtime = runtime
        self.period = period

    def __repr__(self):
        return f"<Config {self.runtime:.0f}/{self.period:.0f} on {self.num_cpus} CPUs>"

    def format_out(self):
        return f"{self.num_cpus} {self.runtime} {self.period}"

def gen_tasksets(options: TaskgenLibOptions) -> list[Taskset]:
    from taskgen_lib import StaffordRandFixedSum, gen_periods

    x = StaffordRandFixedSum(options.n, options.util, options.nsets)
    periods: Any = gen_periods(options.n, options.nsets, options.permin, options.permax, options.pergran, options.perdist)
    #iterate through each row (which represents utils for a taskset)
    tasksets = []
    for i in range(numpy.size(x, axis=0)):
        C = x[i] * periods[i]
        if options.round_C:
            C = numpy.round(C, decimals=0)

        taskset_data = numpy.c_[x[i], C / periods[i], periods[i], C]

        taskset = []
        for t in range(numpy.size(taskset_data,0)):
            if taskset_data[t][3] == 0:
                continue

            taskset += [ Task(taskset_data[t][3], taskset_data[t][2], taskset_data[t][0]) ]

        taskset.sort(key= lambda task: task.period)
        tasksets += [ Taskset(taskset, options.util, options.id) ]

    return tasksets

def gen_configs(tasksets: list[Taskset], options: ConfigGenOptions) -> list[tuple[Taskset, list[Config]]]:
    import concurrent.futures

    futures = []
    with concurrent.futures.ProcessPoolExecutor() as executor:
        for i, taskset in enumerate(tasksets):
            for period in range(options.permin, options.permax + 1, options.pergran):
                futures += [ executor.submit(carts_analysis, i, taskset, period) ]

    out = [ (taskset, []) for taskset in tasksets ]
    for future in concurrent.futures.as_completed(futures):
        taskset_id, config = future.result()
        print(f"Generated new config for taskset {taskset_id}/{len(out)}")
        if config is not None:
            if options.max_bw is None:
                out[taskset_id][1].append(config)
            else:
                # Do not save configurations which exceed the maximum allowed bandwidth
                bw = config.runtime / config.period
                if bw <= options.max_bw:
                    out[taskset_id][1].append(config)
        else:
            print(f"* Generation failed for taskset {taskset_id}")

    return out

def carts_analysis(id: int, taskset: Taskset, period: int) -> tuple[int, Config | None]:
    from tempfile import NamedTemporaryFile
    import subprocess

    precision = 10
    def generate_xml_file(taskset: Taskset, period: int) -> str:
        header = \
f'''<system os_scheduler="DM" min_period="0" max_period="0">
  <component name="Test" scheduler="DM" min_period="{period * precision}" max_period="{period * precision}">
'''

        footer = \
'''
  </component>
</system>'''

        tasks = []
        for i, task in enumerate(taskset.tasks):
            tasks += [ f"    <task name=\"T{i}\" p=\"{task.period * precision}\" d=\"{task.deadline * precision}\" e=\"{task.runtime * precision}\"></task>" ]

        tasks = '\n'.join(tasks)
        return f"{header}{tasks}{footer}"

    def parse_output_file(data: str) -> Config | None:
        from math import ceil

        for line in data.splitlines():
            if "cpus" in line:
                line = line.split(' ')
                cpus = int(line[1].split('"')[1])
                if cpus == 0:
                    continue

                period = int(line[2].split('"')[1]) // precision
                runtime = ceil(float(line[3].split('"')[1]) / (cpus * precision))
                return Config(cpus, runtime, period)

        return None

    carts_bin = f"{os.environ["BUILD"]}/SchedTest/Carts/carts-source/bin"

    out_data = ""
    with NamedTemporaryFile(delete_on_close=False) as input, NamedTemporaryFile(delete_on_close=False) as output:
        input.write(generate_xml_file(taskset, period).encode())
        input.close()
        output.close()

        subprocess.run(["java", "-cp", carts_bin, "Carts", input.name, "MPR2", output.name, "MARKO_SCHEDTEST"], shell=False, capture_output=True)

        with open(output.name, mode='r') as f:
            out_data = f.read()

    return (id, parse_output_file(out_data))

def mpr_analysis(id: int, taskset: Taskset, config: Config) -> tuple[int, Config | None]:
    from tempfile import NamedTemporaryFile
    import subprocess

    def parse_output_file(out_data: str, config: Config) -> Config | None:
        print(out_data)
        if "Schedulable M-SBF: 1" in out_data or "Schedulable H-BCL: 1" in out_data:
            return config
        return None

    mpr_bin = "build/MultiContainerAnalysis/SchedAnalysis/m-h_test"

    out_data = ""
    with NamedTemporaryFile(delete_on_close=False) as input:
        input.write(taskset.format_out().encode())
        input.close()

        cpus = [ str(config.runtime), str(config.period) ] * config.num_cpus
        proc = subprocess.run([mpr_bin, input.name] + cpus, shell=False, capture_output=True)

        out_data = proc.stdout.decode()

    return (id, parse_output_file(out_data, config))

def save_configs(tasksets: list[tuple[Taskset, list[Config]]], options: OutputOptions):
    import os

    for i, (taskset, configs) in enumerate(tasksets):
        if len(configs) == 0:
            continue

        name = taskset.name()
        path = f"{options.out_folder}/{name}"
        os.makedirs(path)

        with open(f"{path}/taskset.txt", "w") as f:
            f.write(taskset.format_out())

        for j, config in enumerate(configs):
            name = f"config{j:03d}"
            with open(f"{path}/{name}.txt", "w") as f:
                f.write(config.format_out())

def float_range(min: float, max: float, step: float):
    assert(min < max)
    assert(step > 0)

    val = min
    while val < max:
        yield val
        val += step

def parse_args() -> tuple[TaskgenOptions, ConfigGenOptions, OutputOptions]:
    import argparse, sys

    parser = argparse.ArgumentParser(prog="Taskset Generator")

    parser.add_argument("-o", "--outdir", type=str, required=True)
    parser.add_argument("-n", "--taskset-per-util", default=3, type=int, help="default: 3")
    parser.add_argument("-t", "--num-tasks-min", default=6, type=int, help="default: 6")
    parser.add_argument("-T", "--num-tasks-max", default=16, type=int, help="default: 16")
    parser.add_argument("-u", "--tasks-util-min", default=0.2, type=float, help="default: 0.5")
    parser.add_argument("-U", "--tasks-util-max", default=2.5, type=float, help="default: 2.5")
    parser.add_argument("--tasks-util-step", default=0.2, type=float, help="default: 0.2")
    parser.add_argument("-p", "--tasks-period-min", default=100, type=int, help="default: 100")
    parser.add_argument("-P", "--tasks-period-max", default=500, type=int, help="default: 500")
    parser.add_argument("--tasks-period-step", default=10, type=int, help="default: 10")
    parser.add_argument("-c", "--cgroup-period-min", default=20, type=int, help="default: 20")
    parser.add_argument("-C", "--cgroup-period-max", default=100, type=int, help="default: 100")
    parser.add_argument("--cgroup-period-step", default=40, type=int, help="default: 40")
    parser.add_argument("--max-bw", default=0.9, type=float, help="default: 0.9")
    parser.add_argument("-R", "--seed", default=42, type=int, help="default: 42")

    parsed = parser.parse_args(sys.argv[1:])

    taskgen_options = TaskgenOptions(
        parsed.taskset_per_util,
        (parsed.num_tasks_min, parsed.num_tasks_max),
        [ i for i in float_range(parsed.tasks_util_min, parsed.tasks_util_max + 0.01, parsed.tasks_util_step) ],
        (parsed.tasks_period_min, parsed.tasks_period_max),
        parsed.tasks_period_step,
        parsed.seed,
    )

    config_gen_options = ConfigGenOptions(
        (parsed.cgroup_period_min, parsed.cgroup_period_max),
        parsed.cgroup_period_step,
        parsed.max_bw
    )

    output_options = OutputOptions(
        parsed.outdir
    )

    return (taskgen_options, config_gen_options, output_options)

def main():
    import os
    import random, math

    taskgen_options, config_gen_options, output_options = parse_args()

    if os.path.exists(output_options.out_folder):
        print(f"Out folder already exists: {output_options.out_folder}")
        exit(1)

    # generate tasksets
    tasksets = []
    print("Generating tasksets...")
    min_tasks, max_tasks = taskgen_options.num_tasks_minmax[0], taskgen_options.num_tasks_minmax[1]
    num_tasks_step = math.ceil((max_tasks - min_tasks) / taskgen_options.num_tasksets_per_utilization)

    for utilization in taskgen_options.utilizations:
        for num_tasks in range(min_tasks, max_tasks + 1, num_tasks_step):
            num_tasks = max(num_tasks, math.floor(utilization) + 1)

            task_options = TaskgenLibOptions(
                0,
                1,
                num_tasks,
                utilization,
                taskgen_options.period_minmax,
                taskgen_options.period_step,
                'logunif',
                True
            )

            tasksets += gen_tasksets(task_options)
    print("Generating configs...")
    configs = gen_configs(tasksets, config_gen_options)
    print("Saving...")
    save_configs(configs, output_options)

if __name__ == "__main__":
    main()