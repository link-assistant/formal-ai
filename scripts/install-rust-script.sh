#!/usr/bin/env bash
set -euo pipefail

if command -v rust-script >/dev/null 2>&1; then
  rust-script --version
  exit 0
fi

attempts="${RUST_SCRIPT_INSTALL_ATTEMPTS:-3}"
delay="${RUST_SCRIPT_INSTALL_RETRY_DELAY_SECONDS:-10}"

for attempt in $(seq 1 "$attempts"); do
  echo "Installing rust-script (attempt ${attempt}/${attempts})"
  if cargo install rust-script --locked; then
    rust-script --version
    exit 0
  fi

  status=$?
  if [ "$attempt" -eq "$attempts" ]; then
    echo "rust-script install failed after ${attempts} attempts"
    exit "$status"
  fi

  echo "rust-script install failed with status ${status}; retrying in ${delay}s"
  sleep "$delay"
done
