#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BIN="${BIN:-$ROOT/target/release/formal-ai}"
AGENT="${AGENT:-agent}"
OUT="${OUT:-$ROOT/docs/case-studies/issue-1028/agent-tree-run}"
TREE_DEPTH="${TREE_DEPTH:-5}"
NODE_FILTER="${NODE_FILTER:-}"
BASE_PORT="${BASE_PORT:-8870}"
VERIFY_NODE="$ROOT/experiments/issue_1028_agent_cli_ladder/verify-node.sh"

[[ -x "$BIN" ]] || { echo "build first: cargo build --release --bin formal-ai" >&2; exit 2; }
command -v "$AGENT" >/dev/null || { echo "Agent CLI not installed" >&2; exit 2; }
command -v git >/dev/null || { echo "git is required" >&2; exit 2; }
command -v curl >/dev/null || { echo "curl is required" >&2; exit 2; }
command -v python3 >/dev/null || { echo "python3 is required" >&2; exit 2; }

case "$TREE_DEPTH" in
  0|1|2|3|4|5|all) ;;
  *) echo "TREE_DEPTH must be 0, 1, 2, 3, 4, 5, or all" >&2; exit 2 ;;
esac
if [[ -n "$NODE_FILTER" && ! "$NODE_FILTER" =~ ^(R|[12](\.[12]){0,4})$ ]]; then
  echo "NODE_FILTER must be R or a binary path such as 2.1.2.2.1" >&2
  exit 2
fi

mkdir -p "$OUT"

# A full depth-five run is over half an hour of real Agent CLI turns, and it
# shares `target/` with whatever else is building on the machine. A rebuild or a
# cache prune inside that window swaps -- or removes -- the binary under
# measurement, and a node that fails because its server never started is
# indistinguishable in the log from a node that failed on its merits. Copy the
# binary out once, and every node is measured against the same bytes.
STAGE="$(mktemp -d)"
trap 'rm -rf "$STAGE"' EXIT
cp "$BIN" "$STAGE/formal-ai"
BIN="$STAGE/formal-ai"

NODES="$OUT/tree.tsv"
RUN_LOG="$OUT/run.log"
: > "$NODES"
: > "$RUN_LOG"
declare -A VERIFIED_EFFECTS=()

