#!/usr/bin/env bash
# Put the `formal-ai` binary on the runner, taken from the published container.
#
# Issue #1085 (D2.3). A repository installing this action has no formal-ai
# sources to build, and should not spend a compile on them: the release
# container already carries the binary. It is copied out and run directly on
# the runner, so the agent's workspace and the server share a filesystem the
# way they do in a source build.
set -euo pipefail

image="${IMAGE:?an image is required}"
docker pull --quiet "$image"
container=$(docker create "$image")
trap 'docker rm --force "$container" >/dev/null 2>&1 || true' EXIT

mkdir -p target/release
docker cp "$container:/usr/local/bin/formal-ai" target/release/formal-ai
chmod +x target/release/formal-ai

printf 'formal-ai from %s: %s\n' "$image" "$(target/release/formal-ai --version)"
