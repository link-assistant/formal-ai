# Output and session debugging

Formal AI renders ordinary answers as friendly text. Structured values use a
JSON code fence rather than leaking a raw protocol envelope:

```json
{"status":"ok","items":["a","b"]}
```

Tool results are retained byte-for-byte in the client conversation, then
normalized into a localized friendly answer. A follow-up can therefore request
a particular line, URL, or the complete result without losing the transcript.

After `formal-ai with` exits, it reports only the session artifact created or
changed by that run and prints a resume command where supported. Temporary
homes containing a transcript are preserved. Set `FORMAL_AI_PROXY_LOG` to an
existing logging-proxy file to include it in the same report.

| Harness | Session/transcript location | Resume form |
| --- | --- | --- |
| Codex | `$CODEX_HOME/sessions/YYYY/MM/DD/rollout-*.jsonl` (normally `~/.codex/sessions/...`) | `codex resume <id>` |
| T3 Code (Codex) | isolated `CODEX_HOME/sessions/**/*.jsonl` | open the preserved T3/Codex session |
| OpenCode | `~/.local/share/opencode/opencode.db` | `opencode --session <id>` |
| Agent CLI | `~/.local/share/link-assistant-agent/storage/session/**/*.json` | `agent --resume <id>` |
| Gemini | `$GEMINI_CLI_HOME/.gemini/tmp/**/*.jsonl` | `gemini --resume <id>` |
| Claude | `$CLAUDE_CONFIG_DIR/projects/**/*.jsonl` | `claude --resume <id>` |
| Qwen | `~/.qwen/projects/**/*.jsonl` inside its selected home | `qwen --resume <id>` |
| Grok | `~/.grok/**/*.jsonl` inside its selected home | client-specific |
| Aider | no registry-declared session artifact | client-specific |
| Cursor | no registry-declared session artifact | Cursor history UI |

The displayed path is authoritative for an isolated one-shot run; do not copy a
hardcoded home from this table. For server-side debugging enable
`FORMAL_AI_TRACE_REQUESTS=1`, redirect server stderr/stdout to a log, and match
its request timeline to the client transcript or `FORMAL_AI_PROXY_LOG`.

For a complete request/response record grouped by dialog, set
`FORMAL_AI_DIALOG_LOG_DIR` to a directory before starting the server. This is
off by default because it records full prompt, source, tool-result, and response
bodies. Each dialog is appended to its own JSONL file. Clients may send
`X-Formal-AI-Dialog-ID` to select an exact grouping key; without it, the server
derives a stable key from the first user prompt in the conversation.

```bash
FORMAL_AI_DIALOG_LOG_DIR=./dialog-logs formal-ai serve
gh-upload-log ./dialog-logs/dialog_*.jsonl --private \
  --description "Formal AI dialog reproduction"
```

Review and redact sensitive values before uploading. `gh-upload-log` is an
optional external uploader; recording never installs it or sends logs off the
machine automatically.

## Export a complete agentic conversation

The `context` command turns a captured dialog or supported harness session into
one reviewable document. Links Notation is the default so messages, tool calls,
tool results, metadata, and server exchanges remain readable without losing
their native repeated sequence structure.

Start a server with complete dialog logging before reproducing the problem:

```bash
mkdir -p dialog-logs
FORMAL_AI_DIALOG_LOG_DIR=./dialog-logs formal-ai serve --agent-mode
```

Pass `X-Formal-AI-Dialog-ID` from the client when a stable identifier is useful:

```bash
curl http://127.0.0.1:8080/api/openai/v1/chat/completions \
  -H 'content-type: application/json' \
  -H 'X-Formal-AI-Dialog-ID: checkout-reproduction' \
  -d '{"model":"formal-ai","messages":[{"role":"user","content":"Reproduce it"}]}'
```

Export that dialog locally:

```bash
formal-ai context export \
  --session checkout-reproduction \
  --source both \
  --log-dir ./dialog-logs \
  --output checkout-reproduction.lino
```

Use `--format json` only when another program needs JSON:

```bash
formal-ai context export \
  --session checkout-reproduction \
  --source both \
  --log-dir ./dialog-logs \
  --format json \
  --output checkout-reproduction.json
```

The global diagnostic default is verbose. Put `--silent` before `context` when
only the exported document should be written to stdout:

```bash
formal-ai --silent context export \
  --session checkout-reproduction \
  --source both \
  --log-dir ./dialog-logs
```

### Choose the source deliberately

| Source | Included data | Typical use |
| --- | --- | --- |
| `auto` | Canonical server capture when present, then OpenCode fallback | Normal local debugging |
| `both` | Reconstructed messages plus complete matching server exchanges | Full issue reports and replay evidence |
| `server` | Matching request/response exchange records without the reconstructed message list | Transport and protocol debugging |
| `harness` | OpenCode session when available, otherwise reconstructed messages without server exchanges | Client-side behavior reports |
| `opencode` | One read-only OpenCode SQLite session | Existing OpenCode sessions or offline extraction |

