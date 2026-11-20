# CodeQL result and extraction checkpoint

Issue #1580 remains unfinished. Keep pull request #1581 draft and unmerged.

The security workflow previously trusted a successful analysis/upload and
allowed a failed Rust sysroot pin to continue. Neither proves complete analysis
or absence of findings. The pinned Rust template already checks generated SARIF;
this adaptation also rejects unsuccessful invocations and error/warning extraction
or configuration notifications. Malformed reports and diagnostics fail closed.
All files and runs are inspected, and findings cannot disappear through suppression
metadata. Diagnostic strings stay on one log line.

The production workflow writes to an explicit report directory, checks that same
directory, and preserves its reports even when the gate fails. Rust toolchain
installation, sysroot discovery and missing standard-library sources stop before
analysis and never export partial paths. Existing main-only conditions, language
matrix, analysis permissions, compiled examples and archive exclusions remain.

## Regression evidence

`rust/tests/web/security-codeql-results.test.mjs` invokes the actual helper and
executes the actual sysroot shell against bounded mock `rustup` fixtures. Eight
groups cover missing/malformed reports, multiple files and runs, findings,
extraction diagnostics, aggregation, log-command injection, workflow output binding
and all four prerequisite outcomes. Seven groups fail against the original state;
all eight pass after repair. These checks do not execute CodeQL itself.

The two existing Rust CodeQL tests move intact into a 98-line module, with the
lost-sysroot assertion updated to require failure. Both the full unit suite and
the existing focused workflow-contract target include it. A separate regression
fails before admission repair and passes afterward: editing either the native
module or its archived excerpt must select native CI. The common fixture reader
is shared; both existing sparse checkout attempts already include these inputs.
Rust formatting passes. Actual native compilation and the two new focused test
executions still require hosted verification.

The initial full gate run failed its abbreviation inventory because ten historical
archive filenames added in the preceding batch crossed the existing ceiling.
Those archives have descriptive names now. Their captured bytes and hashes are
unchanged; only their manifest locators move. No ceiling was raised. The reviewed
full gate run passes all 140 executable-or-explicitly-excluded entries. The final
run after the native admission update also passes; its exact source identities
and complete logs are retained in `final-validation/`.

Evidence is under `experiments/issue-1580/evidence/checks/codeql-result-gate/`.
`before.log.gz`, `native-admission/`, `reviewed-validation/`, `final-validation/`,
`archive-relocations.json` and `upstream-report.json` preserve distinct scopes.
Supplied commands, fixtures, patches and reviews earn zero independent Formal AI
authorship or reasoning credit.

## Existing main findings and limits

The latest main Security run examined here is
https://github.com/link-assistant/formal-ai/actions/runs/44374193717,
head `53cbe4430f8b65330f47728401fc83b09977129f`. It succeeded. Its CodeQL 2.27.4
Rust analysis has zero results; its Actions analysis has two open high-severity
`actions/cache-poisoning/poisonable-step` results, alerts 130 and 131 in
`.github/workflows/hive-mind-draft.yml`. Neither alert is dismissed or suppressed.

The version-matched query's output-name heuristic classifies an immutable trusted
main commit output named `sha` as a possible PR-head source. Reading that model
supports the suspected false positive; it is not an executed query fixture or a
verified upstream fix. The report is
https://github.com/github/codeql/issues/23176. Do not claim a repair from filing it.
Renaming a field to evade the heuristic or excluding the workflow would not
establish the required security property.

The original producer reports were not retained by that workflow. API-exported
SARIF lacks invocation metadata; this normalized export cannot establish whether
the producer included it. Running the new gate against that export refuses its
missing metadata and both real findings. This is diagnostic evidence, not a
successful live gate execution. Original producer-report compatibility, complete
extraction, finding resolution and main publication remain unverified.

## Latest published registry batch

All 23 workflows for `c5b6850813d20fa7d3d5e9f4a496b6b8c5220009` finish: 22 pass;
the pipeline fails its real registry CLI fixture, then correctly propagates that
failure. Aggregate raw lines 29740 and 29749 identify the version-floor case and
missing `docs/status.md` Git path; lines 42919–42920 are the terminal cascade.
The fixture correction is local commit `14ce8c0995b25024639b860443bd6e315e99023d`.
Its new regression fails before and passes after adding the writer's three actual
status surfaces. No local native build ran. The corrected eight CLI cases still
need hosted execution.

All existing native contract families pass on that published head: workflow 102,
coding 32, reasoning 141, interfaces 78, benchmarks 11 and Hive 20/10/7. The shared
registry helper passes three actual native tests; six of seven actual CLI tests
pass. The pipeline's 124 source-size warnings remain acceptance work; extracting
the CodeQL tests is no proof that every warning is resolved.

The Rust template review now covers 180/373 pinned files and 26,781 literal source
lines. The 193 unread files and all unverified applicable practices remain open.
Strict original-source replay coverage stays 31/53; the three historical budget
criteria, broad template/runtime/delivery requirements and independently accepted
Formal AI reasoning (0/20) remain unfinished.