cat > "$OUT/leaves.tsv" <<'EOF'
L01	Edit the tracked file `src/web_search_core.rs`: add "wikiquote" to the WEB_SEARCH_PROVIDERS list. Change only that file and keep it valid Rust.	src/web_search_core.rs	"wikiquote"	WEB_SEARCH_PROVIDERS
L02	Edit the tracked file `src/thinking.rs`: add "sensor_fusion" to the PLAIN_STEPS list. Change only that file and keep it valid Rust.	src/thinking.rs	"sensor_fusion"	PLAIN_STEPS
L03	Edit the tracked file `src/seed/meanings/parse.rs`: add "provenance" to the FACET_KINDS list. Change only that file and keep it valid Rust.	src/seed/meanings/parse.rs	"provenance"	FACET_KINDS
L04	Edit the tracked file `src/solver_handlers/pattern_inference.rs`: add "alternating" to the INTENT_MARKERS list. Change only that file and keep it valid Rust.	src/solver_handlers/pattern_inference.rs	"alternating"	INTENT_MARKERS
L05	Edit the tracked file `src/how_to_guide.rs`: add "about" to the TOPIC_STOPWORDS list. Change only that file and keep it valid Rust.	src/how_to_guide.rs	"about"	TOPIC_STOPWORDS
L06	Edit the tracked file `src/solver_handlers/web_requests.rs`: add "deep-foundation" to the PROMOTED_PROJECT_ORGS list. Change only that file and keep it valid Rust.	src/solver_handlers/web_requests.rs	"deep-foundation"	PROMOTED_PROJECT_ORGS
L07	Edit the tracked file `src/engine_responses.rs`: add "Good morning" to the GREETING_EXAMPLES list. Change only that file and keep it valid Rust.	src/engine_responses.rs	"Good morning"	GREETING_EXAMPLES
L08	Edit the tracked file `src/solver_dispatch.rs`: add "workspace_change" to the CONTEXTUAL_HANDLER_NAMES list. Change only that file and keep it valid Rust.	src/solver_dispatch.rs	"workspace_change"	CONTEXTUAL_HANDLER_NAMES
L09	Edit the tracked file `src/agentic_coding/shell_command_policy.rs`: add "kindly" to the PROSE_WORDS list. Change only that file and keep it valid Rust.	src/agentic_coding/shell_command_policy.rs	"kindly"	PROSE_WORDS
L10	Edit the tracked file `src/solver_handlers/document_request.rs`: add "toward " to the TARGET_MARKERS list. Change only that file and keep it valid Rust.	src/solver_handlers/document_request.rs	"toward "	TARGET_MARKERS
L11	Edit the tracked file `src/program_skill_gap.rs`: add "structured_edit" to the SYNTHESIS_ROUTES list. Change only that file and keep it valid Rust.	src/program_skill_gap.rs	"structured_edit"	SYNTHESIS_ROUTES
L12	In the file src/protocol_memory.rs, replace "request_history" with "conversation_history". Change only that file and keep it valid Rust.	src/protocol_memory.rs	conversation_history	REQUEST_HISTORY_CONVERSATION_ID
L13	In the file src/dialog_log.rs, replace "x-formal-ai-dialog-id" with "x-formal-ai-conversation-id". Change only that file and keep it valid Rust.	src/dialog_log.rs	x-formal-ai-conversation-id	DIALOG_ID_HEADER
L14	In the file src/learning_adoption_ledger.rs, replace "unknown" with "unspecified". Change only that file and keep it valid Rust.	src/learning_adoption_ledger.rs	unspecified	UNKNOWN_INTENT
L15	In the file src/google_trends_catalog.rs, replace "{query}" with "{search_query}". Change only that file and keep it valid Rust.	src/google_trends_catalog.rs	{search_query}	QUERY_PLACEHOLDER
L16	In the file src/service_accessibility.rs, replace "ttl_seconds" with "time_to_live_seconds". Change only that file and keep it valid Rust.	src/service_accessibility.rs	time_to_live_seconds	FIELD_TTL_SECONDS
L17	In the file src/web_search_fusion_core.rs, replace "statement_negation_cue" with "statement_negation_marker". Change only that file and keep it valid Rust.	src/web_search_fusion_core.rs	statement_negation_marker	NEGATION_ROLE
L18	In the file src/client_integrations.rs, replace "http://127.0.0.1:8080" with "http://127.0.0.1:8099". Change only that file and keep it valid Rust.	src/client_integrations.rs	http://127.0.0.1:8099	DEFAULT_BASE_URL
L19	In the file src/cli_report.rs, replace "agentic-cli" with "agentic-command-line". Change only that file and keep it valid Rust.	src/cli_report.rs	agentic-command-line	DEFAULT_SURFACE
L20	In the file src/entity_resolution.rs, replace "{term}" with "{entity_term}". Change only that file and keep it valid Rust.	src/entity_resolution.rs	{entity_term}	TERM_PLACEHOLDER
L21	In the file src/solver_handler_how_synthesis.rs, replace "FORMAL_AI_SOURCE_CACHE_DIR" with "FORMAL_AI_HOW_SOURCE_CACHE_DIR". Change only that file and keep it valid Rust.	src/solver_handler_how_synthesis.rs	FORMAL_AI_HOW_SOURCE_CACHE_DIR	CACHE_DIR_ENV
L22	In the file src/skill_procedure.rs, replace "https://example.com/article" with "https://example.com/document". Change only that file and keep it valid Rust.	src/skill_procedure.rs	https://example.com/document	PROCEDURE_CONFORMANCE_TRIGGER
L23	In the file src/cli_context.rs, rename the constant ERROR_JOIN to ERROR_LIST_JOIN. Change only that file and keep it valid Rust.	src/cli_context.rs	ERROR_LIST_JOIN	"; "
L24	In the file src/issue_report.rs, rename the constant TITLE_JOIN to TITLE_SEGMENT_JOIN. Change only that file and keep it valid Rust.	src/issue_report.rs	TITLE_SEGMENT_JOIN	"` + `"
L25	In the file src/service_accessibility.rs, rename the constant RECORD_INDENT to RECORD_LINE_INDENT. Change only that file and keep it valid Rust.	src/service_accessibility.rs	RECORD_LINE_INDENT	FIELD_INDENT
L26	In the file src/web_search_fusion_core.rs, rename the constant ENTITY_ROLE to WIKIDATA_ENTITY_ROLE. Change only that file and keep it valid Rust.	src/web_search_fusion_core.rs	WIKIDATA_ENTITY_ROLE	"wikidata_entity_anchor"
L27	In the file src/cli_report.rs, rename the constant TRACE_SEPARATOR to TRACE_FIELD_SEPARATOR. Change only that file and keep it valid Rust.	src/cli_report.rs	TRACE_FIELD_SEPARATOR	DEFAULT_SURFACE
L28	In the file src/client_integrations.rs, rename the constant EMPTY_BACKUP_SENTINEL to EMPTY_CONFIG_BACKUP_SENTINEL. Change only that file and keep it valid Rust.	src/client_integrations.rs	EMPTY_CONFIG_BACKUP_SENTINEL	formal-ai-empty-config-backup-v1
L29	In the file src/google_trends_catalog.rs, rename the constant QUERY_PLACEHOLDER to TRENDS_QUERY_PLACEHOLDER. Change only that file and keep it valid Rust.	src/google_trends_catalog.rs	TRENDS_QUERY_PLACEHOLDER	"{query}"
L30	In the file src/entity_resolution.rs, rename the constant CORRECTED_PLACEHOLDER to CORRECTED_TERM_PLACEHOLDER. Change only that file and keep it valid Rust.	src/entity_resolution.rs	CORRECTED_TERM_PLACEHOLDER	"{corrected}"
L31	In the file src/learning_adoption_ledger.rs, rename the constant UNKNOWN_INTENT to UNKNOWN_INTENT_NAME. Change only that file and keep it valid Rust.	src/learning_adoption_ledger.rs	UNKNOWN_INTENT_NAME	"unknown"
L32	In the file src/links_format.rs, rename the constant PROBE to NOTATION_PROBE. Change only that file and keep it valid Rust.	src/links_format.rs	NOTATION_PROBE	: &str = "v";
EOF

