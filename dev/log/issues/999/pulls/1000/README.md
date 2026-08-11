# Issue 999 / pull request 1000 evidence and analysis

Collected on 2026-08-11 UTC for [formal-ai issue 999](https://github.com/link-assistant/formal-ai/issues/999) and [pull request 1000](https://github.com/link-assistant/formal-ai/pull/1000). This directory is the durable record behind the fix. It contains more than 430 raw or derived files (35 MiB), including the original GitHub records and final local verification.

## Executive conclusion

The latest default-branch pipeline did not contain a failing test. Its Intel macOS test job was killed at the unchanged 35-minute job limit while tests were still passing. This happened twice in the three most recent relevant runs; the last successful macOS job took 28m37s, leaving only 6m23s of margin. Cold compilation and one monolithic test invocation consumed that margin as the suite grew. Splitting only the macOS lane into `core` and `specification` shards removes the critical path without weakening platform coverage or concealing the timeout by raising its budget.

The annotation audit also found four recurring false-warning categories: three intentional closure-cache exemptions, an informational self-hosting dry-run report, a Pages timeout value above the action's accepted maximum, and five files in their size-warning bands. The first two are now notices, Pages uses its real 600,000 ms maximum, and every observed file is below its warning threshold.

The complete template comparison found two missing preventive controls (CodeQL/dependency review and link validation) and a workflow-wide concurrency defect. Mixed read/write workflows could cancel repository writers, while the default GitHub concurrency queue can replace an older pending writer. All material repository writers now share a repository-scoped `queue: max` group; read-only jobs keep branch/ref-specific cancellation. The latest template security and link gates were adapted under the templates' Unlicense terms.

## Evidence map

- `github/issue*` and `github/pull*`: complete issue and PR metadata, comments, reviews, events, timeline, commits, file list, and diff. Empty JSON arrays are retained to prove that a comment/review channel was checked and had no entries.
- `github/run-*.json`, `run-*-jobs*.json`, and `annotations/`: run metadata, every job, and every annotation returned by GitHub for the investigated runs.
- `ci-logs/run-*.log`: full downloaded workflow logs. The large logs are intentionally kept unabridged; `*-diagnostics.txt` and `cross-run-warning-context.txt` are searchable extracts.
- `research/*-tree.txt`: full repository trees for Formal AI and every named reference project.
- `research/*-revision.txt`, `*-changes-since-issue-980.txt`, and `*-diff*`: exact template revisions and whole-tree/change comparisons.
- `research/CI-CD-BEST-PRACTICES.md`: the referenced Hive Mind guidance as inspected.
- `research/all-run-annotations.tsv`: normalized, cross-run annotation inventory.
- `research/*actionlint*.log`: workflow validation at the template-pinned and latest actionlint versions.
- `research/issue-999-regression-before-fix.log` and `issue-999-regression-after-fix.log`: red/green regression evidence.
- `research/upstream-*`: existing upstream issue records and the corrective template comment filed during this audit.
- `EVIDENCE-MANIFEST.sha256`: SHA-256 digest and relative path for every other retained evidence file.

## Requirements inventory

The issue has nine independently verifiable requirements:

| ID | Requirement | Resolution |
| --- | --- | --- |
| R1 | Check all false positives in CI/CD. | Audited all annotations and full logs for the latest commit plus one success, one preceding cancellation, and one preceding real failure. False warnings and the timeout presentation are classified below. |
| R2 | Check all false negatives in CI/CD. | Preserved the terminal pipeline-status gate, added non-cancellable writer queues, and added missing security/dependency/link gates. These close cancellation and missing-check blind spots. |
| R3 | Fix every warning and error found. | Fixed all current actionable warnings. True failures/notices and the historical external cache-rate warning are explicitly classified rather than suppressed. |
| R4 | Compare the full CI/CD file tree with the Rust template. | Compared revision `86dd57e97e404e3c2865da1a3512bb8878ba8ef4`; tree and diffs are retained. |
| R5 | Compare the full CI/CD file tree with the JavaScript template. | Compared revision `9af528fb034643c03b4354e5273a8a20d830ee02`; tree and diffs are retained. |
| R6 | Compare the full CI/CD file tree with the Python template. | Compared revision `bd07d1ce958cbc852a9ec9eae569f2064172b90f`; tree and diffs are retained. |
| R7 | Reuse all applicable template and Hive Mind CI/CD best practices. | Adopted missing security, link, file-size, test-sharding, and mixed-workflow concurrency controls. Existing stronger project-specific controls remain. |
| R8 | If the same issue is present in a template, report it there. | Corrected the Rust template's stale `queue` conclusion with a reproduction, workaround, and code-level fix at [issue 113 comment](https://github.com/link-foundation/rust-ai-driven-development-pipeline-template/issues/113#issuecomment-5257178147). Exact text and prior issue state are retained. |
| R9 | Plan and execute everything in this one PR without deferral. | All applicable changes, regression tests, research, and evidence are in PR 1000. No follow-up implementation is deferred. |

The task instructions add evidence retention, online corroboration, root-cause analysis, known-component research, codebase-wide application, reproducible testing, PR finalization, and CI verification. Those are addressed by this report, the raw archive, tests, and final PR checks.

## Reconstructed timeline

| Time (UTC) | Event and evidence |
| --- | --- |
| 2026-08-10 20:09–20:51 | Pipeline [31427622514](https://github.com/link-assistant/formal-ai/actions/runs/31427622514) had a real macOS test failure: `whole_file_legality_task_runs_documented_sidecar_end_to_end` unwrapped `ENOENT` at `tests/issue_835_cli.rs:100`. This is not the issue-999 timeout. A later run passed, so the failure was not current. |
| 2026-08-11 00:14:45–00:43:22 | The Intel macOS job in successful pipeline [31444469911](https://github.com/link-assistant/formal-ai/actions/runs/31444469911) took 28m37s. Its full test command alone took 596.47s; the job still completed. |
| 2026-08-11 07:09:34–07:44:57 | Intel macOS in pipeline [31467733666](https://github.com/link-assistant/formal-ai/actions/runs/31467733666) exceeded 35 minutes and was cancelled. Passing test output continued until termination. Pipeline Status correctly converted the otherwise easy-to-miss cancellation into a red run. |
| 2026-08-11 16:26:08 | Commit `d00c61686d0a51c9932558f4be858e5a5febe153` dispatched the issue's seven default-branch workflows. Six passed or were intentionally skipped. |
| 2026-08-11 16:27:41–17:03:18 | Intel macOS in pipeline [31512369875](https://github.com/link-assistant/formal-ai/actions/runs/31512369875) again exceeded 35 minutes. It was inside passing `specification::` tests when cancelled; no assertion, panic, deadlock signature, or nonzero command appeared first. |
| 2026-08-11 17:59:29 | Issue 999 was opened from those latest-run results. |
| 2026-08-11 18:00:48 | Draft PR 1000 and branch `issue-999-31976d2c6ce9` were created. |
| 2026-08-11, investigation | Every run/job annotation and full non-skipped log was downloaded. The three templates and Hive Mind were cloned/read at the revisions recorded above. Official GitHub, Cargo, and action/project documentation was checked. |
| 2026-08-11, implementation | A six-test regression suite was run red, the fixes were implemented, and the same suite passed 6/6. Actionlint 1.7.12 passed with only its known schema gap narrowly ignored. |

## Root causes, alternatives, and selected solutions

### 1. Intel macOS job cancellation

Evidence: both cancelled jobs have the sole primary annotation `The job has exceeded the maximum execution time of 35m0s`. The latest log shows continuing successful specification tests until GitHub injected `The operation was canceled`. The last green job was already within 6m23s of the cap, and its monolithic suite used 596.47s after cold build/setup work.

Root cause: accumulated compilation plus a growing, serially scheduled monolithic integration/specification suite on the slower Intel hosted runner. It is a capacity boundary, not a hung test. Raising the timeout would postpone recurrence and contradict the instruction to investigate repeated timeouts. Skipping macOS would create a coverage false negative. Changing runner architecture would alter the platform contract.

Selected solution: keep Ubuntu's full suite unchanged and split macOS into two independently timed matrix lanes:

- `core`: all library, binary, and integration tests except the already focused data/census groups and `specification::`;
- `specification`: `cargo test --test unit --all-features --verbose specification::`;
- data integrity, census, and doc tests remain in `core` and are not duplicated;
- the 35-minute macOS limit remains unchanged;
- both shards emit elapsed-time telemetry and a warning at 70% of budget, so future growth is diagnosable before another hard cancellation.

Cargo's test-name filter is a supported native partitioning mechanism; see the [Cargo test documentation](https://doc.rust-lang.org/cargo/commands/cargo-test.html).

### 2. Mixed read/write workflow cancellation

Evidence: `release.yml`, `desktop-release.yml`, and `external-benchmarks.yml` combined cancellable checks with release, Pages, artifact, changelog, or benchmark writers under workflow-level concurrency. Cancelling a superseded workflow can interrupt an already-active writer. GitHub's default concurrency semantics also retain only one pending run, replacing an older pending writer.

Root cause: cancellation scope was wider than the side-effect boundary. This was a false negative because an interrupted writer could leave partial external state even when a newer run looked healthy.

Selected solution: move concurrency to jobs. Read-only jobs use workflow/ref/lane groups and cancel superseded PR/ref work. All repository-mutating release, changelog, Pages-finalization, desktop-finalization, and external-benchmark jobs share `formal-ai-repository-writes` with `queue: max`. Desktop build/vscode artifact writers use tag/lane-specific queues because their outputs are disjoint; finalization joins the global group.

GitHub now documents `queue: max`, a maximum of 100 queued jobs, incompatibility with `cancel-in-progress: true`, and FIFO-by-wait-start semantics in [Control workflow concurrency](https://docs.github.com/en/actions/how-tos/write-workflows/choose-when-workflows-run/control-workflow-concurrency). The workflows therefore keep writers idempotent and never combine those options.

### 3. Actionlint's stale concurrency schema

Evidence: current GitHub accepts `concurrency.queue`, but actionlint 1.7.12 still reports `unexpected key \"queue\"`. This is tracked in [actionlint issue 657](https://github.com/rhysd/actionlint/issues/657). The prior Rust-template issue incorrectly concluded the feature was unsupported.

Root cause: third-party static schema lag, not invalid workflow syntax.

Selected solution: upgrade 1.7.7 to 1.7.12 and ignore exactly that diagnostic text. All other actionlint diagnostics remain fatal. The upstream Rust-template issue received the same official-doc reproduction and precise workaround.

### 4. Pages timeout warning

Evidence: `actions/deploy-pages@v5` warned that `timeout: 1200000` exceeded its maximum and was clamped to 600,000 ms. Its implementation and [actions/deploy-pages issue 165](https://github.com/actions/deploy-pages/issues/165) confirm the backend/action ceiling.

Root cause: the workflow attempted to outwait a Pages backlog with an unsupported input. It gained no extra wait but generated a warning on every deploy.

Selected solution: explicitly use `600000`, retain a job budget above it, and correct the regression test that previously required a value above the action's maximum. A longer wait would require upstream Pages support, so no local value can implement it.

### 5. Intentional policy results emitted as warnings

Evidence: three closure-driven caches exceeded the ordinary 128-record cap but were explicitly exempt because every record was referenced. The release report also announced a hypothetical self-hosting-ratchet decrease even though report mode does not enforce or fail it.

Root cause: annotation severity did not match control semantics. Expected exemptions and dry-run information were rendered as warning debt.

Selected solution: emit GitHub `notice` annotations and matching `NOTICE`/`notice:` text. Enforced cache-cap and self-hosting failures remain failures in their enforcing modes.

### 6. File-size warnings

Evidence and selected remediation:

| File | Before | Threshold | After | Treatment |
| --- | ---: | ---: | ---: | --- |
| `.github/workflows/release.yml` | 1,996 | 1,500 | 1,424 | Removed comment-only and blank-line bulk while keeping executable YAML and adding the new controls. |
| `data/seed/multilingual-responses-agentic.lino` | 1,421 | 1,400 | 1,341 | Moved one coherent agentic workspace/tool-result family to a second 81-line registered seed file. |
| `src/web/worker/formal_ai_worker_20.js` | 1,426 | 1,400 | 1,367 | Removed blank lines only; runtime tokens are unchanged. |
| `src/intent_formalization.rs` | 907 | 900 | 897 | Compacted redundant documentation without changing behavior. |
| `src/agentic_coding/general_planner.rs` | 979 | 900 | 893 | Removed blank lines and compacted documentation without changing behavior. |

The new seed file is registered in the production Rust bundle, test-source bundle, `RESPONSE_FILES`, and browser loader. This prevents the superficially easy but incorrect fix of shrinking the file while silently dropping responses from one runtime.

### 7. Missing preventive gates

The Rust, JavaScript, and Python templates all now contain link and security workflows. Formal AI had extensive formatting, linting, secrets, compile, test, coverage, release, desktop, change-detection, file-size, and pipeline-terminal controls, but no equivalent gates.

The imported template condition had its own false negative: it passed when a broken live URL had a Wayback snapshot, even though the checked source still pointed at the dead URL. The local gate always fails a nonzero Lychee result; Wayback is diagnostic evidence and a suggested replacement, not an exemption.

The first local Lychee run exposed 24 previously invisible diagnostics. Five were genuine stale links: three references to the renamed Links Notation repository, one repository under the wrong organization, and the retired CommonsenseQA site. Those now target the current repositories. Two diagnostics came from literal `https://…` examples in reconstructed historical release notes and one from a build-stamped local asset. Because `CHANGELOG.md` is byte-reconstructed from release history, the gate checks a generated mirror in which only that unparseable placeholder is normalized; the original is excluded only while its complete mirror is checked. The build placeholder is narrowly ignored. The remaining 16 were probe false positives: nine Wikipedia requests were throttled, five exact OpenAI pages rejected automation, one authenticated Docker settings page redirected through a login loop, and the AIME host rejected the checker. Per-host concurrency and request spacing fix the Wikipedia burst. Ignore rules name only the exact browser/session-only pages, rather than accepting every 403 or excluding whole useful domains.

Selected solution:

- CodeQL analyzes Rust and GitHub Actions on pushes, PRs, and a weekly schedule;
- dependency review rejects newly introduced high-severity dependencies on PRs;
- Lychee checks repository links while excluding archived case studies and this immutable raw evidence directory;
- known stale documentation links are repaired, while host-aware throttling and narrowly scoped ignores prevent authenticated or automation-blocking pages from creating false positives;
- a Wayback availability helper validates external archival fallback, matching the template pattern.
- any broken live link remains fatal whether or not Wayback has a snapshot.

These are established components rather than custom scanners: [GitHub dependency review](https://docs.github.com/en/pull-requests/how-tos/review-pull-requests/reviewing-dependency-changes-in-a-pull-request), `github/codeql-action`, `actions/dependency-review-action`, and `lycheeverse/lychee-action`.

### 8. Correct true positives and non-actionable historical signals

These were inspected and intentionally not hidden:

- Pipeline Status failure annotations after a timed-out job are correct: GitHub reports a timeout as cancellation, so the terminal gate prevents a false-green main run.
- Run 31427622514's `ENOENT` test failure was a true test failure. It predates the latest commit and the next audited run passed; no current reproduction exists.
- Historical sccache `You've hit a rate limit` warnings came from GitHub's cache service through `mozilla-actions/sccache-action`, not the repository test logic. They were transient and absent from the latest audited runs. GitHub documents cache service limits in [Actions limits](https://docs.github.com/en/actions/reference/limits); the upstream behavior is tracked in [sccache issue 1485](https://github.com/mozilla/sccache/issues/1485). Suppressing all action warnings or disabling the compiler cache would hide real failures and increase the macOS critical path.
- sccache hit-rate summaries, Codecov-not-configured, and coverage-improvement messages are notices, not warning/error debt.

No unidentified root cause remains, so a speculative debug mode was not added. The only timing-sensitive cause now has default-on CI elapsed telemetry at the shard boundary; this is more actionable than an unused application verbose flag.

## Full-tree template disposition

Every file was compared, not only `release.yml`. The raw trees and diffs are in `research/`. Applicable deltas were handled as follows:

| Practice/file family | Formal AI disposition |
| --- | --- |
| Relevant-path detection, merge-result simulation, formatting, lint, test compilation/execution, file-size checks, secrets scan, bounded jobs, pipeline-status terminal gate | Already present, generally more comprehensive than the templates; retained. |
| Changeset/changelog automation and release writers | Existing Rust-specific changelog fragments and release scripts are richer; retained, with concurrency fixed at every writer. |
| Security workflow | Missing and applicable; adapted from current Rust/Python template. |
| Link workflow, `.lycheeignore`, Wayback helper | Missing and applicable; adapted from current Rust template. |
| Desktop release | Formal AI already has a substantially richer cross-platform desktop workflow; retained and repaired at its read/write concurrency boundaries. |
| JS npm/Bun/Deno package publishing and example-app workflow | Not applicable as a replacement: Formal AI's JavaScript is a browser/desktop client embedded in a Rust product, and its existing Node/E2E/desktop checks cover the shipped surfaces. |
| Python PyPI, Ruff, mypy, Sphinx, and wheel publishing | Not applicable: Python is auxiliary test/script content, not a published Python distribution. Existing repository-wide checks still include Python file sizing and relevant scripts. |
| Template sample package code, docs, case studies, and bootstrap scripts | Product/example material, not CI/CD controls; no wholesale copy. |

All four reference repositories use the Unlicense. Copied/adapted workflow/helper material came from Rust template revision `86dd57e97e404e3c2865da1a3512bb8878ba8ef4`; provenance is preserved here and in the git diff.

## Regression and verification protocol

The minimum regression suite is `tests/unit/ci-cd/issue_999.rs`. Before implementation it failed on the missing sharding/concurrency/gates and oversize workflow assertions (`research/issue-999-regression-before-fix.log`). After implementation it passed all six tests (`research/issue-999-regression-after-fix.log`). The suite ensures:

1. exactly two macOS shards retain the original 35-minute budget and complementary filters;
2. every material writer has the correct queue scope and no mixed workflow has top-level cancellation;
3. expected policy outcomes are notices and Pages uses its supported maximum;
4. CodeQL, dependency review, Lychee, and Wayback gates remain wired;
5. actionlint is current and only its exact queue-schema false positive is ignored;
6. every file observed in the warning band remains below its threshold.

Final local verification completed successfully:

- the issue-specific CI/CD suite passed 6/6 and the complete CI/CD unit module passed 183/183;
- the final self-AST census suite passed 8/8 after regenerating the three source-derived documents;
- the all-features repository run completed every integration executable and its final 2,667-test target passed 2,664 with 3 intentional ignores and 0 failures; doc tests also passed;
- all-features Clippy passed with warnings denied, every example compiled, and both normal and docs.rs/no-default-features rustdoc builds passed with warnings denied;
- formatting, actionlint 1.7.12, ShellCheck, changelog reconstruction, fragment-map tests, and `git diff --check` passed;
- the split seed's Rust, browser, worker, response-lookup, and data-integrity regressions passed (51/51 browser tests and the focused Rust suites are retained in `research/`);
- the local live-link gate checked 1,071 links with 0 errors after repairing the discovered stale links and classifying only reproducible probe limitations.

Remote workflow results are retained in `github/` and `ci-logs/` and are checked against the pushed head SHA before finalization.

## Solution alternatives rejected

- Increase the macOS timeout: masks linear growth and does not address the repeated pattern.
- Remove macOS or specification coverage: creates a false negative.
- Mark Pipeline Status green on cancellation: hides timeout failures.
- Keep workflow-level concurrency and set `cancel-in-progress: false`: blocks useful supersession of read-only PR checks and still allows default pending replacement.
- Use only `cancel-in-progress: false` for writers: one pending writer can still be replaced; `queue: max` is required by the issue's no-lost-writer standard.
- Globally ignore actionlint errors: hides unrelated workflow defects.
- Keep the 1,200,000 ms Pages value: the action clamps it, so it cannot help.
- Disable sccache: worsens cold-build time to avoid a transient service annotation.
- Copy language-template release stacks wholesale: would introduce npm/PyPI publishing paths for products this repository does not ship.

## External reports

- Rust template concurrency correction: [link-foundation/rust-ai-driven-development-pipeline-template#113 comment](https://github.com/link-foundation/rust-ai-driven-development-pipeline-template/issues/113#issuecomment-5257178147). The stored comment includes a minimal YAML reproduction, the narrow actionlint workaround, and the code-level recommendation.
- The archived-link false negative was present in all three named templates. Reproducible reports with workarounds and code-level fixes were filed as [Rust template issue 125](https://github.com/link-foundation/rust-ai-driven-development-pipeline-template/issues/125), [JavaScript template issue 127](https://github.com/link-foundation/js-ai-driven-development-pipeline-template/issues/127), and [Python template issue 54](https://github.com/link-foundation/python-ai-driven-development-pipeline-template/issues/54).
- No duplicate reports were filed for Pages or actionlint because exact open upstream reports already exist and are linked above.
- No new sccache report was filed because the exact cache-rate-limit behavior is already tracked upstream and did not reproduce in the issue's latest run.
