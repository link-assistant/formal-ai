# Released client history, feedback and protected fixture cleanup

PR 1417 published v0.355.11 from prepared source
`c4137c0e722f07dace2b9dd8bad965c159c09f3e`. Main CI, configured publishers,
public download verification and full/slim multiarchitecture container checks
passed. These are publication results, not successful original-client delivery.

The actual released-image owner
[40274202613](https://github.com/link-assistant/formal-ai/actions/runs/40274202613)
failed after all three clients produced first-pass ready pull requests. PR 1418
adds the following fixes without rerunning that owner or weakening its task,
source, exact-output, no-op, retry or independent verification assertions.

## Authoritative wire history

Claude's endpoint was `/api/anthropic/v1/messages?beta=true`. The auditor matched
the complete path suffix and therefore misclassified the protocol. Protocol
selection now uses the URL pathname. Query parameters remain transport metadata.

The persisted conversation view also overlaps cumulative request histories.
Claude's 18 exchanges emitted 17 tool calls, but its aggregated view contained
153 calls; Agent's failure reported 43 against 17. The final complete HTTP
request contains the actual tool history. Normalize that request with the same
production protocol parsers as the server; use recorded accumulated history
only for an explicitly incremental Responses request. Compare every call's
ordered name and canonical arguments against independently recorded model
emissions, then bind every receipt and enforce the existing repeated-failure
budget. Responses custom tools carry literal text, so their arguments must not
be assumed to be JSON.

Offline source-only audits of the complete sanitized Claude first pass and
Codex first/second passes now bind respectively 17, 17 and 8 calls and receipts.
Codex's second pass contains no source mutation calls. This does not substitute
for a fresh released-client execution. Small declarative wire projections retain
actual protocol paths, call/receipt identities and argument equality; large
argument strings have declared content-hash projections. Their provenance names
the exact public artifacts. The tests never execute the captured commands.

## Session notices are not review instructions

Codex's second pass exited 5 after interpreting Hive Mind's own generated
working-session summary and session-start notice as change requests. The exact
sanitized successful tool receipt is preserved in
`rust/tests/fixtures/agentic/hive-own-session-feedback.json`.

JavaScript and Rust now read the same complete framework framing patterns from
`data/meta/work-item-steps.lino`. Only a notice with authenticated
`viewerDidAuthor: true`, recognized complete framing and no created edit is
bookkeeping. Account names are unnecessary and absent from the public fixture.
Other authors, missing authorship, edited or appended notices, ordinary bot
feedback and `CHANGES_REQUESTED` reviews still require action. Failed or
incomplete tool receipts still refuse readiness. Ignoring bookkeeping provides
no credit for the notice's verification claims: source, execution, CI and no-op
proof remain independently required.

The shared keys use full English words and hyphens:
`pull-request-session-summary-pattern` and
`pull-request-session-started-pattern`. The first hosted coverage attempt caught
the underscore/abbreviation form through the existing notation gate
([job 121559007224](https://github.com/link-assistant/formal-ai/actions/runs/40318587888/job/121559007224)).
Correct the names in both readers and the mirrored data; do not raise the
notation ceilings. Source formatting, package-data mirroring and the notation
gate are required source-only preflight checks for this change.

The maintained TypeScript twin must also be regenerated through
`scripts/translate-es.mjs` when this JavaScript logic changes. Hosted coverage
[job 121570546692](https://github.com/link-assistant/formal-ai/actions/runs/40321904871/job/121570546692)
caught its stale feedback predicate. Regenerate from the canonical JavaScript
through the existing layout-preserving token-tree/meta projection, then check
the entire maintained tree and actual TypeScript worker/planner inputs. Keep
the missing/tampered-input refusals and the ban on owned JavaScript fallback.
This source projection does not establish unrestricted runtime feature parity.

The existing full-API two-pass regressions now return these actual notices
instead of an empty comments list for all three protocol clients. They still
require unchanged owned source and no second-pass write or commit.

## Preserve repository deletion protection

Trusted cleanup
[40282996263](https://github.com/link-assistant/formal-ai/actions/runs/40282996263)
closed the owned issues and pull requests but retained both branches per task.
The outcome previously hid the individual errors behind `2 !== 0`.

Read-only live metadata confirmed active ruleset `21300712` applies `deletion`
and `non_fast_forward` protection to `~ALL` refs, with no bypass actors. Cleanup
must not weaken that rule or obtain another credential. For a fresh effective
deletion rule, preserve the exact branch head and ruleset identities explicitly
in `cleanup.protectedBranches`. Independently re-read those identities after
closing the owned issue/PR and when acknowledging cleanup. An unprotected branch
still requires confirmed deletion. Changed heads, unknown protection, missing
rules, incomplete closes and uncertain mutations remain failures; no mutation
is retried. Diagnostics identify the failing operation within the existing bound.
The durable per-client cleanup receipt retains the verified protected branch
heads and ruleset identities, so the final outcome does not discard that proof
when it omits intermediate discovery records.

Protected branch residue is not an open test issue and grants no task-delivery
credit. Historical immutable cleanup claims remain retained; do not retry them
or rewrite their failed outcomes.

## Delivery gate

The offline recorded-session replay also needs a bounded source checkout.
[Job 121584796548](https://github.com/link-assistant/formal-ai/actions/runs/40326352911/job/121584796548)
spent almost its complete five-minute job clock fetching the full repository;
GitHub cancelled checkout before any transcript assertion ran. Select only
`experiments/agentic_cli_matrix` through sparse checkout. The pinned checkout
action uses `blob:none` for that selection. Bound checkout at two minutes with
Git low-speed detection inside that deadline, preserving the original
five-minute job budget and every replay assertion. A regression executes all
original transcripts from an isolated harness-only copy with only Bash, jq and
shell utilities available; no GitHub, product binary, compiler or network tool
is available to that test.

Batch these fixes with the coverage and translation corrections in PR 1418.
Require fresh exact-head CI, including instrumented native coverage and native
feedback regressions. The old controller is trusted from the default branch,
so its terminal failed client/cleanup outcomes cannot certify this fix and must
not become a circular pre-merge success requirement. Authenticate their terminal
disposition and require no active publication, real-task owner or cleanup before
advancing main. After merge, independently verify the new release, public
downloads, containers, all three actual clients and trusted cleanup.

Issue 1364 remains acceptance pending, including historical external transport
requirements. The default-token same-repository staged workflow transport does
not prove delivery to those historical external branches. Broader work stays in
the [critical delivery scope](critical-delivery-scope.md).
