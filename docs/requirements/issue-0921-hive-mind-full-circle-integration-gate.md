## Issue #921 Hive-Mind Full-Circle Integration Gate

Issue [#921](https://github.com/link-assistant/formal-ai/issues/921) closes E74
after upstream Hive Mind added native Formal AI dispatch. The complete evidence,
root-cause analysis, and regeneration protocol are in
`docs/case-studies/issue-921/`.

| ID | Requirement | Status / Evidence |
| --- | --- | --- |
| R921-1 | Prove the exact Hive Mind Agent/Formal-AI invocation and run its production executor through the real Agent CLI and candidate Formal AI server. | Public `solve` command preparation selects `agent --model formalai/formal-ai --verbose`; the shipped `executeAgentCommand` then creates and commits the byte-exact Hive Mind directional effect. |
| R921-2 | Dispatch an external Agent CLI from Formal AI on a hive-mind-shaped issue task. | Public `formal-ai agent run` executes the extracted acceptance payload through the installed Agent CLI and live Formal AI server; its exact effect and fixture commit are preserved. |
| R921-3 | Preserve deterministic, replayable proof rather than log-only success. | Both directions record exact result bytes, patches, and full commit IDs; the reverse direction commits canonical session JSON whose hash-chained events replay and include the workspace effect. |
| R921-4 | Propagate nonzero Agent failures honestly in both directions. | Exit-23 probes prove Hive Mind returns 23 with no effect commit and Formal AI returns nonzero with a failed session retaining exit code 23. |
| R921-5 | Run the full circle continuously with traceable diagnostics. | The focused regression pins evidence and workflow wiring; release CI installs Hive Mind, runs the harness, and uploads complete raw traces on failure. |
| R921-6 | An unmarked caller preamble must not outrank the objective stated after it, and a policy sentence naming a privileged command must not authorize it. | `plan_chat_step` routes the text after a line-anchored objective delimiter, and `named_shell_command` reads one sentence at a time and skips a clause opened by a seed-declared `policy_lead`. Rungs `R916-09`/`R916-10` judge both from a real workspace; `tests/unit/issue_907.rs` pins the boundary, the seed data, and the inverse imperative case. |
| R921-7 | A dispatched repository work item must read what it names before concluding nothing can be executed, and `planned_not_executed` must survive only for a genuinely unavailable capability. | The composed plan opens with a `Fetch` step for the work item's own URL; once fetched, the plan is re-composed from the issue text and the existing execution routes act on the artifact it names. `tests/unit/issue_904.rs` pins the read-first order, the executed artifact, the unavailable-capability case, and that an artifact is never invented for a work item naming none. |
