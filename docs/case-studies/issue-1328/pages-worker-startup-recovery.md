# Recovering transient worker startup failures

The v0.355.4 Pages deployment served the expected source, but its live smoke
failed because the chat composer remained disabled. The complete
[failed job log](https://github.com/link-assistant/formal-ai/actions/runs/39902286553/job/120291504219)
has SHA-256 `e31c58d069cf45bb730ac4b6ab75a6f7c880e867a25bc17afd2f14f104038a55`.
The pipeline-status failure was derived from this failed test; the Linux image
publication jobs completed successfully.

An isolated public-browser reproduction captured HTTP 503 responses for
`seed_loader.js` and a worker module. The latter caused an uncaught
`importScripts` NetworkError. The application remained at `loading worker`
because it did not handle worker error events. A bootstrap catch also concealed
required seed-loader failures. The failure report contained the disabled
composer assertion, without the worker error or failed HTTP requests.

The central worker import wrapper now permits at most two attempts for a
NetworkError naming the exact imported resource. Exhausted budgets persist
across later loader calls. Successful scripts are not replayed during recovery,
and syntax errors, runtime errors, and errors from nested services are not
retried. Required bootstrap failures propagate. The application reports fatal
worker errors explicitly and keeps the dead worker's composer disabled.
Consolidated asset-version handling keeps the existing worker line ceiling.
The maintained translator regenerates its TypeScript twin, and the Rust
package receives a byte-identical copy of the canonical worker entry.

The original live smoke retains its native worker, arithmetic, landing, and
documentation assertions, its 20-second worker bound, and zero test retries.
On failure it now prints and attaches bounded worker, console, HTTP, and request
diagnostics, then rethrows the original failed assertion. This distinguishes
transport failure from an engine or assertion bug without hiding either.

The captured transport conditions are stored as a small declarative fixture.
Unit regressions exercise transient recovery, exhaustion across later calls,
and prevention of repeated script side effects. Local-server Playwright tests
inject the same HTTP failures and require the real native worker to answer
arithmetic after recovery; persistent failures must produce an explicit error
after two requests. These emulations create no GitHub issues or pull requests.
They run in CI without local Rust compilation.
The explicit local-suite configuration selects these tests, and a configuration
regression ensures duration-based partitioning assigns them to one project.
Persistent-failure checks derive their UI language cases from the maintained
language registry, verify the advertised UI language or its browser fallback,
and require the same two-request bound and disabled composer in every case.
The partial Spanish solver language uses the English browser fallback; the
four fully supported UI languages retain their exact language assertions.

The PR's native full-suite shard also exposed a pre-existing literal-file replay
failure: the unchanged semicolon request returned no target write. Its failure
log omitted the intermediate plans and tool receipts, so the root cause remains
unresolved. Under the release-first scope, that full native replay is temporarily
ignored as `PR1401-native-literal-replay`, with its original request, expected
bytes, and six-turn bound preserved. Native plan composition still checks both
clauses. The unchanged JavaScript physical replay remains required and passes
with real filesystem writes and shell verification. Empty native replay results
now print the stopping plan and actual bounded conversation for diagnosis.
Restore the full native replay after identifying and repairing its cause; this
debt grants no native file-delivery or three-client acceptance credit.

This repair uses the authorized manual critical-release workflow and receives
no Formal AI authorship credit. It does not confer the real three-client
acceptance required by [issue #1364](https://github.com/link-assistant/formal-ai/issues/1364).
