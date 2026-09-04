#!/usr/bin/env bash
# Reclaim Docker artifacts left by interrupted local and CI harness runs.
#
# The routine is intentionally best-effort: Docker may be absent, stopped, or
# owned by another user, and none of those conditions may block a commit. The
# ordinary pass removes only stopped containers and dangling images. A broader
# `docker system prune` is reserved for hosts above the configured ceiling and
# still never removes volumes or running containers.

set -u

if [[ "${DOCKER_NO_PRUNE:-}" =~ ^(1|true|yes|on)$ ]]; then
  echo "prune-docker: skipped (DOCKER_NO_PRUNE)"
  exit 0
fi

if ! command -v docker >/dev/null 2>&1; then
  echo "prune-docker: docker is unavailable, nothing to prune"
  exit 0
fi

if ! docker info >/dev/null 2>&1; then
  echo "prune-docker: daemon is unavailable, nothing to prune"
  exit 0
fi

echo "prune-docker: containers before cleanup"
docker ps -a || true
echo "prune-docker: dangling images before cleanup"
docker images -f dangling=true || true

# A stopped container cannot be serving traffic, but its writable layer still
# consumes disk. Dangling images are untagged layers left by replaced builds.
docker container prune --force || true
docker image prune --force || true

docker_size_bytes() {
  docker system df --format '{{.Size}}' 2>/dev/null | awk '
    {
      value = toupper($0)
      gsub(/[[:space:]]/, "", value)
      number = value
      sub(/[^0-9.].*$/, "", number)
      unit = value
      sub(/^[0-9.]+/, "", unit)
      multiplier = 1
      if (unit == "KB") multiplier = 1000
      else if (unit == "MB") multiplier = 1000000
      else if (unit == "GB") multiplier = 1000000000
      else if (unit == "TB") multiplier = 1000000000000
      if (number ~ /^[0-9]+([.][0-9]+)?$/) total += number * multiplier
    }
    END { printf "%.0f\n", total }
  '
}

max_size_gb=${DOCKER_MAX_SIZE_GB:-20}
if [[ "$max_size_gb" =~ ^[0-9]+([.][0-9]+)?$ ]]; then
  current_bytes=$(docker_size_bytes)
  max_bytes=$(awk -v gigabytes="$max_size_gb" 'BEGIN { printf "%.0f", gigabytes * 1000000000 }')
  if [[ "$current_bytes" =~ ^[0-9]+$ ]] && (( current_bytes > max_bytes )); then
    echo "prune-docker: Docker uses ${current_bytes} bytes, above ${max_size_gb}GB ceiling"
    docker system prune --force || true
  else
    echo "prune-docker: Docker is within ${max_size_gb}GB ceiling (${current_bytes:-unknown} bytes)"
  fi
else
  echo "prune-docker: invalid DOCKER_MAX_SIZE_GB=$max_size_gb; ceiling check skipped" >&2
fi

echo "prune-docker: containers after cleanup"
docker ps -a || true
echo "prune-docker: dangling images after cleanup"
docker images -f dangling=true || true

exit 0
