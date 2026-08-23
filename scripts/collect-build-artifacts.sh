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

cp target/release/formal-ai "$destination/formal-ai"

collected=$(find "$destination/tests" -maxdepth 1 -type f | wc -l | tr -d ' ')
printf 'collected %s test executables and the release binary into %s/\n' \
  "$collected" "$destination"
