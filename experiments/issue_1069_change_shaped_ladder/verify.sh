#!/usr/bin/env bash
# Change-shaped verifier, parameterised by a change contract (issue #1069).
#
# The issue #1028 ladder accepted a self-describing side file as a node's
# "effect", so every leaf was satisfiable without touching a tracked source.
# This verifier reads *only* the tracked target, and passes solely when that
# file differs from its pristine baseline in the named way. It runs inside the
# delegated workspace, so it is deliberately git-free and compiler-free: the
# workspace walk in `src/orchestration/workspace.rs` strips `.git` and
# `target/`, so neither is available to a verifier that ships with the fixture.
#
# The contract arrives as a file, `change-contract.env`, rather than as
# arguments, because the orchestrator's `--verify` argv is fixed for the whole
# run while the contract differs per node.
set -euo pipefail

CONTRACT="${CONTRACT:-change-contract.env}"
[ -f "$CONTRACT" ] || { echo "missing_contract:$CONTRACT" >&2; exit 2; }
# shellcheck disable=SC1090
. "./$CONTRACT"

: "${CHANGE_PATH:?contract must set CHANGE_PATH}"
: "${CHANGE_MARKER:?contract must set CHANGE_MARKER}"
: "${CHANGE_GUARD:?contract must set CHANGE_GUARD}"

BASELINE=".baseline/$CHANGE_PATH"

[ -f "$CHANGE_PATH" ] || { echo "missing_target:$CHANGE_PATH" >&2; exit 1; }
[ -f "$BASELINE" ] || { echo "missing_baseline:$BASELINE" >&2; exit 1; }

# 1. The marker must be absent from the pristine baseline. Without this a
#    contract could name text the file already contains, and doing nothing at
#    all would pass.
if grep -qF -- "$CHANGE_MARKER" "$BASELINE"; then
  echo "marker_already_present:$CHANGE_MARKER" >&2
  exit 2
fi

# 2. The tracked file must actually differ. An added side file is not a
#    modification, which is the escape this whole verifier exists to close.
if cmp -s "$CHANGE_PATH" "$BASELINE"; then
  echo "unchanged_target:$CHANGE_PATH" >&2
  exit 1
fi

# 3. The difference must be the one that was asked for.
grep -qF -- "$CHANGE_MARKER" "$CHANGE_PATH" || {
  echo "missing_change_marker:$CHANGE_MARKER" >&2; exit 1; }

# 4. The anchor the change attaches to must survive: a file replaced wholesale
#    by a line containing the marker is not the requested edit.
grep -qF -- "$CHANGE_GUARD" "$CHANGE_PATH" || {
  echo "clobbered_anchor:$CHANGE_GUARD" >&2; exit 1; }

# 5. Rust targets must still parse. `rustfmt` fails on a parse error, so a
#    syntactically broken edit cannot pass. Skipped when rustfmt is absent
#    rather than silently accepted as a pass of a check that never ran.
#
#    The parse output goes to a scratch file outside the workspace: an earlier
#    revision redirected it to `parse.err` in the working directory, and the
#    orchestrator -- correctly -- committed that file alongside the edit as part
#    of the verified effect. A verifier that adds a file to the tree it is
#    judging contaminates the evidence it exists to produce.
case "$CHANGE_PATH" in
  *.rs)
    if command -v rustfmt >/dev/null 2>&1; then
      parse_err="$(mktemp)"
      trap 'rm -f "$parse_err"' EXIT
      rustfmt --edition 2024 --emit stdout "$CHANGE_PATH" >/dev/null 2>"$parse_err" || {
        echo "target_does_not_parse" >&2; cat "$parse_err" >&2; exit 1; }
    fi
    ;;
esac

# 6. Nothing else under the baseline may be disturbed.
while IFS= read -r -d '' base; do
  rel="${base#.baseline/}"
  [ "$rel" = "$CHANGE_PATH" ] && continue
  cmp -s "$rel" "$base" || { echo "collateral_change:$rel" >&2; exit 1; }
done < <(find .baseline -type f -print0)

echo "change-shaped verifier passed for $CHANGE_PATH"