python3 - "$OUT/leaves.tsv" "$NODES" <<'PY'
import sys
from pathlib import Path
leaves = {}
for line in Path(sys.argv[1]).read_text().splitlines():
    leaf, text, change_path, change_marker, change_guard = line.split('\t', 4)
    leaves[int(leaf[1:])] = (text, change_path, change_marker, change_guard)

def child(path, branch):
    return path + ("." if path else "") + str(branch)

def leaf_index(path):
    bits = ''.join('0' if p == '1' else '1' for p in path.split('.'))
    return int(bits, 2) + 1

def emit(path, depth, out):
    if depth == 0:
        text = 'Verify Formal AI supports recursive binary task decomposition from atomic leaves through the complete 32-leaf level.'
        criterion = 'new_composite_effect'
        node_id = 'R'
    elif depth == 5:
        i = leaf_index(path)
        node_id = path
        leaf_text, criterion_path, criterion_marker, criterion_guard = leaves[i]
        text = f'Atomic task L{i:02d}: {leaf_text}'
        criterion = 'tracked_source_change'
    else:
        node_id = path
        bits = ''.join('0' if p == '1' else '1' for p in path.split('.'))
        prefix = int(bits, 2)
        span = 2 ** (5 - depth)
        start = prefix * span + 1
        end = (prefix + 1) * span
        text = f'Complete recursive decomposition node {path}, covering atomic tasks L{start:02d}–L{end:02d}; both child nodes must produce independently checkable evidence.'
        criterion = 'new_composite_effect'
    if depth < 5:
        criterion_path = ''
        criterion_marker = ''
        criterion_guard = ''
    left = child(path, 1) if depth < 5 else ''
    right = child(path, 2) if depth < 5 else ''
    out.append((node_id, depth, text, criterion, left, right, criterion_path,
                criterion_marker, criterion_guard))
    if depth < 5:
        emit(child(path,1), depth+1, out)
        emit(child(path,2), depth+1, out)

rows=[]
emit('',0,rows)
# `depth` is an int, and str.join refuses a non-str item, so joining the row
# straight raised TypeError before a single node was ever selected. Render
# every field before joining rather than trusting the tuple to be all strings.
# Empty optional fields belong to the in-memory row, but emitting their trailing
# separators makes the committed TSV fail git's whitespace check. Readers pad
# omitted tail fields back to the eight-field schema below.
Path(sys.argv[2]).write_text(
    '\n'.join('\t'.join(map(str, r)).rstrip('\t') for r in rows) + '\n'
)
PY

