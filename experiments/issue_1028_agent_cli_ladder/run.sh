#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BIN="${BIN:-$ROOT/target/release/formal-ai}"
AGENT="${AGENT:-agent}"
OUT="${OUT:-$ROOT/docs/case-studies/issue-1028/agent-ladder-run}"
BASE_PORT="${BASE_PORT:-8870}"
MAX_LEAVES="${MAX_LEAVES:-32}"

[[ -x "$BIN" ]] || { echo "build first: cargo build --release --bin formal-ai" >&2; exit 2; }
command -v "$AGENT" >/dev/null || { echo "Agent CLI not installed" >&2; exit 2; }
command -v git >/dev/null || { echo "git is required" >&2; exit 2; }
command -v curl >/dev/null || { echo "curl is required" >&2; exit 2; }

mkdir -p "$OUT"
LEAVES="$OUT/leaves.tsv"
RUN_LOG="$OUT/run.log"
: > "$LEAVES"
: > "$RUN_LOG"

# The same 32 leaves committed in docs/case-studies/issue-1028/task-decomposition.md.
# Each prompt is intentionally phrased differently so the run tests the general
# coding capability rather than a memorized command sequence.
cat > "$LEAVES" <<'EOF'
L01	Read issue #1028 and create .agent-ladder/L01-proof.md containing a concise list of its four concrete requirements. Do not change production code.
L02	Inspect scripts/apt-install-with-retry.sh and record its actual retry variables and defaults in .agent-ladder/L02-proof.md. Use repository source, not memory.
L03	Inspect tests/unit/ci-cd/issue_1021.rs and identify the reusable apt retry test harness in .agent-ladder/L03-proof.md. Do not modify unrelated tests.
L04	Inspect .github/workflows/agentic-cli-matrix.yml and record the Xvfb retry budget settings in .agent-ladder/L04-proof.md.
L05	Calculate the old 3-attempt, 90-second, 5-second-delay worst case from the issue and record the arithmetic in .agent-ladder/L05-proof.md.
L06	Calculate a geometric 1:2:4 retry schedule for the remaining execution budget and explain the integer-rounding rule in .agent-ladder/L06-proof.md.
L07	Verify from the wrapper that calls without TEST_BUDGET_SECONDS keep the fixed attempt deadline. Record the exact branch in .agent-ladder/L07-proof.md.
L08	State the general invariant that a retry schedule must fit inside its enclosing step budget. Record the formula and failure condition in .agent-ladder/L08-proof.md.
L09	Add or verify a focused issue-1028 test for geometric deadline allocation. Keep the implementation general and write the test name and result to .agent-ladder/L09-proof.md.
L10	Add or verify a deterministic slow-mirror stand-in for issue #1028, without network access. Prove it in .agent-ladder/L10-proof.md.
L11	Exercise the old flat per-attempt schedule against the slow-mirror shape and record why it fails in .agent-ladder/L11-proof.md.
L12	Exercise the budget-aware escalating schedule on the same slow mirror and record the successful recovery in .agent-ladder/L12-proof.md.
L13	Verify that retry delays are reserved before attempt deadlines are allocated. Add a regression assertion if necessary and document it in .agent-ladder/L13-proof.md.
L14	Verify that later retries receive strictly more execution time than earlier retries. Add a general regression test if the repository lacks one, then document it in .agent-ladder/L14-proof.md.
L15	Verify that the first retry share is positive and smaller than the final share. Record the test/evidence in .agent-ladder/L15-proof.md.
L16	Verify that failure diagnostics report the actual deadline used by the failing attempt, not the historical fixed value. Fix generally if needed and record evidence in .agent-ladder/L16-proof.md.
L17	Verify that a persistent non-timeout apt failure returns apt's own exit status after all retries. Record evidence in .agent-ladder/L17-proof.md.
L18	Verify that timeout status 124 is distinguished from apt's ordinary failure statuses. Add a general assertion if needed and record it in .agent-ladder/L18-proof.md.
L19	Review the retry wrapper comments and make them describe the budget-aware geometric algorithm without relying on issue-specific magic numbers. Record the result in .agent-ladder/L19-proof.md.
L20	Review the Xvfb workflow environment and wrapper interface together; make any general consistency correction needed and document it in .agent-ladder/L20-proof.md.
L21	Complete the issue-1028 case-study evidence for the implemented retry behavior. Record the concrete evidence paths in .agent-ladder/L21-proof.md.
L22	Complete the changelog fragment for the retry scheduling fix. Confirm it matches repository contribution rules and record evidence in .agent-ladder/L22-proof.md.
L23	Run bash syntax validation against scripts/apt-install-with-retry.sh and record the command and result in .agent-ladder/L23-proof.md.
L24	Run the focused issue-1028 Rust tests and report their actual result in .agent-ladder/L24-proof.md.
L25	Inspect the working diff and remove or avoid unrelated changes. Record the final changed-file set in .agent-ladder/L25-proof.md.
L26	Draft a PR summary that explains the reusable budget-aware retry generalization, not only the observed incident. Put it in .agent-ladder/L26-proof.md.
L27	Add or verify requirement-traceability evidence for the four issue requirements. Record what is delivered and how it is tested in .agent-ladder/L27-proof.md.
L28	Validate docs/case-studies/issue-1028/task-decomposition.md has exactly 32 unique L01-L32 leaves and record the checker output in .agent-ladder/L28-proof.md.
L29	Round-trip the task-decomposition contract/artifact using the repository's existing Links Notation test path and record the result in .agent-ladder/L29-proof.md.
L30	Inspect any failure evidence available to this leaf, classify the root cause by observable evidence, and record the classification in .agent-ladder/L30-proof.md.
L31	If this fresh-copy check reveals a capability gap, generalize the production/tooling fix and add a differently worded regression test; never add a prompt-specific branch. Record exactly what generalized capability changed in .agent-ladder/L31-proof.md.
L32	Produce the final leaf evidence bundle in .agent-ladder/L32-proof.md: completed checks, test commands, changed files, and the agent session identifier.
EOF

