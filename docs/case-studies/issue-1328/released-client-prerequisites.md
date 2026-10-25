# Released client prerequisites and admission race

Release [v0.355.9](https://github.com/link-assistant/formal-ai/releases/tag/v0.355.9)
was published from `43fa87e4a0dd7745737be9e41237fabde05fef8b`, the guarded
version-only child of merged PR 1407. Its main pipeline, desktop/native
publication, browser/source-network/VS Code publication and public-download
verification all passed. Independent metadata checks verified the native
attestations, macOS ad hoc signing receipts, full/slim multiarchitecture GHCR
images, their version/default aliases, and the Docker Hub mirror. This did not
establish original-client coding acceptance.

Fresh released-image execution
[40173640886](https://github.com/link-assistant/formal-ai/actions/runs/40173640886)
used the published immutable full image and shipped binary. All four native
PID1 memory lifecycle cycles passed, including supported migration, unchanged
restart, incompatible-original preservation, exact answer recording and active
connection shutdown. The receipt explicitly does not claim default DinD
entrypoint lifecycle coverage.

## Observed causes

Agent passed the default-token permission probe, then failed before claiming a
task. Scheduled proposal-only learning execution 40176454890 started after
admission. Fresh revalidation treated that unrelated proposal generator as a
new required release check. Its generic false/true assertion omitted the
workflow and reason. Proposal generation is read-only and has no push or PR
trigger; it is not a release producer or code validation gate.

Codex and Claude reached their real claims after that learning execution
completed. They created owned issues 1409 and 1408 using `GITHUB_TOKEN`; no
additional automation credential was needed. Both then stopped before an AI
working session. The actual published image is built on `box-dind`, while the
earlier candidate task image used Hive's larger base. The published runtime
tool manifest installed Agent and launchers, but omitted Claude, Codex and
Playwright MCP. Hive's normal MCP preflight reported missing client binaries
and an unavailable MCP package. These observations do not establish a coding
or model failure.

The child harness put its useful diagnostic in stdout and left stderr empty.
The replay executor retained the text artifacts but printed an uninformative
exit-code assertion. Small authenticated UTF-8 evidence archives exposed the
actual missing-prerequisite error; no image or executable was downloaded to the
local machine.

## Repair and verification contract

The shared runtime manifest pins the clients to the versions used by the
repository's CLI checks. Installation retains Bun's existing lifecycle trust
policy. The pinned Claude package ships a placeholder executable; its reviewed
`install.cjs` links the installed matching native optional dependency. Only
that official linker is explicitly run, once within the setup deadline.
The first ARM64 image check also showed Bun's existing executable link pointing
directly at the official platform binary. The installer hard-links the generic
package executable to that same file. Accept this identical filesystem object
as well as the canonical path, preserve the existing link, and continue to
refuse a different executable even if its current bytes happen to match.

Both images check actual client versions and local, pinned MCP registration.
The same MCP package is linked into the discovered npm global directory so
Hive's Agent package-availability probe can find it. This is one package
instance, not a second global installation. Browser navigation is not claimed
by those setup checks. The final task-image verifier additionally executes all
three actual client version commands and the original pinned Hive MCP
preflight before archive transfer or task issue creation. Its owned probe
container is removed on success or failure.

One shared policy excludes only the read-only learning workflow's schedule and
manual proposal events. Actual push/PR checks, real publishers and unknown
scheduled workflows remain required. Missing, pending, failed, foreign-source
or incomplete authority continues to refuse execution. Denials include the
blocking run, attempt, path, event and status. Replay command failures include
both output streams after secret/account redaction and bounded truncation;
uncertain cleanup, signals and timeouts still fail without a command retry.

Regression controls replay the observed post-admission proposal race, the
native placeholder, absent clients/MCP, unhealthy registration, global npm
prefix discovery and stdout-only failure. Native execution, dependency
installation and Docker builds remain CI-only. Local checks use source,
metadata, temporary filesystem fixtures and mocked commands.

The failed effect owner is terminal. Its existing scheduled finalizer
preparation was requested once for bounded recovery because GitHub's
workflow-run depth limit prevents another completion-chain level and the
independent schedule was delayed. Verify owned issue/branch cleanup from that
trusted outcome. That scheduled recovery retained both issues and all three
client rows without preserving the refusal reasons. A source-level read-only
replay recognized the two actual owned fixtures, while Agent's failed
pre-claim check was incorrectly treated as an ambiguous task. This difference
does not establish a permissions failure or justify new credentials.

The default-token PR audit in
[40182009633](https://github.com/link-assistant/formal-ai/actions/runs/40182009633)
then exposed the exact transport difference: the hosted GitHub CLI rejected
all three raw logs because they contained terminal escape sequences. The
local older CLI allowed them. No issue write was reached. For captured log
data only, the shared reader probes local CLI help once and uses its supported
[`--allow-escape-sequences`](https://cli.github.com/manual/gh_api) option.
The API read is never retried for this negotiation, existing transfer/time
bounds remain, and raw bytes are parsed or hashed rather than printed to a
terminal. Legacy CLIs keep their original argument list. Tests emulate both
versions and reject the raw opt-in for writes or arbitrary endpoints.
The qualified native-data collector uses the same captured-log reader. Complete
UTF-8 text is decoded after joining retained buffers so a multibyte character
at their boundary is preserved.

The repair recognizes the terminal failed read-only claim check followed by
skipped persistence and solve gates as no task effects. Incomplete logs,
unknown gates or any created resource still refuse that classification.
Retained cleanup receipts now include bounded transport/ownership reasons.
An explicitly selected owner can also be audited in a same-repository PR or
manual workflow using only the default token's read permissions; the audit
cannot enter either mutating cleanup job and grants no cleanup credit. The
audit explicitly uses Bash pipefail so an incomplete JSON result cannot become
a successful workflow merely because its final `tee` command succeeded. The
existing immutable cleanup claim continues to prohibit automatic repetition
after uncertainty. Use the audit to diagnose the actual hosted difference
before another recovery decision.

Never rerun the effectful Original workflow. Require a fresh
execution and actual released-image client success before closing issue 1364.

## Hive Mind source-callback failure

The bounded shared collector also retrieved the original client logs from
[Hive Mind run 40124638032](https://github.com/link-assistant/hive-mind/actions/runs/40124638032).
That run explicitly used Formal AI 0.355.7. Claude job 120963507697 and Agent
job 120963507679 returned a complete `workspace-source-v1` observation with
`success: true`, `absent: true` and `error_code: ENOENT`. Their real callbacks
contained stdout without a shell exit code. The unchanged current recipe
continuation required exit zero before decoding that source observation, so
it reported a failed write and stopped before creating the program.

A focused JavaScript replay reproduced that failure on current source. The
source constructors now decode their own complete semantic receipts after
exact command/callback binding and harness-failure checks. Missing process
status remains unknown. Explicit nonzero status, error callbacks, partial or
truncated output, signals, wrong paths and duplicate bindings still refuse
source evidence. Source updates retain preimage validation and force a new
exact-byte read before any artifact credit. Existing source mutation epochs
still invalidate an earlier read. Both Rust and JavaScript use this boundary;
native execution is checked only by CI. No task solution or recorded answer
was added to production dispatch.

The full API replay also exposed raw Git porcelain status losing its first
column through a presentation normalizer. Its exact status grammar now parses
the raw bytes first in both runtimes; structured transport fallback still
rejects explicit failures and incomplete observations.

Codex job 120963507681 in the same Hive run delivered and committed its source,
then stopped with `no checks reported` for a staged `ci/` workflow. Hosted
checks cannot be assumed for these default-token task PRs. The source-owned
continuation now observes readiness separately: it verifies the exact staged
workflow at the clean, pushed current head, reads the live same-head PR, and
makes one ready transition if needed. A repeated run only reads already-ready
state. Its distinct receipt reports zero hosted checks and `ciVerified: false`;
it cannot satisfy the unchanged normal hosted-CI decoder. The final report
keeps workflow verification pending for the original Hive harness, which must
still execute the staged workflow and every original delivery assertion.

The follow-up batch also regenerates the source-selector and requirements
derivatives, adds the three publisher registry answers without relabeling the
older snapshot cohort, and records the audited CLI compatibility pins against
issue 1364. The read-only audit supports standard bot checks-mode dispatch and
excludes E2E base branches; neither dispatch mode can enter mutating cleanup.

A final older-head native shard exposed an order-dependent Dockerfile test:
it required the recovery helper to be adjacent to the COPY destination. The
real image matrix passed, including the newly added configurator. The test now
reads the continued COPY instruction and requires every runtime helper in that
closure independently of source order. Runtime and build probes stay required.

The same owner exposed a second deadline problem before the next execution:
Codex job 121114230265 used 28 minutes 13 seconds before starting its bounded
21-minute solve; Claude job 121114230383 used 27 minutes 26 seconds. The old
30-minute parent therefore could not preserve that solve budget. Only the
delivery job now has a temporary, explicit 55-minute CI-speed exception for
observed setup, solve and five minutes of evidence reserve. Every inner limit,
single-attempt rule and other job cap stays unchanged. A regression uses the
actual public timestamps. Remove this transfer debt once the source-bound image
handoff and full solve fit within 30 minutes; no duration record was invented.

The final native lint run found Clippy's case-sensitive-extension diagnostic
in the new staged-workflow selector. Splitting at the final dot preserves its
exact lowercase `yml`/`yaml` predicate without that lint violation. No workflow
verification or readiness condition changes. Native validation remains a CI
obligation after the final push.

Read-only repository settings on 25 October also confirmed that Formal AI and
Hive Mind both use `default_workflow_permissions: write` with
`can_approve_pull_request_reviews: true`. Their existing default-token policy
supports the task transport; no repository setting or credential was changed.
