import sys
from typing import Any
import numpy

class OutputOptions:
    def __init__(self, out_folder: str):
        self.out_folder = out_folder

class ConfigGenOptions:
    def __init__(self, period_minmax: tuple[int, int], period_step: int | None):
        self.permin, self.permax = period_minmax

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
    def __init__(self, num_tasksets: int, num_tasks: int, utilization: float, period_minmax: tuple[int, int], period_step: int | None = None, distribution: str = 'logunif', round: bool = True):
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
    def __init__(self, tasks: list[Task]):
        self.tasks = tasks

    def __repr__(self):
        return f"<Taskset {self.tasks}>"

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
            taskset += [ Task(taskset_data[t][3], taskset_data[t][2], taskset_data[t][0]) ]

        tasksets += [ Taskset(taskset) ]

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
        if config is not None:
            out[taskset_id][1].append(config)

    return out

def carts_analysis(id: int, taskset: Taskset, period: int) -> tuple[int, Config | None]:
    from tempfile import NamedTemporaryFile
    import subprocess

    def generate_xml_file(taskset: Taskset, period: int) -> str:
        header = \
f'''<system os_scheduler="DM" min_period="0" max_period="0">
  <component name="Test" scheduler="DM" min_period="{period}" max_period="{period}">
'''

        footer = \
'''
  </component>
</system>'''

        tasks = []
        multiplier = 10
        for i, task in enumerate(taskset.tasks):
            tasks += [ f"    <task name=\"T{i}\" p=\"{task.period * multiplier}\" d=\"{task.deadline * multiplier}\" e=\"{task.runtime * multiplier}\"></task>" ]

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

                period = int(line[2].split('"')[1])
                runtime = ceil(float(line[3].split('"')[1]) / cpus)
                return Config(cpus, runtime, period)

        return None

    carts_bin = "../build/CARTS/carts-source/bin"

    out_data = ""
    with NamedTemporaryFile(delete_on_close=False) as input, NamedTemporaryFile(delete_on_close=False) as output:
        input.write(generate_xml_file(taskset, period).encode())
        input.close()
        output.close()

        subprocess.run(["java", "-cp", carts_bin, "Carts", input.name, "MPR2", output.name, "MARKO_SCHEDTEST"], shell=False, capture_output=True)

        with open(output.name, mode='r') as f:
            out_data = f.read()

    return (id, parse_output_file(out_data))

def save_configs(tasksets: list[tuple[Taskset, list[Config]]], options: OutputOptions):
    import os

    for i, (taskset, configs) in enumerate(tasksets):
        name = f"taskset{i:03d}"
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

def main():
    import os
    import random, math

    taskgen_options = TaskgenOptions(
        3,
        (2,16),
        [ i for i in float_range(0.2, 6.01, 0.2) ],
        (100, 500),
        10,
        42
    )

    config_gen_options = ConfigGenOptions(
        (100, 500),
        200
    )

    output_options = OutputOptions(
        "../build/tasksets/root/tasksets"
    )

    if os.path.exists(output_options.out_folder):
        print(f"Out folder already exists: {output_options.out_folder}")
        exit(1)

    # generate tasksets
    tasksets = []
    rand = random.Random(taskgen_options.seed)
    for utilization in taskgen_options.utilizations:
        for _ in range(taskgen_options.num_tasksets_per_utilization):
            min_tasks = max(taskgen_options.num_tasks_minmax[0], math.floor(utilization) + 1)
            max_tasks = taskgen_options.num_tasks_minmax[1]

            task_options = TaskgenLibOptions(
                1,
                rand.randrange(min_tasks, max_tasks + 1),
                utilization,
                taskgen_options.period_minmax,
                taskgen_options.period_step,
                'logunif',
                True
            )

            tasksets += gen_tasksets(task_options)
    configs = gen_configs(tasksets, config_gen_options)
    save_configs(configs, output_options)

if __name__ == "__main__":
    main()