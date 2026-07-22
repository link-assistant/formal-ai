# Issue 822: complete agentic error reports

- Issue: <https://github.com/link-assistant/formal-ai/issues/822>
- Pull request: <https://github.com/link-assistant/formal-ai/pull/823>
- Solver failure notice:
  <https://github.com/link-assistant/formal-ai/issues/822#issuecomment-5042705154>

## Summary

Before this change, the report planner immediately built a bounded Markdown
summary from user and assistant text. It omitted tool-call inputs/results,
harness metadata, and the matching server trace. Complete JSONL exchanges were
available only when an opt-in environment variable enabled dialog logging, and
there was no API or CLI for a harness to retrieve them.

The implemented flow now asks what the user wants before collecting context.
GitHub reports ask a second question about which logs to include. Structured
question tools are preferred when advertised; other clients receive the same
localized choices as plain text. Only after confirmation does the planner
export a harness log, fetch a matching server log, submit context to Formal AI,
or file a GitHub issue.

GitHub bodies embed the complete Links Notation payload when it fits. Larger
payloads retain a bounded preview of the latest complete context lines and link
a secret gist containing the full `.lino` document, keeping the issue under
GitHub's body limit without losing context.

## Root cause

Three independent gaps combined into the incomplete reports:

1. The report recipe treated `Report` as an unconditional GitHub action. It had
   no destination/content state and serialized only selected chat text.
2. Server tracing and exchange persistence were opt-in. Even a new exporter
   could not reliably retrieve a complete matching trace.
3. The existing JSON formatter served a cache-specific `entry` shape. It had no
   general repeated-record sequence encoding, and no OpenCode SQLite adapter or
   conversation-context endpoint existed.

## Requirement evidence

| Requirement | Implementation and regression evidence |
| --- | --- |
| R0: confirm before collection | `report_issue` is a two-stage state machine. Unit tests cover structured and plain questions, all supported languages, no pre-confirmation `curl`/`gh`, and all four destinations. |
| R1: automatic full context | Dialog exchanges are persisted by default under a stable id derived from the first user prompt, while an explicit safe harness id is preserved verbatim. `context export` merges cumulative or incremental request histories with final and streamed server responses in append order. |
| R2: matching server logs | Conversation export includes every matching exchange record under `server_logs` in physical JSONL order; the report command selects `harness`, `server`, or `both`. |
| R3: `/api` source of truth | `GET /api/formal-ai/v1/conversations/{session}` (plus the `/v1` alias) returns context. `POST .../{session}/learn` submits the same context for learning. |
| R4: Links Notation default | The API and CLI default to `.lino`; `?format=json`/`--format json` are explicit opt-ins. Canonical-parser regressions verify repeated `message`/`part` records, colon-safe quoting, delimiter choice, `b64:` fallback, and physical-line escaping. |
| R5: verbose by default | Server request tracing, full response/thinking output, proxy bodies, and dialog persistence are enabled by default. Global `--silent` disables them. |
| R6: arbitrary JSON conversion | `formal-ai context json-to-lino --path ...` uses the general native serializer; integration coverage invokes stdin and file paths. |
| R7: deterministic OpenCode adapter | The shipped stdlib-only extractor opens SQLite with `mode=ro`, orders messages/parts by `(time_created, id)`, and preserves full JSON metadata. It emits JSON for the CLI to pass through the shared Rust LiNo serializer, eliminating formatter drift. A fixture test verifies determinism, byte-identical serialization, and that the database remains unchanged. |
| Whole flow | `run_issue_822.sh` drives the real Agent CLI through destination and contents confirmation, executes a PATH-local fake `gh`, and verifies the resulting issue carries inline or linked complete transcript and server logs. |

The review follow-up makes R3 operational rather than archival. Symbolic
responses containing a verified unknown-rule candidate now expose structured
`learning_trace` metadata. `POST .../{session}/learn` recovers those events from
the full report, runs `learn_rules_from_unknown_traces`, and returns staged
proposal counts while explicitly leaving promotion awaiting human review. A
subsequent exact repeated failure queries the approved `LearningLedger` before
attempting fresh rule synthesis.