leaf_count=$(wc -l < "$LEAVES" | tr -d ' ')
[ "$leaf_count" -eq "$MAX_LEAVES" ] || {
  echo "expected $MAX_LEAVES leaves, got $leaf_count" >&2
  exit 1
}

check_leaf_28() {
  local count duplicates
  count=$(grep -cE '^\| [0-9]+\.[0-9]+\.[0-9]+\.[0-9]+\.[0-9]+ \| L[0-9]{2} ' \
    "$ROOT/docs/case-studies/issue-1028/task-decomposition.md")
  duplicates=$(grep -Eo '\| L[0-9]{2} ' "$ROOT/docs/case-studies/issue-1028/task-decomposition.md" | sort | uniq -d)
  [ "$count" -eq 32 ] || { echo "decomposition rows: $count"; return 1; }
  [ -z "$duplicates" ] || { echo "duplicate leaf ids: $duplicates"; return 1; }
}

run_one() {
  local id prompt work port server_pid session_dir proof status num
  id="$1"
  prompt="$2"
  num="${id#L}"
  port=$((BASE_PORT + 10#$num - 1))
  session_dir="$OUT/$id"
  work=$(mktemp -d)
  mkdir -p "$session_dir"
  cleanup_one() {
    kill "${server_pid:-}" 2>/dev/null || true
    rm -rf "$work"
  }
  trap cleanup_one RETURN

  git -C "$ROOT" archive HEAD | tar -x -C "$work"
  git -C "$work" init -q
  git -C "$work" config user.email agent-ladder@example.invalid
  git -C "$work" config user.name agent-ladder
  git -C "$work" add .
  git -C "$work" commit -qm ladder-fixture
  mkdir -p "$work/.agent-ladder"

  FORMAL_AI_AGENT_MODE=1 FORMAL_AI_TRACE_REQUESTS=1 \
    FORMAL_AI_MEMORY_PATH="$work/.agent-ladder/memory.lino" \
    FORMAL_AI_DREAMING=0 "$BIN" serve --agent-mode --host 127.0.0.1 --port "$port" \
    >"$session_dir/formal-ai.log" 2>&1 &
  server_pid=$!
  curl -fsS --retry 30 --retry-delay 1 --retry-connrefused "http://127.0.0.1:$port/health" >/dev/null

  local config
  config="$(printf '{\"provider\":{\"formalai\":{\"name\":\"Formal AI\",\"npm\":\"@ai-sdk/openai-compatible\",\"options\":{\"baseURL\":\"http://127.0.0.1:%s/api/openai/v1\",\"apiKey\":\"local\"},\"models\":{\"formal-ai\":{\"name\":\"Formal AI\"}}}},\"model\":\"formalai/formal-ai\"}' "$port")"

  set +e
  (cd "$work" && \
    FORMAL_AI_API_KEY=local \
    LINK_ASSISTANT_AGENT_CONFIG_CONTENT="$config" \
    "$AGENT" --model formalai/formal-ai --permission-mode auto \
      --output-format stream-json --compact-json --disable-stdin \
      --prompt "$prompt\n\nWrite your evidence to .agent-ladder/$id-proof.md. The first line must be exactly leaf_id=$id. Work only in this temporary repository copy; use web research when it materially improves factual accuracy.") \
      >"$session_dir/agent-stream.jsonl" 2>"$session_dir/agent-stderr.log"
  status=$?
  set -e
  [ "$status" -eq 0 ] || {
    echo "$id\tFAIL\tagent_exit_$status" >> "$RUN_LOG"
    return 1
  }

  proof="$work/.agent-ladder/$id-proof.md"
  [ -s "$proof" ] || { echo "$id\tFAIL\tmissing_proof" >> "$RUN_LOG"; return 1; }
  grep -q "^leaf_id=$id$" "$proof" || {
    echo "$id\tFAIL\tbad_proof_marker" >> "$RUN_LOG"
    return 1
  }

  if [ "$id" = L23 ]; then
    bash -n "$work/scripts/apt-install-with-retry.sh"
  elif [ "$id" = L24 ]; then
    (cd "$work" && cargo test --test unit -- issue_1028 --nocapture)
  elif [ "$id" = L28 ]; then
    check_leaf_28
  fi

  cp "$proof" "$session_dir/proof.md"
  echo "$id\tPASS\t$proof" >> "$RUN_LOG"
}

# Run every leaf against a fresh copy. A failure is retained in its directory;
# later leaves still run so the report gives the complete capability matrix.
failed=0
while IFS=$'\t' read -r id prompt; do
  echo "=== $id ===" | tee -a "$RUN_LOG"
  if run_one "$id" "$prompt"; then
    :
  else
    failed=1
  fi
done < "$LEAVES"

check_leaf_28 | tee "$OUT/decomposition-check.txt"

cat > "$OUT/README.md" <<EOF
# Issue #1028 Agent-CLI ladder run

This directory is generated by \`experiments/issue_1028_agent_cli_ladder/run.sh\`.
Each of the 32 leaves runs in a fresh temporary repository copy against the
real \`@link-assistant/agent\` CLI and a local \`formal-ai serve --agent-mode\`.

- leaf count: $leaf_count
- failures: $failed
- decomposition: docs/case-studies/issue-1028/task-decomposition.md
- runner log: run.log

A leaf is successful only when the real Agent CLI exits successfully and leaves
its required evidence marker in the temporary copy. Leaf L23, L24 and L28 also
run direct post-agent checks because those checks are deterministic and local.
EOF

exit "$failed"
