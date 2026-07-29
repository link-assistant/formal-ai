#!/usr/bin/env bash
# Differential harness for issue #840: run the SAME prompt against reference
# agents (claude, codex, and every opencode-zen free model) in a sandbox that
# reproduces the #838 desktop layout, and keep every transcript.
#
# This is the automated form of the manual comparison asked for in #819 — the
# point is that nobody should have to hand-run these to find out whether we
# regressed against freely available models.
#
# Usage:
#   experiments/issue_840_reference_agents/run_reference_agents.sh
#   PROMPT="Найди папку ..." experiments/issue_840_reference_agents/run_reference_agents.sh
#
# Environment knobs:
#   PROMPT     The task (default: the canonical #838 request)
#   OUTDIR     Where transcripts land (default: <scriptdir>/transcripts)
#   MODELS     Space-separated opencode model ids (default: all *-free)
#   SKIP       Space-separated agent names to skip (e.g. SKIP="claude codex")
#   TIMEOUT    Per-agent seconds (default: 420)
#
# RATE LIMITS: each opencode-zen free model has its OWN quota, so they are run
# in parallel — serialising them does not help and wastes wall-clock. A model
# that exhausts its quota must be recorded as INCONCLUSIVE, never as a pass or
# a fail; `laguna-s-2.1-free` retried for ~7 minutes before succeeding during
# the #840 investigation, and an impatient harness would have mislabelled the
# single best reference run in the whole set.
#
# Exits 0 always: this is a measurement harness, not a gate.

set -uo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"
DEFAULT_PROMPT="Find hive-mind-control center folder on my desktop. My desktop is the ./Desktop directory here."
PROMPT="${PROMPT:-$DEFAULT_PROMPT}"
OUTDIR="${OUTDIR:-$HERE/transcripts}"
TIMEOUT="${TIMEOUT:-420}"
SKIP="${SKIP:-}"
DEFAULT_MODELS="opencode/laguna-s-2.1-free opencode/deepseek-v4-flash-free opencode/ling-3.0-flash-free opencode/mimo-v2.5-free opencode/nemotron-3-ultra-free opencode/north-mini-code-free"
MODELS="${MODELS:-$DEFAULT_MODELS}"

mkdir -p "$OUTDIR"

skipped() { case " $SKIP " in *" $1 "*) return 0 ;; *) return 1 ;; esac; }

# --- sandbox reproducing the #838 desktop layout -----------------------------
SANDBOX="$(mktemp -d "${TMPDIR:-/tmp}/formal-ai-refagents.XXXXXX")"
trap 'rm -rf "$SANDBOX"' EXIT
mkdir -p "$SANDBOX/Desktop/Archive/hive-control-center"
: >"$SANDBOX/Desktop/Archive/hive-mind-bot.2025-12-26.private-key.pem"
# codex refuses to run outside a trusted/git directory.
git -C "$SANDBOX" init -q 2>/dev/null || true
echo "sandbox: $SANDBOX"
echo "prompt:  $PROMPT"
echo

# --- frontier agents ---------------------------------------------------------
if ! skipped claude && command -v claude >/dev/null 2>&1; then
  echo "=> claude"
  ( cd "$SANDBOX" && timeout "$TIMEOUT" claude -p "$PROMPT" \
      --output-format json --allowedTools "Bash,Glob" ) \
      >"$OUTDIR/claude.json" 2>"$OUTDIR/claude.err" &
fi

if ! skipped codex && command -v codex >/dev/null 2>&1; then
  echo "=> codex"
  ( cd "$SANDBOX" && timeout "$TIMEOUT" codex exec "$PROMPT" \
      --json --sandbox read-only --skip-git-repo-check </dev/null ) \
      >"$OUTDIR/codex.jsonl" 2>"$OUTDIR/codex.err" &
fi

# --- opencode free models, all in parallel (separate rate limits) ------------
if command -v opencode >/dev/null 2>&1; then
  for model in $MODELS; do
    skipped "$model" && continue
    tag="$(printf '%s' "$model" | tr '/' '_')"
    echo "=> $model"
    ( cd "$SANDBOX" && timeout "$TIMEOUT" opencode run --model "$model" --print-logs "$PROMPT" ) \
        >"$OUTDIR/oc-$tag.txt" 2>&1 &
  done
fi

wait
echo
echo "transcripts in $OUTDIR"

# --- classify ---------------------------------------------------------------
echo
echo "== outcome per agent (ground truth: Desktop/Archive/hive-control-center) =="
for f in "$OUTDIR"/*; do
  case "$f" in *.err) continue ;; esac
  name="$(basename "$f")"
  if grep -qi "rate limit" "$f" 2>/dev/null && ! grep -q "hive-control-center" "$f" 2>/dev/null; then
    printf '  %-40s INCONCLUSIVE (rate limited)\n' "$name"
  elif grep -q "hive-control-center" "$f" 2>/dev/null; then
    printf '  %-40s FOUND\n' "$name"
  elif grep -qi "permission\|rejected" "$f" 2>/dev/null; then
    printf '  %-40s INCONCLUSIVE (sandbox permission)\n' "$name"
  else
    printf '  %-40s NOT FOUND\n' "$name"
  fi
done
