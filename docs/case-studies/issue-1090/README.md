# Issue #1090 (E112) — manual-confirmation column finished or retired

## The decision

716 of 776 rows (at the issue's measurement; 1,132 of 1,214 at today's
rebuild) read `not yet confirmed`. The issue offered two branches:

1. **Fill** the column from the replayable session captures the
   agentic-CLI matrix already produces.
2. **Retire**: mark the column aspirational in its header and add no new
   ledger until an existing one is complete.

This branch takes **retire**, for the reason the table itself already
carried: a `not yet confirmed` row names the recorded truth — machinery
pinned by an automated test, no hand run recorded — so the honest move is
to stop treating the column's emptiness as 1,132 units of debt and start
treating it as the resting state it always was. The fill branch remains
the way the column eventually earns its place; nothing here blocks it.

## Deliverables

| Surface | Change |
|---|---|
| `docs/requirements-traceability.md` | Header carries the status block: *aspirational*, decided 2026-09-30, `not yet confirmed` carries no debt and gates nothing, the retire rule quoted, pointers to the manifest line and the pinning test. |
| `CONTRIBUTING.md` | "Manual confirmation is aspirational (issue #1090)" section: no new manual-confirmation ledger until an existing one completes; rows fill from replayed session captures or stay `not yet confirmed`. |
| `scripts/generate-requirement-status.rs` | `render_manifest` emits `manual_column "aspirational since 2026-09-30 (issue #1090): not yet confirmed rows carry no debt"`, so every regeneration keeps the decision. |
| `data/meta/requirement-status-ledger.lino` | The same line hand-applied to the committed manifest (byte-identical to what the generator now emits). |
| `rust/tests/unit/issue_1090_manual_column_retired.rs` | Pins the branch: the header carries the status and the rule, CONTRIBUTING states it, the manifest line exists, the generator emits it, and every requirement keeps its manual row (>1,000) — the rows' vocabulary is deliberately unchanged, because `docs_requirements/issue_1021` pins `not yet confirmed` as the honest cell text. |

## How to test

The unit test above is the issue's "a test reads the table and asserts
whichever branch was taken". Unconfirmed rows remain above 100 — the
retire branch's other acceptance leg (`the header carries the status and
CONTRIBUTING.md states the rule`) is what is satisfied instead.
