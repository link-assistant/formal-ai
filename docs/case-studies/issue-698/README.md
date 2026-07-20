# Issue 698 — real external benchmark harness

Issue #698 asks for the measurement this repository did not have: not another
curated slice that the repository itself authored, but the *unmodified upstream*
case set, executed end to end, with the resulting number published exactly as
measured — even when that number is zero.

- Requirement trace: [`requirements.md`](./requirements.md)
- Per-requirement design: [`solution-plans.md`](./solution-plans.md)
- Survey of existing harnesses (upstream and in-repo): [`survey.md`](./survey.md)
- Raw run logs: [`raw-data/`](./raw-data/)

## What was built

| Piece | Path |
| --- | --- |
| Suite registry with pinned upstream revisions and licenses | `src/external_benchmarks/manifest.rs` |
| Run-time download into the build-artifact cache | `src/external_benchmarks/fetch.rs` |
| Upstream record → gradable case mapping | `src/external_benchmarks/cases.rs` |
| Per-suite grading (Python execution, numeric, boxed, text, diff) | `src/external_benchmarks/grade.rs` |
| Committed results ledger | `src/external_benchmarks/ledger.rs`, `data/benchmarks/external-results.lino` |
| Monotonic per-suite ratchet | `src/external_benchmarks/ratchet.rs` |
| CLI (`formal-ai benchmark list \| run \| ratchet`) | `src/cli_benchmark.rs` |
| Weekly scheduled job | `.github/workflows/external-benchmarks.yml` |
| Requirement tests | `tests/unit/specification/external_benchmarks.rs` |
| Published numbers | `docs/benchmarks.md` § "External (upstream) results" |

## The honest first measurement

Slice 20, solver `0.300.0`, offline deterministic configuration, recorded
`2026-07-20` (`raw-data/all-suites-first-run.log`):

```
suite=humaneval passed=0 failed=20 total=20
suite=mbpp passed=0 failed=20 total=20
suite=gsm8k passed=2 failed=18 total=20
suite=math passed=0 failed=20 total=20
suite=object_counting passed=0 failed=20 total=20
suite=coedit passed=0 failed=20 total=20
suite=editeval benchmark_unavailable: …
suite=swebench_lite passed=0 failed=20 total=20
```

These are the real numbers. GSM8K's `2 / 20` comes from two word problems whose
final number the solver produced correctly; everything else the offline solver
does not currently answer. The failures are ordinary solver output, not harness
artifacts — on HumanEval the solver echoes the prompt and appends its
"cannot infer a verified answer" message, which is then executed against the
upstream test and fails (see `raw-data/humaneval-first-run.log` and the produced
files under `target/formal-ai-benchmarks/run/humaneval/`).

## Timeline

1. **Read the issue and the existing benchmark surface.** The repository already
   had five `.lino` suites with local ratchets and a download-on-test pattern
   from issue #362, plus a "no vendored datasets, permissive licenses only"
   policy in `docs/benchmarks.md`. None of them ran an upstream case set.
2. **Verified every candidate source before writing code.** Each URL was fetched
   and its license checked. Two findings changed the design:
   - The MATH split on `raw.githubusercontent.com` returns a 131-byte Git LFS
     pointer. The payload is served by `media.githubusercontent.com`
     (446 564 bytes, 500 rows), so the manifest pins the media URL and records
     why.
   - EditEval hosts no task payload, and its corpora (ASSET CC BY-NC 4.0, JFLEG
     CC BY-NC-SA 4.0) fail the permissive-only policy. It is therefore encoded
     as unavailable rather than approximated.
3. **Built the harness** as `manifest → fetch → cases → grade → ledger`, with no
   new dependency: downloads shell out to `curl`/`gzip` exactly as the issue
   #362 benchmark does, and parquet-only datasets (CoEdIT, SWE-bench Lite) are
   read through the Hugging Face datasets-server `rows` API, the same route
   issue #482 uses.
4. **Ran it for real** and recorded the numbers above, including the
   `benchmark_unavailable` row for EditEval.
5. **Added the ratchet, the schedule, the tests, and the documentation**, then
   confirmed the acceptance criterion:
   `cargo test --test unit external_benchmarks -- --ignored --nocapture` prints
   `suite=humaneval passed=0 failed=20 total=20`
   (`raw-data/acceptance-ignored-test.log`).

## Design decisions worth reviewing

- **Grading is per suite, not per guess.** `Grading` is carried from the manifest
  into `grade_case`, so a SWE-bench patch is never graded as a number and a
  CoEdIT edit is never graded as Python. An earlier draft inferred the mode from
  the case id; that would have silently mis-scored two suites.
- **The floor equals the best measurement.** `minimum_pass_count` is raised only
  by a run that actually achieved it, and a test asserts the floor equals the
  best recorded pass count — so the ledger cannot carry an aspirational number.
- **Unavailability is data, not silence.** `Availability::Unavailable { reason }`
  flows into a `benchmark_unavailable` ledger row with the concrete blocker.
- **Nothing is vendored.** Payloads land in `target/formal-ai-benchmarks`; the
  test asserts `data/benchmarks/` contains only `.lino` and `.md` files.
