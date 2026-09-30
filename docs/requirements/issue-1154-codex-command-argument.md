# Issue #1154: One Shell-Argument Reading for the Whole Agentic Module

Issue [#1154](https://github.com/link-assistant/formal-ai/issues/1154) (E119):
Codex's shell tool `exec_command` sends the command under the argument key
`cmd`, while `progress.rs` read only `command`. The work-item read was never
recorded as attempted, and the stateless planner re-derived the identical
`gh issue view` call 78 times in a row (6.2M input tokens) until the client
compacted the thread. The full incident account, with the transcript excerpts,
is `docs/case-studies/issue-1154/`.

| ID | Requirement | Status / Evidence |
| --- | --- | --- |
| R1154-1 | One shared `command_argument` for the whole `agentic_coding` module, reading the argument key from the client's declared tool schema (the key whose schema describes the shell command), falling back to `command`, `cmd`, `script`, and an array form joined for `shell`-style tools; the seven local copies are deleted. | Implemented: `rust/src/agentic_coding/tool_result.rs` gains the shared `command_argument` (string and joined-array forms) and `command_argument_key` (schema-driven key resolution for callers that hold the request's tool definitions); the seven former copies in `progress.rs`, `file_read.rs`, `tool_result.rs`, `narration.rs`, `command_reroute.rs`, `evidence_record.rs` and `workspace_change.rs` now delegate to it. |
| R1154-2 | Invariant independent of parsing: never plan a tool call whose name and arguments are byte-identical to one already attempted in the current turn window; if the planner would, report the stuck step and its last result instead of repeating it. | Implemented: `Progress::repeated_call` (`rust/src/agentic_coding/progress.rs`) recognizes a completed call by byte-identical arguments or the same canonical command/URL operand (the protocol layer may project `command` onto `cmd` before the transcript echoes the call), and `stop_repeated_call` (`rust/src/agentic_coding/planner.rs`) replaces the repeat with the `stuck_step_report` answer from `data/meta/work-item-steps.lino`. A completed call is the loop signature; a failed one keeps its existing bounded retry and the two-failure stop of issue #1133. |
| R1154-3 | After a successful read, the fetched text drives `plan_work_item_execution` exactly as on the Claude and Agent paths. | Implemented by the shared reading: `issue_view_url` now resolves under `cmd`, the read lands in `fetched_pages`, and `repository_work_item_objective` feeds `plan_work_item_execution` unchanged. Pinned by `codex_cmd_read_drives_execution_instead_of_repeating`. |
| R1154-4 | Tests: replay the Codex transcript's first two Responses items (`function_call` `exec_command {"cmd": …}` + `function_call_output`) through the Responses input conversion and assert the next plan is not another `gh issue view`; a table test over `{command}`, `{cmd}`, `{script}`, `{command:[…]}` yields the same recorded read. | Implemented in `rust/tests/unit/issue_1154_progress_arguments.rs`: the replay goes through `ResponsesRequest::to_chat_completion_request` (the server's own conversion), and the table covers all four spellings plus the twice-repeated transcript shape. |
