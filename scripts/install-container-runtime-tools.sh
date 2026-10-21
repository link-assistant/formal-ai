#!/usr/bin/env bash
# Shared image install; preserve Bun's default lifecycle-script trust policy.
set -euo pipefail
runtime_home="${1:?runtime tools project directory is required}"
[[ "$runtime_home" == /* && -f "$runtime_home/package.json" ]] || {
  echo "an absolute runtime tools project with package.json is required" >&2
  exit 2
}
script_dir="$(CDPATH= cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
# One bounded attempt, no dependency-resolution retries or ignored failures.
bash "$script_dir/run-with-deadline.sh" 600 \
  bun install --production --cwd "$runtime_home"
node "$script_dir/verify-container-runtime-tools.mjs" "$runtime_home"
