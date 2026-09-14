#!/usr/bin/env bash
# Prune `target/` down to the artifacts of the most recent build.
#
# Cargo never removes anything: every branch, every dependency version and every
# incremental session leaves artifacts behind, and this repository's debug tree
# reaches several gigabytes within a few days of ordinary work. On CI the same
# growth is saved into the actions cache and restored on the next run, so the
# cache slowly fills with builds nobody will use again.
#
# Two pruners live here, and the difference matters.
#
#   cargo-sweep   asks cargo which artifacts the current build actually
#                 references and removes the rest. It reads `.fingerprint`
#                 metadata, so an artifact the latest build still depends on
#                 survives even when it was compiled weeks ago and never
#                 recompiled since. This is the one that leaves a cache the
#                 *next* build can still link against.
#   the fallback  compares modification times against a marker. It cannot tell a
#                 stale artifact from a current one that simply did not need
#                 rebuilding, so it deletes dependencies the next build then has
#                 to compile again. Used only when cargo-sweep is absent.
#
# Install the good one with `cargo install cargo-sweep`; see CONTRIBUTING.md.
#
# "Most recent build" means artifacts newer than a reference point. Callers that
# know when their build started should pass that marker file as `$1`
# (`scripts/cargo-test.sh` does); with no argument the newest fingerprint cargo
# wrote is used instead, which is the best available stand-in for "this build".
#
# `CARGO_TARGET_MAX_SIZE_MB` puts a ceiling on the tree after the sweep above.
# Sweeping keeps one build, but one build of this repository is itself large,
# and a laptop shared with everything else the maintainer is doing has a budget
# that a correct-but-unbounded cache can still exceed. Unset means no ceiling,
# which is what CI wants: an ephemeral runner is billed for the rebuild, not the
# disk. Defaults to 4096 (4GB) locally.
#
# Set `CARGO_TEST_NO_PRUNE=1` to skip pruning entirely.
#
# On CI this runs as a step of the `test` job in .github/workflows/release.yml,
# gated on `!cancelled()` so a red suite is still pruned -- it leaves the same
# stale artifacts a green one does. That gate is `!cancelled()` rather than the
# unconditional status function because issue #808 and CI-CD-BEST-PRACTICES.md
# section 10 forbid the latter anywhere in that job: it also fires when the run
# itself is cancelled, and pruning a half-written tree is pointless.
# `ci_cd::workflow_release::test_job_skips_non_code_changes` pins the rule by
# substring across the whole job block, so the forbidden name must not appear
# there even inside a comment.
#
# Usage:
#   scripts/prune-build-cache.sh [marker-file]
set -euo pipefail

if [ -n "${CARGO_TEST_NO_PRUNE:-}" ]; then
  echo "prune-build-cache: skipped (CARGO_TEST_NO_PRUNE)"
  exit 0
fi

if [ ! -d target ]; then
  echo "prune-build-cache: no target/ directory, nothing to prune"
  exit 0
fi

target_size_mb() {
  echo $(( $(du -sk target 2>/dev/null | cut -f1 || echo 0) / 1024 ))
}

before=$(target_size_mb)

if command -v cargo-sweep >/dev/null 2>&1; then
  # `--installed` keeps only what the toolchains rustup currently has can use.
  # A toolchain upgrade orphans every artifact the old compiler produced, and
  # those are pure waste: nothing can ever link them again.
  cargo sweep --installed >/dev/null 2>&1 || true

  # The marker records when this build started, so `--file` removes everything
  # cargo did not touch for it while keeping the dependencies it still
  # references. Without a marker there is no "before this build" instant to
  # compare against, and `--installed` above has already done what it can.
  marker=${1:-}
  # The path given to `--file` is the *project* to sweep, and cargo-sweep looks
  # for `sweep.timestamp` inside it -- it is not a "where is the stamp" flag, so
  # the stamp has to be written here and removed afterwards. Verified against
  # cargo-sweep 0.8.0: pointing `--file` at a temp directory fails with
  # "manifest path ... does not exist" and sweeps nothing at all, silently.
  #
  # The stamp holds a JSON epoch instant, which cargo-sweep parses; its mtime is
  # never read. Everything cargo did not touch for the build that started at
  # that instant goes, and everything the build still references stays.
  marker_epoch=""
  if [ -n "${1:-}" ] && [ -e "${1:-}" ]; then
    marker_epoch=$(stat -f %m "$1" 2>/dev/null || stat -c %Y "$1" 2>/dev/null || echo "")
  fi
  # Without a caller-supplied marker, "now" is the right instant: the build that
  # just finished is the one to keep, and anything it did not touch is stale.
  #
  # `+ 1`, because cargo-sweep compares whole seconds. A stamp written in the
  # same second as the build it follows makes every artifact look "not older
  # than" the stamp, and the sweep keeps the entire tree -- measured against
  # cargo-sweep 0.8.0, where an orphaned dependency survived a same-second stamp
  # and was removed once the stamp was a second later. Rounding up keeps the
  # build that just finished (its artifacts are newer than the marker) while
  # still catching everything from before it.
  [ -z "$marker_epoch" ] && marker_epoch=$(( $(date +%s) + 1 ))

  if [ -e sweep.timestamp ]; then
    # Someone else's stamp. Leave it alone rather than clobbering it.
    cargo sweep --file . >/dev/null 2>&1 || true
  else
    printf '{"secs_since_epoch":%s,"nanos_since_epoch":0}' "$marker_epoch" \
      > sweep.timestamp
    trap 'rm -f sweep.timestamp' EXIT
    cargo sweep --file . >/dev/null 2>&1 || true
    rm -f sweep.timestamp
    trap - EXIT
  fi
  pruner="cargo-sweep"
