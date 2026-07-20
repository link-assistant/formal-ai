# Issue 698 — per-requirement solution plans

Each plan states the requirement, the approach taken, the alternative rejected,
and how the result is verified.

## R698-01 — download at run time, cache, never vendor

**Approach.** `SuiteSource` describes the wire format (`JsonLines` with optional
gzip, `BigBenchTask`, `DatasetsServerRows`, `Unavailable`). `fetch.rs` downloads
with `curl -fSL --retry 3 --retry-delay 2` into
`target/formal-ai-benchmarks/<cache_file>`, writing a `.partial` file first and
renaming on success so an interrupted run cannot leave a truncated cache.
Gzipped payloads are expanded through `gzip -dc`. Parquet-only datasets are read
page-by-page from the Hugging Face datasets-server `rows` endpoint (100 rows per
page), which returns JSON this crate can already parse.

**Rejected.** Adding an HTTP client and a parquet decoder as dependencies. The
repository already shells out to `curl` for issue #362 downloads and already
uses the datasets-server for issue #482; matching those keeps the dependency
surface unchanged.

**Verified by.** `upstream_slices_are_downloaded_at_test_time_and_never_vendored`
asserts https-only URLs, cache files confined to the cache directory, and that
`data/benchmarks/` contains no payload files.

## R698-02 — honest `passed / total` over the upstream case set

**Approach.** `parse_cases` takes the first N records **in upstream order** and
refuses to run when the source yields fewer than N. Every case is graded by the
upstream criterion: HumanEval concatenates the produced code with the upstream
test and runs `check(entry_point)`; MBPP runs the upstream `test_list` asserts;
GSM8K compares the final number with the gold after `####`; MATH compares the
final `\boxed{...}`; object counting compares the final number with the target;
CoEdIT compares the edited text; SWE-bench Lite compares the produced patch with
the gold patch. `SuiteRun::summary()` prints
`suite=<id> passed=<n> failed=<m> total=<t>` and nothing is filtered out.

**Rejected.** Selecting cases the solver is likely to handle, or grading with a
loose similarity threshold. Both would produce a number that is not the upstream
number, which is the exact failure mode the issue names.

**Verified by.** `recorded_scores_are_honest_passed_over_total` asserts
`passed + failed == total == slice` for every recorded row and that each suite's
floor equals its best measured pass count.

## R698-03 — scheduled job publishing to a committed ledger

**Approach.** `.github/workflows/external-benchmarks.yml` runs weekly
(`17 4 * * 1`) and on `workflow_dispatch` with `slice`, `suite`, and `commit`
inputs. It builds the release binary, installs Python (needed to grade HumanEval
and MBPP), runs `benchmark run --suite all --slice N --append`, appends the
`suite=` lines to the job summary, verifies the ratchet, uploads the log, and
commits the refreshed ledger. Each ledger row carries `date`, `suite`, `slice`,
`passed`, `failed`, `total`, `solver_version`, `runner`, and a note.

**Rejected.** Publishing to a build artifact only. The issue asks for a
*committed* ledger so history is reviewable in the repository.

**Verified by.** `scheduled_workflow_publishes_to_the_committed_ledger`.

## R698-04 / R698-10 — the monotonic ratchet

**Approach.** Two pure functions over parsed ledgers, so the ratchet is testable
without network:

- `ratchet::violations(&Ledger)` — internal consistency: every result row has a
  suite record, `passed + failed == total == slice`, every run at the ratchet
  slice clears `minimum_pass_count`, the floor is never below the best recorded
  pass count, and the per-suite history at a fixed slice never falls over time.
- `ratchet::regressions(&previous, &current)` — the pull-request check: a suite
  record removed, a floor lowered, a recorded row deleted, or a recorded pass
  count rewritten downwards are all reported.

`Ledger::raise_floor` only ever raises, and only at the matching ratchet slice,
so an unlucky rerun cannot erase a recorded achievement. `benchmark run --append`
refuses to write a ledger that violates the ratchet.

**Verified by.** `recorded_upstream_pass_count_may_never_regress` builds four
synthetic regressions (lowered pass count, deleted row, lowered floor, fresh run
below the floor) and asserts each one is reported.

## R698-05 — license discipline

**Approach.** Every manifest entry carries `license`, `license_url`, `source_url`
and a pinned `source_ref`. `PERMISSIVE_LICENSES` is `MIT`, `Apache-2.0`,
`CC-BY-4.0` — the same set `docs/benchmarks.md` already declares. The same
provenance is duplicated into the ledger (so the committed record is
self-contained) and into `data/benchmarks/LICENSES.md`.

**Verified by.**
`only_permissively_licensed_suites_are_fetched_and_licenses_are_recorded`
asserts every runnable suite's license is in the permissive set and that both the
suite id and its license appear in `LICENSES.md`.

## R698-06 — `benchmark_unavailable`

**Approach.** `Availability::Unavailable { reason }` short-circuits `run_suite`,
which returns a `SuiteRun` with `total = 0` and the reason set; `--append` then
writes a `benchmark_unavailable` record carrying the reason and the evidence
note. The same path covers a missing `python3` interpreter, so a machine without
Python records "no python3 interpreter is available" instead of a fabricated 0.

EditEval is the concrete case: no task payload upstream, and corpora under
CC BY-NC 4.0 / CC BY-NC-SA 4.0, which the permissive-only policy excludes.

**Rejected.** Substituting the repository's own text-manipulation suite for
EditEval. That is precisely the "silently substituting a local proxy" the issue
forbids. Instructed text editing is instead measured on Apache-2.0 CoEdIT, which
is a real upstream suite in the same task family.

**Verified by.** `an_unrunnable_suite_is_recorded_as_benchmark_unavailable`,
which also asserts no substituted score exists for `editeval` and that CoEdIT
covers the same `task_family`.

## R698-07 — suite coverage

**Approach.** Eight suites in the order the issue lists them: HumanEval, MBPP,
GSM8K, MATH, BIG-bench `object_counting`, then CoEdIT and EditEval for text
editing (#408), then SWE-bench Lite (dev split) for agentic coding. MATH uses the
`openai/prm800k` 500-problem split, which is MIT-licensed and pinned by revision;
the original `hendrycks/math` archive is distributed as a tarball on a mirror
that is not license-pinned in the same way.

**Verified by.** `issue_698_external_benchmark_harness_is_wired_end_to_end`
asserts each suite id and each task family is present in both manifest and
ledger, and that the ledger's provenance matches the manifest field for field.

## R698-08 — the acceptance run

**Approach.** `humaneval_slice_of_twenty_real_upstream_cases_runs_end_to_end` is
`#[ignore]`d (it downloads and executes Python) and lives in a module named
`external_benchmarks`, so the command in the issue selects it exactly:
`cargo test --test unit external_benchmarks -- --ignored`. It asserts the slice
is 20 real cases in upstream order (`HumanEval/0` … `HumanEval/19`), prints the
report, and asserts the pass count is at or above the recorded floor.

**Verified by.** `raw-data/acceptance-ignored-test.log`:
`suite=humaneval passed=0 failed=20 total=20`.

## R698-09 / R698-11 — documentation and case study

**Approach.** `docs/benchmarks.md` gains an "External (upstream) results"
section with the measured table, the ratchet rule, and the commands. The
whole-task test asserts every recorded `passed`/`total` pair actually appears in
the document, so the published numbers cannot drift from the ledger. This
directory holds the timeline, requirement trace, these plans, the harness
survey, and the raw logs.
