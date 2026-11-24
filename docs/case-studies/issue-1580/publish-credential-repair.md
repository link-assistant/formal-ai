# Cargo publication credentials and native admission

The publishing helper in [PR #1581](https://github.com/link-assistant/formal-ai/pull/1581)
forwarded its selected token to Cargo with two process arguments. It also warned
that credentials were absent before Cargo could try a configured credential
provider. An empty command-line token suppressed the environment fallback.

Selected nonempty tokens now reach Cargo through CARGO_REGISTRY_TOKEN. The
existing command-line option and native/legacy environment precedence remain.
An empty override falls back; no explicit token leaves Cargo's provider handling
available and prints a neutral message. A rejected credential still returns
failure with publish_result=auth_failed.

The five new process regressions run the compiled helper against a recording
Cargo command. Synthetic fixture tokens cover precedence, empty overrides,
argument privacy, premature warnings and authentication failure. A hashed copy
of the original helper proves the former token argument and premature warning.
The fixture is compiled inside the existing locked release-registry harness.
No package or lockfile dependency is added. The native admission classifier now
covers that entire harness directory, including existing modules and fixtures
which previously could change without selecting native CI.

Run the small JavaScript contract with:
node --test --test-concurrency=1 experiments/issue-1580/native-regression-groups.test.mjs

Its new case fails against the original helper and passes after repair; all
66 cases pass. Two Python files parse, and both recording-Cargo stub controls
run locally. Eleven affected JavaScript gates, whitespace checks and rustfmt
with edition 2024 pass. The first formatting attempt used the wrong default
edition; its failure remains retained separately from the corrected check.
The earlier admission-contract failure is also retained alongside its repair.

The native regression command is:
python3 experiments/issue-1580/release-registry/cli_regressions.py

That command compiles the harness and belongs in the existing hosted native
job. It was not run locally. The five new native process methods, actual Cargo
credential-provider behavior and live publication authority remain unverified.
The recording stub does not implement a real credential provider.

The [receipt](../../../experiments/issue-1580/evidence/checks/publish-credential-repair-20261123/manifest.json)
retains original/current source, failed drafts, complete local logs and source
bindings. Every archive member was reopened and checked against its byte length
and SHA-256 hash. Hosted qualification and the broader original acceptance
criteria remain unfinished; the PR stays draft and unmerged. Supplied work earns
zero independent authorship credit.

## Bound the release credential probes

The five-minute release preflight job sets aside up to two minutes for checkout.
Its credential script retried transient HTTP results, but an unresponsive
server could use the remaining job time in one curl call. The other credential
probes and the aggregate verdict would then be lost to cancellation.

Each curl attempt now allows three seconds to connect and five seconds total.
With the existing three attempts and two five-second delays, seven exchanges
can consume at most 175 seconds of request time and backoff. The checkout bound
and credential verdicts are preserved. The Bash script still works from the
existing one-file checkout. Legacy crates.io token authority remains unknown.
Trusted publishing would require a crate owner to configure the workflow and a
successful official token exchange from that workflow.

Run `node --test --test-concurrency=1 rust/tests/web/release-preflight-checkout.test.mjs`.
The loopback fixture stalls the first public crate lookup for seven seconds and
returns HTTP 503 on retry. The old script waits for the late HTTP 200, skips the
retry and fails the assertion. The repaired script retries, reports the crate
as unknown, verifies and cancels a GHCR upload session, reports Docker Hub's
HTTP 403 and blocks publication. All three tests pass. Bash syntax and
ShellCheck 0.11.0 also pass. The request fixture uses only synthetic credentials.
The [complete before log](../../../experiments/issue-1580/evidence/checks/preflight-deadlines/preflight-before.log.gz)
and [passing log](../../../experiments/issue-1580/evidence/checks/preflight-deadlines/preflight-after.log.gz)
retain both results.

## Compare the deadline with current templates

The current [Hive Mind guide](https://github.com/link-assistant/hive-mind/blob/18258198fe16ee19e80cb5586a48ae41134fc81e/docs/CI-CD-BEST-PRACTICES.md#16-prove-you-can-publish-before-you-build)
requires the preflight to report all failures and retain an unknown result on
timeout. JavaScript, Rust, Python and C++ bound their curl probe attempts. The
PHP HTTP helper also sets a timeout. In C#, the NuGet requests have a 20-second
abort signal. The earlier GitHub repository request and JSON body have no
caller deadline, and the later checks wait for both to finish.

The [pinned C# source](https://github.com/link-foundation/csharp-ai-driven-development-pipeline-template/blob/10edf2e5268f62d905fe116d28d9ad6662a23d25/scripts/preflight-credentials.mjs#L38)
matches Git blob `2eeab16623ad3a381730a81323963ecc8b55b561`.
With an extracted copy of that repository, run
`CSHARP_TEMPLATE_ROOT=/path/to/extracted-template bun test --timeout 4000 experiments/issue-1580/csharp-preflight-deadline.test.mjs`.
The healthy case passes. Both real loopback stalls delay the next probe for the
entire 900-millisecond fixture window and fail the deadline assertion. A local
candidate passes a signal to the GitHub fetch and reports an unreadable JSON
body as unknown. The 150-millisecond fixture signal lets all three cases pass
with Bun 1.4.4 and Node. [Original run](../../../experiments/issue-1580/evidence/checks/preflight-deadlines/csharp-preflight-bun-before.log.gz)
 · [Candidate patch](../../../experiments/issue-1580/evidence/checks/preflight-deadlines/csharp-bounded-probe.patch.gz)
 · [Passing run](../../../experiments/issue-1580/evidence/checks/preflight-deadlines/csharp-preflight-bun-after.log.gz)
