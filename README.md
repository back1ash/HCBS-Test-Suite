# HCBS Rust Initrd

Rust-based tests for Hierarchical Constant Bandwidth Server (available at https://github.com/Yurand2000/HCBS-patch)

## 🚀 Quick Start

### Prerequisites

- **Make**
- **Rust** nightly
- **Python3**

**Extra Requirements to build an INITRAMFS.** Useful for QEMU users.
- **GCC**
- **Git**

### Installation

The building process is based on Make, and will produce a compressed archive with all the tests.

### Basic Usage

The tests can be manually run one by one, useful for development and debugging, or may be run using the dedicated `run_tests.sh` script.

The following concerns the `run_tests.sh` script:

```sh
# Step 1
# Get a running kernel with the patchset applied.
# Make sure that you have a kernel with cgroups-v2 enabled.
# The script will setup the real-time cgorup system in a default way,
# make sure you don't have any SCHED_DEADLINE task active. Also, the
# script will migrate all SCHED_FIFO/SCHED_RR tasks in the root cgroup.

# Step 2
# Run all standard tests
> sh run_tests.sh all

# OR Run all standard tests + randomly generated tests
> sh run_tests.sh full

# ---------------- #
# Get available test suites
> sh run_tests.sh help

# Run a single test suite
> sh run_tests.sh random-stress
```

### Advanced Usage

Tests executables can be run manually. They can be found at `test_suite_v2`. Just run any executable without argument to get the help screen. Take a look at section Available Tests for more information.

```bash
# Examples of manually run tests.
> ./test_suite_v2/regression fair-server -t 10

> ./test_suite_v2/stress migrate -r 10 -p 100 -P 0.1 -t 60

# There is also useful tools
> ./test_suite_v2/tools -h
Usage: tools <COMMAND>

Commands:
  hog              CPU hog
  mount-cgroup-fs  Mount CGroup filesystem and CPU controller
  mount-debug-fs   Mount DebugFS
  rt-bw-change     Change the badwidth allocated to real-time tasks
                   (both FIFO/RR and DEADLINE)
  move-to-root     Move all real-time tasks to the root control group
  cgroup-setup     Change the runtime and period to the given control group
  hrtick           Enable/Disable the HRTICK_DL scheduler feature
[...]
```

## 🛠️ Available Tests

### 1. Constraints
### 2. Regression
### 3. Stress
### 4. Time
### 5. Taskset
### Extra Tools

## 📄 License