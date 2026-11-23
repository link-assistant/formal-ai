# Missing link-checker observations must fail

The Rust template security contract compares the checker exit output to a
string. Formal AI's final link gate instead compared it with numeric zero.
GitHub's [expression rules](https://docs.github.com/en/actions/reference/workflows-and-actions/expressions#operators)
coerce an empty string to zero. A missing checker observation could therefore
skip the failure step, even with no successful recovery. This is a false
negative in [PR #1581](https://github.com/link-assistant/formal-ai/pull/1581).

## Reproduction and correction

Run the maintained regression directly:

    node --test --test-concurrency=1 experiments/issue-1580/link-observation-boundaries.test.mjs

On source aa84dfc247891313dc5dc20847950e91c9528064, the original 33 controls
pass and the two new tests fail: the missing-output case and malformed whitespace
case incorrectly clear the deployed final shell gate. The repair requires a
nonempty checker observation and compares its exit value as a string. The
same 35 tests then pass, retaining explicit success, confirmed recovery,
runtime errors, skipped recovery, malformed output and quiet cancellation.

The regression parses the actual YAML condition and runs its actual failure
shell. Its small VM adapter qualifies only these lower-case string/Boolean
fixtures; it does not implement arbitrary Actions expressions or an Actions
runner.

An independent bounded replay uses GitHub's official @actions/expressions
0.3.62, publication gitHead 8e5973b4c9a14ebd526a44799823ac168dbcd291.
The 37,986-byte registry archive's complete SHA-512 integrity is verified
before extraction. Across 120 before/after comparisons, the original has
15 false-negative controls. All fixed observations match their required
verdicts. Every string fixture agrees with the narrow maintained adapter.

The engine's actual absent-property/null values also expose an incomplete
intermediate string-only repair. That retained failure is why the final
condition explicitly checks absence in addition to comparing strings.
The engine stays outside the repository dependency graph:

    node --max-old-space-size=128 experiments/issue-1580/link-terminal-official-engine.mjs /path/to/verified/package /tmp/link-terminal-report.json

## Comparison boundary

The complete 349-line pinned Rust workflow_security contract covers untrusted
inputs, writer queues, CodeQL modes, scheduled lock audits, dependency review
and link recovery. Formal AI retains parsed workflow policy, environment-bound
event inputs, scoped writers, complete committed-lock inventory and strict SARIF
diagnostics. Existing link controls separately qualify real loopback HTTP and
mocked archive transports.

This local expression-library replay is not a hosted runner or live external
link audit. Main-only production, CodeQL semantic coverage, original template
practice execution and dependency-score criteria remain unfinished. The current
hosted cohort predates this repair. No independent authorship or reasoning
credit is claimed.
