# Issue 980 / PR 981 CI investigation

Collected: 2026-08-08 UTC. Raw issue, pull-request, review, workflow, template,
and run-log evidence is preserved beside this report. The issue and PR had no
comments, reviews, or inline review comments when collected.

## Requirements ledger

| ID | Requirement | Disposition |
|---|---|---|
| R980-1 | Inspect every referenced default-branch run for errors, warnings, false positives, and false negatives. | All seven run records and logs downloaded. The six successful workflows had no actionable failure. Run 31186108359 had two failed jobs and one retry-masked flaky test. |
| R980-2 | Root-cause and fix every observed CI defect. | Fixed the rustfmt violation, isolated local opener parity from live providers, and made the permission test wait for the actual pending-task state. |
| R980-3 | Compare the full CI/workflow/script trees with the Rust, JS, and Python templates. | Current template SHAs, GitHub trees, tracked-file inventories, workflow/script sources, and control indexes are preserved. The reusable controls were already adopted by PRs 809 and 971; no new template-owned defect was found. |
| R980-4 | Apply Hive Mind CI/CD best practices. | Revalidated fail-fast linting, bounded jobs, cancellation guards, diff-aware execution, fresh-merge checks, secrets checks, file-size checks, changelog fragments, artifacts, and aggregate status. The remaining failures were test implementation defects, not missing workflow architecture. |
| R980-5 | Execute everything in PR 981 and add regression coverage. | `issue_980.rs` pins all three fixes; focused browser repetition and repository checks are recorded in this directory. |

## Timeline

1. PR 965 commit `9f772b53` introduced the unformatted assertion in
   `tests/unit/total_closure.rs`; merge commit
   `5625a53a29291cc55f1c587c13d9bdb3c2c0f4c8` put it on `main`.
2. At 2026-08-07 14:08 UTC, the Coverage, Task Ladder, Write-Effect Ladder,
   Agentic CLI Matrix, and CI/CD workflows started on exactly that SHA.
3. CI/CD run 31186108359 failed formatting at 14:11:43. The exact diff is in
   `red-rustfmt.log` and the run log around lines 3180-3320.
4. The same run's local E2E suite completed at 14:18 with 465 passing, one
   failing, one flaky, and one skipped test. The Chinese opener case received a
   live ranked-search result instead of a local WASM opener. The permission
   cold-start case first saw `data-has-pending-task=false`, then passed on retry.
5. Pipeline Status correctly named `lint` and `test-e2e-local` and failed at
   14:32. Coverage and the three ladder/matrix workflows succeeded. Desktop
   Release later succeeded, as did the later learning-cycle workflow.
6. On 2026-08-08 the issue was opened and PR 981 was prepared. The evidence was
   collected and every observed defect was reproduced or traced to its exact
   asynchronous boundary before the fixes were applied.

## Root causes and fixes

### 1. Deterministic formatting failure

`cargo fmt --all -- --check` rejected a long `misplaced.push(format!(...))`
line in `tests/unit/total_closure.rs`. This was a true positive, introduced by
PR 965 and missed before merge. The source is now formatted and the regression
gate retains the exact rejected text so that this particular error cannot
silently return.

### 2. Live search intercepted a local parity test

`issue-282.spec.js` describes a Rust/WASM unknown-opener contract but sent
ordinary multilingual terms with external research enabled. The Chinese prompt
`未知词` was accepted by a live provider, producing a ranked result instead of
the local fallback. The assertion was valid; the fixture boundary was not.

The test now routes every request and aborts cross-origin traffic while allowing
the already-loaded local origin. Playwright's documented `page.route()` /
`route.abort()` mechanism is the existing component intended for hermetic
network tests: <https://playwright.dev/docs/network> and
<https://playwright.dev/docs/api/class-route>. This avoids provider-specific
mocks and covers every current or future external endpoint.

### 3. Permission test read state before the worker finished

`sendPrompt()` returned as soon as *any* message appeared. The click itself
appends the user message, so the helper did not prove that the worker had
captured `pendingAgentTask`. On the first CI attempt, the CTA was read during
that window; retry timing hid the defect.

The first draft waited for exactly two messages. Repetition falsified that
assumption: this flow can emit two assistant messages (three total). The final
test uses Playwright's retrying web-first assertions to wait for more than the
user append and, critically, for `data-has-pending-task=true`. This asserts the
observable state the test depends on rather than an incidental message count.
Playwright documents this retry-until-ready behavior at
<https://playwright.dev/docs/api/class-playwrightassertions>.

## Template and best-practice comparison

The fetched heads were Rust `c867f78`, JS `7b70923`, and Python `98d6dca`.
The template snapshots and indexes cover their complete tracked trees, not only
similarly named files. `local-ci-controls.txt` and the three
`*-template-ci-controls.txt` files make the control-by-control comparison
searchable.

Formal AI already contains the materially applicable union:

- formatter/linter, actionlint, shellcheck, lockfile, docs, file-size, WASM-size,
  language, coverage, and generated-bundle gates;
- change detection, PR fresh-merge simulation, aggregate status, explicit
  timeouts, PR-only cancellation, and failure artifacts;
- secret scanning and resilient Docker buildx setup;
- changelog fragments, release gates, desktop packaging dry runs, and
  self-hosting evidence checks.

PR 809 imported fresh-merge simulation, scoped secrets scanning, resilient
buildx, and corrected cancellation guards from the templates/Hive Mind guide.
PR 971 revalidated the full four-template audit and filed the still-confirmed
upstream gaps: JS #122; Rust #115, #116, #117; Python #48, #49 (plus C# findings
outside this issue's requested template set). Duplicating those reports would
add noise. No current template has either application-specific Playwright test,
and no new template-owned defect was found, so no additional upstream report is
warranted.

The broader controls align with GitHub's documented model: nonzero exit codes
must fail checks, failure artifacts preserve diagnostics, and workflow-specific
concurrency groups avoid cross-workflow cancellation. References:
<https://docs.github.com/en/actions/how-tos/create-and-publish-actions/set-exit-codes>,
<https://docs.github.com/en/actions/tutorials/store-and-share-data>, and
<https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax>.

## Reproduction and verification evidence

- `red-rustfmt.log`: formatter failure before the source fix.
- `red-tests.log`: cold Rust build plus the initial regression-gate run.
- `green-rustfmt.log`: clean formatter check.
- `green-unit-tests.log`: issue-specific Rust regression tests.
- `green-focused-e2e.log`: initial browser-environment failure and experimental
  run; preserved rather than overwritten or hidden.
- `green-issue-541-e2e.log`: repeated, single-worker permission-flow result.
- `green-issue-282-e2e.log`: all four multilingual opener cases passed three
  consecutive times (12/12) behind the hermetic network boundary.
- `ci-logs/run-31186108359.log` and `*-findings.txt`: authoritative CI failure.

No default-on production debug logging was added: the evidence was sufficient
to identify all root causes, and the existing Playwright trace-on-first-retry,
HTML failure report, diagnostics mode, and uploaded failure artifacts already
provide the requested opt-in verbose path.
