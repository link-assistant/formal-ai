# Formal AI attempts on real (non-fixture) issues in link-assistant/formal-ai

Collected 2026-11-10, read-only, from `gh` (PRs authored by `app/github-actions`).
Raw data: the pull request list, comments and file lists returned by `gh` on 2026-11-10; `prs.tsv` next to this file,
and the pull requests themselves. "Non-log files" excludes `dev/log/**`, `.gitkeep`
and `.formal-ai/general-change-plan.lino` (same rule as `scripts/hive-mind-draft-result.mjs`).

Classes: USEFUL (change by Formal AI that was merged), NO-CHANGE, WRONG (change or claim that was
rejected or false), INFRA (failure outside the model).

## A. "Hive Mind draft" workflow (`solve ... --tool agent --model formal-ai`), 19 PRs

All 19: Tool `Agent CLI`, model `formalai/formal-ai`, exactly one Formal AI session per PR.
Result: 0 USEFUL, 18 NO-CHANGE, 1 WRONG (false completion claim).

| PR -> issue | State | Non-log / total files | Class | Verbatim reason | Comment |
|---|---|---|---|---|---|
| 1236 -> 1235 | open draft | 0/3 | NO-CHANGE (planned_not_executed) | "Planificado, no ejecutado: https://github.com/link-assistant/formal-ai/issues/1235. El trabajo solicitado sigue sin verificarse." | https://github.com/link-assistant/formal-ai/pull/1236#issuecomment-6251365901 |
| 1329 -> 1328 | MERGED | 786/789 | WRONG for the Formal AI session (see note 1) | "Completed the general change request for v0.354.1 and verified it with `cat 'v0.354.1'`." (`terminal_state "executed"`) | https://github.com/link-assistant/formal-ai/pull/1329#issuecomment-6292367520 |
| 1331 -> 1330 | open draft | 0/3 | NO-CHANGE (planned_not_executed) | "Запланировано, но не выполнено: https://github.com/link-assistant/formal-ai/issues/1330." (Russian) | https://github.com/link-assistant/formal-ai/pull/1331#issuecomment-6294510558 |
| 1369 -> 1364 | open draft | 0/3 | NO-CHANGE (planned_not_executed) | "Planned, not executed: https://github.com/link-assistant/formal-ai/issues/1364. The requested work remains unverified." | https://github.com/link-assistant/formal-ai/pull/1369#issuecomment-6303813148 |
| 1397 -> 1395 | open draft | 0/3 | NO-CHANGE (planned_not_executed) | "Planned, not executed: https://github.com/link-assistant/formal-ai/issues/1395." | https://github.com/link-assistant/formal-ai/pull/1397#issuecomment-6339782108 |
| 1398 -> 1396 | open draft | 0/3 | NO-CHANGE (planned_not_executed) | "Запланировано, но не выполнено: https://github.com/link-assistant/formal-ai/issues/1396." (Russian) | https://github.com/link-assistant/formal-ai/pull/1398#issuecomment-6339797378 |
| 1439 -> 1438 | open draft | 0/3 | NO-CHANGE (planned_not_executed) | "Planned, not executed: https://github.com/link-assistant/formal-ai/issues/1438." | https://github.com/link-assistant/formal-ai/pull/1439#issuecomment-6426742343 |
| 1448 -> 1446 | open draft | 0/3 | NO-CHANGE (planned_not_executed) | "Planificado, no ejecutado: https://github.com/link-assistant/formal-ai/issues/1446." (Spanish) | https://github.com/link-assistant/formal-ai/pull/1448#issuecomment-6440920963 |
| 1456 -> 1455 | closed draft | 0/3 | NO-CHANGE (planned_not_executed) | "Planned, not executed: https://github.com/link-assistant/formal-ai/issues/1455." | https://github.com/link-assistant/formal-ai/pull/1456#issuecomment-6474575472 |
| 1493 -> 1485 | open draft | 0/3 | NO-CHANGE (repeated-action stop) | "Repeated action stopped after 3 attempts (limit 3): read {"filePath":"<workspace>/hive-delivery-attempt.mjs"}. Last result: Error: File not found: <workspace>/hive-delivery-attempt.mjs" | https://github.com/link-assistant/formal-ai/pull/1493#issuecomment-6542210231 |
| 1494 -> 1484 | open draft | 0/3 | NO-CHANGE (repeated-action stop) | "Repeated action stopped after 3 attempts (limit 3): read {"filePath":"<workspace>/formal-ai-draft.mjs"}. Last result: Error: File not found" | https://github.com/link-assistant/formal-ai/pull/1494#issuecomment-6542225128 |
| 1495 -> 1483 | MERGED | 179/182 | NO-CHANGE for the Formal AI session (see note 1) | "Reason: Formal AI produced no change (terminal state: planned_not_executed)." | https://github.com/link-assistant/formal-ai/pull/1495#issuecomment-6542227196 |
| 1496 -> 1488 | open draft | 0/3 | NO-CHANGE (repeated-action stop) | "Repeated action stopped after 3 attempts (limit 3): read {"filePath":"<workspace>/formal-ai-draft.mjs"}." | https://github.com/link-assistant/formal-ai/pull/1496#issuecomment-6542233036 |
| 1497 -> 1487 | open draft | 0/3 | NO-CHANGE (repeated-action stop) | "Repeated action stopped after 3 attempts (limit 3): read {"filePath":"<workspace>/text_formalization.mjs"}." | https://github.com/link-assistant/formal-ai/pull/1497#issuecomment-6542231761 |
| 1498 -> 1486 | open draft | 0/3 | NO-CHANGE (evidence refused) | `{"authored":false,...,"reason":"ReadPolicyRefused",...}` (raw JSON posted as the summary) | https://github.com/link-assistant/formal-ai/pull/1498#issuecomment-6542235255 |
| 1499 -> 1489 | open draft | 0/3 | NO-CHANGE (evidence refused) | `{"authored":false,...,"reason":"ReadPolicyRefused",...}` | https://github.com/link-assistant/formal-ai/pull/1499#issuecomment-6542264202 |
| 1502 -> 1501 | MERGED | 55/57 | NO-CHANGE for the Formal AI session (see note 1) | `{"authored":false,...,"reason":"ReadPolicyRefused",...}`; the JSON embeds the signed-out GitHub HTML page of the issue ("Navigation MenuSign in...") as the requirement text | https://github.com/link-assistant/formal-ai/pull/1502#issuecomment-6558350192 |
| 1508 -> 1507 | open draft | 0/3 | NO-CHANGE (repeated-action stop) | "Repeated action stopped after 3 attempts (limit 3): read {"filePath":"<workspace>/Node.js"}. Last result: Error: File not found: <workspace>/Node.js" | https://github.com/link-assistant/formal-ai/pull/1508#issuecomment-6563598758 |
| 1549 -> 1548 | open draft | 0/3 | NO-CHANGE (planned_not_executed) | "Planificado, no ejecutado: https://github.com/link-assistant/formal-ai/issues/1548." then "Reason: Formal AI produced no change (terminal state: planned_not_executed)." | https://github.com/link-assistant/formal-ai/pull/1549#issuecomment-6760365706 and https://github.com/link-assistant/formal-ai/pull/1549#issuecomment-6760368333 |

