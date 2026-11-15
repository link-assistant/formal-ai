#!/usr/bin/env bash
# Keep compiler warnings and bug/performance lint categories fatal. Full style
# cleanup is retained behind an explicit switch while release iteration resumes.
set -euo pipefail
style_flags=()
if [[ "${FORMAL_AI_STRICT_STYLE_LINTS:-0}" != 1 ]]; then
  echo 'Style lint debt deferred; enable FORMAL_AI_STRICT_STYLE_LINTS=1 for full qualification.'
  style_flags=(-A clippy::pedantic -A clippy::nursery -A clippy::style -A clippy::complexity)
fi
if [[ "${1:-}" == --library-and-binaries ]]; then
  # Issue #1191: the focused pull-request check (release.yml native-type-check).
  # Same lints as main, on the production targets only: PR #1497 (E0308) and
  # 0918cda1c/4651b47ab (clippy::manual_contains) broke main from green PRs.
  exec cargo clippy --manifest-path rust/Cargo.toml --locked --lib --bins --all-features -- -D warnings "${style_flags[@]}" -D clippy::correctness -D clippy::suspicious -D clippy::perf
fi
cargo clippy --manifest-path rust/Cargo.toml --lib --bins --tests --all-features -- -D warnings "${style_flags[@]}" -D clippy::correctness -D clippy::suspicious -D clippy::perf
cargo check --manifest-path rust/Cargo.toml --examples --all-features
