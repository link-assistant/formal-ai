#!/usr/bin/env bash
# Issue #844 self-hosting evidence.
#
# The summarization work on this branch (statement-level deduplication,
# evidence-weighted importance, recursive gathering, recheck, the merge into a
# `world_model::Context`, and the identifier rung) is hand-authored, so its
# commits carry NO self-authorship trailers — CONTRIBUTING.md is explicit that
# "an honest 0% release is valid" and that the trailers must never be attached
# to human-authored work. That honesty is exactly what makes the differential
# self-hosting ratchet (`Self-Hosting Evidence Check`) fall on this branch, so
# the answer is the same one issue #839 established: let Formal AI author
# genuine release work of its own here, through its self-inspection recipes.
#
# Nothing in this script is a re-run of #839's bundle for its own sake. Every
# artifact below is a deterministic function of *this* branch's source tree, and
# this branch is precisely the one that adds five modules to `src/summarization/`
# and rewrites `world_model::recalculate`:
#
#   1. self-source-links.lino + whole-repository-projection-NN.lino — the
#      source-to-links projection (issue #558's "recompile itself" recipe) of the
#      tree as this branch leaves it, so the new summarization modules are in the
#      projection and each one is proven to round-trip byte-for-byte.
#   2. self-ast.lino + data/meta/self-ast/** — the CST/AST census recipe
#      (issues #538/#673) over the same tree; the committed census documents for
#      `src/summarization/{dedup,importance,gathering,context,identifier}.rs` are
#      rendered by the very same `ast_census` this session runs on one module.
#   3. how-formal-ai-works.lino — the grounded self-explanation recipe, which
#      resolves every claim against the current owned source manifest and cites
#      module content ids, so its document describes the branch's own sources.
#
# The harness itself is not duplicated: this is a thin wrapper over
# experiments/issue-839-self-hosting-evidence/run.sh, which already takes `OUT`,
# `ONLY`, `LOG` and `PORT` so that a branch can record its own sessions without
# rewriting the transcripts of the sessions already committed. The `.jsonl`
# transcripts and `.log` server traces it writes are the excluded evidence bundle
# (`scripts/self-hosting-metric.rs::CAPTURED_ARTIFACT_EXTENSIONS`) that binds
# each artifact to the real `ses_...` id that authored it.
#
# Prerequisites (same as #839):
#
#   cargo build --release --bin formal-ai \
#     --example project_source_links_sharded --example regenerate_self_ast_census
#   bun install -g @link-assistant/agent   # provides `agent` on PATH
#
# Re-running regenerates every artifact byte-for-byte except the transcripts and
# the session ids, which are new per session by construction.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
HARNESS="$ROOT/experiments/issue-839-self-hosting-evidence/run.sh"
OUT="${OUT:-$ROOT/docs/case-studies/issue-844/self-hosting-evidence}"
PORT="${PORT:-8844}"

mkdir -p "$OUT"
for axis in source-links self-ast explain; do
  echo "=== issue #844 self-hosting evidence: $axis"
  OUT="$OUT" PORT="$PORT" ONLY="$axis" LOG="$OUT/formal-ai-$axis.log" bash "$HARNESS"
done

echo "issue #844 self-hosting evidence written to $OUT"
