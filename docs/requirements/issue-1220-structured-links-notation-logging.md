## Issue #1220 Structured Links Notation Logging

Authority: [issue #1220](https://github.com/link-assistant/formal-ai/issues/1220), fetched 2026-10-15T13:56:10.259Z. Coordinate one implementation and shared format with link-foundation/dedup-log and Hive Mind; verify actual dependency capabilities before choosing an adapter.

| ID | Requirement and acceptance | Status |
| --- | --- | --- |
| R1220-1 | Provide one tracing-compatible structured logging API/layer with typed Links Notation output: RFC3339 nanosecond time, trace/debug/info/warn/error level, component, session/dialog id, message and typed fields. Every emitted line must parse with the actual Links Notation codec. Shared dedup/lazy format semantics must be compatible with Hive Mind. | Recorded requirement; implementation and acceptance pending. |
| R1220-2 | Serve and serve --agent-mode default to debug plus verbose dialog/plan/tool/terminal-state events on stderr and ~/.formal-ai/logs/serve-<pid>.lino. Respect --silent, FORMAL_AI_SILENT and FORMAL_AI_LOG_LEVEL as explicit reductions or configured levels; a quieter implicit default cannot hide normal diagnostic evidence. Preserve protocol stdout and tool stdout byte contracts. | Recorded requirement; implementation and acceptance pending. |
| R1220-3 | At debug, large prompts/files/payloads are lazily represented by hash and byte size; full payload evaluation/output is trace-only under explicit custody/redaction. Test that suppressed fields are not eagerly computed and that deduplication does not erase event identity, order or terminal evidence. | Recorded requirement; implementation and acceptance pending. |
| R1220-4 | Migrate new dialog logs to .lino through the same API, keep a reader for existing .jsonl evidence, and provide formal-ai logs --format lino,json,pretty conversions with typed roundtrip/refusal controls. Retain old failures and their exact provenance. | Recorded requirement; implementation and acceptance pending. |
| R1220-5 | Log every terminal result and reason, including success, planned_not_executed, repeated-call guard, tool failure, cancellation, timeout, refusal and Unknown. A quiet/exit0/no-write outcome must not masquerade as a successful task or PR delivery. | Recorded requirement; implementation and acceptance pending. |
| R1220-6 | Redact secrets at the shared logger boundary, including FORMAL_AI_API_KEY, GH_TOKEN and GITHUB_TOKEN in nested fields/messages/errors. Test adversarial multiline/structured cases without reading real credentials. No secret values enter normal logs or public provenance attachments. | Recorded requirement; implementation and acceptance pending. |
| R1220-7 | Migrate ad-hoc eprintln! diagnostics through the logger and lint against remaining bypasses outside its central implementation. JavaScript first follows standing doctrine; retain equivalent typed semantics and subsequent Rust/native parity without claiming an unexecuted target passes. | Recorded requirement; implementation and acceptance pending. |
| R1220-8 | Bind structured records into the #1213 session chain by hash and have Hive Mind retain the log with the session for all three clients. Real no-flags serve acceptance must parse actual persisted/stderr Lino events, with silent/level reductions and failure paths separately exercised. | Recorded requirement; implementation and acceptance pending. |

Current boundary: rust/src/dialog_log.rs:38 defaults dialog verbosity on, while rust/src/dialog_log.rs:148 writes JSONL and existing diagnostics use ad-hoc stderr. js/agentic/node-host.mjs:177 supplies JavaScript host integration; no current source reading establishes the new typed logger/default-debug/native acceptance.

### Integration boundary clarifications

Use one canonical typed codec and shared hash backend with provenance. Existing seed parsers lose some quoting/types and are not sufficient evidence of this contract. Hash the logical canonical record after secret redaction, before physical deduplication, preserving codec/dictionary references and availability. Raw exact-message commitments and protected replay availability remain separate from public redacted logs. The audited dedup-log live logger prints timestamp/level text and JSON objects; closed-file Lino deduplication alone does not satisfy immediate typed Lino stderr and serve-PID output. Use a compatible structured formatter or sink adapter, preserve lazy evaluation and prove live parsing; a dependency addition alone is not acceptance.

### Exact Hive Mind logging child contract

[Hive Mind #3333](https://github.com/link-assistant/hive-mind/issues/3333) refines existing R1220-2/-5/-8 without adding completion credit. Start serve with the actual debug/verbose defaults, honoring --verbose without passing it to other tools. On both success and failure, retain the matching session time window from in-process serve.log or sidecar docker logs --since <start> --until <end>, together with matching dialog logs. Sanitize all of it, append the fenced Formal AI server log section to the session log, and upload it with --attach-logs. Parse the actual Links Notation using the compatible dedup-log integration, and put observed terminal reasons such as planned_not_executed/repeated_call_guard/errors in the PR comment. The Formal AI Draft workflow must also print that server-log section in its step output. Actual claude/codex/agent delivery plus sidecar/in-process controls remain Pending; a private log fixture or stdout summary cannot satisfy attachment acceptance.

### PR #1221 resumed bulk source checkpoint

The resumed PR #1221 source adds typed .lino dialog/conversation persistence with legacy JSONL readers, JavaScript logs conversion, actual terminal-reason observations and a shared redacting native diagnostic sink. The native bypass guard is wired into hosted CI; native writers validate reader roundtrip before append. Controls are in rust/tests/web/server-dialog-lino.test.mjs, rust/tests/web/server-terminal-events.test.mjs and native dialog source tests. Actual native compilation/runtime, a tracing subscriber, complete shared dedup/lazy policy, native conversion, chain integration and Hive Mind log attachments remain Pending. Full acceptance statuses above remain unchanged. All issues and subissues share [one delivery scope](../case-studies/pull-request-1221/scope.md).


### Current CI evidence and self-coding clarification

The [CI self-coding evidence contract](doctrine-self-coding-ci-evidence.md) clarifies the
existing useful-change and complete-log requirements. Historical plan-only
attempts retain their failure classification; source assistance is not
autonomous implementation. Console recovery and source-qualified native log
mounting are independently tested. Hosted capture remains unverified until a
real run's artifacts and completeness manifests are inspected.
