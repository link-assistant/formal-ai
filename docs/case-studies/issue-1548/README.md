# Issue #1548 — what we do most wrong, that blocks our vision, and the plan to fix that

Issue: <https://github.com/link-assistant/formal-ai/issues/1548>

Pull request: <https://github.com/link-assistant/formal-ai/pull/1549>

Parent issue of the critical blockers:
<https://github.com/link-assistant/formal-ai/issues/1561>

The issue sets its own scope: "We scope of this issue is documentation and
issues planing, so the result here should be documentation changes and issues
created/organized". This case study is the evidence behind both: what
was measured, what was written to GitHub, and what was changed in the
documents. The requirements are listed in [requirements.md](requirements.md);
the options, the selected option and the plan per requirement are in
[solution-plan.md](solution-plan.md).

## 1. Collected data

| Path | Content |
| --- | --- |
| `raw-data/github/issue-1548.json`, `issue-1548-comments.json` | The issue as edited on 2026-11-10T08:42:55Z and its one comment |
| `raw-data/github/issue-graph-before.json` | Every open issue of this repository with type, labels, parent, sub-issues and `blocked by` relations, before any write |
| `raw-data/github/issue-graph-after.json` | The same snapshot after the last write |
| `raw-data/github/writes/issue-types.tsv` | 84 writes of an issue type and the matching label |
| `raw-data/github/writes/created-issues.tsv` | The 13 issues created in this repository |
| `raw-data/github/writes/sub-issues.tsv`, `blocked-by.tsv` | Every sub-issue and `blocked by` relation added, with the time of the write |
| `raw-data/github/writes/comments.tsv` | The ten evidence comments written to existing issues of both repositories |
| `raw-data/audit/vision-vs-code.md` | Audit of the vision against the source at `70935ad8e` (v0.355.34): ten questions, each answered with file and line, the list of contradictions, the ten findings |
| `raw-data/test-repositories/*-comment-history.md` | Comment history of the three Hello World pull requests, per day |
| `raw-data/attempts/real-issue-attempts.md`, `prs.tsv` | Every pull request Formal AI opened on a real issue of this repository, with the verbatim reason of each outcome |
| `raw-data/attempts/hive-mind-attempts.md` | The same inventory for link-assistant/hive-mind |
| `raw-data/hive-mind/reports.md` | Orchestrator-side findings: reported, not reported with the reason, refuted, not verified |
| `raw-data/research/online-notes.md` | Notes of the online research with the address of each source |

The helpers that wrote to GitHub are in `experiments/issue-1548/`.

## 2. Timeline (2026-11-10, UTC)

