# Issue 492 Case Study

Issue: <https://github.com/link-assistant/formal-ai/issues/492>
PR: <https://github.com/link-assistant/formal-ai/pull/583>
Template report: <https://github.com/link-foundation/rust-ai-driven-development-pipeline-template/issues/85>

## 1. Summary

Issue #492 reported that GitHub release notes showed artifact badges as
`invalid` and `failing`. The screenshot captures release `v0.205.0`, published
on 2026-06-15 at 19:55:53Z, whose release body started with live badge
endpoints:

- `https://img.shields.io/crates/v/formal-ai?label=crates.io`
- `https://docs.rs/formal-ai/badge.svg`

Those endpoints describe current registry or documentation status, not the
already-published release artifact. Historical release notes therefore inherit
transient or later status problems. The fix changes release-note crate badges to
static version badges that still link to the exact crates.io and docs.rs version
pages, and restores the traditional README badge block.

## 2. Collected Data

| Data | File |
| --- | --- |
| Issue metadata and comments | `raw-data/issue-492.json`, `raw-data/issue-492-comments.json` |
| PR metadata, comments, reviews | `raw-data/pr-583.json`, `raw-data/pr-583-conversation-comments.json`, `raw-data/pr-583-review-comments.json`, `raw-data/pr-583-reviews.json` |
| Issue screenshot | `assets/issue-screenshot.png` |
| Release list and affected release body | `raw-data/release-list-summary.json`, `raw-data/release-v0.205.0.json` |
| Release tag and commit lookup | `raw-data/release-v0.205.0-tag-ref.txt`, `raw-data/release-v0.205.0-peeled-tag-ref.txt` |
| Release-adjacent CI runs and log | `raw-data/release-v0.205.0-peeled-commit-runs.json`, `raw-data/desktop-release-27572474798.log` |
| Branch CI snapshot before the fix | `raw-data/pr-branch-ci-runs-initial.json` |
| Template audit inputs | `raw-data/template-*-head.txt`, `raw-data/template-*-file-tree.txt`, `raw-data/template-workflow-files.txt`, `raw-data/template-script-files.txt`, `raw-data/template-badge-release-patterns.txt` |
| README badge excerpts from templates | `raw-data/template-*-readme-badges.txt` |
| Upstream template report | `raw-data/reported-template-issues.txt`, `raw-data/reported-rust-template-issue-85.json` |
| External references | `raw-data/online-research.md` |

## 3. Timeline

- 2026-06-15 19:55:53Z: GitHub release `v0.205.0` was published with live
  crates.io and docs.rs badge URLs in the first release-body line.
- 2026-06-15 19:56:00Z: the Desktop Release workflow started for the same
  release commit and later failed in macOS signing. This failure is preserved as
  release evidence but is not the badge root cause.
- 2026-06-16 06:51:00Z: issue #492 was opened with a screenshot showing
  `crates.io invalid` and `docs failing` on release `v0.205.0`.
- 2026-06-28: this investigation reproduced the release-note badge problem,
  audited the template repositories, reported the Rust template docs.rs badge
  regression upstream as issue #85, and changed formal-ai release notes to use
  static version badges.

## 4. Requirements

The issue expands the repository requirements matrix with R360-R369:

- R360: release notes must not use live status badge endpoints for crates.io or
  docs.rs.
- R361: release notes must still link to the exact crate and docs artifact
  version.
- R362: README traditional badges must be restored.
- R363: README artifact badges must use current project-specific badge sources.
- R364: issue evidence, screenshot, release data, CI data, and logs must be
  preserved under this case-study directory.
- R365: the case study must capture the timeline, root cause, solution plan, and
  verification.
- R366: the four AI-driven development pipeline templates must be compared.
- R367: matching template issues must be reported upstream.
- R368: regression tests must protect the release-note and README badge
  behavior.
- R369: all work must land through PR #583 on branch
  `issue-492-c714d50efef8`.

## 5. Root Cause