19 rows = 16 unmerged drafts + 3 merged PRs (1329, 1495, 1502).

Note 1 (merged PRs 1329, 1495, 1502). Each has a single Hive Mind session comment (Agent CLI,
formalai/formal-ai) and that session added only development-log files. The implementation commits
appear hours later with hand-style subjects on differently named local branches, e.g.
`db5f9c16 2026-11-01T05:50:10Z fix(agent): connect observed source to guarded work-item edits`
after `Merge remote-tracking branch 'origin/main' into work/issue-1483-evidence-to-edit` (PR 1495);
`bfb3dcf3 2026-11-01T16:15:07Z Fix file-command intent routing and main qualification regressions`
after `Merge ... into work/issue-1501-routing` (PR 1502); `2253676f 2026-10-20T16:11:26Z fix(agent):
bound repeated actions and preserve failure evidence (#1328)` (PR 1329). Commit author is
`github-actions[bot]` for all commits, and no Hive Mind session comment accompanies them.
So the merged content was NOT produced by the recorded Formal AI session. Who produced it is
UNVERIFIED (no tool/model line exists for those commits).
PR 1329's session is classed WRONG because it reported `terminal_state "executed"` and
"Completed the general change request for v0.354.1" although its commits contain only log files
(commit list of the pull request); context line
"83.9K / 39798.1M (0%) input tokens, 13.0K / 8.2K (158%) output tokens".

