## Reproduction

The latest audited Release run emits 33 `local/no-changelog-comments` warnings from files already committed on the default branch, including scripts, tests, experiments, and the issue-3 case-study fixture:

https://github.com/link-foundation/js-ai-driven-development-pipeline-template/actions/runs/29434577797

Examples include `scripts/publish-to-npm.mjs`, `tests/create-github-release.test.js`, and `docs/case-studies/issue-3/original-format-release-notes.mjs`. The warnings recur even when those files are unrelated to the triggering change, obscuring new actionable findings.

## Expected / suggested fix

Continue linting all source for errors, but annotate warning-level history findings only on changed lines/files. Exclude intentional historical fixtures under `docs/case-studies` and the rule's own tests, or give them narrow inline fixture suppressions. Add a test proving an unchanged baseline finding is not re-annotated while a newly changed finding is.

Found while comparing all four pipeline templates for https://github.com/link-assistant/formal-ai/issues/717.