`scripts/create-github-release.rs` generated release-note badges from live
status endpoints. The crates.io badge endpoint reports package availability
through Shields at render time, while the docs.rs badge endpoint reports current
documentation build status. A release page is historical, so badges embedded in
it should describe the published artifact version, not whatever the external
status endpoint says later.

The README badge block had also drifted out of the top of `README.md`, so the
project no longer had the traditional CI, artifact, Rust version, Codecov, and
license badges visible at first glance.

The CI log for Desktop Release run `27572474798` shows a separate release
problem: release `v0.205.0` had no desktop assets, then macOS builds failed with
signing errors (`code has no resources but signature indicates they must be
present` and `code object is not signed at all`). That explains the desktop
workflow failure near the release but not the `invalid`/`failing` badge text in
the issue screenshot.

## 6. Implemented Design

The release script now renders crate release badges through
`crate_release_badges(crate_name, version)`. The badge images are stable static
Shields URLs with the release version in the badge text:

- `https://img.shields.io/badge/crates.io-<version>-orange?logo=rust`
- `https://img.shields.io/badge/docs.rs-<version>-blue`

The links still point to the exact version artifacts:

- `https://crates.io/crates/<crate_name>/<version>`
- `https://docs.rs/<crate_name>/<version>`

The README now restores the project badge block for CI/CD, Desktop Release,
crates.io, docs.rs, Rust version, Codecov, and license. The docs.rs README badge
uses the current Shields docs.rs endpoint instead of the legacy docs.rs
`/badge.svg` status endpoint.

## 7. Template Comparison

| Template | HEAD captured | Result |
| --- | --- | --- |
| JavaScript | `raw-data/template-js-head.txt` | Uses static package-version release badges; no matching release-note docs badge issue found. |
| Rust | `raw-data/template-rust-head.txt` | Crates.io badge is already static, but release notes still include `https://docs.rs/{crate_name}/badge.svg`; reported upstream as rust-ai-driven-development-pipeline-template#85. |
| Python | `raw-data/template-python-head.txt` | Uses static PyPI version badges; no matching issue found. |
| C# | `raw-data/template-csharp-head.txt` | Uses static NuGet/version badge patterns; no matching issue found. |

The raw grep results are in `raw-data/template-badge-release-patterns.txt`.

## 8. Prior Art And Online Research

The researched sources are recorded in `raw-data/online-research.md`.

- docs.rs now documents badge generation through Shields.io for crate
  documentation status and version badges.
- Shields.io supports static badges, which are appropriate for immutable
  historical release notes.
- GitHub Actions documents workflow status badge URLs with a branch query, which
  matches the restored README workflow badges.

## 9. Verification

Reproduction before the fix:

- `cargo test --test unit crate_release_badges_use_static_artifact_links_not_live_status -- --nocapture`
- Saved output: `raw-data/repro-crate-release-badges-before.log`
- Result: failed because the release body used live crates.io and docs.rs status
  badge endpoints.

Focused verification after the fix:

- `cargo test --test unit crate_release_badges_use_static_artifact_links_not_live_status -- --nocapture`
- Saved output: `raw-data/repro-crate-release-badges-after.log`
- Result: passed.

The before-fix reproducer output is preserved as
`raw-data/repro-crate-release-badges-before.log`, and the focused badge suite is
preserved as `raw-data/focused-badge-tests.log`.

Final local verification:

- `cargo fmt --all -- --check`
  (`raw-data/cargo-fmt-check.log`)
- `cargo test --test unit badges -- --nocapture`
  (`raw-data/focused-badge-tests.log`)
- `cargo test --test unit issue_492_release_badge_documents_are_traceable -- --nocapture`
  (`raw-data/issue-492-traceability-test.log`)
- `cargo test --test unit check_file_size -- --nocapture`
  (`raw-data/check-file-size-unit-tests.log`)
- Manual file-size threshold scan matching the configured `.rs` and `.lino`
  limits because `rust-script` was not installed locally
  (`raw-data/manual-file-size-scan.log`)
- `cargo test --test unit` (`raw-data/cargo-test-unit.log`)
