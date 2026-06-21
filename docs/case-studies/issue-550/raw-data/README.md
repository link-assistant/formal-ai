# Raw Data — Issue #550

Snapshots downloaded for the case study (the issue's M4 "download all logs and data"
requirement). Captured 2026-06-21 via the GitHub CLI.

| File | Source | Notes |
|---|---|---|
| `issue-550.json` | `gh issue view 550 --json …` | The issue body (five-point defect list + meta-requirements) and metadata. Created 2026-06-21T13:07:31Z by `konard`. |
| `issue-550-comments.json` | `gh api repos/link-assistant/formal-ai/issues/550/comments` | Conversation comments — the deep-analysis comment (2026-06-21T13:55:46Z) linking the tracking-repo case study. |
| `pr-551.json` | `gh pr view 551 --json …` | The fix PR metadata (branch `issue-550-c636b0e4075d`, created 2026-06-21T13:08:48Z). |
| `pr-551-comments.json` | `gh api repos/link-assistant/formal-ai/issues/551/comments` | PR conversation comments. |
| `formal-ai-issue-488.json` | `gh issue view 488 --json …` | Predecessor issue — the thinking-preview scrolling/fade work that P1 regressed from. |
| `formal-ai-issue-541.json` | `gh issue view 541 --json …` | Predecessor issue — the collapsed-thinking sizing rules referenced by P1/P2. |
| `code-snapshots-v0.214.0.md` | `git show @ v0.214.0` | The exact buggy source regions (`styles.css`, `thinking.rs`, `app.js`) at the reported version, referenced by the root-cause analysis. |

The annotated issue screenshot is saved one level up as
[`../screenshot-main.png`](../screenshot-main.png) (the image embedded in the issue
body). Before/after renders of each affected surface are in
[`../screenshots/`](../screenshots/).

> **Mirror.** This issue is mirrored as [hive-mind#1963][hm-issue]; that tracking repo
> held a sibling case study at `docs/case-studies/issue-1963/` (tracking PR
> [hive-mind#1964][hm-pr]). All of its findings — including the cross-repo raw data above
> (predecessor issues #488/#541 and the v0.214.0 code snapshots) — are now incorporated
> into this folder, so this is the single authoritative, formal-ai-native record.

[hm-pr]: https://github.com/link-assistant/hive-mind/pull/1964

[hm-issue]: https://github.com/link-assistant/hive-mind/issues/1963
