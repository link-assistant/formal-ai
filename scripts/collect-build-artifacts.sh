#!/usr/bin/env bash
# Gather the products of one `cargo test --release --no-run --bins --tests`
# into `dist/`, ready to upload as artifacts.
#
# Issue #1055: the pipeline compiled the same project in four jobs. One build
# now produces the binary *and* the three test executables, and every consumer
# downloads them. Cargo names each test target `<name>-<hash>` and keeps older
# hashes beside newer ones, so picking "the newest of each name" matters: a
# stale hash restored from cache would otherwise ship as this run's tests.
#
# Usage: collect-build-artifacts.sh [destination]   (default: dist)
set -euo pipefail

destination=${1:-dist}
mkdir -p "$destination/tests"

for target in unit integration source; do
  newest=$(find target/release/deps -maxdepth 1 -type f -perm -u+x \
    -name "$target-*" ! -name '*.d' -exec ls -t {} + | head -1)
  if [ -z "$newest" ]; then
    echo "::error title=Missing test executable::\
'cargo test --no-run' produced no executable for '$target'. Its \
'[[test]]' entry in Cargo.toml and this list have drifted apart." >&2
    exit 1
  fi
  cp "$newest" "$destination/tests/$target"
done

# Every binary, not just `formal-ai`. `src/bin/with-formal-ai.rs` is
# auto-discovered by cargo rather than declared in Cargo.toml, and
# `tests/integration/with_formal_ai.rs` spawns it through
# `CARGO_BIN_EXE_with-formal-ai` -- so collecting one binary by name left that
# test pointing at a file the artifact never carried. Copying what `--bins`
# actually produced means a new binary is picked up without editing this list.
find target/release -maxdepth 1 -type f -perm -u+x ! -name "*.d" \
  -exec cp {} "$destination/" ";"

collected=$(find "$destination/tests" -maxdepth 1 -type f | wc -l | tr -d ' ')
printf 'collected %s test executables and the release binary into %s/\n' \
  "$collected" "$destination"
