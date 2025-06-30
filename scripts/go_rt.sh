setup_hyperthreading() {
  for C in ${NO_HT_CPUS}
   do
    echo 0 > /sys/devices/system/cpu/cpu${C}/online
   done
}

disable_hyperthreading() {
  echo off > /sys/devices/system/cpu/smt/control
}

fix_frequency_intel_pstate() {
  echo 100 > /sys/devices/system/cpu/intel_pstate/max_perf_pct 
  echo 100 > /sys/devices/system/cpu/intel_pstate/min_perf_pct
  echo   1 > /sys/devices/system/cpu/intel_pstate/no_turbo
  echo 100 > /sys/devices/system/cpu/intel_pstate/min_perf_pct
}

setup_amd_pstate() {
  echo passive > /sys/devices/system/cpu/amd_pstate/status
  echo       0 > /sys/devices/system/cpu/cpufreq/boost
}

fix_frequency_governor() {
  for C in $CPUDIRS
   do	  
#    echo 1800000 > ${C}/cpufreq/scaling_max_freq
#    echo userspace > ${C}/cpufreq/scaling_governor
#    echo powersave > ${C}/cpufreq/scaling_governor
    echo performance > ${C}/cpufreq/scaling_governor
    echo 0         > ${C}/cpufreq/cpb
    echo 1800000 > ${C}/cpufreq/scaling_setspeed
    echo 3000000 > ${C}/cpufreq/scaling_setspeed
   done
}

disable_deep_idle_states() {
  for C in $CPUDIRS
   do
    LAST_STATE=$(ls ${C}/cpuidle | tail -n 1 | cut -d 'e' -f 2)
    STATES=$(seq ${MIN_STATE} ${LAST_STATE})
    for S in ${STATES}
     do
      echo 1 > ${C}/cpuidle/state${S}/disable
     done
   done
}

MIN_STATE=${MIN_STATE:-0}
CPUDIRS=""
for C in $RT_CPUS
 do
  CPUDIRS="${CPUDIRS} /sys/devices/system/cpu/cpu${C}"
 done

#if [ x${CPUDIRS} = x ]
if [ "x${CPUDIRS}" = "x" ]
 then
  CPUDIRS=$(ls -d /sys/devices/system/cpu/cpu[0-9]*)
 fi
 
if [ x${NOHT} = x ]
 then
  disable_hyperthreading
 fi

if [ x${NOF} = x ]
 then
  if test -f /sys/devices/system/cpu/intel_pstate/max_perf_pct
   then
    fix_frequency_intel_pstate
   else
    if test -f /sys/devices/system/cpu/amd_pstate/status
     then
      setup_amd_pstate
     fi
    fix_frequency_governor
   fi
 fi

# MISSING:
# 	echo -1 > /proc/sys/kernel/sched_rt_runtime_us
# 	echo 0 > /sys/devices/system/cpu/cpufreq/boost
echo You might want to do:
echo	'echo -1 > /proc/sys/kernel/sched_rt_runtime_us'
echo 	'echo 0 > /sys/devices/system/cpu/cpufreq/boost'

#setup_hyperthreading
disable_deep_idle_states
