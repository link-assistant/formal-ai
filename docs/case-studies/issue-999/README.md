# Issue #999 — complete CI/CD diagnostic audit

Issue: <https://github.com/link-assistant/formal-ai/issues/999>

Pull request: <https://github.com/link-assistant/formal-ai/pull/1000>

Issue #999 asked for every CI/CD error, warning, false positive, and false
negative to be investigated and fixed, with a full-tree comparison against the
current Rust, JavaScript, and Python pipeline templates and Hive Mind's CI/CD
guidance.

## Preserved evidence

The complete 450-plus-file evidence archive (35 MiB) and the full analysis are kept
at [`dev/log/issues/999/pulls/1000/`](../../../dev/log/issues/999/pulls/1000/README.md),
the path required by the task. It includes unabridged logs for the latest seven
default-branch workflows, two earlier timeout runs, the preceding successful
run, and the preceding real test failure; every job annotation; all issue and
PR discussion surfaces; exact reference revisions; whole repository trees; and
template diffs. `raw-data/` repeats the small issue/run metadata needed to read
this case study without the large log archive.

## Requirements

The issue's nine requirements are tracked individually in the detailed
[requirements inventory](../../../dev/log/issues/999/pulls/1000/README.md#requirements-inventory):
audit false positives, audit false negatives, fix warning/error debt, compare
the full Rust/JavaScript/Python template trees, apply all relevant template and
Hive Mind practices, report shared template defects upstream, and complete all
work in this one PR.

## Root causes and fixes

1. The Intel macOS job did not have a failed or hanging test. Cold compilation
   plus the monolithic suite crossed 35 minutes twice; the prior green job was
   already 28m37s. The unchanged-budget macOS lane is now split into
   complementary `core` and `specification` shards with elapsed-time telemetry.
2. Workflow-level concurrency covered both read checks and write jobs. A newer
   run could interrupt an active writer, while GitHub's default queue can
   replace an older pending writer. Read jobs now cancel only superseded
   non-main work; material writers queue in safe repository/tag scopes with
   `queue: max`.
3. Actionlint 1.7.12 lags GitHub's documented `concurrency.queue` schema. Only
   its exact false diagnostic is ignored; all other findings remain fatal.
4. Pages silently clamped 1,200,000 ms to its supported 600,000 ms maximum.
   The workflow and prior regression now use the real maximum.
5. Expected cache exemptions and self-hosting report mode emitted warnings.
   Their enforced paths still fail, while informational paths now emit notices.
6. Five source/data/workflow files were inside warning bands. Documentation and
   whitespace were compacted; the large agentic response family was split into
   a second file registered by every Rust and browser consumer.
7. The latest templates exposed missing CodeQL, dependency-review, and broken
   link gates. They are now present. The template's own archived-link exemption
   was a false negative, so Formal AI fails on every dead live link and uses
   Wayback only to suggest a replacement. The first local run then exposed five
   genuine stale documentation links, all repaired; host throttling and exact
   browser-only ignores classify the remaining checker false positives without
   accepting 403 responses globally. The first remote Dependency Review run
   also exposed its missing repository-level Dependency Graph prerequisite.
   GitHub's documented vulnerability-alerts endpoint was reapplied; SBOM export
   and base/head dependency comparison now succeed, so the gate remains active
   instead of being skipped.
8. The first replacement pipeline correctly rejected the response-file split
   because no changed test covered the supported-language matrix. The existing
   sixth issue regression now checks all four moved intents for every
   `registered_languages()` entry and asserts that production Rust, test-source,
   and browser loaders all register the split file. The policy and focused suite
   pass afterward.
9. A fresh link run found one real 502 and 18 successful redirects, but the
   copied Wayback helper reported all 19 as broken because its permissive URL
   expression crossed Lychee's section boundary. The parser now reads only
   `## Errors per input` (with a fallback for old/plain output), covered by a
   red/green Node regression. The unavailable reference now points to live
   University of Calgary course material instead of being ignored or excused
   by an archive.

The complete evidence-to-root-cause table, alternatives, known components, and
source links are in the [durable analysis](../../../dev/log/issues/999/pulls/1000/README.md).

## Upstream reports

- The Rust template's stale claim that GitHub does not support `queue` was
  corrected in [issue 113](https://github.com/link-foundation/rust-ai-driven-development-pipeline-template/issues/113#issuecomment-5257178147).
- The archived-link false negative was reported with reproductions,
  workarounds, and code fixes as [Rust #125](https://github.com/link-foundation/rust-ai-driven-development-pipeline-template/issues/125),
  [JavaScript #127](https://github.com/link-foundation/js-ai-driven-development-pipeline-template/issues/127),
  and [Python #54](https://github.com/link-foundation/python-ai-driven-development-pipeline-template/issues/54).
- The redirect-parser false positive was reported as
  [Rust #126](https://github.com/link-foundation/rust-ai-driven-development-pipeline-template/issues/126)
  and [JavaScript #128](https://github.com/link-foundation/js-ai-driven-development-pipeline-template/issues/128).
- Exact existing upstream reports were reused for the Pages maximum,
  actionlint's schema lag, and transient sccache cache-service rate limits; no
  duplicate issues were filed.

## Third-party provenance

The link/security workflow patterns, `.lycheeignore`, and Wayback helper were
adapted from `link-foundation/rust-ai-driven-development-pipeline-template` at
immutable revision `86dd57e97e404e3c2865da1a3512bb8878ba8ef4`. The repository
uses the Unlicense; that license has no attribution, notice, naming, field-of-use,
copyleft, patent-retaliation, or redistribution conditions. Formal AI also uses
the Unlicense. No training data, personal data, access-controlled material,
secrets, model output, or large copyrighted payload was acquired. The exact
source tree, revision, license check, and diff are retained in the evidence
archive.

## Verification

The issue-specific suite first failed and then passed all six regressions. The
complete CI/CD unit module passes 183/183, and the generated-census suite passes
8/8. The all-features repository run completed with zero failures (2,664 pass
and 3 intentional ignores in its final 2,667-test target), as did Clippy with
warnings denied, examples, both rustdoc profiles, moved response lookups,
data-file integrity, 53 browser tests, actionlint 1.7.12, ShellCheck,
formatting, file-size enforcement, changelog checks, and zero-error local link
gates (including a clean-tree rerun after the remote source failed). Their logs
and SHA-256 manifest live in the evidence archive. Fresh
remote checks are matched to the final pushed head SHA.

No visual UI behavior changed, so browser screenshots and visual-regression
artifacts do not apply.

## Timeline

- 2026-08-10 20:09 UTC: a preceding pipeline had a real, unrelated `ENOENT`
  test failure.
- 2026-08-11 00:14–00:43 UTC: the last successful Intel macOS job took 28m37s.
- 2026-08-11 07:09–07:44 UTC: the first audited 35-minute macOS timeout.
- 2026-08-11 16:27–17:03 UTC: the latest macOS job repeated the timeout while
  specification tests were passing.
- 2026-08-11 17:59 UTC: issue #999 was created; draft PR #1000 followed at
  18:00 UTC.
- 2026-08-11: complete evidence collection, template/online research, red/green
  tests, implementation, upstream reports, and local verification were
  performed in the same PR.
- 2026-08-11 20:36–20:48 UTC: a fresh remote link failure exposed the live 502
  and shared redirect-parser defect; both were reproduced, fixed, tested, and
  reported upstream.
