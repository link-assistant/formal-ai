# Plans for the remaining #1085 work (PR #1086)

Written before the code, so the work can be resumed from any point if a session
is lost. Each plan is a checklist; a box is ticked in the same commit that
lands the step, and a step that turns out wrong is struck through with the
reason, never deleted.

Maintainer instructions this batch answers (2026-09-09):

- Draft everything at once; wait for CI only in the final preparation step.
- The PR must close every open issue about coding or self-coding.
- CI must run only what a push actually changed; a check that was green on an
  identical input is not run twice.
- Find the root cause of the longest CI steps and fix it without dropping any
  requirement checked.
- Plan in markdown before code -- and make Formal AI do the same when it codes.

## Scope: issue -> plan -> how the PR closes it

| Issue | Title (short) | Plan | Closed by |
| --- | --- | --- | --- |
| #1072 | Ladder duplicates the repository per node (31 GB) | [01](01-ladder-speed.md) | worktree + sparse checkout + stable path |
| #1107 | A branch push costs 310 job-minutes | [02](02-ci-green-ledger.md) | items 1-4 landed; green ledger closes the rest |
| #1095 | Continuation cue routed to web search | [04](04-ladder-leaf-fixes.md) | cue resumes the task in en/ru/hi/zh |
| #1096 | Replacement verified against a generated file | [04](04-ladder-leaf-fixes.md) | edit claim wins over generation claim |
| #1099 | Two-artifact task ends after the first | [03](03-plan-first-coding.md) | plan items; `Final` only when all are done |
| #1105 | Report flow: five live root causes | [05](05-issue-1105-report-flow.md) | RC2, RC4, RC5, RC6, RC7 |
| #1104 | Originating session report | [05](05-issue-1105-report-flow.md) | closes with #1105 |
| #1106 | Server wedge + per-request rebuild | landed | threading + incremental projection |
| #1109 | Rebuild leaks its replacement database | [01](01-ladder-speed.md) §5 | sweep dead-pid `.tmp` on open |
| #1091 | Gemfile.lock as a lockfile | landed (#1103) | -- |

Sub-issues of #1085 that stay open, with the reason:

| Issue | Why not this PR |
| --- | --- |
| #1087 (E109) frontier queue | six user-prompt issues, each with a four-language paraphrase set; separate work by its own definition |
| #1088 (E110) evidence out of the repo | needs a second repository; its item 3 (#1072) lands here |
| #1089 (E111) collapse the gates | a 48 -> 5 test-file consolidation; item 4 (a wall-clock ceiling) is measured here in plan 02 |
| #1090 (E112) traceability column | 716 rows of manual confirmation |
| #959 (E107) handler ledger ratchet | already ratcheted by `kernel-ratchet.lino`; the seed migration is E108's D1 |
| #1101 (E116) docs question in ru/hi/zh | routing rule fix; small, added to plan 04 as a stretch item |

## Order of work

1. Plan 01 (ladder speed) -- it is the longest CI step and blocks every push.
2. Plan 02 (green ledger) -- every later push benefits.
3. Plan 04 (leaf fixes) -- raises the ladder record; the reason the ladder exists.
4. Plan 03 (plan-first coding) -- closes #1099 and answers the maintainer's ask.
5. Plan 05 (#1105) -- the report flow.
6. Final preparation: PR body with `Closes` lines, changelog, wait for CI once.

## Measurements that motivated plan 01

Last successful ladder run (34326451343, 2026-09-09 07:56 -> 09:46, 110 min,
32 leaves selected):

- Agent CLI time summed over the 32 leaves: **~25 min**. The other 85 minutes
  are the harness.
- `git archive HEAD` is **966 MB** (`dev/` 693 MB, `docs/` 249 MB); each leaf
  extracts it, `git init`s, `git add .`s and commits it.
- Each leaf's workspace is a fresh `mktemp -d`, so cargo's fingerprint changes
  and `cargo check --lib` + `cargo test --test unit` recompile the crate per
  leaf, shared target directory notwithstanding.
- No per-node timeout: leaves that produced no proof ran 236 s, 162 s, 125 s.
- The four runs before the current one were all cancelled by
  `cancel-in-progress` (20-68 min each) because pushes arrived faster than the
  ladder finished. Plan 02 makes an unchanged-input push skip it entirely.
