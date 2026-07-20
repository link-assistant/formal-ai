# Issue #698 — real external benchmark harness

Session: `issue-698-claude-20260720`
Pull request: https://github.com/link-assistant/formal-ai/pull/816
Authored by: formal-ai (Claude Code session, autonomous issue solver)

This document is the committed evidence behind the `Formal-AI-Session` /
`Formal-AI-Evidence` trailers on every commit of branch
`issue-698-5abc758210fb`. It records what was measured, what was decided, and
which decisions were rejected — so the numbers published by this pull request
can be audited without re-running anything.

## 1. What the issue asked for

Not another repository-authored slice: the *unmodified upstream* case set, run
end to end, with the resulting score published exactly as measured. Twelve
requirements were extracted and are traced one-by-one in
`docs/case-studies/issue-698/requirements.md` (R698-01 … R698-12) and mirrored
into the repository-wide register as R528 … R533.

## 2. Evidence collected before writing code

Every candidate upstream source was fetched and its license read first. Two
findings changed the design rather than being worked around:

| Finding | Consequence |
| --- | --- |
| `raw.githubusercontent.com` serves a 131-byte Git LFS pointer for the MATH split | the manifest pins the `media.githubusercontent.com` payload URL and records why in `download_note` |
| EditEval ships a harness with no task payload, and its corpora are CC BY-NC / CC BY-NC-SA | the suite is recorded as `benchmark_unavailable` with that reason; instructed text editing runs through Apache-2.0 CoEdIT instead |

Both are the honest outcome the issue asks for: a suite that cannot run is
declared unavailable, never quietly replaced by a repository-local proxy.

## 3. The honest first measurement

Slice 20 per suite, solver `0.300.0`, recorded `2026-07-20`
(`docs/case-studies/issue-698/raw-data/all-suites-first-run.log`):

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

GSM8K's `2 / 20` is two word problems whose final number the solver produced
correctly. Everything else is zero, and zero is what the ledger records. The
failures are ordinary solver output graded by the upstream criterion, not
harness artifacts — the produced Python files are kept under
`target/formal-ai-benchmarks/run/` and quoted in the raw-data logs.

## 4. Decisions that were rejected

- **Vendoring a slice of each dataset.** Rejected: it would violate the
  repository's no-vendored-datasets policy and would let the case set drift from
  upstream. Payloads are downloaded at run time into the build-artifact cache
  `target/formal-ai-benchmarks/`, and a test asserts no dataset payload is
  committed.
- **Adding an HTTP client dependency.** Rejected: the repository already fetches
  through `curl`/`gzip` (issue #362 pattern). The harness reuses it.
- **A non-zero floor so the numbers "look" like progress.** Rejected outright:
  the issue names this as the failure mode being fixed.
- **Rewriting the new diagnostic literals into positional `{}` form so the R379
  hardcoded-language lint stops matching them.** Rejected as lint-gaming. The 52
  new strings are developer diagnostics of exactly the class the allowlist
  already inventories (`src/cli_improve.rs  could not parse {}: {error}`,
  `src/github_logs.rs  failed to create output directory {}: {error}`), so they
  are recorded in the burn-down ledger `scripts/hardcoded-language-allowlist.txt`
  (1317 → 1369 entries) rather than hidden from the scanner.

## 5. Verification

| Check | Result |
| --- | --- |
| `cargo test` (all targets) | pass — 1963 unit tests, 3 ignored |
| `cargo test --test unit external_benchmarks -- --ignored` (network) | runs 20 real upstream HumanEval cases, prints `passed=0 failed=20 total=20` |
| `cargo fmt --check`, `cargo clippy --all-targets --all-features` | clean |
| `actionlint` | clean, including the new `.github/workflows/external-benchmarks.yml` |
| `rust-script scripts/check-hardcoded-language.rs` | in sync (1369 entries) |

## 6. Self-hosting attribution

CI run 29775831898 reported that merging this branch would lower the projected
self-hosting share from 17.14% to 15.74%, because the branch's commits carried
no attribution trailers although formal-ai authored all of them. The remedy the
gate names is to record the trailers while the commits can still be amended, so
the branch was rebased onto `main` with

```
Formal-AI-Session: issue-698-claude-20260720
Formal-AI-Evidence: dev/log/issues/698/pulls/816/analysis.md
```

on every commit. The share is corrected by attributing work that was in fact
machine-authored, not by shrinking the diff.
