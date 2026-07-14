# Code references — root cause of issue #680

Issue #680 reports that tool-call emission in `formal-ai serve` is gated on a
small set of hard-coded phrasings rather than on natural-language intent. The
routing logic lives in the shared "solver handler" / planner layer, which every
wire surface (OpenAI Chat Completions, OpenAI Responses, Gemini
`generateContent`) funnels through. Because it is a single shared root cause,
the same phrasing that works on one surface works on all, and the same phrasing
that fails, fails on all.

The source files copied into this `code/` folder are snapshots taken at repo
commit `33a7d07fff32105e7dd1dc8134db4a882a83c87c` (formal-ai `0.282.0`), the same
version the issue was filed against.

## Files copied here

| Copied path | Repo path | Why it matters |
|-------------|-----------|----------------|
| `solver_handlers/natural_language_tools.rs` | `src/solver_handlers/natural_language_tools.rs` | Natural-language → tool_call dispatch (shell / calculator / web_search / javascript). Contains the `try_local_shell_tool_call` phrasing gate and `tool_call` / `tool_call_refused` log points. |
| `solver_handlers/web_requests.rs` | `src/solver_handlers/web_requests.rs` | Emits the canned **prose** for web search / web fetch instead of a tool_call. |
| `solver_handlers/feature_capability.rs` | `src/solver_handlers/feature_capability.rs` | Capability descriptions returned as prose. |
| `agentic_coding/planner.rs` | `src/agentic_coding/planner.rs` | Shared planner that decides tool vs. prose. |
| `solver_handler_how.rs` | `src/solver_handler_how.rs` | "How" handler that also surfaces canned tool descriptions. |

## Key lines (as of the snapshot)

### Canned prose instead of a `tool_call`

`src/solver_handlers/web_requests.rs`
- L42, L64 — `"HTTP fetch requested for `{url}`. …"` (web_fetch → prose, 0/11 phrasings emit a tool_call)
- L200, L215, L233 — `"Provider: duckduckgo (default) … Combined ranking: reciprocal rank fusion (k = …)"` (web_search → prose, 0/11 phrasings emit a tool_call)

### Phrasing-gated shell dispatch

`src/solver_handlers/natural_language_tools.rs`
- L30 — `try_local_shell_tool_call(prompt, normalized, log, agent_mode)` entry point
- L49–50, L96–97, L156–157, L202–203, L216–217 — `tool_call_refused` / `response:tool_call_refused` log points (the "PROSE" fall-through)
- L56 — `log.append("tool_call", "javascript_execution")`
- L102 — `log.append("tool_call", "calculator")`
- L163 — `log.append("tool_call", "web_search")`
- L187 — `fn try_local_shell_tool_call(...)`
- L274–322 — token-matching comments describing the "named tool + verb" whole-token match (the phrasing gate)

## Other handlers in the same routing layer (not copied, for reference)

These live under `src/solver_handlers/` and `src/agentic_coding/` and participate
in the same intent-vs-phrasing routing; relevant to the `edit` / `write` gaps:

- `src/solver_handlers/web_search_intent.rs`
- `src/solver_handlers/user_intent.rs`
- `src/solver_handlers/text_edit_ops.rs`
- `src/solver_handlers/shell_command_transform.rs`
- `src/agentic_coding/shell_command.rs`
- `src/agentic_coding/change_request.rs`
- `src/agentic_coding/file_read.rs`
- `src/agentic_coding/driver.rs`

## Suggested direction (from the issue)

Route tool calls on **intent** (advertised tool set + request semantics) rather
than matching a small set of literal phrasings. When a client advertises
`write` / `edit` / `websearch` / `webfetch` / `bash` and the request is the
corresponding intent in any phrasing, emit the tool_call instead of returning a
prose description.
