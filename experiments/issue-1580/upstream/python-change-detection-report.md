Audited source: `c532c173801dfb12e681b0730c4abbf93aefd0ca`. The actual unmodified change detector can return success and `any-code-changed=false` when Git cannot establish the changed-file set.

Source: [`scripts/detect_code_changes.py`](https://github.com/link-foundation/python-ai-driven-development-pipeline-template/blob/c532c173801dfb12e681b0730c4abbf93aefd0ca/scripts/detect_code_changes.py).

The helper catches `CalledProcessError` and returns an empty string, so unavailable history is indistinguishable from an empty diff. An available first-commit source tree also reports false. On PRs an empty result can fall back to `HEAD^..HEAD`, which is narrower than the intended full PR.

## Bounded reproduction

The retained regression executes `python3 scripts/detect_code_changes.py` in a temporary directory, with a small mock Git executable ahead of Git on `PATH`. It runs both `GITHUB_EVENT_NAME=push` and `pull_request`, sets 40-character base/head SHAs, and captures the actual `GITHUB_OUTPUT` file. Git comparisons/tree reads print `fatal: unavailable comparison/repository` and exit 128; PR `cat-file`/fetch probes succeed. The script exits **0** and writes **`any-code-changed=false`** for both events. No credentials, network access, installation, or application build is required.

Controls execute the same unmodified source: a successful diff containing `src/main.py` reports true; successful docs-only and genuinely empty diffs report false. A separate PR case makes the full comparison fail while `HEAD^` contains only `docs/guide.md`: the detector still reports false. This cannot establish that earlier PR commits contain no code.

Executable test, Git blob/SHA-256 provenance, original licenses, and unmodified fixtures: [formal-ai PR #1581](https://github.com/link-assistant/formal-ai/pull/1581), `experiments/issue-1580/template-shared-git-observation.test.mjs`. Each child is limited to five seconds; Java/PHP memory is capped at 64 MiB. All three language reproductions ran locally, without runtime skips. This is an executed subprocess reproduction, not a claim that a hosted publishing workflow was dispatched.

## Expected behavior and repair direction

Only a successful complete comparison should justify a negative code-change output. Propagate failed observations (or deliberately run checks conservatively), distinguish a genuinely initial commit from missing/shallow/corrupt history, and do not silently narrow an unknown full-PR comparison to its last commit. Preserve successful empty/docs-only cases. A plain diagnostic followed by exit 0 does not make an unknown observation fail closed.

Workaround: require successful complete change detection before using a negative output to skip checks; when comparison history is unavailable, run checks conservatively or fail the observation.

Related: #40/#41 corrected excluded-path gates and #18/#19 corrected manual-release skip propagation. Neither repairs this failed-Git observation boundary. Reproduction used Python 3.15; the pinned upstream syntax needs Python 3.14 or newer.