`auto` never combines unrelated sources. The stable session/dialog id selects
one server JSONL file or one OpenCode session. An explicit `--log-dir` avoids
depending on the process environment when exporting after the server exits.

### Conversation API

A running server exposes the same canonical capture over HTTP:

```text
GET /api/formal-ai/v1/conversations/<dialog-id>
GET /api/formal-ai/v1/conversations/<dialog-id>?format=json
GET /api/formal-ai/v1/conversations/<dialog-id>?include=server
GET /api/formal-ai/v1/conversations/<dialog-id>?include=both
```

Links Notation is the default response. `format=json` is the explicit machine
format. `include=server` returns the matching transport records;
`include=both` returns those records alongside the reconstructed transcript.
Unsafe path characters in a dialog id are rejected rather than interpreted as
filesystem paths.

### What "complete" means

Dialog JSONL is append-only, and export preserves that physical append order.
Timestamps remain metadata; they are not used to reorder exchanges because two
requests can share the same millisecond timestamp.

For every exchange, the exporter merges the request history and then the server
response. Many clients resend the entire conversation on each request, while
others send only the newest turn. The exporter finds the largest exact overlap
between the transcript suffix and incoming history prefix. This retains
incremental turns and avoids duplicating cumulative histories.

Response extraction understands the protocol envelopes served by Formal AI:

- OpenAI Chat Completions `choices[].message`;
- OpenAI Responses `output[]` message items, excluding reasoning metadata;
- Gemini `candidates[].content`;
- direct objects carrying `role` and `content`;
- streamed Chat Completions SSE deltas, including fragmented content and
  indexed, fragmented tool calls.

The raw request and response bodies remain available in `server_logs` even
when a future response envelope cannot yet be reconstructed as messages.
Consequently, protocol evidence is not discarded when transcript normalization
does not recognize a vendor-specific field.

### Links Notation conversion contract

The same Rust serializer handles server dialogs, OpenCode extraction, and
arbitrary JSON. This keeps quoting and sequence behavior identical across every
entry point.

Record arrays use native repeated singular keys:

```text
messages
  message
    role user
    content "a:b"
  message
    role assistant
    content complete
```

Scalar arrays stay inline. Unsafe object keys become explicit `field`, `name`,
and `value` nodes. Strings that could be confused with `true`, `false`, `null`,
or a number are quoted. Strings containing both quote styles use a tagged
`b64:` value, and carriage returns, newlines, and tabs stay on one physical
Links Notation line through escaping.

Convert any JSON document with that same implementation:

```bash
formal-ai context json-to-lino --path request.json --output request.lino
printf '%s\n' '{"messages":[{"role":"user","content":"a:b"}]}' \
  | formal-ai --silent context json-to-lino
```

### OpenCode SQLite sessions

OpenCode stores sessions under `${XDG_DATA_HOME:-$HOME/.local/share}/opencode/opencode.db`.
The extractor opens the database with SQLite `mode=ro` and orders messages and
parts by `(time_created, id)` for deterministic output. It first produces a
structured JSON tree; the Formal AI CLI then applies the shared Rust serializer
for Links Notation output.

```bash
formal-ai context export \
  --session ses_XXXX \
  --source opencode \
  --db ~/.local/share/opencode/opencode.db \
  --output opencode-session.lino
```

The export includes session columns, decoded JSON metadata, every message,
every part, roles, timestamps, token/cost data, and tool state. The database is
never migrated or modified by this command.

### Reporting and privacy

A complete context can contain system prompts, credentials printed by tools,
source code, filesystem paths, and model responses. Inspect the local `.lino`
or `.json` file and redact secrets before attaching it to an issue. Prefer a
private upload when the remaining reproduction is still sensitive.

The agentic `Report` flow asks twice before acting: first where to report, then
which context to include. Selecting a GitHub issue, Formal AI learning, a
harness log, or a server log is confirmation for that destination only. No
report is filed and no complete log is uploaded before those choices.

### Troubleshooting

- `dialog log unavailable` means the server did not start with
  `FORMAL_AI_DIALOG_LOG_DIR`, or export needs an explicit `--log-dir`.
- `session not found` from OpenCode means `--db` points at a different data home
  or the requested `ses_...` id is absent.
- An empty server file is not a conversation; reproduce at least one exchange.
- If two records have the same timestamp, inspect their order in the JSONL file;
  the exporter intentionally uses append order.
- Use `--format json` to inspect exact types if a quoted Links Notation scalar is
  surprising.
- Keep verbose output enabled while diagnosing transport behavior, and use
  `--silent` only when a clean stdout payload is required.
