Audited source: `65c686846d7327d8c835a0d30b1a76cc6345cfc6`. The latest merged CI change, #16, retains this dependency shape; I checked all open/closed issues through #17 and found no report of this skip chain.

Both manual release modes can complete with a successful terminal status while their selected release job never runs.

In the [pinned workflow](https://github.com/link-foundation/go-ai-driven-development-pipeline-template/blob/65c686846d7327d8c835a0d30b1a76cc6345cfc6/.github/workflows/release.yml):

1. `detect-changes` is skipped for `workflow_dispatch` (line 42).
2. `lint` needs `detect-changes` (line 107). Its `if` explicitly admits `workflow_dispatch`, but contains no status check function.
3. GitHub applies the default `success()` condition to such an expression, so the skipped dependency also skips lint. `build` then skips because it needs lint; `instant-release` and `changeset-release` need lint/test/build and also skip.
4. `test` has `!cancelled()`, so it can run despite the intentional skipped dependencies. The terminal gate uses `always()` and the actual helper returns 0 when everything succeeded or was skipped. No release is required to have succeeded.

GitHub documents both [skipped dependency propagation](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#jobsjob_idneeds) and [the implicit status check](https://docs.github.com/en/actions/reference/workflows-and-actions/expressions#status-check-functions).

Reproduction: dispatch **CI/CD Pipeline** on main with `release_mode=instant`, then with `release_mode=changeset`; inspect the conclusions of detect-changes, lint, build, the selected release job and Pipeline Status. Expected: the selected release runs after its checks. The documented dependency rules predict that it is skipped and the terminal checker accepts the result.

Verification scope: I did **not** dispatch a publishing workflow. A bounded offline scheduler model using the exact pinned YAML reproduces the dependency chain for both modes, and invokes the unmodified upstream `evaluate()` terminal checker on those modeled job results. The checker returns 0 and prints `Every required job succeeded or was skipped.` This is source-grounded model evidence, not a claimed hosted reproduction. Full source identities and the executable four-test reproduction are retained in [formal-ai PR #1581](https://github.com/link-assistant/formal-ai/pull/1581), at `experiments/issue-1580/template-go-result-integrity.test.mjs` and its content-addressed fixtures.

A minimal candidate repair in that model is to allow `detect-changes` to run on dispatch. Both release modes then run their selected release; forcing lint, test, or build to fail still skips publication and makes the actual terminal checker return 1. Alternatively, explicitly handle the intentionally skipped dependency in lint without weakening failed-prerequisite checks. Consider asserting the expected selected release result in the terminal gate for manual events, so an unexpected skip cannot look successful.

Workaround: use the normal tested push-to-main release route until manual dispatch dependency admission is corrected. Do not force release jobs past failed checks.
