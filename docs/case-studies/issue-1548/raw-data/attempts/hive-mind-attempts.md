# Formal AI attempts on real issues of link-assistant/hive-mind

Measured on 2026-11-10 with read-only `gh` calls: pull request lists, pull request
comments, commit file lists and workflow runs of `formal-ai-draft.yml`. The draft
workflow of that repository runs `solve <issue> --tool agent --model formal-ai`
for a new issue. A session is counted as a Formal AI session when its comments are
written by `github-actions[bot]`; 70 of the 135 sessions of the first population
also carry the lines "Tool: Agent CLI" and "Model: formalai/formal-ai".

## Population

- 133 pull requests for 132 issues, created 2026-10-04T21:50Z to 2026-11-10T05:55Z:
  122 with the label `formal-ai-draft` and 11 without it, opened by the workflow on
  `issue-N-*` branches. The 272 "Implement Hello World in ..." fixture pull requests
  are not counted.
- 130 of the 133 have a Formal AI session comment; they hold 135 sessions (129 pull
  requests with one session, pull request 3249 with six).
- 5 more pull requests (4036 to 4040) were opened on 2026-11-10 between 10:57Z and
  10:59Z for the five reports of section 6 of the case study; they are listed
  separately below. Formal AI version of these five sessions: 0.355.33.

## Result of the 135 sessions

| Count | Final message of the session | Pull requests | Result |
| --- | --- | --- | --- |
| 65 | "AGENT execution failed with Agent reported error: Error: File not found: `<workspace>`/e.g" | 2499 to 3225, 2026-10-04 to 2026-10-14 | no change |
| 56 | "Planned, not executed: `<issue address>`." | 3280 to 4030, 2026-10-14 to 2026-11-10 | no change |
| 5 | "Pull request feedback needs source review before this prepared work can be marked ready: Exit code 1 specify only one of --comments or --json" | 3249, restarts 1 to 5 | no change |
| 3 | "Did not write `<file>`: the request names it only inside quoted text, not as the file to write, so nothing was changed." | 3575, 3615, 3726 | no change |
| 2 | "Repeated action stopped after 3 attempts (limit 3): read {"filePath":"`<workspace>`/`<name>`.mjs"}. Last result: Error: File not found" | 3952, 3959 | no change |
| 1 | "Left `.gitmodules` unchanged: it already has content, and the request does not say to replace the whole file." | 3436 | no change |
| 1 | "The work item at `<issue address>` could not be read, so nothing was planned for it and no plan record was written." | 3437 | no change |
| 1 | raw JSON as the summary, `"reason":"ReadPolicyRefused"` | 4028 | no change |
| 1 | "Added "file", "HTML", "text", ... to the list in `e.g` and observed the result." | 3249, first session | false completion claim |

No summary of the 70 sessions that have one starts in a language other than
English (first sentence scanned; the language of each issue was not checked).

Files: 371 commits carry the `github-actions[bot]` identity on 129 pull requests:
121 "Initial commit with task details" (`.gitkeep`), 249 "Add development log",
1 revert of the initial commit. No file other than `.gitkeep` and logs was
changed by a Formal AI session inside any of the 130 pull requests. The first
session of pull request 3249 wrote `e.g` (391 lines) and `formal-ai-draft.yml`
(391 lines) at the repository root and 20 files under `data/cache/derivations/`;
the orchestrator pushed them to `recovery/issue-3248-3f2da58dcbb9` (commit
`82ac9c40`), not to the pull request.

Merged content: 42 of the 133 pull requests are merged, 35 are open and ready, 56
are open drafts. In all 42 merged pull requests every file other than logs comes
from commits with the identity of the repository owner; the session log comments
on them name Claude Code only (28), Codex only (11) or both (3). Before the
merge, the Formal AI session had ended with the `e.g` failure (22), with
"Planned, not executed" (18), or there was none (2: pull requests 2924 and 3507).

