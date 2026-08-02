# Data log — Issue #681 / PR #684

Compiled data related to
[issue #681](https://github.com/link-assistant/formal-ai/issues/681) and
[pull request #684](https://github.com/link-assistant/formal-ai/pull/684).

- **Repo:** `link-assistant/formal-ai`
- **Version at collection:** `0.282.0`
- **Base commit (main):** `e25d521fe51d6ab437de6a53f0ff2db9a18c770c`
- **Branch:** `issue-681-5dc4419f2eb7`
- **Collected:** 2026-07-13

## Issue summary

> **Agentic CLIs: a file-creation request emits a `read` tool_call on the
> (nonexistent) target instead of `write`.**

A natural-language **file-creation** request
(`"Create a file named hello.txt with the content hello world"`) makes the
OpenAI-compatible endpoint emit a **`read` tool_call on the target file** (which
does not exist yet) instead of a `write` tool_call — even when the client
advertises both `read` and `write` tools. The CLI then tries to read a
nonexistent file and the write never happens.

This is distinct from the umbrella issue #680 ("no tool_call is emitted"): here a
tool_call *is* emitted, but it is the **wrong tool** — a correctness bug in its
own right.

### Observed vs. expected

| | |
|---|---|
| **Observed** | assistant message contains `tool_calls: ["read"]` targeting `hello.txt` |
| **Expected** | a `write` tool_call creating `hello.txt` with content `hello world` |
| **write-only variant** | returns prose `"I can read \`hello.txt\` when the client advertises a file read tool or a shell tool."` — a *write* request classified as a *read* intent |
| **End-to-end** | `with-formal-ai --non-interactive opencode "Create a file named hello.txt ..."` finishes with `File not found: .../hello.txt` and writes nothing |

### Live matrix (from the issue, 5 CLIs × 6 tools × 10 phrasings = 300 runs)

Every CLI that hit the write/edit case emitted the *wrong* tool — a read:
`write → read` (agent, opencode), `write → read_file` (qwen, gemini); likewise
`edit → read` / `edit → read_file`. **Only 1 of 50** write runs across all CLIs
actually created the file.

## Root cause (see `code/code-references.md`)

1. The read-intent classifier `has_file_read_intent`
   (`src/agentic_coding/file_read.rs:229`) matches the keyword **`"content"`**,
   which appears in "with **content** hello world", so a write request is
   classified as a read.
2. In `plan_chat_step` (`src/agentic_coding/planner.rs:205-207`) the file-read
   recipe is checked **before** the general write/create planner, so the read
   interception wins.
3. Secondary: even if the write planner were reached, `extract_target`
   (`src/agentic_coding/general_planner.rs:120`) only accepts a filename after
   `file/in/create/…` — not after **"named"** — so the write path would also miss
   this phrasing.

## Folder contents

```
dev/log/issues/681/pulls/684/
├── README.md                              ← this index
├── issue/
│   ├── issue-681.json                     ← issue #681 (structured)
│   ├── issue-681.md                       ← issue #681 (rendered)
│   └── issue-681-comments.json            ← issue comments (empty: [])
├── pull/
│   ├── pr-684.json                        ← PR #684 (structured)
│   ├── pr-684.md                          ← PR #684 (rendered)
│   ├── pr-684.diff                        ← PR #684 diff (only the .gitkeep bootstrap so far)
│   ├── pr-684-conversation-comments.json  ← (empty: [])
│   ├── pr-684-review-comments.json        ← (empty: [])
│   └── pr-684-reviews.json                ← (empty: [])
├── related/
│   ├── issue-680.json / .md               ← umbrella: tool calls are phrasing-gated, not intent-based
│   ├── issue-680-comments.json            ← (empty: [])
│   ├── issue-607.md   ← agent CLI could not run `ls` (prior art)
│   ├── issue-602.md   ← Codex CLI: no SSE streaming on /v1/responses
│   ├── issue-604.md   ← Chat Completions streaming malformed
│   ├── issue-628.md   ← docs: agentic CLI tools testing guide
│   ├── issue-671.md   ← E52: multi-CLI agentic E2E matrix in CI
│   └── related-prs-search.json            ← PRs matching planner/intent/tool_call
└── code/
    ├── code-references.md                 ← where the root cause lives (start here)
    ├── endpoint-dispatch-excerpts.md      ← server.rs / protocol.rs excerpts
    ├── agentic_coding/
    │   ├── planner.rs                     ← plan_chat_step router (read checked before write)
    │   ├── file_read.rs                   ← has_file_read_intent("content") misfire
    │   ├── general_planner.rs             ← write path; extract_target excludes "named"
    │   ├── change_request.rs              ← pinned change-request write recipe
    │   └── driver.rs                      ← executes write_file against workspace
    ├── solver_handlers/
    │   └── natural_language_tools.rs      ← capability / agent-mode gating
    └── tests/
        ├── issue_627.rs                   ← direct_file_read_prompts_emit_read_tool_calls
        └── agentic_general_planner.rs     ← create-file write-path coverage
```

> Source files under `code/` are read-only snapshots at commit
> `e25d521fe51d6ab437de6a53f0ff2db9a18c770c` (formal-ai `0.282.0`). The live
> files are at `src/…` / `tests/…`.

## Reproduction (server-level, from the issue)

```bash
curl -sS http://127.0.0.1:8080/api/openai/v1/chat/completions \
  -H 'content-type: application/json' -H 'authorization: Bearer formal-ai' \
  -d '{
    "model":"formal-ai",
    "messages":[{"role":"user","content":"Create a file named hello.txt with the content hello world"}],
    "tools":[
      {"type":"function","function":{"name":"write","parameters":{"type":"object","properties":{"filePath":{"type":"string"},"content":{"type":"string"}}}}},
      {"type":"function","function":{"name":"read","parameters":{"type":"object","properties":{"filePath":{"type":"string"}}}}}
    ]
  }'
```

## Related issues / PRs

| # | Kind | Title |
|---|------|-------|
| 680 | issue (umbrella) | Agentic CLIs: tool calls are phrasing-gated, not intent-based |
| 683 | PR | `docs(issue-680): compile issue/PR/related data into dev/log` (the template this log mirrors) |
| 607 | issue | Agent CLI cannot run shell commands (`ls`) via natural language |
| 602 | issue | OpenAI server cannot be driven by Codex CLI: no SSE streaming on `/v1/responses` |
| 604 | issue | OpenAI Chat Completions streaming is malformed |
| 628 | issue | docs: add an agentic CLI tools testing guide |
| 671 | issue | E52: Multi-CLI agentic end-to-end matrix in CI |
| 677 | PR (merged) | Generalize agentic planning beyond pinned recipes |
| 632 | PR (merged) | Fix agent-mode natural-language directory listings |

## Suggested direction (from the issue)

Classify create/write/save/generate-file intents as the `write` capability and
emit a `write` tool_call (path + content) when a write tool is advertised; never
route a file-creation request to `read`.
