Audited source: `65c686846d7327d8c835a0d30b1a76cc6345cfc6`. I reviewed the latest merged CI PR #16 and all open/closed issues through #17; none reports this error boundary.

[`scripts/detect-code-changes.mjs`](https://github.com/link-foundation/go-ai-driven-development-pipeline-template/blob/65c686846d7327d8c835a0d30b1a76cc6345cfc6/scripts/detect-code-changes.mjs) turns a failed Git comparison into a successful all-false classification:

```js
function exec(command) {
  try {
    return execSync(command, { encoding: "utf-8" }).trim();
  } catch (error) {
    // Logs the failure, then:
    return "";
  }
}
```

`getChangedFiles()` cannot catch the failure, because `exec()` already swallowed it. Both PR and push paths interpret the empty string as an empty changed-file list. The intended first-commit `git ls-tree` fallback is also unreachable for a failed `HEAD^` comparison.

I reproduced this by executing the **actual unmodified script** in a temporary directory with a bounded mock Git executable: `git diff` writes `fatal: unavailable comparison commit` and exits 128, while the PR's `git cat-file` succeeds. The script exits **0** and writes `go-changed=false` and `any-code-changed=false`. It behaves the same on push; with a first-commit mock, the available `git ls-tree` result is never consulted. A separate test creates a real Git repository containing only an initial `main.go` commit: running the unchanged detector reports `go-changed=false`, while the candidate error-propagation change correctly reports true. A successful diff of `pkg/sample.go` returns true, and a successful docs-only diff returns false, as controls.

On a PR, these false outputs can suppress lint and changeset validation even though the comparison is unknown. Logging a plain error does not turn the successful exit/output into a failing observation.

The executable bounded reproduction, source hashes and unmodified fixtures are retained in [formal-ai PR #1581](https://github.com/link-assistant/formal-ai/pull/1581): `experiments/issue-1580/template-go-result-integrity.test.mjs`. No GitHub token, network request, writes outside the disposable local fixture, or large build is needed by the test.

Candidate repair: propagate the original error from `exec()`, then keep a deliberate fallback where the caller can establish the changed-file set. Replacing only the swallowed `return ""` with `throw error` makes the first-commit fixture enumerate `pkg/sample.go`, and makes an entirely unavailable comparison/repository exit nonzero instead of emitting false “no code changed” outputs. Successful code and docs controls remain unchanged. This one-line candidate is not a complete fix: the PR catch path can still narrow an unavailable full-PR comparison to `HEAD^..HEAD`, which can omit earlier code commits. Require the intended complete comparison, establish a genuinely initial commit before using its tree, and propagate any remaining unknown observation. Failing clearly or conservatively running all affected checks avoids classifying an unknown comparison as docs-only.

Workaround: require successful change detection before consuming its negative outputs; if the comparison cannot be established, run the checks conservatively or fail that observation.
