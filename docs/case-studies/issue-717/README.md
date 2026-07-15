# Issue 717: CI/CD false-positive, warning, and failure audit

## Executive result

The audit found one reproducible release failure, three repository-controlled warning classes, one warning-policy design problem, two CI false negatives, and several third-party package notices. The release failure was not flaky: the deprecated attestation wrapper parsed an LF-only checksum manifest on Windows using the host EOL (`\r\n`), joined every checksum line into one subject name, and GitHub rejected that subject at 256 characters. The upstream parser fix is in `actions/attest` v4.

The repository changes therefore:

1. use `actions/attest@v4` for both desktop and VS Code checksum manifests;
2. always retain LCOV coverage, use fail-closed Codecov v7 when configured, and use download-artifact v8, removing Node 20 and `Buffer()` deprecations controlled by this repository;
3. configure Git's default initial branch before actions invoke Git, removing repeated runner hints;
4. package the repository license in the VSIX;
5. classify the documented ad-hoc macOS fallback as a notice rather than a false warning;
6. preserve repository-wide hard file-size enforcement while annotating only warning-band files changed by the event; and
7. compare the complete PR or push range so a docs-only final commit cannot hide earlier code changes;
8. classify only the two exact, reviewed Agent CLI 0.25.0 diagnostics while rejecting new dependency stderr; and
9. add policy and behavior regression tests.

## Scope and evidence collection

The audit covered every CI/CD file in formal-ai and complete file inventories of the four requested templates:

- `link-foundation/js-ai-driven-development-pipeline-template`
- `link-foundation/rust-ai-driven-development-pipeline-template`
- `link-foundation/python-ai-driven-development-pipeline-template`
- `link-foundation/csharp-ai-driven-development-pipeline-template`

The exact trees are in `raw-data/*-file-tree.txt`; formal-ai's inventory is in `raw-data/formal-ai-ci-file-inventory.txt`. The Rust template workflow diff is preserved as `raw-data/rust-template-release.diff`. Recent workflow metadata, full logs, stderr, and extracted diagnostics are under `raw-data/ci-logs/` and `raw-data/templates/`. The evidence directory is intentionally large because issue 717 explicitly requires complete downloaded logs rather than excerpts that cannot be independently rechecked.

The investigation queried all three PR feedback channels—conversation comments, inline review comments, and reviews—and the issue comments. PR 729 had no review feedback when the audit began. The required high-level `solve` run failed because `solve` 2.6.0 rejected its configured `formal-ai` model; its real log is `agent-cli-live-run.log`, and the automated failure comment is https://github.com/link-assistant/formal-ai/issues/717#issuecomment-4976770399. Work continued through a locally served Formal AI endpoint and the actual Agent CLI, with its server and stream logs retained beside this study.

That integration exposed a separate general Agent CLI defect: the deterministic edit router tried to edit before reading, while the CLI correctly enforces read-before-write. The router now schedules an advertised read capability first, then the requested edit. Four phrasing/tool combinations cover the state transition, and the real CLI subsequently read and authored the attestation, Codecov, artifact-download, and signing-severity changes. The rejected first attempt and successful sessions are preserved as `agent-cli-attest.log`, `agent-cli-attest-success.log`, `agent-cli-codecov.log`, `agent-cli-download.log`, and `agent-cli-signing_notice.log`.

## Timeline

