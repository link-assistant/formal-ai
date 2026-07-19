#!/usr/bin/env bash
set -euo pipefail

version="${1:?usage: smoke-test-published-crate.sh VERSION}"
install_root="$(mktemp -d)"
trap 'rm -rf "$install_root"' EXIT

# Install from the registry, never from the checkout, so this proves the exact
# artifact users receive can compile and start. `wait-for-crate.rs` runs first
# and handles registry-index propagation.
cargo install formal-ai \
  --version "=${version}" \
  --locked \
  --root "$install_root" \
  --bin formal-ai
"$install_root/bin/formal-ai" --help >/dev/null

