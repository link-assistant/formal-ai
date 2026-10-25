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

The repair recognizes the terminal failed read-only claim check followed by
skipped persistence and solve gates as no task effects. Incomplete logs,
unknown gates or any created resource still refuse that classification.
Retained cleanup receipts now include bounded transport/ownership reasons.
An explicitly selected owner can also be audited in a same-repository PR or
manual workflow using only the default token's read permissions; the audit
cannot enter either mutating cleanup job and grants no cleanup credit. The
existing immutable cleanup claim continues to prohibit automatic repetition
after uncertainty. Use the audit to diagnose the actual hosted difference
before another recovery decision.

Never rerun the effectful Original workflow. Require a fresh
execution and actual released-image client success before closing issue 1364.