| UTC date | Event and relevance |
| --- | --- |
| 2026-05-12 | PR 5 removed a Pages deployment false positive. |
| 2026-05-15 | PR 26 fixed Pages subpath handling in live E2E. |
| 2026-05-16 | PR 86 added crates.io retry behavior. |
| 2026-05-19 | PR 122 fixed package-size CI. |
| 2026-06-04 | PR 391 repaired release recovery. |
| 2026-06-13 | PR 443 stopped unnecessary test runs for non-code changes. |
| 2026-06-14–17 | PRs 480, 487, and 510 restored desktop assets and ad-hoc macOS signing; PR 524 addressed runner disk exhaustion. |
| 2026-06-24 | PR 562 pinned Pages deployment to the release commit. |
| 2026-06-28 | PR 583 restored release badge checks. |
| 2026-07-04 | actions/attest issue 440 reported LF checksum parsing failure on Windows. |
| 2026-07-09 | actions/attest PR 443 fixed the parser with `/\r?\n/` and shipped in v4. |
| 2026-07-14 16:58 | formal-ai main run 29351401385 passed but emitted repeated file-size and Codecov Node warnings. |
| 2026-07-14 17:30 | desktop run 29354019108 failed Windows x64 and arm64 attestations with an oversized subject name. |
| 2026-07-15 | issue 717 audit reproduced the policy failures, compared all templates, filed upstream reports, and implemented regression coverage. |
| 2026-07-15 04:53 | first post-push run 29390080057 exposed the multi-commit change-detection false negative: Rust changes in earlier commits were hidden by the final evidence commit. |
| 2026-07-15 05:00 | run 29390577430 exercised every required job successfully, then a complete log scan exposed Codecov's silently failed tokenless upload and Agent CLI 0.25.0 warnings. |
| 2026-07-15 05:33 | run 29391752563 proved GitHub OIDC issuance worked but Codecov rejected the unprovisioned repository; every other required job passed. |

Earlier failed/cancelled runs after the issue opened belonged to the concurrent PR 716 development sequence. Runs 29371095276 and 29372273229 failed tests; 29380834035 failed lint/coverage/tests plus changelog policy; several later runs were superseded and cancelled. Run 29386600637 passed at the final PR 716 SHA. Those are timeline evidence, not latent failures on issue 717's branch. The branch's pre-change run 29387999474 passed.

## Findings and root causes

### 1. Windows build provenance failed

Evidence: `raw-data/ci-logs/run-29354019108.log`, lines 1755 and 3036.

The workflow generated checksum fragments with Node's explicit `"\n"`, so Windows received valid LF-only `sha256sum` text. `actions/attest-build-provenance@v2` invoked an older `actions/attest` implementation that split on `os.EOL`. On Windows that is CRLF, so no split occurred. Every subsequent digest and filename became part of the first subject name, exceeding GitHub's 256-character API limit.

This matches upstream actions/attest issue https://github.com/actions/attest/issues/440 and fix https://github.com/actions/attest/pull/443. The downloaded upstream issue, comments, PR metadata, and current parser source are `raw-data/actions-attest-*` and `raw-data/actions-attest-subject.ts`.

Alternatives considered:

- Emit `os.EOL` from the manifest generator. Rejected: checksum files are portable text artifacts and consumers should accept LF; it would leave the deprecated Node action warning.
- Convert files to CRLF only on Windows. Rejected: platform-specific workaround with the same obsolete wrapper.
- Pass individual subject paths. Rejected: it duplicates the manifest's authoritative artifact selection.
- Upgrade to `actions/attest@v4`. Selected: fixes newline parsing upstream, removes the wrapper, and targets the supported runtime.

### 2. Node 20 and Buffer deprecations

Main run 29351401385 records Codecov's Node warning around lines 7715–7983. Desktop run 29354019108 records attestation Node warnings repeatedly and a deprecated `Buffer()` call in download-artifact v7 at lines 11876–11877.

The fix upgrades the repository-controlled action majors. Setting `ACTIONS_ALLOW_USE_UNSECURE_NODE_VERSION` was rejected because it suppresses the migration signal rather than updating the code.

### 3. Repeated Git initial-branch hints

Every checkout creates a temporary repository before fetching. Without `init.defaultBranch`, current Git announces its future default-branch change. The Rust, Python, C#, and JS templates already use Git's count/key/value environment configuration, which applies before checkout starts. formal-ai adopts the same pattern in both workflows.

### 4. VSIX license warning

The extension declares `Unlicense`, but `vsce` packages from `vscode/` and did not find the root `LICENSE`. The resource preparation step now copies the authoritative root license into the extension package directory before `vsce package`. This fixes the artifact rather than suppressing the warning.

### 5. Expected ad-hoc signing was labeled as a warning

