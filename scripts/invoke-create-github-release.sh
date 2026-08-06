#!/usr/bin/env bash
# Build the argument list for scripts/create-github-release.rs and run it.
#
# Issue #977: `auto-release` and `manual-release` assembled this argv inline and
# almost identically, which is how they drifted -- and this is the step that
# eleven releases (0.326.2 .. 0.333.0) never reached, because the job was killed
# by `timeout-minutes` during the uncached Docker build that precedes it. One
# shared script keeps the two release paths honest.
#
# INPUT (environment)
#   RELEASE_VERSION  version to release; required.
#   GHCR_URL         GHCR package page.
#   DOCKER_HUB_URL   Docker Hub page; empty when Docker Hub publishing is off.
#   GH_TOKEN         token for the GitHub API.
set -euo pipefail

: "${RELEASE_VERSION:?RELEASE_VERSION is required}"

release_args=(
  --release-version "$RELEASE_VERSION"
  --repository "${GITHUB_REPOSITORY}"
  --self-hosting-ledger data/meta/self-hosting-ledger.lino
  --ghcr-url "${GHCR_URL:-}"
)

if [ -n "${DOCKER_HUB_URL:-}" ]; then
  release_args+=(--docker-hub-url "$DOCKER_HUB_URL")
fi

rust-script scripts/create-github-release.rs "${release_args[@]}"
