# Issue Survey — Repository State At Planning Time

Collected with `gh issue list --state all` at planning time for issue #244. The
full machine dump is intentionally **not** vendored: the per-issue requirement
history already lives in `REQUIREMENTS.md`, and reproducing every historical
issue title verbatim trips the repository-hygiene guard
(`tests/unit/docs_requirements.rs::repository_text_avoids_deferred_labels_requested_by_issue_103`).
This file records the survey conclusions instead.

## Counts

- **Total issues (open + closed):** 127.
- **Open issues:** 1 — only **#244 "Plan issues to implement our vision fully"**
  (this issue) is open.

## Conclusions used by the plan

- **No duplicate planning issue exists.** Because #244 is the only open issue,
  none of the proposed epics (E1–E14) duplicate a pre-existing open issue. They
  are all new.
- **History is reconstructable from `REQUIREMENTS.md`.** Closed issues map to the
  R-row blocks in `REQUIREMENTS.md` (R1 … R236 across issue sections up to
  Issue #244), so the implemented baseline the plan builds on is already tracked
  there and in `ROADMAP.md` §2 ("the regression floor").
- **The vision gaps are encoded as tests, not open issues.** The remaining work
  is enumerated by the 69 `#[ignore]` "tracked requirement" tests under
  `tests/unit/specification/` (see `code-audit.md`), which the epics graduate.
