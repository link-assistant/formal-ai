# Issue 716: agentic commands must execute in the client harness

Issue: [#716](https://github.com/link-assistant/formal-ai/issues/716)
Pull request: [#728](https://github.com/link-assistant/formal-ai/pull/728)
Investigation date: 2026-07-15 UTC

## Executive summary

The screenshot exposed two related boundary failures. A tool-bearing API request for a Rust program fell through the agentic planner and returned the ordinary catalog answer, including prose that said compilation and execution happened in an issue-specific local verification harness. No `write` or `bash` call reached the Agent CLI. A follow-up asking for `Hello 2` then regenerated the unchanged catalog template.

The fix preserves the deterministic catalog as the source of the program recipe, but changes who performs its side effects. Language, source, path, and ordered commands now remain a typed `ExecutionRecipe` on the symbolic answer; rendered prose is presentation only. In HTTP agent mode that artifact is lowered into an advertised client `write` call, each advertised client shell call in order, and a final answer grounded in actual tool results. The adapter classifies tool names by capability, so `write`, `write_file`, `create_file`, `bash`, `shell`, `exec`, and `run_command`-style harnesses share one path. The existing catalog output-edit pass now transforms both rendered output and typed source before lowering. HTTP solver instances are also prohibited from running the older embedded temporary-workspace executor.

After maintainer review, the investigation was replayed through Formal AI's production associative auto-learning adapter. Persisted observations link the presentation-coupling, workspace-ownership, client-capability, and follow-up-source failures to three architectural amendments. Usage and graph degree rank those expressions; promotion stays human-review gated until the protocol matrix, presentation variations, and external Agent CLI E2E pass. This replaces the obsolete text-scraping implementation while keeping learning auditable rather than silently mutating production rules.

## Preserved evidence

The `raw-data` directory contains the immutable inputs used in this investigation:

- `issue-716.json` and `issue-716-comments.json`: full issue metadata and all conversation comments. There were no issue comments at investigation time.
- `issue-716.png`: the authenticated GitHub attachment from the issue body. Its eight-byte PNG signature was validated before visual inspection.
- `pr-728.json`, `pr-728-initial.diff`, `pr-728-conversation-comments.json`, `pr-728-review-comments.json`, and `pr-728-reviews.json`: the current PR state, initial diff, and all three GitHub review channels, including the maintainer's post-implementation request for deeper auto-learning and generalization.
- `related-issue-{680,681,682}.json` and `related-pr-{683,684,685}.json`: the recent intent-routing, create-vs-read, and null-content fixes studied for shared planner and API-surface conventions.
- `related-pr-{689,719,726}.json`: the merged associative persistence, semantic-frame routing, and client-execution memory work studied during the post-review architecture pass.
- `agent-cli-e2e.log`: the real `@link-assistant/agent` reproduction after the fix, including the Formal AI request trace.
- `agent-cli-typed-command/`: the post-review external Agent CLI replay using typed execution data, including its source artifact and full request trace.
- `structured-recipe-red.log`: the pre-fix regression proving that harmless label rewording disabled the old prose scraper.
- `agent-cli-execution-learning/`: the real Agent CLI stream, server trace, and generated associative-learning report from the post-review replay.
- `focused-regressions.log`: the six unit and four protocol-integration regressions passing after the fix.
- `manifest.sha256`: checksums for the evidence files.

## Reconstructed timeline

1. On 2026-07-14 at 20:37:54 UTC, issue #716 was opened with an OpenCode screenshot. Formal AI answered “Give me hello world program in Rust” with source and instructions, but the CLI displayed no write or shell tool calls.
2. The response claimed a check and run in an “issue-8 local verification harness.” That execution, even if true inside Formal AI, was not observable or controllable by the API client.
3. The user then sent “Change the output message to `Hello 2`.” Formal AI returned the same `Hello, world!` source and repeated the same claimed verification.
4. On 2026-07-15 at 02:03:04 UTC, draft PR #728 was created from `issue-716-c3806a7105d9` with only `.gitkeep` and a placeholder description.
5. The investigation reproduced both defects in focused tests, traced the fallthrough from `agentic_outcome` to the ordinary symbolic program catalog, and identified that the Responses path also discarded the reconstructed conversation history during this fallback.
6. The implementation added a shared command-recipe lowering step after symbolic solving, kept the client permission/tool gate authoritative, restored Responses history, and disabled embedded agent-workspace execution on the HTTP surface.
7. Unit tests, four API protocol tests, and a real Agent CLI E2E run verified source creation, compilation, execution, output edits, and failure handling.
8. Maintainer review identified the remaining presentation-coupled architecture. A pre-fix regression demonstrated that rewording labels disabled execution; typed symbolic execution metadata removed that dependency across all ten catalog languages.
9. Formal AI ranked the persisted failure/amendment network and used the real Agent CLI to write and verify the review-gated learning artifact.

## Complete requirement audit

| Requirement | Evidence / implementation |
| --- | --- |
| Route CLI commands through the actual CLI shell tool | `command_reroute` consumes the typed ordered commands and emits the client-advertised run capability; the E2E trace asserts `rustc main.rs -o main` and `./main`. |
| Apply the rule to other tools and supported agentic CLIs | The same lowering requires and selects advertised write/run capabilities by semantic tool-name class, not a client brand. OpenAI Chat/Responses, Anthropic Messages, and Gemini API tests cover the protocol adapters used by Codex/OpenCode/Agent/Claude/Gemini-style clients. |
| Preserve all related logs/data and perform a deep case study | This directory contains the issue attachment, issue/PR API records, every comment stream, initial diff, E2E log, checksum manifest, timeline, root-cause analysis, alternatives, and authoritative external references. |
| Add tracing if evidence is insufficient | Existing opt-in `FORMAL_AI_TRACE_REQUESTS=1` records inbound tool schemas, assistant calls replayed by the harness, and tool results. It was sufficient and remains off by default. The E2E harness enables it. |
| API mode must prefer actual agentic tools and memory over embedded tools | The protocol invokes the client tool loop after symbolic/memory-aware solving. `ExecutionSurface::HttpServer` cannot invoke `try_agent_workspace_task`; no server-private side effect can masquerade as a client tool operation. |
| Web, desktop, and Telegram may use configurable embedded tools | The HTTP restriction is surface-specific and does not remove the existing library/CLI/browser/Telegram embedded path. Desktop tools remain default-deny and configurable through the existing tool grants/router. |
| Embedded code execution must be isolated from the user's computer | Existing desktop `code_exec`/`eval_js` and Docker-isolated shell routing uses the pinned `konard/box-dind:2.1.1` sandbox and refuses when unavailable. The packaged Telegram/agent environment uses DinD without the host socket and `start-command --isolated docker --auto-remove-docker-container`, providing disposable inner containers. This PR does not weaken those boundaries. |
| Report defects to related projects when applicable | No upstream defect was found. The Agent/OpenCode harness already advertised and executed its tools correctly; Formal AI chose a text fallback before emitting a call. Filing an upstream issue would therefore be misleading. |
| Apply the correction everywhere in the codebase | The shared Chat request representation feeds Chat Completions and Anthropic; Responses uses the same adapter after history reconstruction; Gemini converts through the same planning contract. Ordinary clients without both capabilities remain text-only. |
| Use auto-learning and execute the same task through Formal AI via Agent CLI | `issue-716-execution-learning.lino` persists observations and evidence-linked amendments. The production associative adapter ranks them, while both the deterministic driver and external Agent CLI write and verify the human-review-gated report. |
| Complete everything in one PR | Code, regression tests, E2E workflow, changelog, raw evidence, and analysis are all in PR #728. |

## Root-cause analysis

### 1. The planner covered curated agent recipes, not ordinary catalog programs

`agentic_outcome` first asked `plan_chat_step` for a tool plan. That planner handled explicit file, web, diagram, audit, and general change recipes, but an ordinary “hello world in Rust” request was intentionally unknown and fell through. The universal solver then selected `write_program` and rendered a normal prose answer. Nothing revisited that answer to determine whether its source and commands should be executed by the requesting harness.

### 2. Execution metadata was flattened into presentation

The program catalog already carried structured execution facts: source filename, optional check command, run command, and expected output. The first implementation rendered them into fenced code plus `Check command:` / `Run command:` prose and then parsed those labels back out in the protocol layer. That worked for the initial screenshot but contradicted generalization: localization or harmless copy editing silently disabled execution. The preserved red regression changes only the labels and proves the failure. `ExecutionRecipe` now projects directly from the selected catalog rule and survives presentation changes, serialization, diagnostic decoration, and follow-up source edits.

### 3. Follow-up context recovered the task, but not the literal edit

The solver could recover “Rust hello world” from history, and an existing language-independent edit pass could replace the catalog's default output. Its replacement vocabulary recognized “replace,” “instead,” and localized equivalents, but not the screenshot's verb “change.” Adding that semantic cue makes the already-generic pass update the entire rendered recipe—including source, expected output, and explanation—before the tool lowering sees it. A Go regression proves this does not mistake the `"fmt"` import for the output string.

### 4. Responses fallback lost history

The Responses implementation reconstructed `memory_prompt` and `history`, but then called the standing-requirements solver with the raw prompt and an empty history. Chat did not have this discrepancy. Passing the reconstructed values is required for the same multi-turn behavior across API shapes.

### 5. The HTTP solver could execute in the wrong workspace

For explicit `[agent]` actions, `try_agent_workspace_task` could run commands in a server-created temporary directory. That executor is useful for local embedded surfaces, but wrong for an HTTP agentic harness: the client cannot approve, observe, or retain the operation. The execution-surface gate now leaves HTTP actions declarative so protocol tools remain the only side-effect boundary.

## Design and alternatives

### Selected: project and lower a typed symbolic execution artifact

The selected design projects the selected catalog rule into `ExecutionRecipe { language, source, path, commands }` before presentation rendering and reuses the planner's capability classifier. It stages one call per turn because agent protocols require each tool result to be returned before the next decision. A failed write/check/run ends the sequence and is surfaced verbatim; successful completion reports the actual final non-empty tool output. A catalog-wide test derives expected metadata from `PROGRAM_LANGUAGES`, so future language entries join the same path without planner changes.

This keeps ordinary chat unchanged, never invents a tool the client did not advertise, preserves the existing agent-mode and per-tool permission gates, and works through each protocol adapter. Blueprint programs that need multiple files, dependencies, reviewed assumptions, or network placeholders intentionally remain non-executable until their model can express those prerequisites without fabricating a runnable workspace.

### Auto-learning promotion rule

The learning report is derived from persisted Links Notation through `AssociativeMemory`, not from a canned checklist. Reads, writes, incoming links, and outgoing links determine retention ranking, and evidence qualifiers preserve why each amendment exists. Learning proposes the typed-artifact and side-effect-boundary rules but does not automatically promote them; human review plus the exact protocol, variation, and Agent CLI checks is the gate.

### Rejected: add every catalog program to `plan_chat_step`

Duplicating every language/task combination in the agentic planner would create two sources of truth and would miss future catalog additions. Lowering the solver's already-selected recipe generalizes automatically.

### Rejected: execute commands on the Formal AI server and copy back prose

This is the behavior the issue rejects. It uses the wrong filesystem, bypasses client approvals/audit, cannot update the user's actual working tree, and makes claimed results unverifiable.

### Rejected: ask each CLI brand to parse prose

Tool-call protocols already define a structured boundary. Teaching Codex, OpenCode, Agent, Claude, and Gemini clients to scrape Formal AI text would be brittle and would incorrectly move a server defect into every downstream project.

## External facts and existing components

- OpenAI's API tool contract represents tools as request definitions and tool calls as structured response items; the application executes the call and returns its result. See the official [OpenAI API reference](https://platform.openai.com/docs/api-reference/responses).
- Google's official [Gemini function-calling guide](https://ai.google.dev/gemini-api/docs/function-calling) likewise distinguishes the model's `functionCall` from the application's function execution and returned `functionResponse`.
- OpenCode exposes file-write and shell tools to its model provider; its maintained implementation and schemas are in the official [OpenCode repository](https://github.com/opencode-ai/opencode). The screenshot therefore did not demonstrate a missing client capability.
- Formal AI already had a cross-client capability classifier and protocol conversions. Reusing them avoided a new dependency or a per-brand compatibility table.
- Formal AI's existing disposable-container component is `start-command` inside the DinD image, configured with `--isolated docker --auto-remove-docker-container`; the desktop router uses the same pinned sandbox image for code tools.

## Verification strategy

The minimum regression walks the full conversation:

1. user requests a Rust hello-world program;
2. server emits `write(main.rs, source)`;
3. harness returns the write result;
4. server emits `bash("rustc main.rs -o main")`;
5. harness returns compile success;
6. server emits `bash("./main")`;
7. harness returns `Hello, world!`;
8. server returns a final answer containing that actual output;
9. a fresh follow-up asking for `Hello 2` emits updated source rather than the original template.

Additional tests prove that presentation changes do not alter executable intent, all ten catalog languages project their own filenames and ordered commands, compile errors prevent the run command, missing write/run capabilities do not produce fabricated calls, the HTTP solver never invokes its embedded workspace, and OpenAI Chat, OpenAI Responses, Anthropic Messages, and Gemini all emit their native tool-call shapes.

The CI E2E launches `formal-ai serve` with agent mode and tracing, drives the released binary through the real `@link-assistant/agent` CLI, asserts `main.rs` exists in the CLI workspace, checks its source canary, requires at least four API turns, and checks the trace for both exact Rust commands. A second live task asks Formal AI to auto-learn from issue #716's persisted network, then requires the Agent CLI to write a report containing the typed-artifact amendment.
