title:	Agent CLI cannot run shell commands (ls) via natural language: server never emits tool_calls for the bash tool
state:	CLOSED
author:	konard (Konstantin Diachenko)
labels:	bug, enhancement
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
number:	607
--
## Summary

Formal AI, driven through our own **Agent CLI** (`@link-assistant/agent`), cannot execute a shell command (e.g. `ls`) when asked in natural language. The server never emits a `tool_calls` response for the standard `bash` tool that agent CLIs advertise. Two independent defects combine to block the whole flow.

This was found while verifying the issue-#538 premise that Formal AI should be drivable by its own Agent CLI to *"call bash commands"* and complete real tasks.

## Setup

- `@link-assistant/agent` **0.24.0** (updated via `bun add -g @link-assistant/agent@latest`).
- Provider config at `~/.config/link-assistant-agent/opencode.json`: a `formalai` provider using `@ai-sdk/openai-compatible`, `baseURL: http://127.0.0.1:8080/v1`, model `formal-symbolic-production`.
- Server: `formal-ai serve` (v0.254.0, stable rustc 1.96.1).

## Defect 1 — streaming hides all model output (see #604)

Agent CLI uses the same `@ai-sdk/openai-compatible` streaming path as opencode, so it hits **#604**: even `agent -p "hi"` returns an **empty** assistant turn. The `step_finish` event shows work happened but no text surfaced:

```json
{"type":"step_finish","part":{"reason":"unknown","tokens":{"input":1,"output":21,"reasoning":0},
 "model":{"providerID":"formalai","requestedModelID":"formal-symbolic-production"}}}
```

Because the streamed payload is `chat.completion` (with `message`) rather than `chat.completion.chunk` (with `delta`), **nothing the model emits — including any `tool_calls` — reaches the client.** This alone blocks tool use over the Agent CLI. Fixing #604 is a prerequisite.

## Defect 2 — the `bash` tool is never granted / never mapped (new)

Tested **server-direct** (bypassing streaming) with `FORMAL_AI_AGENT_MODE=1` so tool execution is enabled. The server still **never emits `tool_calls`** for a shell request. Failing prompts, by advertised tool name:

| Advertised tool | `finish_reason` | `tool_calls` | Server text |
| --- | --- | --- | --- |
| `bash` | `stop` | **NONE** | `Tool calls are not allowed for tool:bash: no installed associative package grants tool:bash.` |
| `shell` | `stop` | **NONE** | `Tool calls are not allowed for tool:shell: no installed associative package grants tool:shell.` |
| `run_command` | `stop` | **NONE** | misroutes to `write_program`: `no template for language the and task list_files` |

Reproduction (agent mode on):

```sh
FORMAL_AI_AGENT_MODE=1 ./target/debug/formal-ai serve &
curl -s http://127.0.0.1:8080/v1/chat/completions -d '{
  "model":"formal-symbolic-production",
  "messages":[{"role":"user","content":"Run the ls command to list files in the current directory."}],
  "tools":[{"type":"function","function":{"name":"bash","description":"Run a bash command",
    "parameters":{"type":"object","properties":{"command":{"type":"string"}},"required":["command"]}}}]
}'
# -> finish_reason: stop, tool_calls: NONE, content: "...no installed associative package grants tool:bash..."
```

### Root causes (in code)

1. **Agent mode is server-env-gated only.** Tool execution requires `FORMAL_AI_AGENT_MODE=1` (`src/solver_helpers.rs:842`). Without it, every tool request is refused with *"Tool calls and function execution are not allowed without explicit agent mode."* There is no per-request way for a CLI to opt in.
2. **No package grants `tool:bash` / `tool:shell`.** `default_associative_packages()` (`src/associative_package.rs:566`) grants `tool:calculator`, `tool:web_search`, `tool:javascript_execution`, `tool:write_program`, `tool:concept_lookup`, `tool:web_fetch`, `tool:write_file`, `tool:run_command` — **but not `tool:bash` or `tool:shell`**.
3. **No tool-name → capability mapping.** Every agent CLI (codex, opencode, Agent CLI, claude) advertises a tool literally named **`bash`**. The server looks up capability `tool:bash` verbatim and finds nothing, even though a `tool:run_command` capability exists. Nothing maps the advertised `bash`/`shell` tool name onto `run_command`.

## Proposed fix

- **Map the conventional shell tool names** (`bash`, `shell`) to the shell/run-command capability, so a CLI advertising `bash` triggers the granted command-execution path.
- **Grant a shell/bash capability by default** (in an agent-mode package), or document the exact package to import — executed in the existing bounded/isolated agent workspace (honoring `--read-only` / `permission-mode`, per NON-GOALS on bounded autonomy). Reuse the isolated `AgentWorkspace` the agentic-coding loop already uses.
- **Emit real `tool_calls`** (OpenAI `tool_calls` / streamed `delta.tool_calls`) so the CLI executes the command in its own sandbox and feeds the result back — the normal agent loop — rather than the server refusing or answering in prose.

## Acceptance criteria

- [ ] With #604 fixed, `agent -p "run ls to list files here" --model formalai/formal-symbolic-production` results in a `bash`/shell `tool_calls` with `command: "ls"`, the CLI runs it, and the file list is summarized back.
- [ ] The server emits `tool_calls` (not prose) for a natural-language shell request when a `bash`/`shell` tool is advertised and execution is permitted.
- [ ] `bash` and `shell` tool names resolve to the shell/run-command capability.
- [ ] Shell execution respects `--read-only` / `permission-mode` and runs in the bounded isolated workspace.
- [ ] A per-request / config way to enable agent tool execution exists (not only the `FORMAL_AI_AGENT_MODE` server env), or it is clearly documented.
- [ ] End-to-end test: Agent CLI drives Formal AI to run `ls` and report the listing.

## All failed prompts (as requested — reported for the record)

Every one of these returned `tool_calls: NONE`:
- Via Agent CLI: `agent -p "hi"` → empty turn (Defect 1).
- Server-direct, tool `bash`: "Run the ls command..." → refused, `tool:bash` not granted.
- Server-direct, tool `shell`: same → `tool:shell` not granted.
- Server-direct, tool `run_command`: misrouted to `write_program`.

## Environment

- `formal-ai 0.254.0`, stable rustc 1.96.1, macOS aarch64
- `@link-assistant/agent 0.24.0`

## Related

- #604 — Chat Completions SSE (prerequisite: without it, no tool_calls reach the CLI)
- #603 — multi-protocol server umbrella
- #538 — the premise that Formal AI is drivable by its own Agent CLI to run commands

---

## Documentation requirement (applies to this issue)

This issue is not complete until it ships **clear, copy-pasteable, end-to-end-verified documentation** (in `README.md` / `ARCHITECTURE.md` and/or `docs/`) for how a user drives Formal AI through the Agent CLI to run shell commands (e.g. `ls`) via natural language — such that following the doc produces the command running and its output summarized back, **without reverse-engineering config or capability grants**.
