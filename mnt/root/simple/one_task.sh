#!/bin/sh

runtime=10000
period=100000
self=$$

# mount filesystem
if [ -z "$(mount | grep /sys/fs/cgroup)" ]; then
    mount -t tmpfs tmpfs /sys/fs/cgroup || return 1
    mkdir /sys/fs/cgroup/cpu || return 1
    mount -t cgroup -o cpu cpu-cgroup /sys/fs/cgroup/cpu || return 1
fi

# setup cgroup
if [ ! -d "/sys/fs/cgroup/cpu/g0" ]; then
    mkdir -p /sys/fs/cgroup/cpu/g0 || return 1
fi

old_period=$(cat /sys/fs/cgroup/cpu/g0/cpu.rt_period_us)
if [ $old_period -gt 0 ]; then
    echo 0 > /sys/fs/cgroup/cpu/g0/cpu.rt_runtime_us || return 1
fi
echo $period > /sys/fs/cgroup/cpu/g0/cpu.rt_period_us || return 1
echo $runtime > /sys/fs/cgroup/cpu/g0/cpu.rt_runtime_us || return 1

# migrate shell in cgroup and set sched_fifo
echo $self > /sys/fs/cgroup/cpu/g0/cgroup.procs || return 1
chrt -p 99 $self || return 1

# start yes
yes < /dev/zero > /dev/null 2> /dev/null &
pid=$!

# wait 5 seconds and kill it
sleep 5
kill $pid

# migrate the shell out and destroy the cgroup
chrt -o -p 0 $self || return 1
echo $self > /sys/fs/cgroup/cpu/cgroup.procs || return 1
rmdir /sys/fs/cgroup/cpu/g0