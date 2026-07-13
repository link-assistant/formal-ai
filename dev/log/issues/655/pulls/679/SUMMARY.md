# Data Collection — Issue #655 / PR #679

Collected: 2026-07-13

## Issue #655 — E36: Hive-Mind-dispatched end-to-end issue solve by Formal AI

- State: OPEN
- Parent: #651 (E36 in `docs/case-studies/issue-651/proposed-issues.md`)
- Blocked-by: #654 · Blocking: #657
- Author: konard

**Goal:** A real self-coding loop where Hive Mind (`solve <issue-url> --tool
agent --model formal-ai`) drives the Agent CLI, which drives `formal-ai serve
--agent-mode`, taking a small repo issue from plan to draft PR. A committed,
replayable session must exist plus offline byte-for-byte inner-loop replay,
scratch-issue verification, and a documented recipe in `CONTRIBUTING.md`.

### Issue comment (1)
- 2026-07-13T04:21:51Z — automated **"Solution Draft Failed"**: solver stopped
  before opening a PR; requested model `formal-ai` rejected with `Invalid model
  name` (the same upstream Hive Mind validation gap the PR reports).

## PR #679 — Add replayable Hive Mind self-coding verification

- State: DRAFT · Base: main · Head: issue-655-352ae09c0c14
- +917 / −73
- Mergeable: **DIRTY (conflicts with main)** — see below.

### Conversation comments (5)
1. 2026-07-13T04:32:01Z — working-session summary (implemented, pushed, ready).
2. 2026-07-13T04:32:16Z — solution draft log (Gist link, cost/token stats).
3. 2026-07-13T04:34:47Z — "Ready to merge" (auto-restart-until-mergeable).
4. **2026-07-13T17:22:56Z — NEW feedback comment (konard):** redo the analysis
   and fully implement the vision from #655 using auto learning and Formal AI
   via Agent CLI; generalize logic and advance the meta algorithm; architecture
   changes are permitted; do everything in this single PR until every
   requirement is fully addressed.
5. 2026-07-13T17:53:30Z — AI Work Session Started (PR converted to draft).

### Review comments / reviews
- None.

### CI checks (run 29223851589)
- Pass: Lint and Format Check, Detect Changes, Version Modification Check.
- All test/build/e2e jobs: **skipping** (path filters / draft).

## Merge conflict status

`git merge-tree` against `origin/main` (merge-base
`a78f5600a31be2439148fe2ac54863f6c16c5ee6`) reports **conflicts**. 45 files
changed in both branches; see `conflicting-files.txt`. Notable code conflicts:
`src/agentic_coding/mod.rs`, `src/agentic_coding/planner.rs`,
`src/agentic_coding/self_heal.rs`, `src/engine.rs`, `tests/unit/mod.rs`,
plus many `data/seed/*.lino`, `src/web/*`, and workflow files. main has moved
substantially since the branch point.

## Files in this folder

| File | Contents |
|------|----------|
| `issue-655.json` | Full issue metadata + body |
| `issue-655-comments.json` | Issue comments |
| `pr-679.json` | Full PR metadata, files, commits, checks |
| `pr-679-conversation-comments.json` | PR discussion comments |
| `pr-679-review-comments.json` | Inline review comments (empty) |
| `pr-679-reviews.json` | PR reviews (empty) |
| `pr-679.diff` | Full PR diff |
| `pr-679-checks.txt` | CI check rollup |
| `branch-commits.txt` | Recent branch commits |
| `merge-tree.txt` | `git merge-tree` output vs origin/main |
| `conflicting-files.txt` | Files changed in both branches |
| `main-changed-files.txt` | Files main changed since merge-base |

## Next actions implied by data
1. Resolve the merge conflict with `main` (DIRTY status).
2. Address the 2026-07-13T17:22:56Z feedback: redo/complete the vision per #655.
3. Outer live dispatch still blocked upstream (hive-mind#2059, `formal-ai`
   absent from the model-validation map).
