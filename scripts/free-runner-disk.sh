#!/usr/bin/env bash
#
# Reclaim disk on a hosted GitHub runner before a heavy build.
#
# The hosted ubuntu image ships with ~14 GB free on `/`, and pre-installed SDKs
# we never touch (dotnet, Android, GHC, CodeQL) eat most of it. Jobs that build
# the crate and then build the Docker image (which compiles the crate a second
# time inside a large base image) exhaust `/` and take the whole runner down
# with "No space left on device" — the runner cannot even flush its own diag
# log, so the job dies with no failed step and no log to download.
#
# Seen in issue #523 (Pages job) and again in issue #736 (release jobs, runs
# 29312084458 and 29485000765).
#
# Set RUNNER_DISK_DEBUG=1 for a per-directory breakdown of what was reclaimed.
set -euo pipefail

if [ "${GITHUB_ACTIONS:-}" != true ]; then
  echo "free-runner-disk: hosted Actions runner required" >&2
  exit 2
fi
if [ "${RUNNER_OS:-}" != Linux ]; then
  echo "free-runner-disk: Linux SDK cleanup skipped on ${RUNNER_OS:-unknown}"
  exit 0
fi
target_mib="${RUNNER_DISK_TARGET_MIB:-}"
if [ -n "$target_mib" ]; then
  if ! [[ "$target_mib" =~ ^[0-9]{1,7}$ ]] || (( 10#$target_mib < 1024 || 10#$target_mib > 1048576 )); then
    echo "free-runner-disk: RUNNER_DISK_TARGET_MIB must be 1024..1048576" >&2
    exit 2
  fi
  target_mib=$((10#$target_mib))
fi

# Paths worth reclaiming, largest first. Each is optional: the runner image
# changes over time and a missing path is not an error.
DISPOSABLE_PATHS=(
  /usr/share/dotnet
  /usr/local/lib/android
  /opt/ghc
  /opt/hostedtoolcache/CodeQL
)

avail_kb() {
  # POSIX output keeps the available-kilobytes column stable on GNU and BSD df.
  df -Pk / | awk 'NR == 2 { print $4 }'
}

# Without an explicit reserve, heavy build jobs retain the original full cleanup.
enough_disk() {
  [ -n "$target_mib" ] && [ "$(avail_kb)" -ge "$((target_mib * 1024))" ]
}

before_kb="$(avail_kb)"
echo "Disk usage before cleanup:"
df -h /
if enough_disk; then
  echo "Already enough runner disk: $((before_kb / 1024)) MiB available; target $target_mib MiB. Cleanup skipped."
  exit 0
fi

for path in "${DISPOSABLE_PATHS[@]}"; do
  if [ ! -e "$path" ]; then
    echo "  skip   ${path} (not present)"
    continue
  fi
  if [ "${RUNNER_DISK_DEBUG:-0}" = "1" ]; then
    echo "  remove ${path} ($(sudo du -sh "$path" 2>/dev/null | cut -f1 || echo '?'))"
  else
    echo "  remove ${path}"
  fi
  # Best-effort: a path we cannot remove is not worth failing the release over.
  sudo rm -rf "$path" || echo "  WARNING: could not remove ${path}"
  if enough_disk; then
    break
  fi
done

if ! enough_disk; then
  echo "  prune  dangling and unused docker images"
  sudo docker image prune --all --force >/dev/null 2>&1 || true
fi

after_kb="$(avail_kb)"
echo "Disk usage after cleanup:"
df -h /
echo "Reclaimed $(( (after_kb - before_kb) / 1024 )) MiB; $(( after_kb / 1024 )) MiB now available on /"

# Surface the "we are still nearly full" case loudly. This is a warning rather
# than a failure: the build may still fit, and a hard threshold here would be a
# guess. If a job dies with no logs, check for this annotation first.
min_mib="${RUNNER_DISK_MIN_MIB:-6144}"
if [ "$(( after_kb / 1024 ))" -lt "$min_mib" ]; then
  echo "::warning title=Low runner disk::Only $(( after_kb / 1024 )) MiB free on / after cleanup (want >= ${min_mib} MiB). A heavy Docker build may crash the runner with 'No space left on device'."
fi