The workflow explicitly supports an ad-hoc mode when Apple credentials are absent and validates the resulting signature. Since this is a successful documented fallback, warning severity was a false positive. It is now a notice with the missing credential names retained for diagnosis. The electron-builder library-validation message is advisory; the required entitlement already exists in `desktop/build/entitlements.mac.plist` and is passed to both child and root signing operations.

### 6. File-size warnings obscured new risk

The hard limits are useful and remain repository-wide. The warning band, however, emitted more than 30 annotations on every run for unchanged files. That makes a new 901-line file hard to notice and turns a preventive signal into baseline noise.

The selected policy computes changed paths from the PR base SHA or push `before` SHA. Only warning-band annotations are filtered; hard-limit and embedded-data violations still scan and fail the entire checkout. If no usable event base exists (for example a manual dispatch), the script safely falls back to the full warning set.

Alternatives considered:

- Remove warning thresholds. Rejected: concurrent edits can cross the hard limit unexpectedly.
- Raise limits. Rejected: it postpones the same problem and weakens reviewability.
- Split every historical near-limit file in this PR. Rejected: broad behavior refactors are unrelated to release correctness and carry substantially higher regression risk.
- Changed-file annotations plus repository-wide enforcement. Selected: preserves prevention while making annotations actionable.

### 7. Per-commit change detection skipped required PR jobs

Fresh run 29390080057 checked out GitHub's synthetic merge commit and logged `Comparing HEAD^2^ to HEAD^2 (per-commit diff of PR head)`. That range covered only this branch's final documentation commit. It therefore emitted `rs-changed=false`, `workflow-changed=false`, and `any-code-changed=false`, skipping the Rust test, coverage, changelog, and E2E jobs despite this PR changing Rust and both workflows. The complete downloaded run is `raw-data/ci-logs/run-29390080057.log`; the decisive lines are 1229 and 1317–1326.

For pull requests, the detector now compares the synthetic merge parents (`HEAD^..HEAD^2`), which is the complete base-to-PR-head diff. For pushes, it uses `github.event.before..HEAD`, covering all commits in the push and avoiding the separate mistake of treating a real merge pushed to `main` as a synthetic PR merge. Missing/zero pre-push SHAs retain the previous-commit fallback. Pure range-selection tests cover all three cases, while the original failure and local full-push result are retained in `change-detection-before-fix.log`, `change-detection-unit-tests.log`, and `change-detection-push-check.log`.

### 8. Codecov reported success after rejecting the upload

Fresh run 29390577430 generated `lcov.info`, but Codecov's final response was `Token required - not valid tokenless upload`. The action still exited successfully because the workflow explicitly set `fail_ci_if_error: false`. Thus the green coverage job proved only local report generation, not delivery of the report.

An attempted fail-closed OIDC fix proved the next boundary in run 29391752563: GitHub issued the requested OIDC token, but Codecov returned `Repository not found` because this repository is not provisioned there. Coverage must not depend on unavailable external state. The job now always uploads `lcov.info` as the required `rust-lcov` workflow artifact. When `CODECOV_TOKEN` is configured it additionally invokes Codecov v7 with `fail_ci_if_error: true`; otherwise it emits one explicit notice instead of attempting an upload that cannot succeed. Explicit `disable_search: true` and `plugins: noop` restrict a configured uploader to the generated LCOV file rather than scanning historical evidence and invoking irrelevant language plugins. A regression scopes all assertions to the coverage job and proves the artifact, configured, and unconfigured branches.

### 9. Agent CLI dependency diagnostics

The Hive Mind replay in run 29390577430 emitted eight AI SDK system-message warnings and four provider compatibility warnings from `@link-assistant/agent` 0.25.0. Its `js/src/session/prompt.ts` intentionally inserts system-role entries into the AI SDK 6 `messages` array without the SDK's required explicit acknowledgment. This call site is owned by the Agent CLI, not Formal AI; the upstream report is https://github.com/link-assistant/agent/issues/279.

The self-coding harness now captures dependency stderr and accepts only the two byte-exact reviewed messages, aggregates them into one GitHub notice linked to the upstream issue, and fails on any other non-empty line. This preserves visibility without allowing known external repetition to obscure new warnings. A behavior regression proves both acceptance and fail-closed rejection; a real local Agent CLI run classified 9 system-message and 5 compatibility instances and completed the replay.

