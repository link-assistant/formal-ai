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
| Pinned, content-verified run-time cache | `src/external_benchmarks/fetch.rs` |
| Upstream record → gradable case mapping | `src/external_benchmarks/cases.rs` |
| Per-suite grading (Python execution, numeric, boxed, text, official SWE-bench tests) | `src/external_benchmarks/grade.rs` |
| Committed results ledger | `src/external_benchmarks/ledger.rs`, `data/benchmarks/external-results.lino` |
| Monotonic per-suite ratchet | `src/external_benchmarks/ratchet.rs` |
| CLI (`formal-ai benchmark list \| run \| ratchet`) | `src/cli_benchmark.rs` |
| PR base ratchet and weekly scheduled job | `.github/workflows/external-benchmarks.yml` |
| Failure-derived, review-gated auto-learning | `src/external_benchmarks/learning.rs`, `src/agentic_coding/external_benchmark_learning.rs` |
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
suite=swebench_lite benchmark_unavailable: legacy proxy score withdrawn …
```

The first six scored lines are the retained real measurements. GSM8K's
`2 / 20` comes from two word problems whose
final number the solver produced correctly; everything else the offline solver
does not currently answer. The failures are ordinary solver output, not harness
artifacts — on HumanEval the solver echoes the prompt and appends its
"cannot infer a verified answer" message, which is then executed against the
upstream test and fails (see `raw-data/humaneval-first-run.log` and the produced
files under `target/formal-ai-benchmarks/run/humaneval/`). The original
SWE-bench `0 / 20` was not retained: review found that it compared with the gold
patch rather than running upstream tests, so the ledger explicitly marks that
capture unavailable until the official evaluator produces a valid score.

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
   #362 benchmark does. CoEdIT comes from a revision-pinned JSONL URL;
   SWE-bench Lite comes from a revision-pinned parquet file decoded by the
   Python environment used by its official evaluator. Every cache payload has
   a source-ref/URL/length/content-id sidecar.
4. **Ran it for real** and recorded the numbers above, including the
   `benchmark_unavailable` row for EditEval.
5. **Corrected the semantic boundary during review.** Gold-patch equality is
   not SWE-bench. The harness now installs the official evaluator at
   `f7bbbb2…`, applies candidate patches in its containers, and treats evaluator
   failures as unavailable infrastructure.
6. **Connected failure evidence to Formal AI learning.** Failed case ids and
   evaluator details are converted into the shared associative-memory format,
   ranked into proposal-only reports, and held behind human review plus the
   benchmark/Agent-CLI gate.
7. **Added the base-aware ratchet, the schedule, tests, and documentation**, then
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
  test recursively rejects the exact upstream cache artifacts while allowing
  self-authored nested benchmark definitions.
- **Learning cannot rewrite the score.** Failure reports carry
  `decision "awaiting_human_review"` and are uploaded as workflow artifacts.
  Only a later reviewed change may alter behavior, and the base-aware ratchet
  still forbids lower recorded results.

## Real Agent CLI evidence

The same issue-solver task was sent through the repository's real Agent CLI
surface. The first two attempts exposed a general planner bug: a policy mention
of `./examples` was treated as a file and produced `EISDIR`. A reproducing unit
test led to the shared file-shaped-path predicate and a successful retry past
that failure. The next retry exposed a second general routing defect: a phrase
inside the long issue transcript (“how Formal AI works”) hijacked the compound
coding task into the self-explanation recipe. That now has a reproducing test
and a structural repository-work-item guard. The benchmark learning task is
also replayed separately through the real local Formal AI binary so its written
artifact can be compared byte-for-byte with the shared renderer.
