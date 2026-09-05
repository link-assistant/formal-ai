#!/usr/bin/env bash
# Install rust-script idempotently, reproducibly, and resiliently.
#
# - Short-circuits when rust-script is already on PATH (cached/preinstalled).
# - Uses --locked so the published Cargo.lock is honored (reproducible builds,
#   immune to broken transitive releases).
# - Retries with backoff so a transient crates.io blip does not fail the job.
set -euo pipefail

if command -v rust-script >/dev/null 2>&1; then
  echo "rust-script already present: $(rust-script --version)"
  exit 0
fi

for attempt in 1 2 3; do
  if cargo install rust-script --locked; then
    exit 0
  fi
  echo "cargo install rust-script failed (attempt $attempt/3); retrying..." >&2
  sleep $((attempt * 5))
done

echo "cargo install rust-script failed after 3 attempts" >&2
exit 1