| Time | Event |
| --- | --- |
| 05:47:21 | The issue is opened |
| 06:04:43 | The `Hive Mind draft` workflow opens pull request #1549 with Formal AI as the model |
| 06:05:45 | The session summary is "Planificado, no ejecutado" for an issue written in English; terminal state `planned_not_executed` (<https://github.com/link-assistant/formal-ai/pull/1549#issuecomment-6760365706>) |
| 06:06:27 | The workflow reports on the issue that no verified draft was produced |
| 08:24:06 to 08:42:55 | The issue description is edited five times; the Hello World test repositories, the collection of failed attempts in both repositories and the reports to Hive Mind are added |
| 09:23:20 to 09:25:40 | The three Hello World sessions end (section 4) |
| 09:30:48 | A Hive Mind session of another model starts on pull request #1549 |
| 09:53 to 10:20 | 84 issue types set, 12 issues created, sub-issues and `blocked by` relations added |
| 10:20 to 10:57 | Reports written to link-assistant/hive-mind; documents changed; this case study written |
| 10:57 to 10:59 | The draft workflow of link-assistant/hive-mind starts a Formal AI session on each of the five reports (section 5.2) |
| 11:07 to 11:11 | Issue #1562 created for the defect those sessions showed and attached to #1561; evidence comments written to #1158, #1189, #1483 and #1558 |

Formal AI was asked first, as CONTRIBUTING requires. Its failed ask is retained:
it is pull request #1549 itself, counted in section 5, and its two causes are
issues [#1483](https://github.com/link-assistant/formal-ai/issues/1483) and
[#1557](https://github.com/link-assistant/formal-ai/issues/1557). Nothing in
this pull request after the three development-log commits of that session was
authored by Formal AI.

## 3. What we do most wrong

The audit (`raw-data/audit/vision-vs-code.md`) read the source at `70935ad8e`
and the committed ledgers. No Rust was compiled for it; every statement cites a
file. The ten findings, ranked by how much each blocks "a recursive meta
algorithm that discovers, composes and verifies solutions instead of storing
them":

| # | Finding | Evidence | Carried by |
| --- | --- | --- | --- |
| 1 | The universal problem-solving loop of the vision picture is recorded as a trace; it is not the control flow that produces answers | 11 early returns in `rust/src/solver.rs:327-809`; decomposition is a split of the prompt text on four separators (`rust/src/solver_helpers/mod.rs:135`); validation is `contains("prime")` (`:372`); candidates are three literal names (`:351`); simplification is the constant `"smallest_sufficient"` (`rust/src/solver.rs:815`); `record_meta_core_execution` is dead code (`rust/src/meta_core.rs:134-150`); `rust/src/recipe_interpreter.rs` "changes no routing and no answer" | [#1550](https://github.com/link-assistant/formal-ai/issues/1550) |
| 2 | Programs are stored, not derived | 138 compiled templates and 151 seed rows (13 tasks, 14 languages); an "algorithm" request receives a stored sort (`data/seed/handler-rules.lino:375-428`); every coding handler is `pending` or `justified-native` in `data/meta/handler-migration-ledger.lino` | [#1554](https://github.com/link-assistant/formal-ai/issues/1554), [#1165](https://github.com/link-assistant/formal-ai/issues/1165) |
| 3 | Discovery is called from tests only, the caches ship empty and the network is off by default | `cached_or_research` is called only from `rust/tests/unit/issue_1165_discovery_production.rs:202,239`; `data/cache/coding-procedure-cache.lino` has no procedure row; live fetching needs `FORMAL_AI_LIVE_FETCH` | [#1165](https://github.com/link-assistant/formal-ai/issues/1165), [#1163](https://github.com/link-assistant/formal-ai/issues/1163) |
| 4 | A plan does not become a change | `finish_general_change` ends in `planned_not_executed_answer()` unless the work reduces to a literal file, a quoted command or a catalogued program (`rust/src/agentic_coding/general_execution.rs:786-797`); coding ladder 23 of 130, level 1 0 of 16; self-authored share 0 against a target of 389 basis points | [#1483](https://github.com/link-assistant/formal-ai/issues/1483) |
| 5 | Recursion with attempt, split, bounded depth and recomposition is used only when work is delegated to an external CLI | `solve_recursively` has one caller, `rust/src/orchestration/incremental.rs:145` | [#1550](https://github.com/link-assistant/formal-ai/issues/1550) |
| 6 | Tests are not derived from requirements, not written in the requested language and not run | Six `ObligationKind`s, all for Hello World with CI (`rust/src/intent_formalization/obligations.rs:56-69`); `rust/src/solver_handlers/test_generation.rs:1-18` writes pytest for three shapes and states "The suite is NOT executed" | [#1551](https://github.com/link-assistant/formal-ai/issues/1551), [#1166](https://github.com/link-assistant/formal-ai/issues/1166) |
| 7 | Selection is first match; relative-meta-logic is not a dependency and is not applied to requirements or candidates | `rust/src/meta_method_dispatch.rs:38`; `rust/src/relative_meta_logic.rs` is an in-repository module of 434 lines used for statements, facts and source trust; `SelectionMode` and `RecursionMode` change the trace only | [#1552](https://github.com/link-assistant/formal-ai/issues/1552), [#1179](https://github.com/link-assistant/formal-ai/issues/1179), [#491](https://github.com/link-assistant/formal-ai/issues/491) |
| 8 | Formalization and deformalization lose meaning | `data/meta/round-trip-translation-ratchet.lino`: sentence survival 0.105, term survival 0.436 over 320 sentences; `data/meta/js-rust-translation.lino`: 316 items translated, 2,463 carried, 1 of 156 modules, 49 refused | [#1555](https://github.com/link-assistant/formal-ai/issues/1555), [#1487](https://github.com/link-assistant/formal-ai/issues/1487), [#1167](https://github.com/link-assistant/formal-ai/issues/1167), [#1500](https://github.com/link-assistant/formal-ai/issues/1500) |
| 9 | Source found on the internet has no licence decision | No GitHub source discovery; `rust/src/coding/fragment_catalog.rs:450-457` maps every licence it does not recognise to `ReuseMode::Verbatim` | [#1553](https://github.com/link-assistant/formal-ai/issues/1553), [#1164](https://github.com/link-assistant/formal-ai/issues/1164) |
| 10 | Status sentences say more than the ledgers measure | Audit Q10: 14 contradictions between a document and the source, 7 between documents | corrected in this pull request (section 7); [#1179](https://github.com/link-assistant/formal-ai/issues/1179) |

Root cause, in one sentence: the steps of the vision picture were delivered as
records of what a handler did, and each later requirement (discovery, tests,
options, recursion, configuration) was attached to that record instead of to the
control flow, so the ledgers that count handlers, stored programs and passing
tasks stayed precise while the answer path stayed a table of handlers.

What is in good shape, and is reused by the plan: the ledgers and their
shrink-only checks; the trusted-source registry with tiers, licences, hashes and
cache-on-fetch; the meta reasoner (grounding, shortest-first typed enumeration,
verification on examples, no natural-language literal in its modules);
`rust/src/recursive_execution.rs`, a general attempt, split and recompose engine
without an internal executor; and the planner, which reports
`planned_not_executed` instead of inventing a patch.

Dependencies named by the vision, as declared in `rust/Cargo.toml`:
`links-notation 0.23.0`, `link-calculator 0.22.0`, `meta-language 1.0.0`
(optional), `web-capture 0.3.37`, `web-search 0.6.0`. `relative-meta-logic` is
not among them; upstream has no release
(<https://github.com/link-foundation/relative-meta-logic>, open issues #185 and
#188 on publishing the crate).

## 4. The three Hello World test repositories

The issue asks to "check latest results (wait for finish, if not finished)".
All three sessions were followed to their last comment. Orchestrator: solve
v2.39.10; Formal AI 0.355.27 on the Hive Mind host. In all three repositories
issue #1 is open, and every pull request comment is posted by the account of
the repository owner.

| Repository | Client | Final state | What is wrong |
| --- | --- | --- | --- |
| Scala, <https://github.com/konard/test-hello-world-019fb330-00e1-7cfc-b6d9-a8dce7a52f51/pull/2> | `--tool agent` | Failed: "No progress between sessions" (<https://github.com/konard/test-hello-world-019fb330-00e1-7cfc-b6d9-a8dce7a52f51/pull/2#issuecomment-6762953487>); draft; head `31809eba0`; 6 files, +1495/-0 | 2 of 4 check runs fail: job `run` of `run.yml`, started once by `pull_request` and once by `push`; a second workflow `Main.yml` was added and passes, the failing one was left; a 2 MB `agent-stream.jsonl` is committed; the description still says no file was changed |
| Rust, <https://github.com/konard/test-hello-world-019fb331-c107-78c7-8ff6-9f127a3c593c/pull/2> | `--tool codex` | Failed twice: "Auto-restart 5/5 - limit reached" (<https://github.com/konard/test-hello-world-019fb331-c107-78c7-8ff6-9f127a3c593c/pull/2#issuecomment-6763712572>, <https://github.com/konard/test-hello-world-019fb331-c107-78c7-8ff6-9f127a3c593c/pull/2#issuecomment-6763757247>); marked ready at 09:25:40; head `8972c2085`; 5 files, +16/-0 | 0 check runs: no workflow file on the branch; the workflow and the test script exist only on `recovery/issue-1-09b0c76bd0e4`; a compiled binary of 4.5 MB is committed; the pull request was marked ready 7 times and returned to draft 7 times in 14 minutes; six summaries carry the prompt of the Scala repository |
| Kotlin, <https://github.com/konard/test-hello-world-019fb330-fa49-7d51-b5ba-33d2d9d85a79/pull/2> | `--tool claude` | "Ready to merge" (<https://github.com/konard/test-hello-world-019fb330-fa49-7d51-b5ba-33d2d9d85a79/pull/2#issuecomment-6763753979>); 4 of 4 checks pass; 7 files, +149/-0 | Two workflows that do the same; the session log is committed; title and description were not written by the session |

The observation of the issue, "no Pull Request description was prepared, was not
edited, and also CI/CD was not fixed", holds at the end of all three sessions for
the description, and for CI in two of three.

| Failure | On the side of Formal AI | On the side of the orchestrator |
| --- | --- | --- |
| The restart repeats the same steps and does not read the failing job | [#1185](https://github.com/link-assistant/formal-ai/issues/1185) | job addresses missing in the restart prompt: link-assistant/hive-mind#4035 |
| Build output, logs and duplicate workflows are committed; no test derived from the requirement | [#1551](https://github.com/link-assistant/formal-ai/issues/1551), [#1162](https://github.com/link-assistant/formal-ai/issues/1162) | none |
| The prompt of another session appears in the summary | [#1559](https://github.com/link-assistant/formal-ai/issues/1559) | none |
| Title and description are not written | [#1162](https://github.com/link-assistant/formal-ai/issues/1162) | the "Changes" block is never refreshed: link-assistant/hive-mind#4033 |
| Five restarts with no change in state; ready with no checks | [#1185](https://github.com/link-assistant/formal-ai/issues/1185) | link-assistant/hive-mind#4031, #4032 and the comment on #3434 |

All three runs belong to
[#1162](https://github.com/link-assistant/formal-ai/issues/1162).

## 5. Failed attempts on real GitHub issues

The issue asks to "collect all failed attempts to solve real GitHub issues by
Formal AI, and make sure it can advance them further", in this repository and in
link-assistant/hive-mind. "Advance" is delivered here as the issue that removes
the cause of each class of failure; the source change is the work of those
issues.

### 5.1 link-assistant/formal-ai

`raw-data/attempts/real-issue-attempts.md` lists every pull request with the
verbatim reason. The `Hive Mind draft` workflow opened 19 pull requests on real
issues; none contains a useful change made by the recorded Formal AI session.

| Outcome | Pull requests | Cause | Advanced by |
| --- | --- | --- | --- |
| "Planned, not executed" (10) | #1236, #1331, #1369, #1397, #1398, #1439, #1448, #1456, #1495, #1549 | A work item that is not a literal file, a quoted command or a catalogued program has no executor | [#1483](https://github.com/link-assistant/formal-ai/issues/1483), then [#1550](https://github.com/link-assistant/formal-ai/issues/1550) and [#1552](https://github.com/link-assistant/formal-ai/issues/1552) |
| of these, summary in Spanish or Russian for an English issue (5) | #1236, #1448, #1549; #1331, #1398 | Language markers match word prefixes | [#1557](https://github.com/link-assistant/formal-ai/issues/1557) |
| Stopped after three reads of a missing file (5) | #1493, #1494, #1496, #1497, #1508 | A file name in the issue text is read at the workspace root | [#1558](https://github.com/link-assistant/formal-ai/issues/1558) |
| Raw JSON `ReadPolicyRefused` posted as the summary (3) | #1498, #1499, #1502 | The refusal is not rendered; the page chrome of the issue page is accepted as the issue text | [#1560](https://github.com/link-assistant/formal-ai/issues/1560) |
| "Completed" with only log files committed (1) | #1329 | No obligation is checked before completion is reported | [#1166](https://github.com/link-assistant/formal-ai/issues/1166), [#1551](https://github.com/link-assistant/formal-ai/issues/1551) |

8 of the 19 sessions ended before any reasoning about the issue (5 on a missing
file, 3 on raw JSON), and 5 more wrote the summary in the wrong language. That is
why the plan starts with stage 0.

The three merged pull requests among the 19 (#1329, #1495, #1502) were merged
with commits that no Formal AI session comment accompanies; who produced those
commits is not verified.

The `Self-authored pull request` workflow of September 2026 opened 13 pull
requests: 3 merged (#1103, #1122, #1124), 3 without a change, 4 rejected, 3
failed outside the model. It does not call Hive Mind and is counted separately.

### 5.2 link-assistant/hive-mind

`raw-data/attempts/hive-mind-attempts.md` has one row per pull request, the
verbatim final messages and the method. The draft workflow of that repository
opened 133 pull requests for 132 real issues between 2026-10-04 and 2026-11-10
05:55Z (the 272 Hello World fixture pull requests are not counted). 130 of them
have a Formal AI session comment; they hold 135 sessions. No session left a
file other than `.gitkeep` and logs in a pull request. 42 of the 133 are merged;
in all 42 every file other than logs comes from commits with the identity of the
repository owner, and the session logs on them name other tools.

| Final message of the session | Sessions | Cause | Advanced by |
| --- | --- | --- | --- |
| "AGENT execution failed ... File not found: .../e.g" | 65 (2026-10-04 to 2026-10-14) | The abbreviation "e.g." in the solver prompt is read as a file path; the client then ends the run on the read error | [#1189](https://github.com/link-assistant/formal-ai/issues/1189) for the read, [#1205](https://github.com/link-assistant/formal-ai/issues/1205) for recovery after a tool error; not seen after 2026-10-14, not measured again |
| "Planned, not executed: `<issue address>`." | 56 (2026-10-14 to 2026-11-10) | A work item that is not a literal file, a quoted command or a catalogued program has no executor | [#1483](https://github.com/link-assistant/formal-ai/issues/1483), then [#1550](https://github.com/link-assistant/formal-ai/issues/1550) and [#1552](https://github.com/link-assistant/formal-ai/issues/1552) |
| "Pull request feedback needs source review ...: Exit code 1 specify only one of --comments or --json" | 5 (pull request 3249, restarts 1 to 5) | The session builds a `gh` command with two flags that exclude each other and stops on its error | [#1158](https://github.com/link-assistant/formal-ai/issues/1158) |
| "Did not write `<file>`: the request names it only inside quoted text ..." and "Left `.gitmodules` unchanged ..." | 4 | A file name quoted in the issue is taken as the whole request; the refusal to write is correct, the reading of the issue is not | [#1483](https://github.com/link-assistant/formal-ai/issues/1483), [#1166](https://github.com/link-assistant/formal-ai/issues/1166) |
| "Repeated action stopped after 3 attempts (limit 3): read ... File not found" | 2 | A file name in the issue text is read at the workspace root | [#1558](https://github.com/link-assistant/formal-ai/issues/1558) |
| "The work item at `<issue address>` could not be read ..." | 1 | No second way to read the issue is tried | [#1155](https://github.com/link-assistant/formal-ai/issues/1155) |
| Raw JSON `ReadPolicyRefused` as the summary | 1 | The refusal is not rendered | [#1560](https://github.com/link-assistant/formal-ai/issues/1560) |
| "Added "file", "HTML", "text", ... to the list in `e.g` and observed the result." | 1 (pull request 3249, first session) | Files named after words of the prompt are created and reported as done | [#1211](https://github.com/link-assistant/formal-ai/issues/1211), [#1166](https://github.com/link-assistant/formal-ai/issues/1166) |

Five more sessions ran on 2026-11-10 between 10:57Z and 10:59Z, because the five
reports of section 6 are new issues of that repository and its draft workflow
runs for every new issue. They ran Formal AI 0.355.33 and are the newest
evidence:

| Pull request | Issue | Result | Advanced by |
| --- | --- | --- | --- |
| 4036 | 4034 | `main.js` with `console.log("Hello, world!");` committed and pushed; "Created and verified" | [#1562](https://github.com/link-assistant/formal-ai/issues/1562) |
| 4037 | 4031 | `main.rs` committed ("feat: add main.rs"); then a `gh pr ready` command chain failed with exit code 1 | [#1562](https://github.com/link-assistant/formal-ai/issues/1562) |
| 4039 | 4035 | "`scala run Main.scala` exited with code 127", "scala: command not found" | [#1562](https://github.com/link-assistant/formal-ai/issues/1562) |
| 4038 | 4032 | "Did not write `src/solve.mjs`: the request names it only inside quoted text ..." | [#1483](https://github.com/link-assistant/formal-ai/issues/1483) |
| 4040 | 4033 | "Repeated action stopped after 3 attempts": the same file was read three times, and each read succeeded | [#1558](https://github.com/link-assistant/formal-ai/issues/1558) |

None of the five issues asks for a program. Three sessions produced or tried a
stored Hello World program; the issue texts contain "hello" only inside the
addresses of the test repositories, plus "Rust log" and the workflow name "Run
Main.scala". Before these sessions no Formal AI session had put a file other
than logs into a pull request of a real issue in either repository; the first
two that did put an unrelated program there. That class had no owner, so
[#1562](https://github.com/link-assistant/formal-ai/issues/1562) was created under [#1561](https://github.com/link-assistant/formal-ai/issues/1561). The five pull requests are open drafts
and were left as they are.

What belongs to the orchestrator and not to Formal AI in this inventory: all 65
comments of the first row also say "Log Upload Failed"; 118 of 149 draft runs
since 2026-10-04 ended as failed, which [link-assistant/hive-mind#3910](https://github.com/link-assistant/hive-mind/issues/3910) already reports (a run fails
when the model makes no change); 29 first sessions show output tokens above
100%, reported as [link-assistant/hive-mind#4034](https://github.com/link-assistant/hive-mind/issues/4034).

Not verified: who or what authored each commit that carries the identity of the
repository owner; the commits of the Formal AI session of pull request 3786
(the API lists 250 of 286); the cause of the failed run of pull request 2924
(exit code 125) and of the cancelled run of pull request 3337 (1h30m limit);
seven failed runs without a located pull request; which rule selected a Hello
World program for issues 4031, 4034 and 4035.

## 6. Orchestrator-side defects reported to Hive Mind

The issue: "Any issues on Hive Mind, that block Formal AI on orchestrator side,
must be reported, but Formal AI should not be treated in any special way."
`raw-data/hive-mind/reports.md` has the full list with the reason for every
finding that was not reported.

| Finding | Written to |
| --- | --- |
| The restart loop on uncommitted changes never applies its "No progress" stop; five identical sessions used 821,325 input and 144,710 output tokens | <https://github.com/link-assistant/hive-mind/issues/4031> |
| After that budget is exhausted a second restart engine starts; the first recovery commit is overwritten by a forced push | <https://github.com/link-assistant/hive-mind/issues/4032> |
| The "Changes" block of a pull request description is never refreshed | <https://github.com/link-assistant/hive-mind/issues/4033> |
| "80.0K / 8.2K (976%) output tokens": the output of a whole session is divided by the limit of one response | <https://github.com/link-assistant/hive-mind/issues/4034> |
| The prompt of a restart after a CI failure carries check names only, without the job addresses | <https://github.com/link-assistant/hive-mind/issues/4035> |
| A pull request is marked ready seven times in one run, and once more after "limit reached" | comment <https://github.com/link-assistant/hive-mind/issues/3434#issuecomment-6765415459> |
| The attribution report walks the whole branch history on every session | comment <https://github.com/link-assistant/hive-mind/issues/3334#issuecomment-6765415926> |

Reports that existed before and cover the same runs: link-assistant/hive-mind#3909
(a draft started by a new issue runs the backend built into the image), #3910
(the draft workflow ends with exit 5 when the model makes no change) and #3911
(drafts are not scheduled from the dependency gate). They are native blockers of
formal-ai #1484, #1488 and #1490.

Each report is written for any provider of an OpenAI-compatible API. The one
exception is the attribution report: its code path is enabled only for Formal
AI, and the comment asks to bound it, not to extend it.

Four claims were checked and refuted as orchestrator defects: CI failures are
fed back to the model; the draft did not wait in a queue (19 minutes are build,
disk cleanup and image pull; the model session took 13 seconds, owned by
[#1486](https://github.com/link-assistant/formal-ai/issues/1486)); the Spanish
summary is model output
([#1557](https://github.com/link-assistant/formal-ai/issues/1557)); the context
size "40285.2M" is the value Formal AI serves by design
(`rust/src/context_capacity.rs:41`).

## 7. What was written

### Issues

- 84 open issues received a GitHub issue type and the matching label. The final
  snapshot has 109 open issues: Feature 56, Bug 37, Task 16, none without a type.
- 13 issues were created in this repository: #1550 to #1556 (features the vision
  requires and no issue carried), #1557 to #1560 (defects that end a draft in its
  first minute), the parent #1561, and #1562 (the defect the five sessions of
  section 5.2 showed after the parent was written).
- [#1561](https://github.com/link-assistant/formal-ai/issues/1561) has 31
  sub-issues and 17 native blockers that already had another parent (GitHub
  allows one parent per issue); it is itself a sub-issue of
  [#651](https://github.com/link-assistant/formal-ai/issues/651). The repository
  has 56 `blocked by` relations, 52 with an open blocker, 3 of them to
  link-assistant/hive-mind; the graph has no cycle.
- 14 open issues without a parent are not critical blockers and are listed in
  #1561 as outside the plan.
- Evidence of the audit was added as comments to #1162, #1165, #1179 and #1483;
  evidence of section 5.2 as comments to #1158, #1189, #1483 and #1558
  (`raw-data/github/writes/comments.tsv`).
- 5 issues and 2 comments were written to link-assistant/hive-mind (section 6).

### Documents

| Document | Change |
| --- | --- |
| `docs/architect-notes/2026-11-10-what-we-do-most-wrong.md` | The issue, verbatim |
| `README.md` | The section on the loop says that the recorded steps are not yet the procedure that produces each answer |
| `ROADMAP.md` | Pillars 2, 6, 7, 24 and 26 from Built to Partial; section "2026-11-10 Vision-Blocker Audit (issue #1548)" with the measured state per standing requirement; earlier tables kept as the historical record |
| `VISION.md` | The paragraph on relative meta logic; new section "Requirements are axioms, options are scored, the plan is executed" |
| `GOALS.md` | Seven items, one per new feature issue and #1483 |
| `docs/meta-algorithm.md` | What the page is and is not |
| `docs/configuration/solver.md` | Every solver switch with what it changes today |
| `docs/guidelines/issue-planning.md`, `CONTRIBUTING.md` | How issues are typed, parented and blocked, and how a failed attempt becomes an issue |
| `docs/requirements/doctrine-standing-doctrine-recursive-meta-algorithm-2026-10-06.md` | Dated measurements added to R1001 and R1002; the stale ceiling in R1010 corrected |
| `docs/requirements/issue-1548-vision-blockers-and-plan.md` | Requirements R1548-1 to R1548-19 |

Of the 21 contradictions of audit Q10, these were left as they are, with the
reason:

- the header of `rust/src/discovery_production.rs` (source, outside the scope of
  this issue; evidence posted on #1165);
- the architect note of 2026-10-06 that says "90 handler rows" and the owner
  quote "no solution is hardcoded" (architect notes are kept verbatim; the
  measured numbers are in ROADMAP);
- `LEGAL-COMPLIANCE.md` (the policy is right; the source does not follow it for
  code fragments, owned by #1553);
- `data/meta/self-ast/README.md` (it already says the files are legacy inputs
  until the migration).

### Online research

Sources are in `raw-data/research/online-notes.md`; how each is used is in
[solution-plan.md](solution-plan.md). In short: hierarchical task networks and
Soar for recursion over subgoals; Parsel and CodeT for tests before selection of
a candidate; the 2025 study of failures in automated issue solving for the
classes of section 5; SPDX and REUSE for licence decisions; PLN and NARS for
truth values with strength and confidence; Grammatical Framework, Attempto
Controlled English and FRET for formalization and deformalization through one
abstract syntax.

## 8. Verification

| Check | Result |
| --- | --- |
| Every open issue has a type and the matching label | `raw-data/github/issue-graph-after.json`: 109 open, 0 without a type |
| Parent issue, sub-issues, blockers | Same snapshot: #1561 with 31 sub-issues and 17 blockers; no cycle in 56 relations |
| Documents, case study and requirement rows are present and linked | `rust/tests/web/issue-1548-vision-blockers.test.mjs` |
| Generated requirement documents and ledgers are current | `node scripts/assemble-requirements.mjs`, `node scripts/check-requirement-status.mjs`, `node scripts/render-status.mjs --check`, `node scripts/render-progressive-plan.mjs --check` |
| File sizes | `node scripts/check-file-size.mjs` |
| Rust and browser suites | CI of pull request #1549; nothing was compiled locally |

Not verified, and stated where it applies: who authored the merged commits of
pull requests #1329, #1495 and #1502; the six agent-mode issues of 2026-10-13
were not measured again at v0.355.34; the items listed at the end of
`raw-data/hive-mind/reports.md`.