python3 - "$NODES" "$TREE_DEPTH" "$NODE_FILTER" > "$OUT/selected.tsv" <<'PY'
import sys
from pathlib import Path
rows=[]
for line in Path(sys.argv[1]).read_text().splitlines():
    fields = line.split('\t', 8)
    if not 6 <= len(fields) <= 9:
        raise ValueError(f'node row has {len(fields)} fields, expected 6 through 9: {line!r}')
    fields.extend([''] * (9 - len(fields)))
    (node, depth, text, criterion, left, right,
     criterion_path, criterion_marker, criterion_guard) = fields
    rows.append((node, int(depth), text, criterion, left, right,
                 criterion_path, criterion_marker, criterion_guard))
mode=sys.argv[2]
filt=sys.argv[3]
levels=list(range(5,-1,-1)) if mode=='all' else [int(mode)]
for level in levels:
    for row in rows:
        node, depth, *_ = row
        in_focused_subtree = (
            not filt
            or filt == 'R'
            or node == filt
            or node.startswith(filt + '.')
        )
        if depth == level and in_focused_subtree:
            print('\t'.join(map(str,row)).rstrip('\t'))
PY

selected_count=$(wc -l < "$OUT/selected.tsv" | tr -d ' ')
expected=1
if [[ "$TREE_DEPTH" = all ]]; then
  if [[ -z "$NODE_FILTER" || "$NODE_FILTER" = R ]]; then
    expected=63
  else
    filter_depth=$(awk -F. '{ print NF }' <<< "$NODE_FILTER")
    expected=$(( (1 << (6 - filter_depth)) - 1 ))
  fi
elif [[ -n "$NODE_FILTER" ]]; then
  expected=1
else
  expected=$((1 << TREE_DEPTH))
fi
[[ "$selected_count" -eq "$expected" ]] || { echo "expected $expected selected nodes, got $selected_count" >&2; exit 1; }

