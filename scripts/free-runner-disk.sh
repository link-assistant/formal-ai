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

# Paths worth reclaiming, largest first. Each is optional: the runner image
# changes over time and a missing path is not an error.
DISPOSABLE_PATHS=(
  /usr/share/dotnet
  /usr/local/lib/android
  /opt/ghc
  /opt/hostedtoolcache/CodeQL
)

avail_kb() {
  df --output=avail / | tail -n 1 | tr -d ' '
}

before_kb="$(avail_kb)"
echo "Disk usage before cleanup:"
df -h /

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
done

echo "  prune  dangling and unused docker images"
sudo docker image prune --all --force >/dev/null 2>&1 || true

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
