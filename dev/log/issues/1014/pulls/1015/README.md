# Issue #1014 / pull request #1015 evidence archive

This directory is the canonical evidence record for the complete CI/CD audit
requested by [issue #1014](https://github.com/link-assistant/formal-ai/issues/1014)
and implemented in
[pull request #1015](https://github.com/link-assistant/formal-ai/pull/1015).
It preserves the inputs, the exact failing and warning output, the research,
the tests-first proof, the upstream reports, and the final verification.

## 1. Scope and collection method

The issue named twelve runs at default-branch SHA
`98cb3c803a72161a880968647330358c65d9b83f`. All twelve complete logs were
downloaded with `gh run view --log`; the one failed run also has a
failure-only copy. For every run the run, job, and artifact metadata is in
`raw-data/`. The combined status, all 66 check-run annotation responses, issue
and pull-request timelines, comments, reviews, and events were collected
separately so that a green workflow conclusion could not hide annotations.

The five initial #1015 workflows at prepared SHA
`6fd8cdb620a0ddb7f4f8dfc4557f1401df8b8124` were also downloaded after they
completed. Their stored metadata and logs prove the observations predate this
fix. `raw-data/branch-initial-diagnostics.txt` is the corresponding diagnostic
scan.

The archive contains more than 290 files at the time of analysis:

- `ci-logs/run-*.log`: every issue-listed baseline workflow log.
- `ci-logs/branch-initial/`: all initial pull-request workflow logs.
- `raw-data/run-*-jobs.json`, `run-*-artifacts.json`, and annotations: complete
  GitHub execution metadata.
- `raw-data/audited-warning-error-excerpts.txt` and
  `key-diagnostics-with-lines.txt`: indexed candidates, not substitutes for
  the complete logs.
- `raw-data/references/`: current immutable template heads/trees and Hive Mind
  `CI-CD-BEST-PRACTICES.md`.
- `raw-data/related/`: the latest related issues, merged PRs, three GitHub
  comment surfaces per PR, and previous template reports.
- `raw-data/*audit*`, `*versions*`, and Gemini v0.55.1 source snapshots: online
  package/advisory and upstream implementation research.
- `upstream-reports/`: exact report bodies with reproduction, workaround, and
  suggested source fix.
- `local-tests/`: failing-before-fix and passing-after-fix test evidence.

The older template source captures remain byte-for-byte evidence, but their
manifest basenames now end in `.snapshot`. GitHub's Dependency Graph treats
any `Cargo.toml`, `package.json`, or `pyproject.toml` below the repository as a
live project; the suffix prevents archived evidence from creating projects.

## 2. Reconstructed timeline

| Time (UTC, 2026-08-15) | Event | Evidence |
| --- | --- | --- |
| 12:35–12:36 | Main CI starts JavaScript installs, agent CLI E2E, and the three Intel macOS core lanes. | `ci-logs/run-31884932415.log` |
| 12:35:46 | Two npm installs print `2 high severity vulnerabilities`, but do not fail. | `raw-data/key-diagnostics-with-lines.txt` |
| 12:35:54–12:36:11 | Bun refuses three untrusted global package lifecycle scripts and reports that fact as routine output. | Same indexed evidence and full pipeline log |
| 12:36:26–12:36:38 | macOS core-1/2/3 independently begin `cargo nextest run`; each recompiles the same test graph before applying its partition. | `raw-data/macos-core-timing-lines.txt` |
| 12:45:37–12:45:39 | Gemini warns twice that true color is absent and once that its disappearing `projects.json.lock` directory cannot be scanned. | `raw-data/key-diagnostics-with-lines.txt` |
| 12:53:07–12:55:03 | The three macOS lanes pass after 999, 1,075, and 1,117 seconds, crossing the 840-second warning threshold and approaching the 1,200-second execution budget. | `raw-data/macos-core-timing-lines.txt` |
| 13:08:44 | Auto Release rejects cycle `v0.345.0..HEAD` because no merged PR has valid end-to-end Formal AI attribution. Pipeline Status correctly propagates this job failure. | `ci-logs/run-31884932415-failed.log` |
| 13:41:40 | Issue #1014 opens against main SHA `98cb3c80`, listing twelve runs and one failure. | `raw-data/issue.json` |
| 13:43:02–13:43:05 | Draft PR #1015 opens from prepared SHA `6fd8cdb6`; five initial workflows start and later all succeed. | `raw-data/pull-request.json`, `branch-runs-latest.json` |
| Investigation | Minimal issue tests are committed to the worktree and fail 7/7 against the original implementation. Full lock audits reveal advisories outside the two install-time summaries. | `local-tests/regression-red.log`, `raw-data/*audit*` |
| Fix verification | Issue tests, five explicit lock audits, affected desktop/VS Code suites, web build, and actionlint pass. | `local-tests/` |

## 3. Requirement ledger

| ID | Requirement reconstructed from the issue and repository policy | Result |
| --- | --- | --- |
| R1014-1 | Preserve and inspect all twelve named runs plus PR checks, jobs, artifacts, annotations, discussion, and exact SHAs. | Complete; see section 1 and `raw-data/`. |
| R1014-2 | Find all false positives, false negatives, warnings, and errors; fix every actionable occurrence across the codebase without suppressing true failures. | Complete; see finding ledger below. |
| R1014-3 | Correct the only failed run at its root while retaining release-integrity enforcement. | Automatic release now defers; manual release remains strict. |
| R1014-4 | Correct near-timeout macOS warnings structurally. | One reusable nextest archive replaces three cold builds. |
| R1014-5 | Close dependency-audit false negatives for every JS lock, not only the lock that happened to print a warning. | One dynamically discovered fail-closed gate covers both Bun locks plus three npm locks; all advisories are resolved. |
| R1014-6 | Remove harness and evidence artifacts that make successful CI noisy or misleading. | Gemini environment, Bun lifecycle policy, and archived manifest names corrected everywhere found. |
| R1014-7 | Compare the full current Rust, JS, and Python template trees and Hive Mind guidance, reuse proven components, and report shared defects upstream. | Complete immutable comparison; six upstream issues filed. |
| R1014-8 | Reproduce first and retain automated tests, complete analysis, and verification evidence. | 7/7 initial failures retained; focused and affected green checks retained. |
| R1014-9 | Finish all work in prepared PR #1015, document timeline/root causes/plans, merge current main, and require fresh green CI. | Documentation is here; merge/fresh-CI evidence is appended during finalization. |

## 4. Complete diagnostic and root-cause ledger

| Observed signal | Classification and root cause | Resolution / evidence |
| --- | --- | --- |
| Auto Release error: no eligible Formal AI-authored PR in `v0.345.0..HEAD` | **True policy result, wrong automatic control flow.** PRs #1007, #1011, and #1013 do not put the same canonical PR trailer plus valid session evidence on every introduced non-merge commit. The new issue-#924 check hard-exited before the older report-only release ratchet could run, making immutable main unable to heal itself. | A read-only preflight returns `Deferred` and writes `should_release=false`; automatic publication stops cleanly with a notice. `version-and-commit.rs` still calls the hard gate for manual release. Behavioral and workflow tests pin both halves. |
| Pipeline Status failure | **True positive.** It correctly propagated Auto Release's failure; changing aggregation would hide a real failed dependency. | No suppression or aggregator weakening. Fix the producer only. |
| Three `Core test slice took ...` warnings at 999/1,075/1,117 seconds | **True capacity warning.** nextest partitions execution after compilation, so three clean macOS jobs each compile the same all-feature graph. Raising the timeout would preserve waste. | A reusable workflow creates one nextest archive, uploads it, and fans out three archive consumers. Each consumer keeps a 10-minute bound; the build retains a 25-minute bound and live 1,200-second warning. |
| npm reports two high advisories twice but succeeds | **True vulnerability, false-negative gate.** Install-time npm audit is advisory output, not a required exit status. Separate audits found 2 high advisories in `tests/e2e`, 10 in each desktop lock, 10 in the VS Code lock, and a moderate DOMPurify issue in the root Bun lock; the experiment Bun lock was already clean. | The fail-closed script discovers and audits all five live locks at moderate-or-higher severity. DOMPurify is 3.4.13; npm locks are refreshed; desktop/VS Code use web-capture 1.11.2 plus scoped Puppeteer 25.7 overrides to remove the stale `extract-zip` chain. All five audits return zero. Routine installs use `--no-audit --no-fund` so findings have one authoritative gate. |
| `Blocked 2 postinstalls` / `Blocked 1 postinstall` | **Expected Bun security policy, ambiguous noise.** The global CLIs are executed as command clients and do not require dependency lifecycle scripts. Bun correctly refused untrusted scripts but suggested an action CI must not take. | Every global Bun CLI install found in workflows and the matrix helper explicitly uses `--ignore-scripts`; the regression enumerates all sites. |
| Gemini true-color warnings | **Harness configuration.** Gemini v0.55.1 checks `COLORTERM` for `truecolor`/`24bit`; `TERM=xterm-256color` alone does not satisfy it. | Export `COLORTERM=truecolor` in the E2E harness. Upstream behavior source is preserved in `gemini-...-compatibility.ts`. |
| Gemini `projects.json.lock` ENOENT | **Upstream race exposed by harness layout.** The isolated home was also the scanned project. `ProjectRegistry` creates/removes a proper-lockfile directory while `getFolderStructure` recursively traverses it; the directory may disappear between enumeration and descent. | Use sibling `$WORKDIR/project` and `$WORKDIR/home`. The upstream race is reported as Gemini #28826 with reproduction and proposed ENOENT handling. |
| Dependency Graph workflow for `/dev/log/.../templates/...` | **False-positive project discovery.** Archived full template trees retained exact manifest names, so Dependabot treated evidence as applications and submitted their dependency graph. A full-tree search found the same hazard in older case-study raw data. | Rename all 21 evidence manifests below `dev/log` and `docs/case-studies` with `.snapshot`; a recursive regression fails if any scanner basename returns. Historical bytes remain available. |
| `rehash: warning: skipping ca-certificates.crt` in Dependency Graph | **Third-party container noise.** The message is emitted by the generated Dependabot action's OS certificate update and the run succeeds. It has no repository workflow source. | No unsafe suppression. Removing false project discovery prevents this generated job for the evidence tree. Search evidence is retained in `dependabot-ca-*.json`. |
| CodeQL sources contain deprecated-command and `::error` strings | **Non-emitted candidates.** These are action source/fixture strings in debug expansion, not check annotations or executed diagnostics. | Classified only; complete annotations and logs prove no matching emitted warning. |
| Cache restore misses, Codecov notices, skipped change-filtered jobs | **Informational/intentional.** A cache miss is valid cold-state behavior; Codecov reported successful processing; skips match no-change predicates. | No suppression and no change to pass/fail semantics. |
| npm 11 refuses a regenerated Kreuzberg lock | **Upstream package metadata defect discovered during repair.** `@kreuzberg/html-to-markdown-node@3.7.2` names two musl packages not published at that version. | Locks were generated and clean-installed with CI's npm 10 on glibc. Reported upstream as html-to-markdown #459; do not fabricate absent platform entries. |

Every observed root cause is identified. The existing `FORMAL_AI_CI_VERBOSE`
backend tracing from #1012 remains disabled by default; no new speculative debug
mode was needed for #1014.

## 5. Current template and best-practice comparison

The comparison uses complete repository trees, not a shortlist of workflow
filenames:

| Source | Frozen revision | Relevant result |
| --- | --- | --- |
| Rust pipeline template | `56aa18ac041398afa037cec0da3cf5cae2553e07` | Dependency Review covers changes, but there is no required/scheduled RustSec audit of the current lock. Reported as template #132. |
| JS pipeline template | `77b8f1b520fde96f9a65a0fd7b5e5a5c9d1046d3` | Dependency Review covers changes, but there is no fail-closed audit of committed current npm locks. Reported as template #134. |
| Python pipeline template | `c3a2eb2eaaa5741c9ece6903c7675626d03e7ea3` | Dependency Review covers changes, but there is no pip-audit/OSV check of the resolved current environment. Reported as template #58. |
| Hive Mind CI/CD guidance | `44372fd6` | Supports pinned/reproducible dependencies, fail-fast security checks, isolated jobs, bounded execution, and avoiding duplicate work. The new lock gate and archive topology apply those practices. |

Dependency Review remains valuable for PR deltas, but it cannot detect an
advisory published later against an unchanged lock. The local gate addresses
that temporal false negative. The same gap was reported to all three templates
with ecosystem-specific commands rather than copied implementation.

## 6. Online research and existing components

- [cargo-nextest partitioning](https://nexte.st/docs/ci-features/partitioning/)
  defines `slice:M/N`; [nextest archiving](https://nexte.st/docs/ci-features/archiving/)
  supports compiling once and running an archive elsewhere with workspace
  remapping. These are the native components used instead of a custom test
  scheduler.
- GitHub Dependency Review is retained for changed dependencies. Native
  `bun audit` and `npm audit --package-lock-only` cover current committed locks
  without adding another scanner.
- npm registry metadata captured in `raw-data/*versions.json` confirms the
  current Puppeteer 25 browser downloader no longer depends on the vulnerable
  `extract-zip` path.
- Gemini v0.55.1 source snapshots identify the exact color check, project
  registry lock, storage location, and tree walker involved in the warnings.
- Recent Kreuzberg workflow data and package metadata are preserved to
  distinguish the registry publication defect from this repository's lock.

Upstream reports filed from the retained bodies:

- Gemini CLI [#28826](https://github.com/google-gemini/gemini-cli/issues/28826)
- web-capture [#153](https://github.com/link-assistant/web-capture/issues/153)
- html-to-markdown [#459](https://github.com/xberg-io/html-to-markdown/issues/459)
- Rust template [#132](https://github.com/link-foundation/rust-ai-driven-development-pipeline-template/issues/132)
- JS template [#134](https://github.com/link-foundation/js-ai-driven-development-pipeline-template/issues/134)
- Python template [#58](https://github.com/link-foundation/python-ai-driven-development-pipeline-template/issues/58)

Each `upstream-reports/*.md` file includes a minimum reproduction, an immediate
workaround, and a suggested source-level correction.

## 7. Alternatives considered

| Requirement | Rejected alternative | Why | Selected plan |
| --- | --- | --- | --- |
| Release deadlock | Treat missing attribution as eligible or ignore Auto Release in Pipeline Status. | Both weaken a true release-integrity property. | Preserve strict manual enforcement; defer only the non-mutating automatic publisher. |
| macOS duration | Increase timeout or split into more cold jobs. | Hides the warning or multiplies compilation further. | Compile one nextest archive, then fan out existing slices. |
| JS advisories | Rely on install summaries, Dependabot, or `npm audit fix --force`. | Summaries do not fail; bots are asynchronous; force can make uncontrolled major upgrades. | Required audit gate plus reviewed direct/scoped updates and affected tests. |
| Gemini lock warning | Filter warning text. | Conceals a real upstream race and remains fragile. | Stop scanning mutable home; report race upstream. |
| Evidence dependency graphs | Disable Dependency Graph globally or delete evidence. | Loses a useful service or audit provenance. | Preserve bytes with non-manifest `.snapshot` basenames. |
| Bun lifecycle warning | Trust all scripts. | Expands install-time execution without need. | Explicitly document non-execution through `--ignore-scripts`. |

## 8. Tests-first and verification record

`local-tests/regression-red.log` records the seven implementation-facing issue
tests failing 7/7 before the implementation: release semantics, archive reuse,
Gemini isolation, evidence manifests, complete JS audit coverage,
lifecycle/install noise, and durable case studies. `regression-red.status` is
`101`. Evidence-completeness tests were then added for every R1014 requirement
plus the composed whole task.

The green evidence includes:

- `regression-green.log`: the composed issue #1014 suite.
- `release-preflight.log` and `release-preflight-outputs.log`: the real current
  ineligible cycle exits zero and emits `should_release=false`.
- `javascript-dependency-audits.log`: root Bun, experiment Bun, tests/e2e,
  desktop, and VS Code audits all report zero vulnerabilities.
- `desktop-tests-node20.log`: 140/140 desktop tests pass.
- `vscode-tests-after-update.log`: 51/51 extension tests pass.
- `web-build.log`: the vendored web bundle regenerates successfully.
- `actionlint.log`: both workflow topology and expressions validate.

The repository-wide Rust checks also pass: strict all-feature Clippy, all
examples, 2,804 all-feature tests (plus integration binaries), and doc tests.
Fresh GitHub run IDs are added here only after they execute against the final
#1015 implementation head; stale green checks are never presented as final
evidence.
