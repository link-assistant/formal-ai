#!/usr/bin/env bash
# Decide whether the Docker Hub publish steps should run, and say why not.
#
# Docker Hub publishing is opt-in: a fork that never sets DOCKERHUB_IMAGE must
# still get a green release, so a missing repository variable or a missing
# Dockerfile disables the steps quietly. A half-configured setup -- image set
# but credentials missing -- is a real misconfiguration and fails loudly, since
# silently skipping it would be a false negative: the release would look
# complete while no image was ever pushed.
#
# Issue #977: this ran inline and byte-identically in both `auto-release` and
# `manual-release`. Sharing one script keeps the two release paths from drifting
# and keeps release.yml under the 2000-line ceiling scripts/check-file-size.rs
# enforces.
#
# INPUT (environment)
#   DOCKERHUB_IMAGE     target repository, e.g. "owner/name"; unset disables.
#   DOCKERHUB_USERNAME  Docker Hub user.
#   DOCKERHUB_TOKEN     Docker Hub access token.
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

if [ -z "$DOCKERHUB_IMAGE" ]; then
  disable_dockerhub "Docker Hub publishing disabled: DOCKERHUB_IMAGE repository variable is not set"
  exit 0
fi

if [ ! -f Dockerfile ]; then
  disable_dockerhub "Docker Hub publishing disabled: Dockerfile was not found at repository root"
  exit 0
fi

if [ -z "$DOCKERHUB_USERNAME" ] || [ -z "$DOCKERHUB_TOKEN" ]; then
  echo "::error::Docker Hub publishing requires DOCKERHUB_USERNAME and DOCKERHUB_TOKEN"
  echo "Set DOCKERHUB_USERNAME as a repository variable or secret, and DOCKERHUB_TOKEN as a secret."
  exit 1
fi

echo "enabled=true" >> "$GITHUB_OUTPUT"
echo "docker_hub_url=https://hub.docker.com/r/${DOCKERHUB_IMAGE}" >> "$GITHUB_OUTPUT"