Issue-level comment "Formal AI through Hive Mind did not produce a verified new draft PR in this
attempt." exists once on each of issues 1455, 1483, 1484, 1485, 1486, 1487, 1488, 1489, 1501, 1507,
1548 (run URLs: column `issue_not_verified_runs` of `prs.tsv`). Earlier attempts (issues 1235, 1328,
1330, 1364, 1395, 1396, 1438, 1446) have no such comment and their workflow runs concluded
`success` although the PR held no useful file.

Shared symptoms across these PRs:
- Summary language does not follow the issue language: Spanish (1236, 1448, 1549), Russian (1331, 1398)
  on English issues. The text comes from Formal AI's own seed
  (`data/seed/multilingual-responses-agentic.lino:821`), Hive Mind only relays it.
- Context line shows an implausible window, e.g. "31.1K / 40285.2M (0%) input tokens, 5.4K / 8.2K (66%)
  output tokens"; output over the stated limit on 1329 (158%), 1369 (275%), 1502 (112%).
- Since 2026-10-31 the "repeated action" guard stops the session after three reads of a file name
  taken from the issue text that does not exist at the workspace root.

## B. "Self-authored pull request" workflow (September 2026; does not call Hive Mind), 13 PRs

| PR -> issue | State | Non-log / total | Class | Reason (verbatim where quoted) |
|---|---|---|---|---|
| 1092 -> 1091 | closed | 0/1 | NO-CHANGE | "Formal AI could not author the change in run ..." |
| 1093 -> 1091 | closed | 1/5 | WRONG (duplicate) | konard: "every re-run of the self-authored workflow re-authored the same task here" |
| 1094 -> 1091 | closed | 1/5 | WRONG (duplicate) | same task re-authored |
| 1097 -> 1091 | closed | 1/5 | WRONG | closed over the changelog fragment contract (konard comment) |
| 1098 -> 1091 | closed | 0/1 | NO-CHANGE | "the two-artifact prompt ended after the first clause (#1099)" |
| 1100 -> 1091 | closed | 2/6 | INFRA | branch cut before the base fix |
| 1102 -> 1091 | closed | 214/218 | INFRA | opened against main carrying every commit of the branch |
| 1103 -> 1091 | MERGED | 2/6 | USEFUL | `scripts/self-hosting-attribution.rs` + changelog fragment |
| 1108 -> 1091 | closed | 1/2 | NO-CHANGE | could not author; task already solved by #1103 |
| 1114 -> 1113 | closed | 1/2 | INFRA | konard: "GitHub refuses to let `GITHUB_TOKEN` push a file under `.github/workflows/` without the `workflow`..." |
| 1121 -> 1120 | closed | 2/6 | WRONG | konard: "the artifact does not compile. Formal AI emitted the contract's escape sequences literally" |
| 1122 -> 1120 | MERGED | 2/6 | USEFUL | merged |
| 1124 -> 1123 | MERGED | 2/6 | USEFUL | merged |

Totals: 3 USEFUL, 3 NO-CHANGE, 4 WRONG, 3 INFRA. Exact comment URLs for this section are in
the pull requests; they were not re-read for this file after the first pass.

## C. Not verified
- Who authored the merged implementation commits of PRs 1329, 1495, 1502.
- Why three "Hive Mind draft" runs of 2026-10-31 concluded `success` without a matching PR
  (41250180330, 41191464467, 41191462690).
- CI results of any of these PRs were not inspected.
