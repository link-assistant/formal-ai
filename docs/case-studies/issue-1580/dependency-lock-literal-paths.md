# Literal dependency lockfile paths

The daily updater read Git's quoted, newline-delimited lockfile names as
literal paths. Names containing a tab, newline, backslash, quote or non-ASCII
character were silently omitted. Its Cargo loop also split a classified
manifest path containing a newline into two resolver calls.

The actual unchanged workflow at `2730b2c35e27010c5af16c1306bf31c3b243d659`
resolves six of fourteen JavaScript fixture locks and splits the newline Cargo
path. Five of eight focused controls fail before the repair. The recording
npm, Bun and Cargo commands never install or compile anything.

Both maintained steps now call `scripts/resolve-dependency-lockfiles.mjs`.
The helper reuses the complete NUL-delimited Git inventory, validates it before
any resolver starts, and supplies literal paths directly through argument
arrays and working directories. It preserves root and nested lock ownership,
the existing lock-only flags, historical Cargo classifications, and the first
resolver's nonzero status. Failed, truncated, duplicate and escaping inventory
observations cannot authorize partial updates. Optional ownership tracing is
off unless `FORMAL_AI_DEPENDENCY_DEBUG=1`.

Reproduce the regression with:

```sh
node --test --test-concurrency=1 experiments/issue-1580/dependency-lock-literal-paths.test.mjs
node --test --test-concurrency=1 rust/tests/web/dependencies-maintenance-workflow.test.mjs
```

All eight focused controls and all eighteen maintained controls pass after the
repair. The latter imports the new regression into the existing fast hosted
test inventory. The five neighboring workflow suites pass after replacing
obsolete shell-text assertions with actual recorded Cargo/npm/Bun calls.
Seven affected local gate twins pass. The final staged-source checks and
original complete streams are retained in the receipt below.

The first neighboring scope run refused an inherited development
`NODE_OPTIONS` loader and also observed a missing short-worker PID receipt.
Both failures are retained. One repeat without external loader options passes
all twelve controls with unchanged budgets; the missing receipt's cause is
not established. No production release authority assertion was relaxed.

This comparison follows the tracked-manifest coverage practice in
[rust template dependency policy](https://github.com/link-foundation/rust-ai-driven-development-pipeline-template/blob/309387f761f9f0f1b2ea15bfe181cb51f5a9a1fb/.github/dependency-policy/check.mjs).
The current inventory contains three active and three preserved Cargo
manifests and fifteen npm manifests, including the scanner's locked closure.
No current npm manifest declares peer dependencies. Wider ecosystem coverage,
actual latest dependency resolution, hosted qualification of this new helper,
and the rest of issue #1580 remain unfinished. Supplied implementation earns
zero independent Formal AI authorship credit.

[Complete observations and member hashes](../../../experiments/issue-1580/evidence/checks/dependency-lock-literal-paths-20261123/manifest.json)
are linked from [PR #1581](https://github.com/link-assistant/formal-ai/pull/1581).