run_one() {
  local id depth prompt criterion left right criterion_path criterion_marker criterion_guard row work session_dir server_pid port status proof effect config node_number full_prompt effect_contract verifier_status verifier_verdict
  # Tab is whitespace to Bash, so IFS would collapse the two empty child fields
  # in a leaf row and shift its external criterion into `left`. Translate only
  # for parsing to a non-whitespace separator that preserves empty fields.
  row=${1//$'\t'/$'\x1f'}
  IFS=$'\x1f' read -r id depth prompt criterion left right criterion_path criterion_marker criterion_guard <<< "$row"
  session_dir="$OUT/$id"
  work=$(mktemp -d)
  mkdir -p "$session_dir"
  # A focused replay reuses its stable evidence directory. Remove only the
  # known outputs for this node so a failed attempt cannot retain a prior
  # attempt's proof or effect and appear to have passed.
  rm -f "$session_dir/agent-stream.jsonl"
  rm -f "$session_dir/agent-stderr.log"
  rm -f "$session_dir/formal-ai.log"
  rm -f "$session_dir/proof.md"
  rm -f "$session_dir/effect.lino"
  node_number=$(python3 - "$id" <<'PY'
import sys
node=sys.argv[1]
if node == 'R':
    print(0)
else:
    bits=''.join('0' if x == '1' else '1' for x in node.split('.'))
    print(int(bits, 2) + 1)
PY
)
  port=$((BASE_PORT + node_number))

  # Cleaning the scratch checkout is not what the ladder measures, and it must
  # never decide a node's verdict. `rm -rf` reports ENOTEMPTY for a directory
  # that gained a file while the walk was inside it, which is what a just-killed
  # server flushing its last write looks like; the trap runs under `set -e`, so
  # that single failure ended the whole run after a node the log had already
  # recorded as PASS. Retry briefly, then leave the directory to the operating
  # system's temporary sweeper rather than failing the node.
  cleanup_one() {
    if [[ -n "${server_pid:-}" ]]; then
      kill -- "-${server_pid}" 2>/dev/null || kill "$server_pid" 2>/dev/null || true
      wait "$server_pid" 2>/dev/null || true
    fi
    local attempt
    for attempt in 1 2 3; do
      rm -rf "$work" 2>/dev/null && return 0
      sleep 1
    done
    echo "cleanup: could not remove $work; leaving it in place" >&2
    return 0
  }
  trap cleanup_one RETURN

  git -C "$ROOT" archive HEAD | tar -x -C "$work"
  git -C "$work" init -q
  git -C "$work" config user.email agent-ladder@example.invalid
  git -C "$work" config user.name agent-ladder
  git -C "$work" add .
  git -C "$work" commit -qm ladder-fixture
  mkdir -p "$work/.agent-ladder"

  if [[ "$depth" -lt 5 ]]; then
    local left_effect_source="${VERIFIED_EFFECTS[$left]:-}"
    local right_effect_source="${VERIFIED_EFFECTS[$right]:-}"
    if [[ ! -s "$left_effect_source" || ! -s "$right_effect_source" ]]; then
      printf '%s\tFAIL\tmissing_current_run_child_effect\n' "$id" >> "$RUN_LOG"
      return 1
    fi
    mkdir -p "$work/.agent-ladder/verified-children"
    cp "$left_effect_source" "$work/.agent-ladder/verified-children/node-$left.lino"
    cp "$right_effect_source" "$work/.agent-ladder/verified-children/node-$right.lino"
    git -C "$work" add .agent-ladder/verified-children
    git -C "$work" commit -qm ladder-verified-child-effects
  fi

  setsid env FORMAL_AI_AGENT_MODE=1 FORMAL_AI_TRACE_REQUESTS=1 \
    FORMAL_AI_MEMORY_PATH="$work/.git/formal-ai-memory/memory.lino" \
    FORMAL_AI_DREAMING=0 "$BIN" serve --agent-mode --host 127.0.0.1 --port "$port" \
    >"$session_dir/formal-ai.log" 2>&1 &
  server_pid=$!

  if ! curl -fsS --retry 30 --retry-delay 1 --retry-connrefused "http://127.0.0.1:$port/health" >/dev/null; then
    printf '%s\tFAIL\tformal_ai_server_start\n' "$id" >> "$RUN_LOG"
    tail -100 "$session_dir/formal-ai.log" >&2 || true
    return 1
  fi

  config="$(printf '{\"provider\":{\"formalai\":{\"name\":\"Formal AI\",\"npm\":\"@ai-sdk/openai-compatible\",\"options\":{\"baseURL\":\"http://127.0.0.1:%s/api/openai/v1\",\"apiKey\":\"local\"},\"models\":{\"formal-ai\":{\"name\":\"Formal AI\"}}}},\"model\":\"formalai/formal-ai\"}' "$port")"

  if [[ "$depth" -eq 5 ]]; then
    printf -v effect_contract 'Apply the change to the tracked file `%s` itself -- the file has to end up modified in the Git worktree, and nothing else may change. Then create `agent-ladder-effects/node-%s.lino` with these exact field lines: `node_path=%s`, `node_depth=%s`, `node_kind=leaf`, and `result=` followed by at least four words that state the change you made and that contain the exact text %s.' \
      "$criterion_path" "$id" "$id" "$depth" "$criterion_marker"
  else
    printf -v effect_contract 'Read the committed child effects in `.agent-ladder/verified-children/node-%s.lino` and `.agent-ladder/verified-children/node-%s.lino`. Inspect both files before writing anything. Extract each raw child value with `sed -n "s/^result=//p" FILE` or an equivalent command that returns undecorated file bytes. Treat only the single line beginning exactly `result=` as that child result. Do not copy tool-rendered line numbers, `<file>` wrappers, or any other fields. Create `agent-ladder-effects/node-%s.lino` with these exact field lines: `node_path=%s`, `node_depth=%s`, `node_kind=composite`, `left_child=%s`, `right_child=%s`, `left_result=` followed by the exact left child `result=` value, `right_result=` followed by the exact right child `result=` value, and `result=` followed by at least four words that include both exact child result values and state how they compose.' \
      "$left" "$right" "$id" "$id" "$depth" "$left" "$right"
  fi

  # Built with printf, not interpolated into a double-quoted string: bash does
  # not expand \n there, so the node instructions used to reach the agent as one
  # line with two literal backslash-n in the middle of it.
  printf -v full_prompt '%s\n\nThis is recursive binary-tree node %s at depth %s. Solve only this node'"'"'s task in this fresh temporary repository. Its harness-evaluated completion criterion is: %s. %s Leave supporting evidence in .agent-ladder/node-%s-proof.md. The first line must be exactly node_path=%s and the body must state the concrete result. The harness rejects proof without the separate Git effect. Use web research when it materially improves factual accuracy. Do not claim success without evidence.\n' \
    "$prompt" "$id" "$depth" "$criterion" "$effect_contract" "$id" "$id"

  set +e
  (cd "$work" && \
    FORMAL_AI_API_KEY=local \
    LINK_ASSISTANT_AGENT_CONFIG_CONTENT="$config" \
    "$AGENT" --no-summarize-session --compaction-model same \
      --model formalai/formal-ai --permission-mode auto \
      --output-format stream-json --compact-json --disable-stdin \
      --prompt "$full_prompt") \
      >"$session_dir/agent-stream.jsonl" 2>"$session_dir/agent-stderr.log"
  status=$?
  set -e
  if [[ "$status" -ne 0 ]]; then
    printf '%s\tFAIL\tagent_exit_%s\n' "$id" "$status" >> "$RUN_LOG"
    return 1
  fi

  # Agent CLI terminates stream-json output with a presentation-only blank
  # line. Keep the committed JSONL canonical so every line is a JSON record
  # and the generated evidence passes Git's whitespace check.
  sed -i '${/^$/d;}' "$session_dir/agent-stream.jsonl"

  proof="$work/.agent-ladder/node-${id}-proof.md"
  if [[ ! -s "$proof" ]]; then
    printf '%s\tFAIL\tmissing_proof\n' "$id" >> "$RUN_LOG"
    return 1
  fi
  if ! grep -q "^node_path=$id$" "$proof"; then
    printf '%s\tFAIL\tbad_proof_marker\n' "$id" >> "$RUN_LOG"
    return 1
  fi

  effect="$work/agent-ladder-effects/node-${id}.lino"
  set +e
  verifier_verdict=$("$VERIFY_NODE" "$work" "$proof" "$id" "$depth" "$left" "$right" "$criterion_path" "$criterion_marker" "$criterion_guard")
  verifier_status=$?
  set -e
  if [[ "$verifier_status" -ne 0 ]]; then
    printf '%s\tFAIL\t%s\n' "$id" "$verifier_verdict" >> "$RUN_LOG"
    return 1
  fi

  cp "$proof" "$session_dir/proof.md"
  cp "$effect" "$session_dir/effect.lino"
  VERIFIED_EFFECTS["$id"]="$session_dir/effect.lino"
  printf '%s\tPASS\tdepth=%s\n' "$id" "$depth" >> "$RUN_LOG"
}

failed=0
while IFS= read -r line; do
  [[ -z "$line" ]] && continue
  node=$(printf '%s\n' "$line" | cut -f1)
  echo "=== $node ===" | tee -a "$RUN_LOG"
  if run_one "$line"; then :; else failed=1; break; fi
done < "$OUT/selected.tsv"

cat > "$OUT/README.md" <<EOF
# Recursive Formal AI decomposition tree run

- requested depth: $TREE_DEPTH
- node filter: ${NODE_FILTER:-none}
- selected nodes: $selected_count
- failures: $failed

The canonical decomposition is a complete binary tree: depth 0 has 1 node,
depth 1 has 2, depth 2 has 4, depth 3 has 8, depth 4 has 16, and depth 5 has 32.
Each selected node runs in a fresh temporary repository copy against the real
\`@link-assistant/agent\` CLI and a local \`formal-ai serve --agent-mode\`.
Every leaf is *change-shaped*: its task is a member insertion, a literal
replacement or an identifier rename in a tracked source, and PASS requires the
worktree to show exactly that one file modified, with the marker absent from
\`HEAD\`, the anchor still present, and the file still parsing. An effect file
that merely describes the change never passes.

PASS requires the external harness to accept a non-hollow proof and a newly
added \`agent-ladder-effects/node-<id>.lino\` Git effect. Composite effects must
consume the immutable, committed effects that both children passed earlier in
this invocation, copy both exact child results, and compose them. Agent exit
zero or a self-authored proof alone never pass.

The \`all\` mode verifies the smallest atomic tasks first (32 leaves), then
16, 8, 4, 2, and finally the root, stopping on the first real failure so the
underlying capability can be repaired before larger composite tasks are tested.
EOF

exit "$failed"
