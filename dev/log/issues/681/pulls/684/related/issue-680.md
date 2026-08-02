title:	Agentic CLIs: tool calls are phrasing-gated, not intent-based — web search & web fetch never fire; write/edit/shell mostly fail (all 5 CLIs)
state:	OPEN
author:	konard (Konstantin Diachenko)
labels:	bug
comments:	0
assignees:	
projects:	
milestone:	
issue-type:	
parent:	
sub-issues:	
sub-issues-completed:	
blocked-by:	
blocking:	
number:	680
--
## Summary

Driving `formal-ai serve` with the supported agentic CLIs, **tool-call emission is gated on a handful of hard-coded phrasings rather than on natural-language intent**. Only a few exact wordings cause the planner to emit an OpenAI/Responses/Gemini `tool_call`; almost every other natural phrasing of the *same* request returns prose instead, so the CLI never runs the tool. Two whole capabilities — **web search** and **web fetch** — never emit a tool call for *any* phrasing, so they are effectively unsupported out of the box.

This affects every CLI the project targets (codex, opencode, gemini, qwen, `@link-assistant/agent`) because it is a single root cause in the shared planner: the same phrasing that works on one surface works on all, and the same phrasing that fails, fails on all.

## Environment

- `formal-ai 0.282.0` (global `cargo install formal-ai`), `with-formal-ai 0.282.0`
- Server started with `FORMAL_AI_AGENT_MODE=1 formal-ai serve` (agent mode; tool execution enabled)
- CLIs: codex-cli 0.144.1, opencode 1.17.18, agent 0.24.1, gemini 0.50.0, qwen 0.7.1
- macOS (Darwin 24.6)
- Real tool names advertised by the CLIs (captured via a logging proxy): OpenAI surface — `bash, batch, codesearch, edit, glob, grep, list, read, task, todoread, todowrite, webfetch, websearch, write`; Gemini surface — `enter_plan_mode, glob, google_web_search, grep_search, invoke_agent, list_directory, read_file, update_topic`.

## Live end-to-end matrix — 5 CLIs × 6 tools × 10 English phrasings (300 runs)

Each of the 5 CLIs was driven by `with-formal-ai --non-interactive <cli> "<phrasing>"` against `formal-ai serve`, with a logging proxy recording whether the server emitted the correct `tool_call` for that request. **"correct tool_call emitted / 10 phrasings":**

| tool | agent | opencode | codex | qwen | gemini |
|------|:-----:|:--------:|:-----:|:----:|:------:|
| **shell** | 2/10 | 2/10 | 2/10 | 0/10 | 0/10 |
| **read** | 6/10 | 6/10 | 7/10 | 7/10 | 7/10 |
| **write** | 1/10 | 1/10 | 1/10 | 0/10 | 0/10 |
| **edit** | 0/10 | 0/10 | 0/10 | 0/10 | 0/10 |
| **web_search** | 0/10 | 0/10 | 0/10 | 0/10 | 0/10 |
| **web_fetch** | 0/10 | 0/10 | 0/10 | 0/10 | 0/10 |

(codex measured on its native Responses API surface; the other four via live CLI runs. Totals: web_search 0/50, web_fetch 0/50, edit 0/50, write 4/50, shell 6/50, read 33/50.)

For **write**, even the runs that emitted a `write` tool_call rarely changed the filesystem: across all CLIs only **1/50 write runs actually created the file**. The rest either returned prose or emitted a `read` on the not-yet-existing target (see companion write→read issue). No `edit` run changed a file (0/50).

## Reproduction (server-level, deterministic — no CLI flakiness)

The live matrix above reflects a server-side root cause, reproducible with a single `POST` (no CLI needed). `POST /api/openai/v1/chat/completions` with one tool advertised and a natural-language prompt; count how many of 11 phrasings per tool emit a `tool_calls` entry vs. prose:

| Tool | Phrasings that emitted a `tool_call` |
|------|--------------------------------------|
| **shell** (`bash`) | **3 / 11** |
| **web_search** (`websearch`) | **0 / 11** |
| **web_fetch** (`webfetch`) | **0 / 11** |
| **file write** (`write`) | **1 / 11** |
| **file edit** (`edit`) | **0 / 11** |

