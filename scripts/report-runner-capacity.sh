#!/usr/bin/env bash
# Report the capacity of the host a CI job is running on.
#
# Issue #1076: `Coverage / Code Coverage` ran the *same 358 integration tests*
# in 1572.8s that the previous day's run finished in 213.6s, on an identical
# runner image, and was killed by `timeout-minutes`. The slowdown was global
# (every one of the 18 heaviest test modules, 2.3x-21.6x) and progressive
# (2.2x in the first decile of the run, 14.5x in the last), and it hit tests
# that are pure in-process CPU work with no subprocess, no file I/O and no
# network. That is the fingerprint of a host problem -- CPU steal from a noisy
# neighbour, memory pressure, or a filling disk -- but none of those could be
# confirmed, because no job in this repository has ever recorded them: grepping
# all five collected coverage logs for `no space left`, `Cannot allocate`,
# `out of memory` and `oom-kill` returns nothing at all.
#
# This script closes that gap. It samples what the runner had available so the
# next slow run can be attributed instead of guessed at.
#
# It is **off by default**. Without `FORMAL_AI_CI_VERBOSE=true` it prints
# nothing and exits 0, so it is safe to call unconditionally from a workflow
# and never adds noise to an ordinary green run.
#
# Usage:
#   report-runner-capacity.sh <label>              one snapshot
#   report-runner-capacity.sh --watch <seconds> <label>
#                                                  snapshot every <seconds>
#                                                  until terminated
#
# Environment:
#   FORMAL_AI_CI_VERBOSE   `true` enables all output (default: false)
set -uo pipefail

[ "${FORMAL_AI_CI_VERBOSE:-false}" = "true" ] || exit 0

watch_seconds=0
if [ "${1:-}" = "--watch" ]; then
  watch_seconds="${2:?--watch needs an interval in seconds}"
  shift 2
fi
label="${1:-runner capacity}"

# Total and idle jiffies across all CPUs, plus steal -- the time the hypervisor
# gave to some other tenant. Steal is the number that distinguishes "our tests
# got slower" from "we were given less of the machine".
cpu_jiffies() {
  awk '/^cpu /{
    total = 0
    for (i = 2; i <= NF; i++) { total += $i }
    # user nice system idle iowait irq softirq steal
    print total, $5, $6, $9
    exit
  }' /proc/stat
}

sample() {
  local when
  when="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

  local before after
  before="$(cpu_jiffies)"
  sleep 1
  after="$(cpu_jiffies)"

  local busy_percent idle_percent iowait_percent steal_percent
  read -r busy_percent idle_percent iowait_percent steal_percent <<<"$(
    awk -v b="${before}" -v a="${after}" 'BEGIN {
      split(b, x, " "); split(a, y, " ")
      d = y[1] - x[1]
      if (d <= 0) { print "0.0 0.0 0.0 0.0"; exit }
      idle = (y[2] - x[2]) * 100 / d
      iow  = (y[3] - x[3]) * 100 / d
      steal = (y[4] - x[4]) * 100 / d
      printf "%.1f %.1f %.1f %.1f", 100 - idle, idle, iow, steal
    }'
  )"

  local cpus load mem_total_kb mem_available_kb disk
  cpus="$(nproc 2>/dev/null || echo '?')"
  load="$(cut -d' ' -f1-3 /proc/loadavg 2>/dev/null || echo '?')"
  mem_total_kb="$(awk '/^MemTotal:/{print $2}' /proc/meminfo 2>/dev/null || echo 0)"
  mem_available_kb="$(awk '/^MemAvailable:/{print $2}' /proc/meminfo 2>/dev/null || echo 0)"
  disk="$(df -h / 2>/dev/null | awk 'NR==2 {printf "%s free of %s (%s used)", $4, $2, $5}')"

  printf '::notice title=Runner capacity::%s @ %s | cpus=%s load=%s | cpu busy=%s%% idle=%s%% iowait=%s%% steal=%s%% | mem avail=%sMiB of %sMiB | disk / %s\n' \
    "${label}" "${when}" "${cpus}" "${load}" \
    "${busy_percent}" "${idle_percent}" "${iowait_percent}" "${steal_percent}" \
    "$((mem_available_kb / 1024))" "$((mem_total_kb / 1024))" "${disk}"

  # Steal above a few percent means the runner is sharing its cores with
  # another tenant, which is the one explanation a test-level fix cannot
  # address. Call it out so it is not lost among the notices.
  awk -v s="${steal_percent}" 'BEGIN { exit !(s >= 5.0) }' && printf \
    '::warning title=Runner CPU steal::%s: %s%% of CPU time went to another tenant on this host; wall-clock measurements from this run are not comparable to a quiet runner (issue #1076)\n' \
    "${label}" "${steal_percent}"
}

if [ "${watch_seconds}" -gt 0 ] 2>/dev/null; then
  while true; do
    sample
    sleep "${watch_seconds}"
  done
else
  sample
fi
