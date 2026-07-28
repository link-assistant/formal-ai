# Issue #698 — real external benchmark harness

Sessions: `issue-698-claude-20260720`, `issue-698-codex-20260728`
Pull request: https://github.com/link-assistant/formal-ai/pull/816
Authored by: formal-ai autonomous issue-solver sessions

This document is the committed evidence referenced by the
`Formal-AI-Session` / `Formal-AI-Evidence` trailers on the issue implementation
commits. It records what was measured, what was decided, and which decisions
were rejected — so the numbers published by this pull request can be audited
without re-running anything.

## 1. What the issue asked for

Not another repository-authored slice: the *unmodified upstream* case set, run
end to end, with the resulting score published exactly as measured. Fourteen
requirements were extracted and are traced one-by-one in
`docs/case-studies/issue-698/requirements.md` (R698-01 … R698-14) and mirrored
into the repository-wide register as R528 … R535.

## 2. Evidence collected before writing code

Every candidate upstream source was fetched and its license read first. Two
findings changed the design rather than being worked around:

| Finding | Consequence |
| --- | --- |
| `raw.githubusercontent.com` serves a 131-byte Git LFS pointer for the MATH split | the manifest pins the `media.githubusercontent.com` payload URL and records why in `download_note` |
| EditEval ships a harness with no task payload, and its corpora are CC BY-NC / CC BY-NC-SA | the suite is recorded as `benchmark_unavailable` with that reason; instructed text editing runs through Apache-2.0 CoEdIT instead |
| Hugging Face datasets-server requests do not carry the recorded dataset revision | CoEdIT and SWE-bench Lite use immutable revision-pinned payload URLs with cache provenance sidecars |
| Gold-patch equality is not the SWE-bench semantic criterion | the invalid proxy result was withdrawn; the scheduled harness pins the official evaluator, applies candidate patches, and executes repository tests |

These are the honest outcomes the issue asks for: a suite that cannot run is
declared unavailable, never quietly replaced by a repository-local proxy.

## 3. The honest first measurement

The first raw capture used slice 20, solver `0.300.0`, on `2026-07-20`
(`docs/case-studies/issue-698/raw-data/all-suites-first-run.log`):

```
suite=humaneval passed=0 failed=20 total=20
suite=mbpp passed=0 failed=20 total=20
suite=gsm8k passed=2 failed=18 total=20
suite=math passed=0 failed=20 total=20
suite=object_counting passed=0 failed=20 total=20
suite=coedit passed=0 failed=20 total=20
suite=editeval benchmark_unavailable: …
suite=swebench_lite benchmark_unavailable: historical gold-diff proxy withdrawn
```

GSM8K's `2 / 20` is two word problems whose final number the solver produced
correctly. Everything else is zero, and zero is what the ledger records. The
failures are ordinary solver output graded by the upstream criterion, not
harness artifacts — the produced Python files are kept under
`target/formal-ai-benchmarks/run/` and quoted in the raw-data logs.

The raw log preserves the historical SWE-bench `0 / 20` line as investigation
evidence, but it is not a valid SWE-bench score: that implementation compared
text with the gold patch. The committed ledger withdraws that row as
`benchmark_unavailable`. Future SWE-bench Lite runs use the official evaluator
at `f7bbbb2ccdf479001d6467c9e34af59e44a840f9`, with a separately bounded
one-case default because the evaluator builds and runs upstream containers.

## 4. Decisions that were rejected

- **Vendoring a slice of each dataset.** Rejected: it would violate the
  repository's no-vendored-datasets policy and would let the case set drift from
  upstream. Payloads are downloaded at run time into the build-artifact cache
  `target/formal-ai-benchmarks/`, and a test asserts no dataset payload is
  committed.
- **Adding an HTTP client or Rust parquet dependency.** Rejected: the repository
  already fetches through `curl`/`gzip` (issue #362 pattern), and the pinned
  official SWE-bench Python environment already provides `pyarrow`.
- **A non-zero floor so the numbers "look" like progress.** Rejected outright:
  the issue names this as the failure mode being fixed.
- **Treating failure-derived learning as automatic promotion.** Rejected:
  observed case ids and evaluator details become associative evidence and
  proposal-only reports, but behavior changes remain behind human review, the
  base-aware benchmark ratchet, and a real Agent CLI gate.
- **Adding benchmark diagnostics to the R379 debt allowlist.** Rejected in the
  final implementation. User-visible benchmark and work-item messages live in
  the Links seed; the final policy check reports 1362 detected literals and
  1362 existing allowlist entries, with no new debt.

## 5. Verification

| Check | Result |
| --- | --- |
| `cargo test --locked --all-targets --all-features -- --test-threads=1` | pass — 241 integration tests, 482 library tests, 2136 unit tests, 3 ignored, plus every example target |
| `cargo test --test unit external_benchmarks -- --ignored` (network) | runs 20 real upstream HumanEval cases, prints `passed=0 failed=20 total=20` |
| `cargo fmt --all -- --check`, `cargo clippy --locked --all-targets --all-features -- -D warnings` | clean |
| `python3 scripts/audit-total-closure.py`, `scripts/sync-seed.sh --check` | 0 unresolved values; seed mirrors match |
| `scripts/check-hardcoded-language.rs` | in sync (1362/1362 entries) |
| exact issue-solver task through `formal-ai agent` | 3 turns / 2 tool calls; structurally routed to `repository_work_item` without claiming unobserved edits |

## 6. Self-hosting attribution

CI run 29775831898 reported that merging this branch would lower the projected
self-hosting share from 17.14% to 15.74%, because the branch's commits carried
no attribution trailers although formal-ai authored all of them. The remedy the
gate names is to record the trailers while the commits can still be amended, so
the branch was rebased onto `main` with

```
Formal-AI-Session: issue-698-claude-20260720
Formal-AI-Session: issue-698-codex-20260728
Formal-AI-Evidence: dev/log/issues/698/pulls/816/analysis.md
```

on the original implementation commits. Later merge commits and failed Agent
CLI recovery commits are not represented as attributed implementation work.
The share is corrected by attributing work that was in fact machine-authored,
not by shrinking the diff.