Example — only these shell phrasings route; the rest return prose:

```
OK CALL:bash   | List the files in the current directory
!! PROSE       | Show me what's in this folder
OK CALL:bash   | What files are here?
!! PROSE       | Print the current working directory
OK CALL:bash   | Run pwd
!! PROSE       | Tell me today's date using the shell
!! PROSE       | How much disk space is free?
!! PROSE       | Show the running processes
!! PROSE       | Count the number of lines in Cargo.toml
!! PROSE       | Create a directory called build
!! PROSE       | What is my username?
```

Web search — **0/11** emit a tool call; the server returns a canned description instead of calling the tool:

```
!! PROSE | Search the web for the latest stable Rust version
!! PROSE | Look up who won the 2022 World Cup
!! PROSE | Google the capital of Australia
!! PROSE | Search the internet for the CEO of OpenAI
... (all 11 return prose: "Web search requested for `...`. In the browser demo formal-ai defaults to the DuckDuckGo ...")
```

Web fetch — **0/11**; the server returns prose (`HTTP fetch requested for https://example.com. The browser web app attempts a direct fetch() first ...`) instead of a `webfetch` tool_call, for every phrasing including bare `Fetch https://example.com and summarize it`.

File edit — **0/11**; every phrasing (`In greeting.txt, change hello to goodbye`, `Replace foo with bar in notes.txt`, `Fix the typo teh to the in doc.txt`, …) returns `I could not determine ...`.

### Same defect on the other two wire surfaces (single shared root cause)

Responses API (codex) and Gemini `generateContent` (gemini/qwen) behave identically — only the one canned phrasing routes:

```
== Responses API surface (codex) ==
  shell  CALL:bash | List the files in the current directory
  shell  PROSE     | Show me what's in this folder
  shell  PROSE     | Print the current working directory
  search PROSE     | Search the web for the latest Rust version
  write  PROSE     | Create a file named hello.txt with the content hello world

== Gemini generateContent surface (gemini/qwen) ==
  shell  CALL:run_shell_command | List the files in the current directory
  shell  PROSE                  | Show me what's in this folder
  search PROSE                  | Search the web for the latest Rust version
  write  PROSE                  | Create a file named hello.txt with the content hello world
```

### End-to-end confirmation through a real CLI

`with-formal-ai --non-interactive opencode "Create a file named hello.txt with the content hello world"` → exits 0, **no file written**, and the transcript ends with `File not found: .../hello.txt` — opencode never received a `write` tool_call.

`with-formal-ai --non-interactive opencode "Search the web for the latest stable Rust version"` → the server streams prose ("Provider: duckduckgo (default) … reciprocal rank fusion") instead of a `websearch` tool_call, so opencode's web-search tool is never invoked and no results are returned.

## Impact

Out of the box, an agentic CLI pointed at `formal-ai serve` can only:
- run a shell command if the user phrases it as one of ~3 exact strings,
- read a file (this mostly works across phrasings), and
- **cannot** web-search, web-fetch, write, or edit files via ordinary natural language.

This blocks the core promise (natural-language requests fully supported for shell, web search, web fetch, file read/write/edit, code changes) for all five CLIs.

## Suggested direction

Route tool calls on **intent** (the advertised tool set + the request semantics) rather than matching a small set of literal phrasings. Concretely: when a client advertises `write`/`edit`/`websearch`/`webfetch`/`bash` tools and the request is a file-creation / file-modification / web-search / web-fetch / shell-command intent in any phrasing, emit the corresponding `tool_call` instead of returning a prose description of what would happen.

## Related

- Split out separately: **write requests emit a `read` tool_call** (wrong tool, not just missing) — see #681.
- Split out separately: **qwen 400 `MessageContent` wire error** — see #681.
- Prior art: #607 (agent CLI could not run `ls`), #602 / #604 (SSE surfaces), #628 (agentic CLI testing guide), #671 / E52 (multi-CLI E2E matrix).
