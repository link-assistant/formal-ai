#!/usr/bin/env bash
# Probe every *change family* the ladder intends to use, one delegation each
# (issue #1069).
#
# The issue #1028 ladder's 32 leaves are all Inspect/Verify/Record tasks, and
# every one of them is satisfiable by writing a self-describing side file. The
# reviewer's requirement is that the ladder exercise *change-shaped* tasks whose
# effect is a diff to a tracked source, and that the verifier demand that diff.
#
# Before rewriting 32 leaves it has to be known which change shapes Formal AI
# can actually deliver through the Agent CLI today, so this probe runs one
# representative of each family and records PASS or FAIL. A family that fails
# here is not a reason to soften the ladder: per R924-7 the failure is the next
# thing to fix, and the fix has to generalise rather than special-case the
# probe's wording.
set -uo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BIN="${BIN:-$ROOT/target/debug/formal-ai}"
BASE_PORT="${BASE_PORT:-8970}"
RUN_DIR="${RUN_DIR:-$(mktemp -d)}"
FAMILIES="${FAMILIES:-$ROOT/experiments/issue_1069_change_shaped_ladder/families.tsv}"
FAMILY_FILTER="${FAMILY_FILTER:-}"
PULL_REQUEST_URL="${PULL_REQUEST_URL:-https://github.com/link-assistant/formal-ai/pull/1070}"
VERIFIER="$ROOT/experiments/issue_1069_change_shaped_ladder/verify.sh"

[ -x "$BIN" ] || { echo "build first: cargo build --bin formal-ai" >&2; exit 2; }
command -v agent >/dev/null || { echo "Agent CLI not installed" >&2; exit 2; }
[ -f "$FAMILIES" ] || { echo "missing families table: $FAMILIES" >&2; exit 2; }

REPORT="$RUN_DIR/report.tsv"
: > "$REPORT"
index=0
failed=0

run_family() {
  local family task change_path marker guard row work port server_pid status
  row=${1//$'\t'/$'\x1f'}
  IFS=$'\x1f' read -r family task change_path marker guard <<< "$row"
  [ -n "$FAMILY_FILTER" ] && [ "$family" != "$FAMILY_FILTER" ] && return 0

  index=$((index + 1))
  port=$((BASE_PORT + index))
  work="$RUN_DIR/$family"
  mkdir -p "$work/$(dirname "$change_path")" "$work/.baseline/$(dirname "$change_path")"
  cp "$ROOT/$change_path" "$work/$change_path"
  cp "$ROOT/$change_path" "$work/.baseline/$change_path"
  cp "$VERIFIER" "$work/verify.sh"
  chmod +x "$work/verify.sh"
  # The contract travels as a file because `--verify` argv is fixed for a run
  # while the contract differs per node.
  {
    printf 'CHANGE_PATH=%q\n' "$change_path"
    printf 'CHANGE_MARKER=%q\n' "$marker"
    printf 'CHANGE_GUARD=%q\n' "$guard"
  } > "$work/change-contract.env"

  git -C "$work" init --quiet
  git -C "$work" config user.name "Formal AI"
  git -C "$work" config user.email "formal-ai@example.invalid"
  git -C "$work" add -A
  git -C "$work" commit --quiet -m "test: seed $family change-family probe"
  local base_commit
  base_commit="$(git -C "$work" rev-parse HEAD)"

  FORMAL_AI_AGENT_MODE=1 FORMAL_AI_TRACE_REQUESTS=1 \
    FORMAL_AI_MEMORY_PATH="$work/.git/formal-ai-memory/memory.lino" FORMAL_AI_DREAMING=0 \
    "$BIN" serve --host 127.0.0.1 --port "$port" > "$RUN_DIR/$family-server.log" 2>&1 &
  server_pid=$!

  local ready=0
  for _ in $(seq 1 100); do
    curl -sf "http://127.0.0.1:$port/v1/models" >/dev/null && { ready=1; break; }
    sleep 0.2
  done
  if [ "$ready" -ne 1 ]; then
    kill "$server_pid" 2>/dev/null || true
    printf '%s\tFAIL\tserver_start\n' "$family" >> "$REPORT"
    failed=1
    return 0
  fi

  "$BIN" agent dispatch \
    --incremental --cli agent --task "$task" \
    --workspace "$work" --output-dir "$work/.formal-ai-orchestration" \
    --pull-request "$PULL_REQUEST_URL" --base-url "http://127.0.0.1:$port" \
    --allow-command bash --allow-command rustfmt --allow-command cmp --allow-command find \
    --allow-command sed --allow-command cat \
    --verify '["bash","verify.sh"]' > "$RUN_DIR/$family-dispatch.json" 2>"$RUN_DIR/$family-dispatch.log"
  status=$?
  kill "$server_pid" 2>/dev/null || true

  # The probe's own assertion, independent of what dispatch reports: the
  # tracked file must carry a *modification* commit with all three trailers,
  # and the commit must add nothing but the orchestrator's own evidence. That
  # last clause is not decoration: the first revision of `verify.sh` redirected
  # rustfmt's stderr to `parse.err` inside the workspace, and every attributed
  # commit duly carried a stray file the task never asked for. Only the
  # workspace history shows that; the dispatch report does not.
  local changed=0 commit trailer trailers_ok=1 stray=""
  for commit in $(git -C "$work" rev-list "$base_commit..HEAD"); do
    for trailer in Formal-AI-Session Formal-AI-Evidence Formal-AI-Pull-Request; do
      git -C "$work" show -s --format=%B "$commit" | grep -q "^$trailer:" || trailers_ok=0
    done
    git -C "$work" show --format= --name-status "$commit" \
      | grep -q "^M[[:space:]]*$change_path$" && changed=1
    stray+="$(git -C "$work" show --format= --name-only --diff-filter=A "$commit" \
      | grep -v '^\.formal-ai-orchestration/' || true)"
  done

  if [ "$status" -ne 0 ]; then
    printf '%s\tFAIL\tdispatch_exit_%s\n' "$family" "$status" >> "$REPORT"
    failed=1
  elif [ "$trailers_ok" -ne 1 ]; then
    printf '%s\tFAIL\tmissing_trailers\n' "$family" >> "$REPORT"
    failed=1
  elif [ "$changed" -ne 1 ]; then
    printf '%s\tFAIL\tno_tracked_modification\n' "$family" >> "$REPORT"
    failed=1
  elif [ -n "$stray" ]; then
    printf '%s\tFAIL\tstray_files:%s\n' "$family" "$(echo "$stray" | tr '\n' ',')" >> "$REPORT"
    failed=1
  else
    printf '%s\tPASS\t%s\n' "$family" "$change_path" >> "$REPORT"
  fi
}

while IFS= read -r line; do
  case "$line" in ''|'#'*) continue ;; esac
  run_family "$line"
done < "$FAMILIES"

echo "=== change-family probe report ==="
cat "$REPORT"
echo "run directory: $RUN_DIR"
exit "$failed"
