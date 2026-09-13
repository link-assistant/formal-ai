#!/usr/bin/env bash
# Decide whether the Docker Hub publish steps should run, and say why not.
#
# Docker Hub publishing is opt-in, and the token is what opts in. A fork that
# never holds DOCKERHUB_TOKEN must still get a green release, so a missing
# token or a missing Dockerfile disables the steps quietly. A half-configured
# setup -- token held but no target to push to -- is a real misconfiguration
# and fails loudly, since silently skipping it would be a false negative: the
# release would look complete while no image was ever pushed.
#
# Issue #1131: the image used to be the switch, and nothing ever set it, so
# every release since Docker Hub publishing was added skipped it silently while
# reporting success. The image and the username now carry defaults, which makes
# them useless as a signal -- only the secret distinguishes a repository that
# can push from one that cannot.
#
# Issue #977: this ran inline and byte-identically in both `auto-release` and
# `manual-release`. Sharing one script keeps the two release paths from drifting
# and keeps release.yml under the 2000-line ceiling scripts/check-file-size.rs
# enforces.
#
# INPUT (environment)
#   DOCKERHUB_IMAGE     target repository, e.g. "owner/name"; defaulted.
#   DOCKERHUB_USERNAME  Docker Hub user; defaulted.
#   DOCKERHUB_TOKEN     Docker Hub access token; unset disables.
# OUTPUT (GITHUB_OUTPUT)
#   enabled             "true" | "false"
#   docker_hub_url      browsable URL, only when enabled.
set -euo pipefail

DOCKERHUB_IMAGE="${DOCKERHUB_IMAGE:-}"
DOCKERHUB_USERNAME="${DOCKERHUB_USERNAME:-}"
DOCKERHUB_TOKEN="${DOCKERHUB_TOKEN:-}"

disable_dockerhub() {
  echo "enabled=false" >> "$GITHUB_OUTPUT"
  echo "$1"
}

if [ -z "$DOCKERHUB_TOKEN" ]; then
  disable_dockerhub "Docker Hub publishing disabled: DOCKERHUB_TOKEN is not set"
  exit 0
fi

if [ ! -f Dockerfile ]; then
  disable_dockerhub "Docker Hub publishing disabled: Dockerfile was not found at repository root"
  exit 0
fi

if [ -z "$DOCKERHUB_IMAGE" ] || [ -z "$DOCKERHUB_USERNAME" ]; then
  echo "::error::Docker Hub publishing has a token but no target: DOCKERHUB_IMAGE or DOCKERHUB_USERNAME is empty"
  echo "Both are defaulted in release.yml; an empty one means the default was overridden with a blank value."
  exit 1
fi

echo "enabled=true" >> "$GITHUB_OUTPUT"
echo "docker_hub_url=https://hub.docker.com/r/${DOCKERHUB_IMAGE}" >> "$GITHUB_OUTPUT"
