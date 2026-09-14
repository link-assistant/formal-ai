#!/usr/bin/env bash
set -euo pipefail

NEXTEST_BIN=${1:?usage: run.sh /path/to/cargo-nextest}
issue_tmp=$(mktemp -d)
trap 'rm -rf "$issue_tmp"' EXIT

project="$issue_tmp/project"
mkdir -p "$project/src" "$project/tests"
printf '%s\n' \
  '[package]' \
  'name = "archive-path-probe"' \
  'version = "0.1.0"' \
  'edition = "2021"' \
  > "$project/Cargo.toml"
printf '%s\n' 'fn main() { println!("archive-path-probe"); }' > "$project/src/main.rs"
printf '%s\n' \
  'use std::process::Command;' \
  '#[test]' \
  'fn launches_compile_time_binary_path() {' \
  '    let output = Command::new(env!("CARGO_BIN_EXE_archive-path-probe"))' \
  '        .output().expect("launch archived binary");' \
  '    assert!(output.status.success());' \
  '}' \
  > "$project/tests/archive.rs"

archive="$issue_tmp/tests.tar.zst"
(cd "$project" && "$NEXTEST_BIN" nextest archive --archive-file "$archive")
rm -rf "$project/target"

default_status=0
(cd "$project" && "$NEXTEST_BIN" nextest run --archive-file "$archive") \
  > "$issue_tmp/default.log" 2>&1 || default_status=$?
if [[ "$default_status" -eq 0 ]]; then
  printf '%s\n' 'expected default temporary extraction to fail' >&2
  exit 1
fi

(cd "$project" && "$NEXTEST_BIN" nextest run \
  --archive-file "$archive" \
  --extract-to "$project" \
  --workspace-remap "$project") \
  > "$issue_tmp/workspace.log" 2>&1

printf 'default extraction status: %s\n' "$default_status"
printf '%s\n' 'workspace extraction: passed'