### 10. Third-party npm notices

The desktop and VSIX install logs contain deprecation notices from transitive packaging dependencies (`glob@7`, `rimraf@2`, `inflight`, `boolean`, `prebuild-install`, and `whatwg-encoding`). The direct Electron and electron-builder ranges were already current at audit time. These lines are neither GitHub annotations nor application runtime dependencies, and overriding nested versions can break packaging tools that selected those APIs. They are recorded as upstream dependency debt rather than hidden with npm log-level flags.

## Template comparison and upstream reports

| Template | Finding | Report |
| --- | --- | --- |
| Rust | unchanged file-size annotation; Codecov v5 | https://github.com/link-foundation/rust-ai-driven-development-pipeline-template/issues/93 |
| Python | setup-python v5 and Codecov v4 target obsolete runtimes | https://github.com/link-foundation/python-ai-driven-development-pipeline-template/issues/32 |
| C# | setup-dotnet v4 and Codecov v4; policy test pins the old version | https://github.com/link-foundation/csharp-ai-driven-development-pipeline-template/issues/39 |
| JavaScript | 33 unchanged `no-changelog-comments` warnings in the latest run | https://github.com/link-foundation/js-ai-driven-development-pipeline-template/issues/103 |

The report bodies are preserved in `raw-data/upstream-*-report.md`. The comparison also identified useful practices already present across templates: deterministic Git init configuration, cargo network retry/multiplexing settings in the Rust template, lockfile guards, and resilient Buildx setup. This PR adopts the directly relevant deterministic Git settings. It does not copy unrelated release machinery merely for textual parity; formal-ai already has project-specific release recovery, Pages, desktop, extension, multi-registry, and E2E requirements tested by its existing suite.

## Reproduction and verification

The pre-fix regression log is `raw-data/regression-before-fix.log`. The focused tests require:

- two `actions/attest@v4` checksum attestations and no v2 wrapper;
- an always-retained LCOV artifact plus Codecov v7 explicit-file upload and fail-closed errors when configured;
- download-artifact v8;
- deterministic Git init configuration in both workflows;
- notice severity for the intentional ad-hoc signing fallback;
- license copying before VSIX packaging; and
- changed-file filtering for warning-band findings;
- complete-event change detection for multi-commit pull requests and pushes; and
- fail-closed classification of the reviewed Agent CLI stderr messages.

The post-fix logs record the focused test, all unit tests, format, Clippy, file-size policy, desktop/VSIX tests, and the real Agent CLI session. The first complete Rust pass (`cargo-test-all-features.log`) correctly exposed one integration assertion that still expected edit-before-read; the updated HTTP round-trip assertion passed independently and the complete rerun (`cargo-test-all-features-rerun.log`) finished with 1,575 passed, 0 failed, and 2 ignored. CI run metadata is retained after pushing so timestamps and SHAs can be compared to the final commit rather than trusting stale status.

## Requirements traceability

| Issue requirement | Evidence/result |
| --- | --- |
| Compare all CI/CD files and four templates | Complete file trees, workflow diff, template run logs, and comparison above. |
| Recheck errors, warnings, false positives, and false negatives | Classified in findings 1–10; stale/superseded failures separated in the timeline. |
| Download logs/data | Full raw logs and API metadata under `raw-data/`. |
| Deep root cause, alternatives, libraries, online research | Root-cause and alternatives sections; official action issues, PRs, releases, and source archived. |
| Report matching template issues upstream | Four linked template reports with preserved bodies; Agent CLI diagnostics reported in issue 279. |
| Add diagnostics when insufficient | Existing verbose signing and release diagnostics were sufficient; no always-on debug noise was added. Real CLI/server traces are retained. |
| Test before fix | Failing policy regression captured before action upgrades. |
| Fix everywhere | Both attestation sites and every affected workflow-controlled warning source are covered by assertions. |
| One PR | https://github.com/link-assistant/formal-ai/pull/729 |
