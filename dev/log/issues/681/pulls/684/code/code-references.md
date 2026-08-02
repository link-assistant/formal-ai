# Code references — root cause of issue #681 (write → read misroute)

Issue #681 reports that a natural-language **file-creation** request
(`"Create a file named hello.txt with the content hello world"`) makes the
OpenAI-compatible endpoint emit a **`read` tool_call on the nonexistent target**
instead of a `write` tool_call, even when the client advertises both `read` and
`write`. This is a *wrong-tool* correctness bug, distinct from the umbrella
"no tool_call at all" issue #680.

The source files copied into this `code/` folder are snapshots taken at repo
commit `e25d521fe51d6ab437de6a53f0ff2db9a18c770c` (formal-ai `0.282.0`), the same
version the issue was filed against.

## Root cause (two compounding defects)

For the prompt `Create a file named hello.txt with the content hello world`:

1. **The read-intent classifier false-positives on `"content"`.**
   `has_file_read_intent` (`src/agentic_coding/file_read.rs:225`) matches any of a
   keyword list that includes `"content"` (line 229). The prompt contains
   "with **content** hello world", so it is classified as a *read* intent.

2. **The file-read recipe is checked before the write/create planner.**
   In `plan_chat_step` (`src/agentic_coding/planner.rs:117`), the file-read check
   at lines 205–207 runs *before* the general change/write plan fallback at
   lines 220–221. So once (1) misfires, the read path wins and emits a `read`
   tool_call (or, when no `read` tool is advertised, the prose
   `"I can read \`{path}\` when the client advertises a file read tool or a shell tool."`
   at `src/agentic_coding/file_read.rs:106`).

3. **(Secondary) Even the write path would miss this phrasing.** In
   `general_planner.rs`, `extract_target` (line 108) only accepts a filename when
   the preceding word is in the allowlist `["file","файл","in","в","create","создай"]`
   (line 120). Here the word before `hello.txt` is **"named"**, which is not in
   the list, so the write planner would fail to extract the target even if it were
   reached. `extract_content` (line 125) does recognise the ` with content `
   marker, so content extraction is not the blocker — target extraction is.

## Files copied here

| Copied path | Repo path | Why it matters |
|-------------|-----------|----------------|
| `agentic_coding/planner.rs` | `src/agentic_coding/planner.rs` | Top-level `plan_chat_step` router (L117); file-read checked before write (L205–207); write fallback `compose_general_change_plan` (L220–221); `Capability` enum + `classify_tool` name→capability mapping (L791–839). |
| `agentic_coding/file_read.rs` | `src/agentic_coding/file_read.rs` | Read-intent classifier `file_read_task_for` (L177) and `has_file_read_intent` (L225) whose keyword list contains `"content"` (L229) — the primary misfire. Emits the `read` tool_call / "I can read …" prose (L95–106). |
| `agentic_coding/general_planner.rs` | `src/agentic_coding/general_planner.rs` | The write path `compose_general_change_plan` (L67); `extract_target` preceding-word allowlist excludes "named" (L108–123); `extract_content` markers (L125–143). |
| `agentic_coding/change_request.rs` | `src/agentic_coding/change_request.rs` | The pinned change-request write recipe (`is_change_request_task`, `render_document`), checked at `planner.rs:160`. |
| `agentic_coding/driver.rs` | `src/agentic_coding/driver.rs` | Executes `write_file` against the isolated workspace (`workspace.create_file`, L221–224); `DRIVER_TOOLS` (L34). |
| `solver_handlers/natural_language_tools.rs` | `src/solver_handlers/natural_language_tools.rs` | Capability/agent-mode gating (`require_tool_permission`) for tool execution. |
| `endpoint-dispatch-excerpts.md` | `src/server.rs`, `src/protocol.rs` | Excerpts (files too large to copy whole): the `POST /api/openai/v1/chat/completions` route, `requests_tool_execution` / `requested_tool_names`, `agentic_outcome` → `plan_chat_step`, and `chat_completion_from_plan` (`AgenticPlan::ToolCalls` → OpenAI `tool_calls`). |
| `tests/issue_627.rs` | `tests/unit/issue_627.rs` | Closest existing coverage: `direct_file_read_prompts_emit_read_tool_calls` asserts `call.tool == "read"`. A write-vs-read regression test would live alongside these. |
| `tests/agentic_general_planner.rs` | `tests/unit/agentic_general_planner.rs` | Confirms the write path works for the `"Create file … containing …"` phrasing but not `"named …"` / `"with content …"`. |

## Key lines (as of the snapshot)

### Misfiring read-intent classifier

`src/agentic_coding/file_read.rs`
- L177 — `pub(super) fn file_read_task_for(prompt: &str) -> Option<FileReadTask>`
- L225–243 — `fn has_file_read_intent(lower: &str) -> bool` keyword list
- **L229 — `"content",`** (matches "with **content** …" in a write request)
- L95–96 — emits the `read` `PlannedToolCall`
- L106 — `"I can read \`{path}\` when the client advertises a file read tool or a shell tool."`

### Router order (read before write)

`src/agentic_coding/planner.rs`
- L117 — `pub fn plan_chat_step(...)`
- L160 — `if change_request::is_change_request_task(&task) { … }`
- L205–207 — `if let Some(file_task) = file_read_task_for(&task) { return Some(plan_file_read_step(...)) }`  ← intercepts before write
- L220–221 — `compose_general_change_plan(...)` write fallback
- L801–839 — `fn classify_tool(name)` → `Capability` (`read`, `write`/`write_file`, `edit`/`patch` ruled out, …)

### Write path that never gets a chance (and would also miss "named")

`src/agentic_coding/general_planner.rs`
- L67 — `pub(super) fn compose_general_change_plan(...)`
- L108–123 — `fn extract_target(...)` allowlist `["file","файл","in","в","create","создай"]` (no "named")
- L125–143 — `fn extract_content(...)` markers incl. `" with content "`

## Suggested direction (from the issue)

Classify create / write / save / generate-file intents as the `write` capability
and emit a `write` tool_call (path + content) when a write tool is advertised;
never route a file-creation request to `read`. Concretely: gate
`has_file_read_intent` / `file_read_task_for` so a create/write intent wins over
the `"content"` keyword, check the write/create plan before (or ahead of) the
file-read recipe, and broaden `extract_target` so the filename after "named"
(and similar) is recognised.