The `e.g` failure has two sides, both recorded in the case studies of
link-assistant/hive-mind: the model asked to read a file named `e.g` (the
abbreviation in the solver prompt; `docs/case-studies/issue-3108/README.md`:
"This is a model quality problem"), and the client ended the run on that read
error (`docs/case-studies/issue-3215/README.md`: "A recoverable `File not
found: .../e.g` tool error was treated as fatal"). All 65 comments also say "Log
Upload Failed", so no log of these sessions was read.

## The five sessions of 2026-11-10 (Formal AI 0.355.33)

| Pull request | Issue | Final message | Files other than logs |
| --- | --- | --- | --- |
| [4036](https://github.com/link-assistant/hive-mind/pull/4036#issuecomment-6765594974) | 4034 | "Created and verified `main.js` through the agentic CLI harness." `console.log("Hello, world!");` "Committed and pushed" | `main.js` |
| [4037](https://github.com/link-assistant/hive-mind/pull/4037#issuecomment-6765619204) | 4031 | "The command failed: `test -z "$(git status --porcelain)" && pr_draft=$(gh pr view ... --json isDraft --jq .isDraft) && ... gh pr ready ...` exited with code 1." | `main.rs` (commit "feat: add main.rs") |
| [4038](https://github.com/link-assistant/hive-mind/pull/4038#issuecomment-6765586168) | 4032 | "Did not write `src/solve.mjs`: the request names it only inside quoted text, not as the file to write, so nothing was changed." | none |
| [4039](https://github.com/link-assistant/hive-mind/pull/4039#issuecomment-6765629107) | 4035 | "The command failed: `scala run Main.scala` exited with code 127." "scala: command not found" | none |
| [4040](https://github.com/link-assistant/hive-mind/pull/4040#issuecomment-6765604517) | 4033 | "Repeated action stopped after 3 attempts (limit 3): read {"filePath":"`<workspace>`/src/solve.results.lib.mjs"}. Last result:" followed by the content of the file | none |

None of the five issues asks for a program. The text of issues 4031, 4034 and
4035 contains "hello" only inside the addresses of the three Hello World test
repositories; 4031 also contains "Rust log" and 4035 the workflow name "Run
Main.scala". All five pull requests are open drafts.

## One row per pull request of the first population

Newest first. "Commits" is the number of commits with the `github-actions[bot]`
identity. "Other sessions" lists the session log comments of other tools on the
same pull request; it does not map commits to sessions.

| Pull request | Issue | State | Commits | Final message | Other sessions |
| --- | --- | --- | --- | --- | --- |
| [4030](https://github.com/link-assistant/hive-mind/pull/4030#issuecomment-6760307803) | 4029 | open draft | 3 | planned, not executed | - |
| [4028](https://github.com/link-assistant/hive-mind/pull/4028#issuecomment-6739725900) | 4027 | open draft | 4 | raw JSON | - |
| [4026](https://github.com/link-assistant/hive-mind/pull/4026#issuecomment-6737732350) | 4025 | MERGED 2026-11-09 | 3 | planned, not executed | Claude Code / Claude Opus 5.5 x1 |
| [4024](https://github.com/link-assistant/hive-mind/pull/4024#issuecomment-6690437355) | 4023 | MERGED 2026-11-09 | 4 | planned, not executed | Codex / GPT-6.1 Sol x1 |
| [4022](https://github.com/link-assistant/hive-mind/pull/4022#issuecomment-6635858022) | 4021 | MERGED 2026-11-05 | 4 | planned, not executed | Codex / GPT-6.1 Sol x1 |
| [3959](https://github.com/link-assistant/hive-mind/pull/3959#issuecomment-6559846767) | 3958 | open draft | 4 | repeated read stopped | - |
| [3953](https://github.com/link-assistant/hive-mind/pull/3953#issuecomment-6559557470) | 3951 | open draft | 4 | planned, not executed | - |
| [3952](https://github.com/link-assistant/hive-mind/pull/3952#issuecomment-6559534040) | 3950 | open draft | 4 | repeated read stopped | - |
| [3929](https://github.com/link-assistant/hive-mind/pull/3929#issuecomment-6547735203) | 3924 | open draft | 4 | planned, not executed | - |
| [3923](https://github.com/link-assistant/hive-mind/pull/3923#issuecomment-6547555973) | 3922 | MERGED 2026-10-31 | 3 | planned, not executed | Claude Code / Claude Opus 5.5 x1 |
| [3913](https://github.com/link-assistant/hive-mind/pull/3913#issuecomment-6543877089) | 3910 | open draft | 4 | planned, not executed | - |
| [3912](https://github.com/link-assistant/hive-mind/pull/3912#issuecomment-6543874530) | 3909 | open, ready | 4 | planned, not executed | Claude Code / Claude Opus 5.5 x1 |
| [3864](https://github.com/link-assistant/hive-mind/pull/3864#issuecomment-6526841713) | 3863 | MERGED 2026-11-01 | 4 | planned, not executed | Claude Code / Claude Opus 5.5 x3 |
| [3838](https://github.com/link-assistant/hive-mind/pull/3838#issuecomment-6467848338) | 3837 | MERGED 2026-10-28 | 3 | planned, not executed | Codex / GPT-6.1 Sol x2 |
| [3832](https://github.com/link-assistant/hive-mind/pull/3832#issuecomment-6465108855) | 3831 | MERGED 2026-10-28 | 3 | planned, not executed | Codex / GPT-6.1 Sol x1 |
| [3786](https://github.com/link-assistant/hive-mind/pull/3786#issuecomment-6399326075) | 3785 | MERGED 2026-11-09 | 0 | planned, not executed | Claude Code / Claude Opus 5.5 x3; Codex / GPT-6.1 Sol x2; Codex / model line absent x1 |
| [3762](https://github.com/link-assistant/hive-mind/pull/3762#issuecomment-6386800580) | 3759 | MERGED 2026-10-24 | 4 | planned, not executed | Codex / GPT-6.1 Sol x1 |
| [3748](https://github.com/link-assistant/hive-mind/pull/3748#issuecomment-6375668268) | 3747 | open draft | 4 | planned, not executed | - |
| [3742](https://github.com/link-assistant/hive-mind/pull/3742#issuecomment-6364134143) | 3740 | open draft | 4 | planned, not executed | - |
| [3741](https://github.com/link-assistant/hive-mind/pull/3741#issuecomment-6364031946) | 3739 | open, ready | 4 | planned, not executed | Claude Code / Claude Opus 5.5 x1 |
| [3726](https://github.com/link-assistant/hive-mind/pull/3726#issuecomment-6362486601) | 3724 | open draft | 4 | did not write, quoted text | - |
| [3725](https://github.com/link-assistant/hive-mind/pull/3725#issuecomment-6362477688) | 3723 | open draft | 4 | planned, not executed | - |
| [3720](https://github.com/link-assistant/hive-mind/pull/3720#issuecomment-6361516317) | 3719 | open draft | 4 | planned, not executed | - |
| [3692](https://github.com/link-assistant/hive-mind/pull/3692#issuecomment-6325297838) | 3690 | open draft | 4 | planned, not executed | - |
| [3691](https://github.com/link-assistant/hive-mind/pull/3691#issuecomment-6325214014) | 3689 | open draft | 4 | planned, not executed | - |
| [3686](https://github.com/link-assistant/hive-mind/pull/3686#issuecomment-6322903377) | 3684 | MERGED 2026-10-22 | 4 | planned, not executed | Claude Code / Claude Opus 5.5 x1 |
| [3661](https://github.com/link-assistant/hive-mind/pull/3661#issuecomment-6314164145) | 3656 | open, ready | 4 | planned, not executed | Claude Code / Claude Opus 5.5 x1 |
| [3631](https://github.com/link-assistant/hive-mind/pull/3631#issuecomment-6308782596) | 3630 | open, ready | 4 | planned, not executed | Claude Code / Claude Opus 5.5 x1 |
| [3615](https://github.com/link-assistant/hive-mind/pull/3615#issuecomment-6301440778) | 3614 | open, ready | 4 | did not write, quoted text | Claude Code / Claude Opus 5.5 x2; Codex / GPT-6.1 Sol x1 |
| [3611](https://github.com/link-assistant/hive-mind/pull/3611#issuecomment-6299723949) | 3606 | open draft | 4 | planned, not executed | - |
| [3605](https://github.com/link-assistant/hive-mind/pull/3605#issuecomment-6299144989) | 3604 | open draft | 4 | planned, not executed | - |
| [3585](https://github.com/link-assistant/hive-mind/pull/3585#issuecomment-6292026022) | 3583 | MERGED 2026-10-21 | 4 | planned, not executed | Claude Code / Claude Opus 5.5 x1 |
| [3584](https://github.com/link-assistant/hive-mind/pull/3584#issuecomment-6291972650) | 3582 | open draft | 4 | planned, not executed | - |
| [3575](https://github.com/link-assistant/hive-mind/pull/3575#issuecomment-6290987263) | 3572 | open draft | 4 | did not write, quoted text | - |
| [3574](https://github.com/link-assistant/hive-mind/pull/3574#issuecomment-6290996273) | 3573 | open draft | 4 | planned, not executed | - |
| [3571](https://github.com/link-assistant/hive-mind/pull/3571#issuecomment-6290830898) | 3570 | MERGED 2026-10-20 | 4 | planned, not executed | Claude Code / Claude Opus 5.5 x1 |
| [3559](https://github.com/link-assistant/hive-mind/pull/3559#issuecomment-6272176340) | 3558 | open draft | 4 | planned, not executed | - |
| [3553](https://github.com/link-assistant/hive-mind/pull/3553#issuecomment-6270689671) | 3552 | MERGED 2026-10-20 | 3 | planned, not executed | Claude Code / Claude Opus 5.5 x3 |
| [3535](https://github.com/link-assistant/hive-mind/pull/3535#issuecomment-6261886329) | 3533 | open draft | 4 | planned, not executed | - |
| [3534](https://github.com/link-assistant/hive-mind/pull/3534#issuecomment-6261839689) | 3532 | open draft | 4 | planned, not executed | - |
| [3531](https://github.com/link-assistant/hive-mind/pull/3531#issuecomment-6261533695) | 3530 | open, ready | 4 | planned, not executed | Claude Code / Claude Opus 5.5 x4; Claude Code / model line absent x1 |
| [3507](https://github.com/link-assistant/hive-mind/pull/3507) | 3505 | MERGED 2026-10-18 | 0 | no session | Claude Code / Claude Opus 5.5 x1 |
| [3506](https://github.com/link-assistant/hive-mind/pull/3506#issuecomment-6245747002) | 3505 | open draft | 4 | planned, not executed | - |
| [3470](https://github.com/link-assistant/hive-mind/pull/3470#issuecomment-6232765795) | 3469 | MERGED 2026-10-17 | 4 | planned, not executed | Claude Code / Claude Opus 5.5 x2 |
| [3446](https://github.com/link-assistant/hive-mind/pull/3446#issuecomment-6231901257) | 3445 | MERGED 2026-10-17 | 3 | planned, not executed | Claude Code / Claude Opus 5.5 x4 |
| [3438](https://github.com/link-assistant/hive-mind/pull/3438#issuecomment-6229639858) | 3435 | open draft | 4 | planned, not executed | - |
| [3437](https://github.com/link-assistant/hive-mind/pull/3437#issuecomment-6229579611) | 3434 | open draft | 4 | work item not read | - |
| [3436](https://github.com/link-assistant/hive-mind/pull/3436#issuecomment-6229619363) | 3433 | open draft | 4 | left unchanged | - |
| [3414](https://github.com/link-assistant/hive-mind/pull/3414#issuecomment-6220000538) | 3413 | MERGED 2026-10-17 | 4 | planned, not executed | Claude Code / Claude Opus 5.5 x1 |
| [3396](https://github.com/link-assistant/hive-mind/pull/3396#issuecomment-6210495476) | 3395 | open draft | 4 | planned, not executed | - |
| [3367](https://github.com/link-assistant/hive-mind/pull/3367#issuecomment-6194378559) | 3359 | open draft | 4 | planned, not executed | - |
| [3366](https://github.com/link-assistant/hive-mind/pull/3366#issuecomment-6194315730) | 3354 | open draft | 4 | planned, not executed | - |
| [3365](https://github.com/link-assistant/hive-mind/pull/3365#issuecomment-6194303022) | 3357 | open draft | 4 | planned, not executed | - |
| [3364](https://github.com/link-assistant/hive-mind/pull/3364#issuecomment-6194286797) | 3358 | open draft | 4 | planned, not executed | - |
| [3363](https://github.com/link-assistant/hive-mind/pull/3363#issuecomment-6194219743) | 3356 | open draft | 4 | planned, not executed | - |
| [3362](https://github.com/link-assistant/hive-mind/pull/3362#issuecomment-6194136635) | 3355 | open draft | 4 | planned, not executed | - |
| [3339](https://github.com/link-assistant/hive-mind/pull/3339#issuecomment-6192519298) | 3333 | open draft | 4 | planned, not executed | - |
| [3338](https://github.com/link-assistant/hive-mind/pull/3338#issuecomment-6192506123) | 3334 | open draft | 4 | planned, not executed | - |
| [3331](https://github.com/link-assistant/hive-mind/pull/3331#issuecomment-6192169124) | 3330 | open draft | 4 | planned, not executed | - |
| [3327](https://github.com/link-assistant/hive-mind/pull/3327#issuecomment-6191356877) | 3326 | MERGED 2026-10-15 | 4 | planned, not executed | Claude Code / Claude Opus 5.5 x1 |
| [3295](https://github.com/link-assistant/hive-mind/pull/3295#issuecomment-6174960537) | 3282 | MERGED 2026-10-15 | 4 | planned, not executed | Claude Code / Claude Opus 5.5 x1 |
| [3290](https://github.com/link-assistant/hive-mind/pull/3290#issuecomment-6174920629) | 3278 | open draft | 4 | planned, not executed | - |
| [3285](https://github.com/link-assistant/hive-mind/pull/3285#issuecomment-6174875894) | 3277 | open draft | 4 | planned, not executed | - |
| [3281](https://github.com/link-assistant/hive-mind/pull/3281#issuecomment-6174837082) | 3279 | open draft | 4 | planned, not executed | - |
| [3280](https://github.com/link-assistant/hive-mind/pull/3280#issuecomment-6174787860) | 3276 | open draft | 4 | planned, not executed | - |
| [3249](https://github.com/link-assistant/hive-mind/pull/3249#issuecomment-6164646825) | 3248 | open draft | 10 | false completion claim, then 5 restarts | - |
| [3225](https://github.com/link-assistant/hive-mind/pull/3225#issuecomment-6161640628) | 3215 | MERGED 2026-10-14 | 2 | `e.g` not found | Claude Code / Claude Opus 5.5 x2 |
| [3224](https://github.com/link-assistant/hive-mind/pull/3224#issuecomment-6161623610) | 3207 | open draft | 2 | `e.g` not found | - |
| [3223](https://github.com/link-assistant/hive-mind/pull/3223#issuecomment-6161623353) | 3206 | open draft | 2 | `e.g` not found | - |
| [3222](https://github.com/link-assistant/hive-mind/pull/3222#issuecomment-6161596211) | 3210 | open draft | 2 | `e.g` not found | - |
| [3221](https://github.com/link-assistant/hive-mind/pull/3221#issuecomment-6161604581) | 3213 | open draft | 2 | `e.g` not found | - |
| [3220](https://github.com/link-assistant/hive-mind/pull/3220#issuecomment-6161594164) | 3208 | open draft | 2 | `e.g` not found | - |
| [3219](https://github.com/link-assistant/hive-mind/pull/3219#issuecomment-6161600288) | 3212 | open draft | 2 | `e.g` not found | - |
| [3218](https://github.com/link-assistant/hive-mind/pull/3218#issuecomment-6161615471) | 3214 | open draft | 2 | `e.g` not found | - |
| [3217](https://github.com/link-assistant/hive-mind/pull/3217#issuecomment-6161589909) | 3211 | open draft | 2 | `e.g` not found | - |
| [3216](https://github.com/link-assistant/hive-mind/pull/3216#issuecomment-6161578060) | 3209 | open draft | 2 | `e.g` not found | - |
| [3191](https://github.com/link-assistant/hive-mind/pull/3191#issuecomment-6148182099) | 3190 | MERGED 2026-10-13 | 1 | `e.g` not found | Claude Code / Claude Opus 5.5 x1 |
| [3179](https://github.com/link-assistant/hive-mind/pull/3179#issuecomment-6139498115) | 3176 | open draft | 2 | `e.g` not found | - |
| [3175](https://github.com/link-assistant/hive-mind/pull/3175#issuecomment-6139345162) | 3174 | open draft | 2 | `e.g` not found | - |
| [3127](https://github.com/link-assistant/hive-mind/pull/3127#issuecomment-6111351330) | 3125 | open, ready | 2 | `e.g` not found | Claude Code / Claude Opus 5.5 x1 |
| [3126](https://github.com/link-assistant/hive-mind/pull/3126#issuecomment-6111327901) | 3124 | open draft | 2 | `e.g` not found | - |
| [3109](https://github.com/link-assistant/hive-mind/pull/3109#issuecomment-6109913438) | 3108 | MERGED 2026-10-11 | 2 | `e.g` not found | Claude Code / Claude Opus 5.5 x1 |
| [3049](https://github.com/link-assistant/hive-mind/pull/3049#issuecomment-6101634988) | 3048 | open draft | 2 | `e.g` not found | - |
| [3045](https://github.com/link-assistant/hive-mind/pull/3045#issuecomment-6101464705) | 3043 | open, ready | 2 | `e.g` not found | Claude Code / Claude Opus 5.5 x2 |
| [3018](https://github.com/link-assistant/hive-mind/pull/3018#issuecomment-6096987523) | 3015 | MERGED 2026-10-11 | 2 | `e.g` not found | Claude Code / Claude Opus 5.5 x2 |
| [3003](https://github.com/link-assistant/hive-mind/pull/3003#issuecomment-6096240002) | 3001 | open draft | 2 | `e.g` not found | Claude Code / Claude Opus 5.5 x1 |
| [3002](https://github.com/link-assistant/hive-mind/pull/3002#issuecomment-6096225271) | 3000 | open, ready | 2 | `e.g` not found | Claude Code / Claude Opus 5.5 x1 |
| [2999](https://github.com/link-assistant/hive-mind/pull/2999#issuecomment-6096109696) | 2998 | open, ready | 2 | `e.g` not found | Claude Code / Claude Opus 5.5 x1 |
| [2924](https://github.com/link-assistant/hive-mind/pull/2924) | 2923 | MERGED 2026-10-09 | 0 | no session | Claude Code / Claude Opus 5.5 x1 |
| [2918](https://github.com/link-assistant/hive-mind/pull/2918#issuecomment-6085106910) | 2917 | open, ready | 2 | `e.g` not found | Claude Code / Claude Opus 5 x1; Claude Code / Claude Opus 5.5 x1 |
| [2902](https://github.com/link-assistant/hive-mind/pull/2902#issuecomment-6084136988) | 2900 | open, ready | 2 | `e.g` not found | Claude Code / Claude Opus 5.5 x1 |
| [2901](https://github.com/link-assistant/hive-mind/pull/2901#issuecomment-6083912246) | 2892 | open, ready | 2 | `e.g` not found | Claude Code / Claude Opus 5.5 x1 |
| [2899](https://github.com/link-assistant/hive-mind/pull/2899#issuecomment-6083815070) | 2891 | open, ready | 2 | `e.g` not found | Claude Code / Claude Opus 5.5 x1 |
| [2896](https://github.com/link-assistant/hive-mind/pull/2896#issuecomment-6083738226) | 2890 | open, ready | 2 | `e.g` not found | Claude Code / Claude Opus 5.5 x1 |
| [2895](https://github.com/link-assistant/hive-mind/pull/2895#issuecomment-6083676720) | 2887 | open, ready | 2 | `e.g` not found | Claude Code / Claude Opus 5.5 x1 |
| [2894](https://github.com/link-assistant/hive-mind/pull/2894#issuecomment-6083669335) | 2889 | open, ready | 2 | `e.g` not found | Claude Code / Claude Opus 5 x1; Claude Code / Claude Opus 5.5 x1 |
| [2893](https://github.com/link-assistant/hive-mind/pull/2893#issuecomment-6083619745) | 2888 | open, ready | 2 | `e.g` not found | Claude Code / Claude Opus 5.5 x1 |
| [2886](https://github.com/link-assistant/hive-mind/pull/2886#issuecomment-6083132835) | 2885 | open, ready | 2 | `e.g` not found | Claude Code / Claude Opus 5.5 x1 |
| [2856](https://github.com/link-assistant/hive-mind/pull/2856#issuecomment-6078475155) | 2844 | open, ready | 2 | `e.g` not found | Claude Code / Claude Opus 5.5 x1 |
| [2855](https://github.com/link-assistant/hive-mind/pull/2855#issuecomment-6078472472) | 2838 | open, ready | 2 | `e.g` not found | Claude Code / Claude Opus 5.5 x1 |
| [2854](https://github.com/link-assistant/hive-mind/pull/2854#issuecomment-6078460367) | 2840 | MERGED 2026-10-10 | 2 | `e.g` not found | Claude Code / Claude Opus 5.5 x1 |
| [2853](https://github.com/link-assistant/hive-mind/pull/2853#issuecomment-6078458943) | 2845 | open, ready | 2 | `e.g` not found | Claude Code / Claude Opus 5 x1; Claude Code / Claude Opus 5.5 x1 |
| [2852](https://github.com/link-assistant/hive-mind/pull/2852#issuecomment-6078452090) | 2846 | open, ready | 2 | `e.g` not found | Claude Code / Claude Opus 5.5 x1 |
| [2851](https://github.com/link-assistant/hive-mind/pull/2851#issuecomment-6078452904) | 2839 | MERGED 2026-10-11 | 2 | `e.g` not found | Claude Code / Claude Opus 5.5 x3 |
| [2850](https://github.com/link-assistant/hive-mind/pull/2850#issuecomment-6078451677) | 2841 | open, ready | 2 | `e.g` not found | Claude Code / Claude Opus 5.5 x1 |
| [2849](https://github.com/link-assistant/hive-mind/pull/2849#issuecomment-6078447325) | 2837 | open, ready | 2 | `e.g` not found | Claude Code / Claude Opus 5.5 x1 |
| [2848](https://github.com/link-assistant/hive-mind/pull/2848#issuecomment-6078443638) | 2843 | open, ready | 2 | `e.g` not found | Claude Code / Claude Opus 5.5 x1 |
| [2847](https://github.com/link-assistant/hive-mind/pull/2847#issuecomment-6078441580) | 2842 | open, ready | 2 | `e.g` not found | Claude Code / Claude Opus 5.5 x1 |
| [2824](https://github.com/link-assistant/hive-mind/pull/2824#issuecomment-6075835952) | 2823 | MERGED 2026-10-12 | 2 | `e.g` not found | Claude Code / Claude Opus 5 x1; Claude Code / Claude Opus 5.5 x2 |
| [2810](https://github.com/link-assistant/hive-mind/pull/2810#issuecomment-6063889635) | 2809 | open, ready | 2 | `e.g` not found | Claude Code / Claude Opus 5.5 x1 |
| [2804](https://github.com/link-assistant/hive-mind/pull/2804#issuecomment-6062834009) | 2803 | open, ready | 2 | `e.g` not found | Claude Code / Claude Opus 5.5 x1 |
| [2802](https://github.com/link-assistant/hive-mind/pull/2802#issuecomment-6062507764) | 2801 | MERGED 2026-10-14 | 2 | `e.g` not found | Claude Code / Claude Opus 5.5 x7 |
| [2772](https://github.com/link-assistant/hive-mind/pull/2772#issuecomment-6057433158) | 2771 | MERGED 2026-10-09 | 2 | `e.g` not found | Claude Code / Claude Opus 5.5 x1 |
| [2754](https://github.com/link-assistant/hive-mind/pull/2754#issuecomment-6054572638) | 2751 | MERGED 2026-10-09 | 2 | `e.g` not found | Codex / GPT-6.1 Sol x2 |
| [2750](https://github.com/link-assistant/hive-mind/pull/2750#issuecomment-6054070437) | 2747 | open, ready | 2 | `e.g` not found | Codex / GPT-6.1 Sol x1 |
| [2746](https://github.com/link-assistant/hive-mind/pull/2746#issuecomment-6053545722) | 2745 | MERGED 2026-10-08 | 2 | `e.g` not found | Codex / GPT-6.1 Sol x1 |
| [2700](https://github.com/link-assistant/hive-mind/pull/2700#issuecomment-6048327273) | 2699 | open, ready | 2 | `e.g` not found | Claude Code / Claude Opus 5.5 x1 |
| [2690](https://github.com/link-assistant/hive-mind/pull/2690#issuecomment-6047658804) | 2687 | MERGED 2026-10-08 | 2 | `e.g` not found | Codex / GPT-6.1 Sol x1 |
| [2686](https://github.com/link-assistant/hive-mind/pull/2686#issuecomment-6047485455) | 2685 | MERGED 2026-10-08 | 2 | `e.g` not found | Codex / GPT-6.1 Sol x2 |
| [2648](https://github.com/link-assistant/hive-mind/pull/2648#issuecomment-6043290566) | 2647 | open, ready | 1 | `e.g` not found | Claude Code / Claude Opus 5.5 x1; Codex / GPT-6.1 Sol x1 |
| [2636](https://github.com/link-assistant/hive-mind/pull/2636#issuecomment-6041933621) | 2631 | open, ready | 1 | `e.g` not found | Codex / GPT-6.1 Sol x1 |
| [2635](https://github.com/link-assistant/hive-mind/pull/2635#issuecomment-6041902102) | 2629 | open, ready | 1 | `e.g` not found | Codex / GPT-6.1 Sol x1 |
| [2632](https://github.com/link-assistant/hive-mind/pull/2632#issuecomment-6041895798) | 2630 | open, ready | 1 | `e.g` not found | Codex / GPT-6.1 Sol x1 |
| [2616](https://github.com/link-assistant/hive-mind/pull/2616#issuecomment-6036098584) | 2615 | MERGED 2026-10-07 | 1 | `e.g` not found | Claude Code / Claude Opus 5.5 x1 |
| [2614](https://github.com/link-assistant/hive-mind/pull/2614#issuecomment-6035655582) | 2613 | MERGED 2026-10-07 | 1 | `e.g` not found | Codex / GPT-6.1 Sol x1 |
| [2592](https://github.com/link-assistant/hive-mind/pull/2592#issuecomment-6032993852) | 2591 | MERGED 2026-10-11 | 1 | `e.g` not found | Claude Code / Claude Opus 5 x2; Claude Code / Claude Opus 5.5 x2 |
| [2574](https://github.com/link-assistant/hive-mind/pull/2574#issuecomment-6025878549) | 2573 | MERGED 2026-10-07 | 1 | `e.g` not found | Claude Code / Claude Opus 5.5 x1 |
| [2572](https://github.com/link-assistant/hive-mind/pull/2572#issuecomment-6025553607) | 2571 | MERGED 2026-10-07 | 1 | `e.g` not found | Claude Code / Claude Opus 5.5 x1 |
| [2566](https://github.com/link-assistant/hive-mind/pull/2566#issuecomment-6019889421) | 2563 | MERGED 2026-10-07 | 1 | `e.g` not found | Claude Code / Claude Opus 5.5 x1 |
| [2550](https://github.com/link-assistant/hive-mind/pull/2550#issuecomment-6013399726) | 2549 | MERGED 2026-10-07 | 1 | `e.g` not found | Claude Code / Claude Opus 5.5 x1; Codex / GPT-6.1 Sol x1 |
| [2527](https://github.com/link-assistant/hive-mind/pull/2527#issuecomment-5998443121) | 2526 | MERGED 2026-10-06 | 1 | `e.g` not found | Codex / GPT-6.1 Sol x1 |
| [2499](https://github.com/link-assistant/hive-mind/pull/2499#issuecomment-5984799182) | 2498 | MERGED 2026-10-07 | 1 | `e.g` not found | Claude Code / Claude Opus 5.5 x2; Codex / GPT-6.1 Sol x2 |

## Not verified

- Which person or tool authored each commit that carries the identity of the
  repository owner, beyond what the session log comments name.
- Pull request 3786: 250 of 286 commits can be listed through the API, so the
  commits of its Formal AI session are not visible (shown as 0).
- Pull request 3337 (issue 3332): run 38735501777 was cancelled ("The job has
  exceeded the maximum execution time of 1h30m0s"); it has one `.gitkeep` commit
  and no comment, and is not in the table.
- Pull request 2924 (issue 2923): run 37993481615 failed in the step "Open the
  Formal AI draft" with "Process completed with exit code 125"; the cause was
  not established.
- Seven failed draft runs since 2026-10-04 have no located pull request
  (41547700367, 39648049138, 39211909258, 38515996193, 37993992264, 37367087072,
  37204604042). Conclusions of the `issues`-event runs since that date: 118
  failure, 30 success, 1 cancelled.
- Session logs under `dev/log/` of that repository were not read, except the
  logs of pull requests 4036 and 4039 (for the Formal AI version); the classes
  rest on comments, commit file lists and pull request state.
- Which rule selected a Hello World program for issues 4031, 4034 and 4035.