else
  # Fallback: no cargo-sweep on this machine. Timestamps only.
  marker=${1:-}
  cleanup_marker=""
  if [ -z "$marker" ]; then
    # No caller-supplied start time. Use the newest fingerprint cargo just
    # wrote: everything the current build touched is at least that new, and
    # everything older belongs to a build that no longer exists.
    newest=$(find target -name '.fingerprint' -prune -o -type f -newer Cargo.toml -print 2>/dev/null | head -1 || true)
    marker=$(mktemp)
    cleanup_marker=$marker
    if [ -n "$newest" ]; then
      touch -r "$newest" "$marker"
    else
      # Nothing newer than Cargo.toml: treat the whole tree as current and only
      # drop artifacts older than the manifest.
      touch -r Cargo.toml "$marker"
    fi
  fi
  # shellcheck disable=SC2064  # expand now: the path must survive this scope
  [ -n "$cleanup_marker" ] && trap "rm -f '$cleanup_marker'" EXIT

  # Only build outputs are pruned. Binaries, test executables and cargo's own
  # bookkeeping stay, so the next build still links rather than starting cold.
  find target -type f ! -newer "$marker" \
    \( -path '*/incremental/*' -o -name '*.rlib' -o -name '*.rmeta' -o -name '*.o' \) \
    -delete 2>/dev/null || true
  find target -type d -empty -delete 2>/dev/null || true
  pruner="timestamps (install cargo-sweep for fingerprint-accurate pruning)"
fi

# Linked example binaries, which cargo-sweep leaves alone.
#
# Issue #1049: `target/debug/examples` reached 27GB of a 28GB tree, and
# `cargo sweep --maxsize 4096` cleaned nothing from it. This crate has 116
# examples; each links the whole library into a ~190MB binary, and cargo keeps
# both a hashed and an unhashed copy of every one. cargo-sweep reasons about
# what the *current* build references, and these are current -- so they are
# invisible to it and to `--maxsize` alike, and the tree grows without limit.
#
# Nothing needs them between runs. `cargo check --examples` type-checks an
# example without linking it, which is what both the `run_clippy` CI gate and
# the pre-commit hook use; a linked example binary only appears when someone
# runs `--all-targets` by hand, and it is never read again afterwards.
if [ -d target/debug/examples ] || [ -d target/release/examples ]; then
  rm -rf target/debug/examples target/release/examples
fi

# A ceiling, applied after the sweep. Only local runs get one by default: see
# the header. cargo-sweep drops least-recently-used artifacts until the tree
# fits, which keeps the newest build -- the one about to be extended -- intact.
max_size_mb=${CARGO_TARGET_MAX_SIZE_MB:-}
if [ -z "$max_size_mb" ] && [ -z "${CI:-}" ]; then
  max_size_mb=4096
fi
if [ -n "$max_size_mb" ] && [ "$max_size_mb" -gt 0 ] 2>/dev/null; then
  if [ "$(target_size_mb)" -gt "$max_size_mb" ]; then
    if command -v cargo-sweep >/dev/null 2>&1; then
      cargo sweep --maxsize "$max_size_mb" >/dev/null 2>&1 || true
      echo "prune-build-cache: applied ${max_size_mb}MB ceiling"
    else
      echo "prune-build-cache: target/ exceeds ${max_size_mb}MB; install cargo-sweep to enforce the ceiling"
    fi
  fi
fi

after=$(target_size_mb)
freed=$((before - after))

if [ "$freed" -gt 0 ]; then
  echo "prune-build-cache: freed ${freed}MB via ${pruner} (target/ is now ${after}MB)"
else
  echo "prune-build-cache: nothing stale to remove via ${pruner} (target/ is ${after}MB)"
fi
