## Issue #1014 Complete CI/CD Diagnostic Audit

Issue [#1014](https://github.com/link-assistant/formal-ai/issues/1014) requires
every current CI/CD failure, false positive, false negative, warning, and error
to be traced to evidence and corrected across the repository. It also requires
a full comparison with the current Rust, JavaScript/TypeScript, and Python
pipeline templates and the Hive Mind CI/CD guidance.

| ID | Requirement | Verification |
| --- | --- | --- |
| R1014-1 | Download and inspect all twelve issue-listed default-branch runs, every available job/check annotation, and the initial pull-request runs at the matching SHAs. | The canonical archive is `dev/log/issues/1014/pulls/1015/`; its README maps the collected files to findings. |
| R1014-2 | Classify every failure, warning, and error-shaped line, fix every actionable cause, and retain genuine failures. | `tests/unit/ci-cd/issue_1014.rs`; the finding ledger in the evidence README distinguishes defects, expected policy, third-party noise, and informational messages. |
| R1014-3 | Prevent the automatic release from making the immutable default branch permanently red when a release cycle lacks eligible self-development evidence, without weakening the manual release gate. | `scripts/check-self-development-release.rs`; automatic publication defers with a notice, while `scripts/version-and-commit.rs` still rejects an ineligible manual release. |
| R1014-4 | Remove repeated macOS compilation from the three near-budget core slices instead of raising the timeout, and keep test binaries relocatable in archive consumers. | `.github/workflows/macos-core-tests.yml` builds one nextest archive, extracts it at the original workspace target path, and runs five smaller consumers; a retained red/green nextest experiment and workflow tests pin both properties. |
| R1014-5 | Make dependency security checks fail closed over every committed JavaScript lock and remove the advisories found in the four affected JavaScript dependency surfaces. | `scripts/check-javascript-dependencies.sh` dynamically discovers all five live locks, the web CI gate runs it, affected locks are updated, and retained audit output is clean. |
| R1014-6 | Eliminate harness-generated Gemini warnings, ambiguous package lifecycle noise, and dependency-graph projects discovered inside archived evidence. | Isolated project/home directories with true-color capability, default-denied lifecycle scripts with trust limited to pinned OpenCode, and `.snapshot` evidence filenames. |
| R1014-7 | Compare complete current template trees and Hive Mind practices, research existing components, and report shared or upstream defects with reproductions, workarounds, and code-level suggestions. | Immutable tree metadata and guidance in the archive; seven upstream issues and eight exact report/comment bodies across Gemini, web-capture, html-to-markdown, and the Rust/JS/Python templates. |
| R1014-8 | Prove the defects with minimal failing tests before the fixes, then verify each requirement and the composed change. | Preserved red/green logs under `local-tests/`, issue-specific unit coverage, dependency audits, actionlint, affected JS suites, and repository-wide checks. |
| R1014-9 | Record the reconstructed timeline, root cause and alternatives for every finding, and deliver the complete change in the single prepared PR. | Evidence README, issue and PR case studies, changelog fragment, and pull request #1015. |
