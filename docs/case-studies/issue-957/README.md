# Issue 957 case study — full-history requirements audit (2026-08-04)

Raw data of the complete audit of every issue, pull request, and comment (#1–#929, 944 requirements extracted and individually verified against the codebase) that produced:

- `docs/requirements-traceability.md` (the #957 data layer, commit eff19114)
- the E78–E107 epic batch (#930–#959), three bug issues (#960–#962), and #964
- the living-docs refresh (commit f86259fa)

## Contents of `raw-data/`

- `req-chunk-<range>.ndjson` / `.md` — requirements extracted from konard's issue/PR bodies and comments, per item-number range, with source URLs and thread-resolution status.
- `verified-<range>.ndjson` / `.md` — per-requirement delivery verdicts (DELIVERED / PARTIAL / NOT-DELIVERED / OBSOLETE / UNVERIFIABLE-LOCALLY) with concrete code/test evidence.
- `arch-review.md` — 25-finding architecture review (JS-doctrine inventory, handler-migration state, terminology, memoized surfaces).
- `docs-audit.md` — the 45-edit living-documentation audit plan applied in f86259fa.
- `test-report.md` — local full-suite + hand-check battery results (macOS, 2026-08-04).
- `issues-manifest.md` — consolidation manifest mapping findings → filed issues, dedup table, maintainer actions.

Verdict totals: 681 DELIVERED · 180 PARTIAL · 77 NOT-DELIVERED · 3 OBSOLETE · 3 UNVERIFIABLE-LOCALLY (of 944).
