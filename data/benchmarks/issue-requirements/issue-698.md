Parent: #651

## Motivation and evidence

The repeated requirement to measure Formal AI against **real external benchmarks** has been narrowed into repository-local proxies every time it was asked:

- Issue #408 / PR #416: the maintainer asked to "fully pass at least 10% of each benchmark's tests" (CoEdIT, EditEval, SWE-bench and similar), then "pass all of them", and wrote mid-thread: "My requirement were completely ignored." What shipped instead is 1,440 self-generated "repository-local" cases with a self-defined 3/30 floor. No upstream benchmark case executes.
- Issue #440: "pass all benchmarks we know of for coding tasks" — absent from closing PR #472.
- Issue #303 closed while the coding benchmark slice stood at 0/5; the current industry slice (`data/benchmarks/industry-suite.lino`) is a 10-case hand-picked subset.
- Issue #362: a CI-runnable/nightly bulk suite on external datasets was requested; downloads exist only behind an ignored, env-gated test with no scheduled CI wiring.
- Issue #625: the agentic e2e half (real phrasing matrices asserted in CI) was dropped from PR #631.

`docs/benchmarks.md` states full upstream datasets are never vendored — that policy is fine, but it must not prevent *running* upstream cases.

## Requirements

1. Build a benchmark harness that downloads (at test time, cached under the existing provenance/cache policy — never vendored) real upstream slices and executes them against the solver: HumanEval, MBPP, GSM8K, MATH, BIG-bench object counting first; then CoEdIT/EditEval for text editing (issue #408) and a SWE-bench-lite slice for agentic coding.
2. Report **honest scores**: `passed / total` per suite against the *upstream* case set (not a curated subset). 0% or 2% are acceptable first values; fake floors are not (mirrors the honesty rule of issue #657's self-hosting metric).
3. Wire a scheduled (nightly or weekly) CI job that runs the harness on a bounded slice per suite (e.g. first N deterministic cases, N configurable) and publishes results to a committed ledger `data/benchmarks/external-results.lino` with date, suite, slice size, pass count, and solver version.
4. Add a monotonic per-suite ratchet: a PR may not reduce any recorded upstream pass count (extends the existing `minimum_pass_count` discipline from repository-local suites to upstream ones).
5. Keep license discipline: only permissively licensed suites are fetched; record license per suite in `data/benchmarks/LICENSES.md`.
6. Keep reporting honest: when a suite cannot run in CI (size, license, network policy), record an explicit `benchmark_unavailable` entry with the reason instead of silently substituting a local proxy.

## Acceptance criteria

- `cargo test --test unit external_benchmarks -- --ignored` (or a dedicated `formal-ai benchmark run --suite humaneval --slice 20` CLI) executes ≥ 20 real upstream HumanEval cases end to end and prints `passed=<n> failed=<m> total=20`.
- The scheduled workflow exists in `.github/workflows/`, runs green, and appends to the results ledger.
- `docs/benchmarks.md` gains an "External (upstream) results" section with the honest current numbers.
- The ratchet test fails when a recorded upstream pass count regresses.

## Dependencies

- Blocks #657 (E38 self-hosting metric benefits from honest external scores) and #656 (E37 promotion gates should be able to cite upstream benchmark deltas, not only local ones).
- Related: #662 (E43 search can use benchmark tests as fitness), #671 (E52 agentic matrix), #674 (E55).

## Process

Collect data to `docs/case-studies/issue-{id}` (timeline, requirement list, per-requirement solution plans, survey of existing harnesses — e.g. HuggingFace `evaluate`, OpenAI `simple-evals`, SWE-bench tooling), and land everything in a single PR that is not closed until every requirement above is fully addressed or explicitly recorded as blocked with evidence.