The same follow-up adds a bounded failure-driven recursive executor: try the
whole node, descend after failure, extend and retry one unsupported elementary
leaf, then climb back up and rerun the parent test. The executable evidence tree
and four focused regressions are under
[`recursive-self-coding/`](recursive-self-coding/README.md).

## Interfaces

```text
formal-ai context export --session SESSION --source both
formal-ai context export --session SESSION --source opencode --db ~/.local/share/opencode/opencode.db
formal-ai context json-to-lino --path input.json --output output.lino
curl http://127.0.0.1:3000/api/formal-ai/v1/conversations/SESSION
curl 'http://127.0.0.1:3000/api/formal-ai/v1/conversations/SESSION?format=json'
```

The conversation endpoint and harness commands share
`load_conversation_context` and `conversation_context_to_lino`, so a session id
resolves to one representation rather than independent report serializers.

## Reproduction and verification

The original failing unit run is preserved as
[`test-logs/reproducer-before.log`](test-logs/reproducer-before.log). It records
seven failures: immediate issue creation, no context route, non-native JSON
sequences, opt-in logging, missing global verbosity controls, no general
conversion command, and no installed OpenCode adapter.

Focused post-fix evidence is in
[`test-logs/implementation-slice-4.log`](test-logs/implementation-slice-4.log),
[`test-logs/issue-822-unit-5.log`](test-logs/issue-822-unit-5.log), and
[`test-logs/compatibility-suite.log`](test-logs/compatibility-suite.log). The
compatibility run covers the earlier #687, #714, and #771 reporting contracts.

The final `cargo test --all-features --verbose` run passed 1,965 tests with two
intentional ignores and no failures; its complete 3,182-line output is in
[`test-logs/cargo-test-all-features-final.log`](test-logs/cargo-test-all-features-final.log).
Formatting, Clippy with warnings denied, rustdoc, file-size, terminology,
language, closure, package-content, web, VS Code, desktop, worker, release, and
self-AST checks are preserved in the other `test-logs/*-final.log` files. The
package listing confirms that `scripts/opencode-conversation-to-lino.py` ships
in the crate.

The optimized binary then passed both real-client scenarios. The issue #822
script verified that no `gh` command ran during either question and that the
confirmed report contained both logs. The #714 compatibility script verified
that the earlier agentic tool-call learning path still works. Their concise
results are in
[`test-logs/agent-cli-e2e-final.log`](test-logs/agent-cli-e2e-final.log) and
[`test-logs/issue-714-agent-cli-compatibility-final.log`](test-logs/issue-714-agent-cli-compatibility-final.log).
The full client log, server trace, fake-`gh` argv, rendered issue body, and
linked Lino payload are under [`agent-cli-e2e/`](agent-cli-e2e/).

## Self-coding evidence

The required live self-coding command was attempted before manual work:

```text
examples/self-coding/run.sh --live https://github.com/link-assistant/formal-ai/issues/822
```

The wrapper rejected `formal-ai` as an available Agent model before a session
could start and automatically posted the linked failure comment. The complete
captured output is preserved in
[`raw-data/self-coding-live.log`](raw-data/self-coding-live.log). Because no
Formal AI session authored the original implementation, those commits
intentionally carry no self-authorship trailers.

The follow-up CI repair was subsequently authored through the local Formal AI
server and Agent CLI. Its session, failing regressions, applied fixes, and
scope boundaries are recorded in
[`self-hosting-fix/README.md`](self-hosting-fix/README.md).

After the broad-ambition review, a second real local-server run used the
external Agent CLI to author and verify leaf L5 of a five-leaf recursive
decomposition. The captured session and 20% measurement are in
[`recursive-self-coding/`](recursive-self-coding/README.md); only its generated
artifact/evidence commit carries the paired self-authorship trailers.

## Captured inputs

`raw-data/` contains the issue, paginated issue comments, prepared PR metadata,
all three PR comment/review channels, the supplied gist API snapshot, the most
recent related merged PR reviewed during implementation, the live self-coding
failure, and primary-source research notes. The issue and comments contained no
screenshots or image attachments.
