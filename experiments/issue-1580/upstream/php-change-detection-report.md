Audited source: `3a034b27a56067971ca74dd4c74c7707056b215d`. The actual unmodified change detector can return success and `any-code-changed=false` when Git cannot establish the changed-file set.

Source: [`scripts/src/ChangeDetector.php`](https://github.com/link-foundation/php-ai-driven-development-pipeline-template/blob/3a034b27a56067971ca74dd4c74c7707056b215d/scripts/src/ChangeDetector.php).

The detector falls back to listing the tree, but if both commands fail it returns an empty array. Its PR recovery can also compare only `HEAD^..HEAD` after the full PR comparison fails. A legitimate initial tree does work here; the failure is treating unknown repository/history as an empty changed-file set.

## Bounded reproduction

The retained regression executes `php scripts/detect-code-changes.php` in a temporary directory, with a small mock Git executable ahead of Git on `PATH`. It runs both `GITHUB_EVENT_NAME=push` and `pull_request`, sets 40-character base/head SHAs, and captures the actual `GITHUB_OUTPUT` file. Git comparisons/tree reads print `fatal: unavailable comparison/repository` and exit 128; PR `cat-file`/fetch probes succeed. The script exits **0** and writes **`any-code-changed=false`** for both events. No credentials, network access, installation, or application build is required.

Controls execute the same unmodified source: a successful diff containing `src/Main.php` reports true; successful docs-only and genuinely empty diffs report false. A separate PR case makes the full comparison fail while `HEAD^` contains only `docs/guide.md`: the detector still reports false. This cannot establish that earlier PR commits contain no code.

Executable test, Git blob/SHA-256 provenance, original licenses, and unmodified fixtures: [formal-ai PR #1581](https://github.com/link-assistant/formal-ai/pull/1581), `experiments/issue-1580/template-shared-git-observation.test.mjs`. Each child is limited to five seconds; Java/PHP memory is capped at 64 MiB. All three language reproductions ran locally, without runtime skips. This is an executed subprocess reproduction, not a claim that a hosted publishing workflow was dispatched.

## Expected behavior and repair direction

Only a successful complete comparison should justify a negative code-change output. Propagate failed observations (or deliberately run checks conservatively), distinguish a genuinely initial commit from missing/shallow/corrupt history, and do not silently narrow an unknown full-PR comparison to its last commit. Preserve successful empty/docs-only cases. A plain diagnostic followed by exit 0 does not make an unknown observation fail closed.

Workaround: require successful complete change detection before using a negative output to skip checks; when comparison history is unavailable, run checks conservatively or fail the observation.

Related: the latest merged dependency/CI update is #24. The open Pages configuration report #25 concerns a different failure.
