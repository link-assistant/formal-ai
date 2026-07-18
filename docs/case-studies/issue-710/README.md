# Issue #710 — Dropped-Requirements Regression Backlog

Issue [#710](https://github.com/link-assistant/formal-ai/issues/710) is the
tracked regression backlog produced by the 2026-07-14 full-history requirement
audit: every closed issue (all 329) and every merged pull request (all 317) was
re-read against the maintainer's original requirements, konard's follow-up
comments, and the delivery evidence in each thread, then cross-checked against
the repository state at `main` v0.285.0.

## Raw data

The audit reports live in [`raw-data/`](raw-data/):

| File | Scope |
| --- | --- |
| `report-open-issues.md` | Digest of all 31 open issues (requirements, themes, blocking graph) |
| `report-open-prs.md` | State of all 17 open PRs and their unaddressed feedback |
| `report-closed-issues-1-350.md` | Per-issue verdicts for the 183 closed issues ≤ #350 |
| `report-closed-issues-351-plus.md` | Per-issue verdicts for the 146 closed issues > #350, with the consolidated dropped-requirements list |
| `report-merged-prs-first-half.md` | konard-comment audit of merged PRs #2–#328 |
| `report-merged-prs-second-half.md` | konard-comment audit of merged PRs #328–#683, verified against `main` |
| `report-problem-solving-repo.md` | Digest of the konard/problem-solving methodology this project should follow |
| `local-doc-consistency-findings.md` | Documentation inconsistencies found and fixed in the same pass |

## Headline findings

- Of 183 closed issues ≤ #350, only ~15% show clear in-thread delivery
  evidence; of 146 closed issues > #350, roughly half were partially addressed.
- The dominant failure mode is **silent scope-narrowing**: the reported prompt
  gets fixed while the attached generalization, benchmark, or integration
  requirement is dropped.
- Recurring dropped themes: generalization vs. memoization, "all languages"
  narrowed to four, real external benchmarks replaced by local proxies,
  loopback tests instead of real agentic clients, deferred work despite
  "defer nothing" instructions, and standing process clauses (case studies,
  upstream filings) skipped.

## Follow-up structure

The audit produced the E56–E68 planning batch
([#698](https://github.com/link-assistant/formal-ai/issues/698)–[#710](https://github.com/link-assistant/formal-ai/issues/710)),
all sub-issues of [#651](https://github.com/link-assistant/formal-ai/issues/651)
with explicit blocked-by relationships. Issue #710's checklist enumerates the
smaller silently-dropped items; the twelve sibling issues own the large
capability gaps. `ROADMAP.md` gained a requirement-level status table
(done / partial / not done) in the same pass.
