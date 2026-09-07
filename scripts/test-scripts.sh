#!/usr/bin/env bash
# Run the inline `#[cfg(test)]` suites of the standalone scripts nothing else runs.
#
# Issue #1081. `cargo test` builds this repository's crates; it does not build
# `scripts/*.rs`, which are rust-script programs with their own `[dependencies]`
# manifests. Two mechanisms already made a script's inline suite part of CI:
#
#   1. `#[path = "../../scripts/<name>.rs"] mod ...` in `tests/`, which compiles
#      the script into the unit test crate. Free, but it forces every dependency
#      the script declares into this repository's `Cargo.toml`.
#   2. A gate in `data/meta/ci-gates/` whose `run` is
#      `rust-script --test scripts/<name>.rs`, which builds the script with its
#      own manifest. Costs a compile, and is the only option for a script whose
#      dependencies this repository does not otherwise carry -- `ureq`, in
#      `wait-for-crate.rs`.
#
# Four scripts had neither: `check-hardcoded-language.rs`,
# `check-wasm-worker-size.rs`, `publish-crate.rs` and `wait-for-crate.rs`
# carried 23 passing test cases that no CI job executed. A test suite nobody
# runs is a false negative with a green tick on it -- the exact failure mode
# issue #1081 is about -- and the repository could not tell, because the gate
# registry does run three of them, *without* `--test`, which runs the check and
# not its tests.
#
# The selection is derived, never listed: every `scripts/*.rs` with a
# `cfg(test)` suite that neither of the two mechanisms above already covers. A
# script added tomorrow is picked up without anyone remembering this file
# exists, which is the property a hand-maintained list cannot have.
#
# Adapted from `scripts/test-scripts.sh` in
# link-foundation/rust-ai-driven-development-pipeline-template, which runs the
# same sweep over its whole `scripts/` directory.
#
# `--list` prints the selection and runs nothing, so a test can assert that the
# selection is what it claims to be.
set -uo pipefail

cd "$(dirname "$0")/.." || exit 1

# Match how CI compiles the rest of the tree, so a warning introduced by the
# test harness surfaces here rather than in a later job.
export RUSTFLAGS="${RUSTFLAGS:--Dwarnings}"

# Scripts the unit test crate already compiles. `cargo test` runs their suites,
# so running them again through rust-script would only buy a second compile.
already_compiled=$(
  grep -rhoE '#\[path = "[^"]*scripts/[^"]+\.rs"\]' tests --include='*.rs' 2>/dev/null |
    sed -e 's|.*scripts/||' -e 's|"\]$||' |
    sort -u
)

# Scripts a registered gate already runs with `--test`. Those suites are part
# of the `lint` job's verdict and do not need a second home here.
already_gated=$(
  grep -rhoE 'rust-script --test scripts/[^" ]+\.rs' data/meta/ci-gates .github/workflows 2>/dev/null |
    sed -e 's|.*scripts/||' |
    sort -u
)

selected=()
for script in scripts/*.rs; do
  grep -q 'cfg(test)' "$script" || continue
  name=${script#scripts/}
  if printf '%s\n' "$already_compiled" | grep -qxF "$name"; then
    continue
  fi
  if printf '%s\n' "$already_gated" | grep -qxF "$name"; then
    continue
  fi
  selected+=("$script")
done

if [ "${1:-}" = "--list" ]; then
  printf '%s\n' "${selected[@]}"
  exit 0
fi

if [ ${#selected[@]} -eq 0 ]; then
  echo "No standalone script test suites to run." >&2
  exit 1
fi

status=0
failed=()

for script in "${selected[@]}"; do
  echo "::group::rust-script --test $script"
  if ! rust-script --test "$script"; then
    status=1
    failed+=("$script")
  fi
  echo "::endgroup::"
done

if [ "$status" -ne 0 ]; then
  echo "Failed script test suites:" >&2
  printf '  %s\n' "${failed[@]}" >&2
fi

exit "$status"
