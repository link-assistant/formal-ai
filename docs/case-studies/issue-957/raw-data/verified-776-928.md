# Verification summary — requirements #776–#928 (131 items, verified against local main @ v0.326.0, 2026-08-04)

## Verdict counts

| Verdict | Count |
|---|---|
| DELIVERED | 60 |
| PARTIAL | 25 |
| NOT-DELIVERED | 44 |
| UNVERIFIABLE-LOCALLY | 2 |
| OBSOLETE | 0 |
| **Total** | **131** |

All 44 NOT-DELIVERED items are either tracked by an open issue/epic or flagged `needs_issue`.

## Items needing NEW issues (needs_issue=true, 5)

1. **R838-1 (bug, confirmed in code)** — Report export writes `formal-ai-*-<session>.lino` into the caller's CWD: `export_command` (src/agentic_coding/report_issue.rs:367-378) builds a bare relative output filename (`formal-ai-harness-{dialog_id}.lino`), so the generated report script drops files wherever it runs. The two stray files in the repo root (`formal-ai-harness-latest.lino`, `formal-ai-server-latest.lino`, dated Jul 25) are its artifacts.
2. **R873-2** — Versioned recoverable memory (snapshot/rollback, immutable baseline tests). No artifact anywhere in src/memory*, memory_sync.rs, associative_persistence.rs; not absorbed by any epic (E72 covers research, not memory versioning).
3. **R873-3** — Bounded autonomy: configurable stuck-recovery time limit (default 1h), full-trust auto-select mode, per-command permission mode. Zero artifacts.
4. **R824-1** — Mutating-action ladder rungs (824.L1–L4, mkdir/mv with pre/post verification, sandbox-reset semantics). Explicitly deferred in #824 comment 5073614623; the deferred design was never filed as its own tracker.
5. **R790-1** — Self-coding harness created issues #784/#786/#789/#790/#791 despite explicit instructions not to; no guard exists (may belong upstream in hive-mind, but nothing tracks it anywhere).

## NOT-DELIVERED, tracked by open issues (no new issue needed)

- **August defect cluster**: #905 (PR #927 open, last session FAILED), #906 (PR #928 open), #907, #908, #909 (zero fix artifacts — gemini client effectively unusable; `--global` writes broken headless configs; exit-code-blind verification unpinned). **#904's PR #926 is NOT in local main** — HEAD v0.326.0 immediately follows #925's merge; the chunk's "merged 2026-08-04" note is wrong for this clone (handcheck remote).
- **Untracked-work-wise but open-issue-tracked**: #802 (2-4-6 disproof methodology), #825 (autocomplete — zero refs in src/web app code), #836 (illegal-request warning), #861 (Sentry — zero refs), #901 (TRIZ — zero refs), #862/#863/#865/#866/#867/#868/#872 (agentic-CLI failure reports, none pinned by any test), #873 (research-when-unknown principle).
- **E68 audit children**: #889, #891, #892 (Spider-Man snapshot confirmed still in data/seed/facts.lino), #893, #894, #895 (coverage job exists at release.yml:635 but no ratchet).
- **Epics E69–E77** (#916–#924): all open; R916-1 is PARTIAL (#902/#903 fixed with tests) — the rest have no delivery artifacts. E71's target src/solver_handlers/ still has ~40 files.
- **Unmerged green PRs**: #887 (anticipatory dreaming) and #888 (E68 re-verification) have been CI-green + "Ready to merge" since 2026-08-01 — maintainer action, not agent work, is the blocker.
- **R823-2**: the 20% self-coding floor was never recorded anywhere; scripts/self-hosting-metric.rs + GOALS.md:108 honestly say ~0%. Tracked by #924.
- **R848-2** hive-mind gating → #921.

## UNVERIFIABLE-LOCALLY / hand-check list (15 handchecks total)

Runtime/e2e checks:
- R800-1: run the RU amazon.in charger prompt on v0.326.0; verify non-empty synthesized research.
- R801-1 / R821-1: run "Search (online) for Elon Musk"; verify cited multi-source summary at reference parity.
- R826-1 / R827-1: re-run the RU prompts (ФБС vs ФБО; фуфломицин + anaphora follow-up) against the shipped synthesis/coreference machinery, then close or refile the source reports.
- R781-8 / R781-10 / R876-1: live multi-turn depth per matrix client; orchestration corrective-feedback resume.
- R819-9: minimal-actions behavior.

External-tracker checks:
- R883-2: link-foundation/meta-language filings from the PR #883 window.
- R819-4: OpenCode upstream re-render bug report.
- R841-2: command-stream#175/#180 + agent-commander#43/#46 status; whether published packages now own the capture layer (no dep on either exists in Cargo.toml/package.json today).
- R912-1: post-#912 web-search/web-capture filings.
- R904-1: whether PR #926 merged remotely after this clone's last fetch.
- R887-1/R888-1: merge decision on the two green PRs.

## PR #823 self-coding demands vs E69–E77 coverage

- Recursive-descent-until-solvable protocol → covered by **E77 (#924)** + foundation **E69 (#916)**; learning wiring itself IS live (src/rule_synthesis.rs:84 `approved_lesson_for`, tests/unit/issue_823_recursive_learning.rs, PR #817).
- 20% leaf-task self-coding floor → restated by **E77** but the *measured floor* (ratio + decomposition tree per case study) is not in any epic's acceptance text — partially orphaned; noted under R823-2.
- Learning adoption (traces→ledger→lesson) → delivered (#817) + continued by **E75 (#922)**.
- Guiding-doc sync → delivered (#886/#915; GOALS.md:104-108 honest 0% ratchet, VISION.md decomposition vocabulary, docs/philosophy.md).
- **Orphan found**: no epic carries the "coding ladder runs in CI" definition-of-done from #848 — the coding ladder (experiments/issue_847_coding_ladder, score 65/130, **L1 0/16**) is not referenced by any workflow; only #842's 24-node task ladder (24/24) runs in CI (task-ladder.yml). E69 implies it but doesn't state the CI requirement.

## Surprising discoveries

1. **Stray-file bug is real and current** (see needs_issue #1): report exports land in the caller's CWD.
2. **Coding ladder never made it into CI** despite #848 closing "delivered": recorded score 65/130 with L1 = 0/16 — the exact "honest attempt at L1" bar konard set is still failing, and nothing ratchets it.
3. **PR #926 not on local main** although the requirements dump recorded it as merged — either a remote-race or an over-claim; hand-check.
4. **The task ladder is at 24/24** (from the 8/24 baseline konard set as the bar) — the #838/#827/#826 arc genuinely closed at ladder level, but the three source issues remain open awaiting runtime re-verification.
5. **No local PTY/VT code exists** (R841-2's end-state goal), but neither command-stream nor agent-commander appears as a dependency — the capture layer was re-architected into seed-driven client contracts rather than upstreamed-and-depended-on as konard directed.
6. **#745/#758 were never formally reopened** after konard's own comment proved their acceptance contradicted measured behavior; the ladder ratchet now covers the regression, but the maintainer decision is still dangling.

## Files

- Verdicts: `verified-776-928.ndjson` (131 records; input + verdict/evidence/tracked_by/needs_issue/handcheck)
- This summary: `verified-776-928.md`
