# Issue 859: Codex hello-world execution

Issue: <https://github.com/link-assistant/formal-ai/issues/859>

Pull request: <https://github.com/link-assistant/formal-ai/pull/898>

## Problem

The same Formal AI hello-world recipe worked in Agent but failed in Codex. The
first Codex action called `write_stdin` with process id `0`, even though no
process existed, instead of creating `main.rs`. The issue also required the two
execution actions to say exactly:

- `Let me run a compile this program for you.`
- `Let me run the compiled program for you.`

Finally, `Report issue` must ask for report details rather than search the web.
The original failure and the two expected narration states are preserved in
[`screenshots/`](screenshots/).

## Root causes

The failing tests exposed three independent compatibility gaps:

1. The compatibility classifier treated every name containing `write` as a
   workspace writer. Codex's `write_stdin` therefore won the first write/run
   recipe step and received a fabricated process id.
2. Codex 0.144.1 advertises `apply_patch` as a freeform Responses custom tool.
   Formal AI only emitted and consumed JSON-schema `function_call` items, so it
   could neither send Codex's patch grammar nor resume from a
   `custom_tool_call_output`.
3. A generic `_execute` function inside Codex's document-control namespace was
   classified as runnable and took precedence over the canonical
   `exec_command` tool.

Compile and run actions also shared the generic command narration. The report
route itself was already correct on the current default branch: a Codex-shaped
reproduction selected `request_user_input`. A durable regression now protects
that behavior and proves that no web search is selected.

## Test-first reproduction

The minimum unit reproduction recorded `write_stdin` as `Some(Write)` and
selected it instead of `apply_patch`. The native Responses reproduction then
failed before compilation because the first output was a function call to
`write_stdin`. Adding Codex's current nested document-control namespace exposed
the third failure: the compile step selected `_execute_d_7437ad2e4ffa` instead
of `exec_command`.

The unchanged red outputs are in [`test-logs/`](test-logs/):

- [`red-unit.log`](test-logs/red-unit.log)
- [`red-integration.log`](test-logs/red-integration.log)
- [`red-namespaced-tool.log`](test-logs/red-namespaced-tool.log)

## Solution

- File-write classification now recognizes explicit workspace-write names;
  process input no longer qualifies. A patch-flavoured edit tool may satisfy
  the creation step and is tracked by the exact selected tool name.
- Canonical execution aliases take precedence over generic MCP namespace
  functions. Hosted research keeps its prior MCP-first behavior.
- The Responses adapter supports native `custom_tool_call` and
  `custom_tool_call_output` items, converts semantic file content to Codex's
  add-file patch grammar, emits the custom-input streaming events, and records
  custom calls in proxy summaries.
- Compile and run commands are identified from the seed language catalog, so
  the exact English sentences and their Russian, Hindi, Chinese, and Spanish
  equivalents remain data-defined while unrelated shell commands retain generic
  narration.
- A Codex-shaped report request must call `request_user_input` and must not call
  `web_search`.

## Real client verification

The reusable experiment runs pinned `codex-cli 0.144.1`, matching the issue,
against a real local Formal AI server and body-logging proxy:

```sh
cargo build --bin formal-ai
ARTIFACT_DIR=/tmp/issue-859-codex \
  CODEX_SANDBOX=danger-full-access \
  experiments/issue_859_codex_e2e/run.sh
```

`danger-full-access` was needed only because this container cannot nest Codex's
workspace-write sandbox. The experiment's default remains `workspace-write`.

Codex created `main.rs` through `apply_patch`, compiled it through
`exec_command`, ran it through a second `exec_command`, printed
`Hello, world!`, and displayed both exact narration sentences. A second Codex
run selected `request_user_input` for `Report issue` and never selected web
search. Headless `codex exec` cannot answer that interactive question, so its
non-zero report status is expected after the required tool call.

The raw network/server evidence, generated source, version, harness result, and
client logs with account email/id redacted are in
[`real-codex-e2e/`](real-codex-e2e/).

## Self-coding and authorship

The broad `examples/self-coding/run.sh --live` attempt collapsed the issue task
to the existing `./examples` directory, and Agent correctly failed with
`EISDIR`. That repository-policy gap was reported in PR comments
[#5153268503](https://github.com/link-assistant/formal-ai/pull/898#issuecomment-5153268503)
and
[#5153274247](https://github.com/link-assistant/formal-ai/pull/898#issuecomment-5153274247).

The work was then decomposed into five smallest useful leaves:

1. route Codex creation and execution tools;
2. support native Responses custom patch calls;
3. add precise multilingual compile/run narration;
4. cover hello-world and report behavior with automated and real-client tests;
5. preserve the changelog and case-study evidence.

Formal AI drove a real external Agent CLI for leaf 5. Session
`ses_040c62ea9ffe21iqJOIzaJL4Xe` wrote the final changelog fragment and verified
it with `cat`. The raw transcript is
[`raw-data/agent-cli-changelog-session.log`](raw-data/agent-cli-changelog-session.log),
and a unit test compares the transcript's `write` content to the committed file
byte for byte. One of the five reviewed leaves is therefore Agent-authored
(20%); no authorship is claimed for the manually implemented runtime changes.

## Verification

Focused green logs are preserved beside the red reproductions. They cover the
classifier/recipe, custom-call proxy summary, complete native Responses loop,
streaming custom-input events, report routing, and the prior MCP namespace
regression. The real-client experiment covers the actual filesystem and shell
effects that mocked protocol tests cannot prove.
