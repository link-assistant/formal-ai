#!/usr/bin/env bash
# Change-shaped verifier: the task is only complete when a *tracked source file*
# differs from its pristine baseline in a specific, named way. A self-describing
# side file can never satisfy it, because the check reads the tracked file only.
set -euo pipefail

TARGET="src/orchestration/workspace.rs"
BASELINE=".baseline/$TARGET"

[ -f "$TARGET" ] || { echo "missing_target:$TARGET" >&2; exit 1; }
[ -f "$BASELINE" ] || { echo "missing_baseline:$BASELINE" >&2; exit 1; }

# 1. The tracked file must actually have changed. This is the requirement the
#    evidence-shaped ladder never made: an added file is not a modification.
if cmp -s "$TARGET" "$BASELINE"; then
  echo "unchanged_target:$TARGET" >&2
  exit 1
fi

# 2. The change must be the one asked for, read out of the tracked file itself.
grep -q 'node_modules' "$TARGET" || { echo "missing_node_modules_arm" >&2; exit 1; }
grep -q '"\.git" | "target" | "\.formal-ai" | "\.formal-ai-orchestration" | "node_modules"' "$TARGET" \
  || grep -qE '"node_modules"' "$TARGET" || { echo "node_modules_not_in_ignored_set" >&2; exit 1; }

# 3. The file must still parse as Rust. `rustfmt --check` fails on a parse error,
#    so a syntactically broken edit cannot pass.
if command -v rustfmt >/dev/null 2>&1; then
  rustfmt --edition 2024 --emit stdout "$TARGET" > /dev/null 2>/tmp/rustfmt-verify.err || {
    echo "target_does_not_parse" >&2; cat /tmp/rustfmt-verify.err >&2; exit 1;
  }
fi

# 4. Nothing else in the baseline may be disturbed.
while IFS= read -r -d '' base; do
  rel="${base#.baseline/}"
  [ "$rel" = "$TARGET" ] && continue
  cmp -s "$rel" "$base" || { echo "collateral_change:$rel" >&2; exit 1; }
done < <(find .baseline -type f -print0)

echo "change-shaped verifier passed for $TARGET"
