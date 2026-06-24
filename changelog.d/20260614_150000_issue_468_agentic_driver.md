---
bump: minor
---

### Added

- An **in-repo agentic driver** (`src/agentic_coding/driver.rs`) that plays the
  role of an external agentic CLI against our own OpenAI-compatible server,
  closing issue #468's *"our Formal AI system should have enough skills … to
  actually call all the tools from any agentic CLI, understand errors from
  tools, … do web fetch and web search, to actually complete the task"*. It
  advertises the four-tool set (`web_search`, `web_fetch`, `write_file`,
  `run_command`), and on every `tool_calls` turn the server emits it **executes**
  each call — search/fetch against an offline corpus, file writes and commands in
  a single reused, sandboxed [`AgentWorkspace`] — feeds each result back as a
  `tool` message, and loops until the server returns the finished knowledge base.
  The loop is bounded by a hard turn cap, so unbounded reasoning stays a NON-GOAL
  and no network or neural inference is ever involved. Exposed as
  `run_agentic_task` / `run_agentic_task_in` returning a `DriverOutcome` with the
  full tool-call transcript.
- An **offline, deterministic web corpus** (`src/agentic_coding/corpus.rs`) that
  resolves `web_search` / `web_fetch` tool calls against a fixed page set: a
  search that surfaces the canonical Викитека page for «Сказка о рыбаке и рыбке»
  and a fetch that returns the canonical synopsis (the formalizer's fallback
  text), plus a 404 path for unknown URLs so the driver exercises the
  *"understand errors from tools"* requirement with no live network.
- A new **`agent` CLI subcommand** (`formal-ai agent [--task …] [--transcript]`)
  that drives the whole offline loop and prints the resulting Links Notation
  knowledge base, with `--transcript` showing every executed tool call.
- An `issue_468_agentic_loop` example that runs the driver end to end and prints
  the transcript plus the final knowledge base.

### Changed

- `AgentWorkspace` gained a `last_command_result()` accessor so a long-lived
  workspace reused across a tool-call loop can observe each command's output
  between steps, before `finish` consumes it.
- The default associative packages now include a permission-only
  `pkg_agentic_coding` package granting the client-executed `web_fetch`,
  `write_file`, and `run_command` capabilities, so the full agentic loop passes
  the server's tool-permission gate. `agent_mode` remains the real guard (every
  tool is still refused unless it is explicitly enabled), and capabilities the
  package does not name (e.g. `local_shell`) stay denied.
