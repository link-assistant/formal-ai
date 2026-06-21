# Raw Data — Issue #550

Snapshots downloaded for the case study (the issue's M4 "download all logs and data"
requirement). Captured 2026-06-21 via the GitHub CLI.

| File | Source | Notes |
|---|---|---|
| `issue-550.json` | `gh issue view 550 --json …` | The issue body (five-point defect list + meta-requirements) and metadata. Created 2026-06-21T13:07:31Z by `konard`. |
| `issue-550-comments.json` | `gh api repos/link-assistant/formal-ai/issues/550/comments` | Conversation comments — the deep-analysis comment (2026-06-21T13:55:46Z) linking the tracking-repo case study. |
| `pr-551.json` | `gh pr view 551 --json …` | The fix PR metadata (branch `issue-550-c636b0e4075d`, created 2026-06-21T13:08:48Z). |
| `pr-551-comments.json` | `gh api repos/link-assistant/formal-ai/issues/551/comments` | PR conversation comments. |

The annotated issue screenshot is saved one level up as
[`../screenshot-main.png`](../screenshot-main.png) (the image embedded in the issue
body). Before/after renders of each affected surface are in
[`../screenshots/`](../screenshots/).

> **Mirror.** This issue is mirrored as [hive-mind#1963][hm-issue]; that tracking repo
> holds a sibling case study at `docs/case-studies/issue-1963/` with additional
> cross-repo raw data (predecessor issues #488/#541, code snapshots). This folder is the
> formal-ai-native record.

[hm-issue]: https://github.com/link-assistant/hive-mind/issues/1963
