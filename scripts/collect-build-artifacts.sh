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
# Issue #1569: Rust 1.100 moved test executables from
# `target/release/deps/<name>-<hash>` to the build-dir layout
# `target/release/build/<package>/<hash>/out/<name>-<hash>`. The executable of
# each target is now taken from Cargo's own report -- the `--message-format=json`
# stream the build step writes -- so its place on disk is no longer guessed.
# Without that report (a local run), both layouts are searched for the newest.
#
# Usage: collect-build-artifacts.sh [destination] [cargo-messages.jsonl]
#   destination          default: dist
#   cargo-messages.jsonl default: $RUNNER_TEMP/native-build-messages.jsonl when it exists
# FORMAL_AI_COLLECT_DEBUG=1 prints how each executable was chosen.
set -euo pipefail

destination=${1:-dist}
messages=${2:-${RUNNER_TEMP:+$RUNNER_TEMP/native-build-messages.jsonl}}
mkdir -p "$destination/tests"

debug() {
  if [ "${FORMAL_AI_COLLECT_DEBUG:-0}" = 1 ]; then echo "collect-build-artifacts: $*" >&2; fi
}

# The executable Cargo reported for the test target "$1", or nothing.
reported_executable() {
  [ -n "$messages" ] && [ -f "$messages" ] || return 0
  node -e '
    const [file, target] = process.argv.slice(1);
    const found = require("node:fs").readFileSync(file, "utf8").split(/\r?\n/u)
      .filter(line => line.startsWith("{")).map(line => JSON.parse(line))
      .filter(row => row.reason === "compiler-artifact" && row.target?.name === target
        && row.target.kind?.length === 1 && row.target.kind[0] === "test"
        && row.profile?.test === true && typeof row.executable === "string")
      .map(row => row.executable);
    if (found.length > 1) { console.error("ambiguous Cargo report for " + target); process.exit(1); }
    if (found.length === 1) console.log(found[0]);
  ' "$messages" "$1"
}

# The newest "$1-<hash>" executable in either Cargo layout, or nothing.
newest_executable() {
  local directories=()
  for directory in rust/target/release/deps rust/target/release/build; do
    if [ -d "$directory" ]; then directories+=("$directory"); fi
  done
  [ "${#directories[@]}" -gt 0 ] || return 0
  find "${directories[@]}" -maxdepth 4 -type f -perm -u+x -name "$1-*" ! -name '*.d' \
    \( -path 'rust/target/release/deps/*' -o -path 'rust/target/release/build/*/*/out/*' \) \
    -exec ls -t {} + | head -1
}

for target in unit integration source; do
  chosen=$(reported_executable "$target")
  if [ -n "$chosen" ]; then
    debug "$target: Cargo reported $chosen"
  else
    chosen=$(newest_executable "$target")
    debug "$target: no Cargo report in '${messages:-}', newest on disk is '${chosen}'"
  fi
  if [ -z "$chosen" ] || [ ! -f "$chosen" ]; then
    echo "::error title=Missing test executable::\
'cargo test --no-run' produced no executable for '$target'. Its \
'[[test]]' entry in Cargo.toml and this list have drifted apart." >&2
    exit 1
  fi
  cp "$chosen" "$destination/tests/$target"
done

# Every binary, not just `formal-ai`. `src/bin/with-formal-ai.rs` is
# auto-discovered by cargo rather than declared in Cargo.toml, and
# `tests/integration/with_formal_ai.rs` spawns it through
# `CARGO_BIN_EXE_with-formal-ai` -- so collecting one binary by name left that
# test pointing at a file the artifact never carried. Copying what `--bins`
# actually produced means a new binary is picked up without editing this list.
find rust/target/release -maxdepth 1 -type f -perm -u+x ! -name "*.d" \
  -exec cp {} "$destination/" ";"

collected=$(find "$destination/tests" -maxdepth 1 -type f | wc -l | tr -d ' ')
printf 'collected %s test executables and the release binary into %s/\n' \
  "$collected" "$destination"
