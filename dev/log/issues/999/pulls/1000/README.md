# Issue 999 / pull request 1000 evidence and analysis

Collected on 2026-08-11 UTC for [formal-ai issue 999](https://github.com/link-assistant/formal-ai/issues/999) and [pull request 1000](https://github.com/link-assistant/formal-ai/pull/1000). This directory is the durable record behind the fix. It contains more than 450 raw or derived files (35 MiB), including the original GitHub records and final local verification.

## Executive conclusion

The latest default-branch pipeline did not contain a failing test. Its Intel macOS test job was killed at the unchanged 35-minute job limit while tests were still passing. This happened twice in the three most recent relevant runs; the last successful macOS job took 28m37s, leaving only 6m23s of margin. Cold compilation and one monolithic test invocation consumed that margin as the suite grew. Splitting only the macOS lane into `core` and `specification` shards removes the critical path without weakening platform coverage or concealing the timeout by raising its budget.

The annotation audit also found four recurring false-warning categories: three intentional closure-cache exemptions, an informational self-hosting dry-run report, a Pages timeout value above the action's accepted maximum, and five files in their size-warning bands. The first two are now notices, Pages uses its real 600,000 ms maximum, and every observed file is below its warning threshold.

The complete template comparison found two missing preventive controls (CodeQL/dependency review and link validation) and a workflow-wide concurrency defect. Mixed read/write workflows could cancel repository writers, while the default GitHub concurrency queue can replace an older pending writer. All material repository writers now share a repository-scoped `queue: max` group; read-only jobs keep branch/ref-specific cancellation. The latest template security and link gates were adapted under the templates' Unlicense terms. The first Dependency Review run then exposed a repository-level prerequisite: alerts were enabled but the Dependency Graph had never been generated. Reapplying GitHub's documented vulnerability-alerts setting enabled both features; the SBOM and dependency-diff APIs now succeed. A later live-link run exposed both a newly unavailable external reference and a second shared template bug: the Wayback helper treated every successful redirect bullet as a broken URL. The reference now uses live university material, and the parser is section-aware with a red/green regression test.

## Evidence map

- `github/issue*` and `github/pull*`: complete issue and PR metadata, comments, reviews, events, timeline, commits, file list, and diff at final source SHA `219b3dc7`, immediately before the self-describing evidence commit. GitHub's diff endpoint rejected the 501-file PR with HTTP 406, so `pull.diff.stderr` preserves that response and `pull.diff` is the equivalent complete `origin/main...origin/issue-999-31976d2c6ce9` diff. Empty JSON arrays are retained to prove that a comment/review channel was checked and had no entries.
- `github/run-*.json`, `run-*-jobs*.json`, and `annotations/`: run metadata, every job, and every annotation returned by GitHub for the investigated runs.
- `ci-logs/run-*.log`: full downloaded workflow logs. The large logs are intentionally kept unabridged; `*-diagnostics.txt` and `cross-run-warning-context.txt` are searchable extracts.
- `research/*-tree.txt`: full repository trees for Formal AI and every named reference project.
- `research/*-revision.txt`, `*-changes-since-issue-980.txt`, and `*-diff*`: exact template revisions and whole-tree/change comparisons.
- `research/CI-CD-BEST-PRACTICES.md`: the referenced Hive Mind guidance as inspected.
- `research/all-run-annotations.tsv`: normalized, cross-run annotation inventory.
- `research/*actionlint*.log`: workflow validation at the template-pinned and latest actionlint versions.
- `research/issue-999-regression-before-fix.log` and `issue-999-regression-after-fix.log`: red/green regression evidence.
- `research/upstream-*`: existing upstream issue records, the corrective template comment, and four reproducible template reports filed during this audit.
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
| R7 | Reuse all applicable template and Hive Mind CI/CD best practices. | Adopted missing security, link, file-size, test-sharding, and mixed-workflow concurrency controls. Enabled the repository-level Dependency Graph prerequisite discovered by the first remote run. Existing stronger project-specific controls remain. |
| R8 | If the same issue is present in a template, report it there. | Corrected the Rust template's stale `queue` conclusion and filed reproducible reports for the archived-link false negative and redirect-parser false positive in every affected named template. Links appear under External reports; exact records are retained. |
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
| 2026-08-11 20:13:08–20:13:28 | The first new Dependency Review job failed before analysis because this repository's Dependency Graph was not enabled. Its full log and annotation are archived under run 31531791299/job 93913334294. |
| 2026-08-11 20:15:49–20:15:56 | The documented vulnerability-alerts repository setting was reapplied through GitHub's REST API. The endpoint returned 204; SBOM export and the base/head dependency diff immediately changed from unavailable to successful. The diff found the five newly introduced Actions dependencies and no vulnerabilities. |
| 2026-08-11 20:21:12–20:27:45 | The first full replacement pipeline passed every preceding lint command, then its language-test-coverage policy correctly rejected the seed split because no changed test asserted the supported-language matrix. The complete 3,752-line job log and annotation are archived under run 31532227251/job 93915498383. |
| 2026-08-11, follow-up | The existing file-size regression was strengthened to load every moved response intent for every `registered_languages()` entry and assert registration in the production Rust, test-source, and browser loaders. The previously red language policy, focused 6-test suite, formatting, and Clippy then passed locally. |
| 2026-08-11 20:36:44–20:40:42 | Fresh Broken Link run [31533813546](https://github.com/link-assistant/formal-ai/actions/runs/31533813546) found one real 502 for the Encyclopedia of Mathematics reference and 18 successful redirects. The archive helper incorrectly announced 19 broken URLs and emitted false annotations for redirected links. Its complete 697-line log, metadata, jobs, and annotations are archived. |
| 2026-08-11 20:41–20:48 | A minimum Node regression reproduced the parser defect before implementation. The parser was restricted to Lychee's error section, the unavailable reference was replaced with live University of Calgary course material, and the focused tests, all 53 browser unit tests, syntax check, direct HTTP probes, and a clean-tree full Lychee run passed. The shared defect was reported to both affected templates. |
| 2026-08-11 20:59:08–21:01:11 | Broken Link run [31535677995](https://github.com/link-assistant/formal-ai/actions/runs/31535677995) passed on the exact corrective source SHA `219b3dc782bc4e7b6a485e16bd7e3d990e58c682`. Its complete 450-line log, run and job metadata, and zero-annotation result are archived beside the failing reproduction. |

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

The first remote Dependency Review execution revealed a separate repository-setting dependency. GitHub returned `Dependency review is not supported on this repository` and pointed to Dependency Graph enablement. The repository was public and Dependabot alerts answered as enabled, but SBOM export returned 404 and GraphQL reported zero manifests. GitHub's official REST schema states that enabling vulnerability alerts also enables the Dependency Graph, so the administrator endpoint `PUT /repos/link-assistant/formal-ai/vulnerability-alerts` was called idempotently. It returned 204; SBOM export then produced a GitHub Dependency Graph document, and `dependency-graph/compare/d00c6168...940fbd05` returned the five new Actions dependencies with empty vulnerability lists. The full job log, annotation, REST response, official enablement documentation, SBOM, and comparison are retained in this archive. Skipping the job would have recreated the false negative that the new gate is intended to close.

### 8. Split-seed language coverage

Evidence: the first full replacement pipeline's lint job passed Clippy, rustdoc, file-size enforcement, cache policy, generated-source checks, i18n catalog validation, and language-change parity. `check-language-test-coverage.mjs` then reported the three changed language-facing files, four changed CI test files, and zero languages covered in added test lines. The same command reproduced locally against `origin/main`.

Root cause: splitting a multilingual response family was correctly classified as language-facing, but the first issue regression only checked line limits. Existing older tests happened to cover one moved intent, while no changed test proved that every moved intent remained present for every language or that all runtime registries loaded the new file. The policy failure was therefore a true positive, even though the worker edit itself removed blank lines only.

Selected solution: strengthen the existing sixth issue regression rather than adding language-name comments or bypassing the policy. It now iterates `formal_ai::language::registered_languages()`, requires direct records for all four moved intents, and checks registration in the production Rust bundle, source-test mirror, and browser loader. This both satisfies the diff-aware guard and catches a real missing-file/missing-locale regression. The guard changed from red to `Language test coverage OK against origin/main: en, ru, hi, zh, es`; the same six focused tests and Clippy pass afterward.

### 9. Wayback redirect misclassification and unavailable reference

Evidence: run 31533813546's Lychee summary contained exactly one error and 18 redirects. `extractBrokenUrls` first matched explicit 4xx/5xx statuses, then applied a permissive bullet-URL expression to the entire report. That second pass captured all 18 bullets under `## Redirects per input`; the helper therefore said `Found 19 broken URL(s)` and emitted seven false error annotations when Wayback had no snapshot for healthy redirect sources. The same script blob (`2b8244d7d76d56d9acdf88b4ea766e35c554b1fe`) was current in both the Rust and JavaScript templates. Python does not contain the helper.

Selected solution: when Lychee emits structured Markdown, parse only `## Errors per input` through the next level-two heading. Preserve whole-document parsing for old/plain output. The parser is exported behind a direct-run guard so a `node:test` fixture can assert that one error plus one redirect returns only the error. The test failed before the fix and passes afterward. Reports with the exact fixture, workaround, and proposed patch were filed upstream.

The sole real error was independent: `https://encyclopediaofmath.org/wiki/Normal_algorithm` returned 502 on the runner and on four consecutive local probes. It was not ignored or replaced by an archive. The documentation now links to live University of Calgary course notes that directly define a Markov algorithm as a finite ordered sequence of substitution rules and state its equivalence with Turing computation; the replacement returned 200 and the clean-tree Lychee run finished with zero errors.

### 10. Correct true positives and non-actionable historical signals

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
- the split seed's Rust, browser, worker, response-lookup, data-integrity, and archive-parser regressions passed (53/53 browser tests and the focused Rust suites are retained in `research/`);
- the initial local live-link gate checked 1,071 links with 0 errors after repairing the discovered stale links and classifying only reproducible probe limitations; after the remote 502, a clean-tree rerun checked 991 current inputs with 0 errors;
- the remote Broken Link Checker passed on corrective source SHA `219b3dc782bc4e7b6a485e16bd7e3d990e58c682` after the parser and source corrections;
- the diff-aware language-test-coverage policy passed for all registered locales (`en`, `ru`, `hi`, `zh`, `es`) after the split-response regression was made registry-driven.

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
- The redirect-parser false positive was present in the identical Wayback helper shipped by the Rust and JavaScript templates. Reproducible reports were filed as [Rust template issue 126](https://github.com/link-foundation/rust-ai-driven-development-pipeline-template/issues/126) and [JavaScript template issue 128](https://github.com/link-foundation/js-ai-driven-development-pipeline-template/issues/128). The Python template has no such helper, so no inapplicable report was filed there.
- No duplicate reports were filed for Pages or actionlint because exact open upstream reports already exist and are linked above.
- No new sccache report was filed because the exact cache-rate-limit behavior is already tracked upstream and did not reproduce in the issue's latest run.
