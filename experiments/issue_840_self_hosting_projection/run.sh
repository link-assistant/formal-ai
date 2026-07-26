#!/usr/bin/env bash
# Reproduce issue #840's focused self-hosting release evidence.
#
# The real Agent CLI session performs the whole-repository source-to-links task
# through a memory-isolated Formal AI server. In the same run, the established
# deterministic self-AST generator refreshes the current census and this harness
# preserves the detailed agentic-coding slice: the subsystem issue #840 changes.
# Together the artifacts record both exhaustive round-trip coverage and the
# focused architecture that implements the generalized grounded-action loop.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
OUT="${OUT:-$ROOT/docs/case-studies/issue-840/self-hosting-projection}"
PORT="${PORT:-8840}"
SOURCE_WORKFLOW="$ROOT/experiments/issue-819-self-hosting-evidence/run.sh"
FOCUSED_SOURCE="$ROOT/data/meta/self-ast/src/agentic_coding"
FOCUSED_OUT="$OUT/self-ast/src/agentic_coding"

OUT="$OUT" PORT="$PORT" "$SOURCE_WORKFLOW"

# The checked-in census is itself generated from the embedded current source.
# Refreshing first makes the copied snapshot fail with the normal drift tests if
# it ever stops matching the source tree.
(cd "$ROOT" && cargo run --release --quiet --example regenerate_self_ast_census) \
  2>"$OUT/self-ast-census.summary.log"

rm -rf -- "$OUT/self-ast"
mkdir -p "$FOCUSED_OUT"
cp "$FOCUSED_SOURCE"/*.lino "$FOCUSED_OUT/"

file_count="$(find "$FOCUSED_OUT" -type f -name '*.lino' | wc -l)"
line_count="$(find "$FOCUSED_OUT" -type f -name '*.lino' -print0 | xargs -0 wc -l | tail -1 | awk '{print $1}')"
echo "focused self-AST projection: $line_count lines across $file_count agentic-coding modules"
