# Orchestrator-side findings and what was reported in link-assistant/hive-mind

Collected on 2026-11-10 from the three Hello World runs (solve v2.39.10) and
from the `Hive Mind draft` run on issue #1548 (solve v2.39.14). Orchestrator
source was read at commit `b245deb62fbbca42625753587575416c50d81727` (2.39.14);
whether the cited lines are identical in 2.39.10 was not verified. Every report
is written in provider-neutral words, except C10, whose code path is enabled
only for Formal AI.

## Reported

| Id | Finding | Written to | Blocks a session |
| --- | --- | --- | --- |
| C1 | A pull request is marked ready for review seven times in one run, each time with uncommitted changes or a "No progress" verdict, and once more after "Auto-restart 5/5 - limit reached" | comment <https://github.com/link-assistant/hive-mind/issues/3434#issuecomment-6765415459> | no; misreports the state |
| C2 | The restart loop on uncommitted changes never applies its "No progress" stop: an old session-kill notice is counted as a new comment on every iteration; five identical sessions used 821,325 input and 144,710 output tokens | <https://github.com/link-assistant/hive-mind/issues/4031> | yes |
| C3 | After that budget is exhausted the second restart engine still starts: a second "limit reached" comment, the first recovery commit is overwritten by a forced push, the work stays only on `recovery/<branch>` | <https://github.com/link-assistant/hive-mind/issues/4032> | yes |
| C4 | The "### Changes" block the solver writes into a pull request description is never refreshed: "No files were changed" stays on a pull request with 5 changed files | <https://github.com/link-assistant/hive-mind/issues/4033> | no; misleads the reviewer |
| C5 | "80.0K / 8.2K (976%) output tokens": the cumulative output of a session is divided by the limit of one response | <https://github.com/link-assistant/hive-mind/issues/4034> | no |
| C6 | The prompt of a restart after a CI failure carries only check names ("- run", "- run"); the job URLs go to the log and to the comment | <https://github.com/link-assistant/hive-mind/issues/4035> | partly; not tested |
| C10 | The attribution report walks the whole branch history on every session (191 commits, 210 log lines, 27.9 s after a 15.8 s session) and attributes old commits to the current session; this path runs only for Formal AI | comment <https://github.com/link-assistant/hive-mind/issues/3334#issuecomment-6765415926> | no; cost and log noise |

Reports that existed before 2026-11-10 and cover the same runs:
link-assistant/hive-mind#3909 (a draft started by a new issue runs the backend
built into the image), #3910 (the draft workflow ends with exit 5 when the model
makes no change; candidate C11) and #3911 (drafts are not scheduled from the
dependency gate).

## Effect of the reports themselves

The draft workflow of link-assistant/hive-mind runs for every new issue, so
each of the five new issues started a Formal AI session between 10:57Z and
10:59Z (pull requests 4036 to 4040 of that repository, Formal AI 0.355.33).
Their results are in `../attempts/hive-mind-attempts.md`; two of them committed
a Hello World program to a defect report, owned by formal-ai #1562. The five
pull requests are open drafts and were left as they are.

## Not reported, with the reason

| Id | Finding | Reason |
| --- | --- | --- |
| C7 | "Finished without changes" while three development-log commits are pushed, each starting 11 workflow runs that end `action_required` | The approval requirement is a setting of this repository, and the log commits are requested by the draft workflow of this repository; not a defect of the orchestrator |
| C8 | A working-session summary of 14 KB with the prompt of another session is posted as is | The text is produced by the model; owned by formal-ai #1559 |
| C9 | "Found 10 existing PR(s) for issue #1548" lists pull requests of other issues, then skips each | Log noise only; no effect on the session |
| C11 | Draft workflow run ends red with exit 5 for a model that made no change | Already reported: link-assistant/hive-mind#3910 |

## Checked and refuted as orchestrator defects

| Claim | Result |
| --- | --- |
| CI failures are not fed back to the model | Refuted: "RESTART TRIGGERED: CI failures detected"; what is missing is the job URL (C6) |
| The draft waits long in a queue | Refuted for run 42764310360: jobs started 2 s after creation; 19 minutes are 8.5 min binary build, 2 min disk cleanup, 4.5 min image pull; the model session took 13 s. Owned by formal-ai #1486 |
| The summary in Spanish comes from the orchestrator | Refuted: the text is the model output; owned by formal-ai #1557 |
| The context size "40285.2M" comes from the orchestrator | Refuted: the client prints `contextLimit 40285227008` taken from the model metadata Formal AI serves; the value is free disk space divided by an average token size (`rust/src/context_capacity.rs:41`), which is the documented design, so no issue was opened |

## Not verified

- No published log covers 09:22:51Z to 09:25:40Z of the Rust run; the sequence
  in C3 is derived from the source, the two comment texts and the timestamps.
- C2: the comment filter was checked by matching the comment text against the
  34 markers in the source; the function of the solver was not executed.
- C6: whether the session would have acted differently with the job URLs.
- C10: which step fills the 27.9 s; no other step is logged in that interval.
