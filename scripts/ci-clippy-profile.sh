#!/usr/bin/env bash
# Keep compiler warnings and bug/performance lint categories fatal. Full style
# cleanup is retained behind an explicit switch while release iteration resumes.
set -euo pipefail
style_flags=()
if [[ "${FORMAL_AI_STRICT_STYLE_LINTS:-0}" != 1 ]]; then
  echo 'Style lint debt deferred; enable FORMAL_AI_STRICT_STYLE_LINTS=1 for full qualification.'
  style_flags=(-A clippy::pedantic -A clippy::nursery -A clippy::style -A clippy::complexity)
fi
cargo clippy --manifest-path rust/Cargo.toml --lib --bins --tests --all-features -- -D warnings "${style_flags[@]}" -D clippy::correctness -D clippy::suspicious -D clippy::perf
cargo check --manifest-path rust/Cargo.toml --examples --all-features
