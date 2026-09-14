## Issue #492 Release Badge Stability

Issue [#492](https://github.com/link-assistant/formal-ai/issues/492)
reported that GitHub release notes showed `invalid` and `failing` artifact
badges, and asked to restore the traditional README badge block while auditing
the AI-driven development pipeline templates for the same pattern. PR
[#583](https://github.com/link-assistant/formal-ai/pull/583) replaces live
release-note status badges with stable version artifact badges, restores README
badges, preserves the research in `docs/case-studies/issue-492`, and reports the
template regression upstream as
[rust-ai-driven-development-pipeline-template#85](https://github.com/link-foundation/rust-ai-driven-development-pipeline-template/issues/85).

| ID | Requirement | Status |
| --- | --- | --- |
| R360 | GitHub release notes must not use live crates.io or docs.rs status badge endpoints that can render `invalid` or `failing` after publication. | Implemented by `scripts/create-github-release.rs::crate_release_badges`, which renders static Shields badge URLs for the exact release version instead of `img.shields.io/crates/v/...` and `docs.rs/.../badge.svg`; covered by `crate_release_badges_use_static_artifact_links_not_live_status`. |
| R361 | Release notes must retain direct artifact links for the exact crate and documentation version that the release published. | Implemented by `crate_release_badges`, which links to `https://crates.io/crates/{crate_name}/{version}` and `https://docs.rs/{crate_name}/{version}`; covered by the same release-script unit test. |
| R362 | The README must restore the traditional project badge block at the top of the document. | Implemented in `README.md` with CI/CD, desktop release, crates.io, docs.rs, Rust version, Codecov, and license badges; covered by `readme_keeps_traditional_ci_and_artifact_badges`. |
| R363 | README artifact badges should use current stable badge providers and project-specific identifiers, not template placeholders. | Implemented by README Shields crates.io/docs.rs badges for `formal-ai` and the `example-sum-package-name` regression check in `readme_keeps_traditional_ci_and_artifact_badges`. |
| R364 | Issue #492 evidence, including issue/PR metadata, release data, screenshots, CI data, and downloaded logs, must be preserved under `docs/case-studies/issue-492`. | Implemented by `docs/case-studies/issue-492/README.md`, `assets/issue-screenshot.png`, and `raw-data/*`, including `release-v0.205.0.json`, `release-v0.205.0-peeled-commit-runs.json`, and `desktop-release-27572474798.log`. |
| R365 | The case study must include a timeline, requirements, root cause analysis, solution plan, and verification notes for issue #492. | Implemented by `docs/case-studies/issue-492/README.md`; covered by `issue_492_release_badge_documents_are_traceable`. |
| R366 | The CI/CD and release-template comparison must cover the JavaScript, Rust, Python, and C# AI-driven development pipeline templates. | Implemented by the saved template HEADs, file trees, workflow/script inventories, README badge excerpts, and badge-pattern scan under `docs/case-studies/issue-492/raw-data/`. |
| R367 | If the same release-badge issue exists in a template, it must be reported upstream and linked from the case study. | Implemented by reporting [rust-ai-driven-development-pipeline-template#85](https://github.com/link-foundation/rust-ai-driven-development-pipeline-template/issues/85) and preserving `reported-rust-template-issue-85.json`. |
| R368 | The release badge and README badge behavior must be protected by automated regression tests. | Implemented by `crate_release_badges_use_static_artifact_links_not_live_status`, `readme_keeps_traditional_ci_and_artifact_badges`, and `issue_492_release_badge_documents_are_traceable`. |
| R369 | The issue #492 fix, documentation, test coverage, and PR metadata must land in the prepared PR #583 branch. | Implemented on branch `issue-492-c714d50efef8` and tracked by PR [#583](https://github.com/link-assistant/formal-ai/pull/583). |
