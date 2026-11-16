#!/usr/bin/env bash
# Issue #1562: compare the Rust obligation and binding texts with the JavaScript twin on the
# three hive-mind fixtures and the #1364 Hello World work items, without
# building the formal-ai crate (whose test build needs more than 3 GB).
# Usage: bash experiments/issue-1562/rust-obligation-twin.sh
set -euo pipefail
root=$(cd "$(dirname "$0")/../.." && pwd)
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
# The pure part of the module: everything before unbind_cited_recipe, without
# the inner doc comments (they are only valid at the start of a module).
sed -n '/^\/\/\/ Drop the solver.s recipe/q;p' "$root/rust/src/agentic_coding/work_item_citations.rs" \
  | grep -v '^//!' | sed 's/^fn obligation_text_keeping/pub fn obligation_text_keeping/' > "$work/citations.rs"
OUT_DIR_GENERATED="$work" rustc --edition 2024 -O -o "$work/twin" "$root/experiments/issue-1562/rust-obligation-twin/main.rs"
mkdir -p "$work/texts"
(cd "$root" && node experiments/issue-1562/obligation-texts.mjs "$work/texts")
inputs=$(ls "$work"/texts/*.full.txt)
# shellcheck disable=SC2086
"$work/twin" $inputs > "$work/rust.out"
for full in $inputs; do
  printf '=== %s\n' "$full"; cat "${full%.full.txt}.obl.txt"
  printf '=== binding\n'; cat "${full%.full.txt}.bind.txt"
done > "$work/js.out"
if cmp -s "$work/rust.out" "$work/js.out"; then
  echo "rust and js obligation and binding texts are identical for $(echo "$inputs" | wc -l) work items"
else
  diff "$work/rust.out" "$work/js.out" | head -40
  exit 1
fi
