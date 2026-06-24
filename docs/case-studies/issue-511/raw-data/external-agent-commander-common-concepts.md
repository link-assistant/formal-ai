<!-- Snapshot of link-assistant/agent-commander docs/common-concepts.md (js_0.8.0 / rust_0.2.6) as of 2026-06-17 (issue #511 case study). This is the authoritative source for the per-command approval (ask mode / --approve-each) parity table that motivates the desktop "default = @link-assistant/agent" backend decision. Source of truth is the upstream repo. -->

# Shared Concepts

`agent-commander` has JavaScript and Rust implementations. The packages should keep the same user-facing behavior for common agent orchestration concepts, even when language-specific APIs differ.

## Tools

Both packages support these tool names:

| Tool       | Purpose                   | Read-only mode           |
| ---------- | ------------------------- | ------------------------ |
| `claude`   | Anthropic Claude Code CLI | `--permission-mode plan` |
| `codex`    | OpenAI Codex CLI          | `--sandbox read-only`    |
| `opencode` | OpenCode CLI              | permission deny rules    |
| `qwen`     | Qwen Code CLI             | `--approval-mode plan`   |
| `gemini`   | Gemini CLI                | `--approval-mode plan`   |
| `agent`    | @link-assistant/agent     | `--permission-mode readonly` (`--plan-only` → `plan`) |

Unsupported tools can still be executed through the generic command builder, but read-only planning mode is rejected unless the tool has an enforceable native restriction.

## Per-command approval (ask mode)

Both packages support a uniform `--approve-each` flag (alias: `--permission-mode ask`) that asks the consumer to approve each command the agent wants to run, mapping to the backend's native per-command approval mechanism. When the backend exposes a drivable JSON handshake, each native permission request is relayed to the consumer as a normalized `permission_request` NDJSON event (with an opaque `id`, the `tool`, the resolved `command`/`pattern`, a `title`, and a `scope`); the consumer answers with a normalized `permission_response` carrying a decision of `once`, `always`, or `reject`, which is forwarded back to the native CLI in its own wire format.

`scope` documents what an `always`/allow decision attaches to — it deliberately differs per backend, so consumers should not assume a session-wide grant everywhere.

| Tool       | Native mechanism                                       | Scope              | Relay | Notes                                                                                            |
| ---------- | ------------------------------------------------------ | ------------------ | ----- | ------------------------------------------------------------------------------------------------ |
| `agent`    | `--permission-mode ask` (+ `--input-format stream-json`) | `session`          | ✅    | Native JSON `permission_request`/`permission_response` protocol; `once` \| `always` \| `reject` map 1:1. |
| `claude`   | `--permission-mode default` (stream-json `can_use_tool`) | `tool-input`       | ✅    | `control_request`/`control_response` handshake; no session-wide `always`, so `once` and `always` both allow this call. |
| `codex`    | `--ask-for-approval` (coupled with `--sandbox`)        | `sandbox-coupled`  | ❌    | Approval is coupled with the sandbox policy and not exposed as a tool-agnostic JSON request/response stream. |
| `qwen`     | `--approval-mode default`                              | `interactive-only` | ❌    | Headless mode has no relayable per-command JSON approval handshake.                              |
| `gemini`   | `--approval-mode default`                              | `interactive-only` | ❌    | No JSON stdin channel (prompt is passed via `-p`), so approvals cannot be relayed.               |
| `opencode` | `OPENCODE_PERMISSION` (static `{edit,bash,task}` policy) | `static-policy`    | ❌    | Only a static up-front policy is available; there is no per-command request/response relay.      |

Only `agent` and `claude` can drive the handshake (`relay = ✅`). For every other tool, `--approve-each` is rejected up front with a clear error — the same pattern `--read-only` uses for tools without an enforceable native restriction.

## Isolation

The shared isolation modes are:

- `none`: run the command directly in the working directory.
- `screen`: wrap the command in a named GNU Screen session.
- `docker`: run the command in a container with the working directory mounted.

`--dry-run` should print the command that would be executed without starting a process.

## Prompt Input

Both packages accept `--prompt <text>` / `prompt` for short prompts and `--prompt-file <path>` / `promptFile` / `prompt_file` for prompt content already stored on disk.

For `claude`, `codex`, `opencode`, and `agent`, the controllers also write in-memory prompts to temporary files at execution time and pipe those files into stdin. This keeps large generated prompts out of nested shell command strings while preserving the public API used by hive-mind and similar orchestrators.

For `codex`, `opencode`, and `agent`, the temporary file contains the system prompt followed by a blank line and then the user prompt. For `claude`, the temporary file contains the user prompt and the system prompt remains a Claude CLI system-prompt argument.

## Native Tool Passthrough

Both packages expose raw passthrough controls for the native `claude`, `codex`, `opencode`, and `agent` commands. These controls cover fast-moving upstream features without forcing every native CLI option into agent-commander's typed API:

- JavaScript `toolOptions.executable` / Rust `AgentOptions.executable` / CLI `--tool-executable`
- JavaScript `toolOptions.extraEnv` / Rust `AgentOptions.extra_env` / CLI `--tool-env KEY=VALUE`
- JavaScript `toolOptions.extraArgs` / Rust `AgentOptions.extra_args` / CLI `--tool-arg`
- JavaScript `toolOptions.skipDefaultSafetyFlags` / Rust `AgentOptions.skip_default_safety_flags` / CLI `--skip-default-safety-flags`

Passthrough environment variables are attached to the native tool side of prompt pipelines, so `cat prompt.txt | env KEY=value codex exec ...` applies `KEY` to `codex` without altering prompt-file reads. Raw arguments are appended after typed arguments, allowing callers to override or extend native CLI behavior such as MCP config, reasoning config, permission modes, sandbox modes, approval modes, and custom config paths.

Claude and Codex builders also expose typed `permissionMode` / `permission_mode`, `sandboxMode` / `sandbox_mode`, and `approvalMode` / `approval_mode` fields for callers that build commands directly. The `agent` builder exposes typed `permissionMode` / `permission_mode` (`auto` | `plan` | `readonly` | `ask`) and an OpenCode-compatible `permission` / `permission` JSON policy. `--read-only` maps to `readonly` and `--plan-only` maps to `plan` for `agent`.

## Claude Options

Both packages expose Claude-specific options for model fallback, session management, prompt appending, verbose mode, and replaying user messages:

- `appendSystemPrompt` / `append_system_prompt`
- `fallbackModel` / `fallback_model`
- `sessionId` / `session_id`
- `forkSession` / `fork_session`
- `verbose`
- `replayUserMessages` / `replay_user_messages`

The CLI spellings are kebab-case: `--append-system-prompt`, `--fallback-model`, `--session-id`, `--fork-session`, `--verbose`, and `--replay-user-messages`.

## Releases

The repository publishes separate language packages from one codebase:

- JavaScript releases use Changesets, npm, `js_` tags, and `[JavaScript] vX.Y.Z` release names.
- Rust releases use changelog fragments, crates.io, `rust_` tags, and `[Rust] vX.Y.Z` release names.

Root documentation should link to the language README files. Package README files should include registry version badges so npm and crates.io package pages show the current package status.
