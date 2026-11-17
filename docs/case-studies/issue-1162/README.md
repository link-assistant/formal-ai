# Issue #1162: Hello World issue-to-PR delivery through all Hive clients (E118)

Source: https://github.com/link-assistant/formal-ai/issues/1162 (P0). The work is
in pull request https://github.com/link-assistant/formal-ai/pull/1579.

Formal AI is given a Hello World issue (Kotlin, Scala or Rust) by Hive Mind and
must deliver one pull request: one program file printing exactly
`Hello, World!`, one workflow that runs it, a verify script derived from the
requirement, a clean working tree, a pushed commit, a written title and
description, and a pull request marked ready. This happens through Claude Code
(`--tool claude`), the Agent CLI (`--tool agent`) and Codex (`--tool codex`),
with no LLM behind them.

## 1. Timeline

| Date | Event | Evidence |
| --- | --- | --- |
| 2026-09-15 | Scala run (backend 0.351.0) stops at `scalac: not found`; a restart gets the same result. | Quoted in #1159; PR https://github.com/konard/test-hello-world-019fb330-00e1-73b9-955e-f357a1600d5b/pull/2 |
| 2026-09-27 | Three runs (solve v2.32.0, backend 0.352.1). Kotlin/claude prints the line twice, leaves `Main.jar` untracked, fails "nothing to commit" on restart and leaves a draft. Scala/agent ends `planned_not_executed`. Rust/codex re-plans `gh issue view` 78 times. | `raw-data/hive-mind-logs/*.log.gz` (redacted copies of the gists linked in the issue) |
| 2026-09-29 to 2026-10-15 | Hive Mind and Agent CLI fixes ship; the Formal AI sub-issues #1154 to #1160 stay open. The 2026-10-13 runs fail on #1189, #1205, #1206, #1209 and others (#1207). | Issue body, "Status" sections |
| 2026-10-31 | Execution plan rewritten as nine acceptance items. Runs 40377859689, 40475832633 and 40668396305 give bounded same-repository success. | Issue body, "Current execution plan" |
| 2026-11-10 | Runs on solve v2.39.10 / Formal AI 0.355.27. Scala: two workflows, 2 of 4 checks fail, a session stream committed, no title or description, draft. Rust: no workflow, a 4.5 MB binary committed. Kotlin: two workflows doing the same thing, the session log committed, no title or description. | Issue comment https://github.com/link-assistant/formal-ai/issues/1162#issuecomment-6765454275; snapshots in `raw-data/test-repositories/*.json` |
| 2026-11-16 | This pull request: a per-obligation grader, the variant matrix, the root-cause fixes below, and an offline end-to-end run driven by Hive Mind's own prompts. | Section 3 |

## 2. Root causes found and fixed in this pull request

1. **The pull request was never bound to the delivery.** Hive Mind's formal-ai
   prompt says "Update the pull request at <url>." The work-item parser only
   bound "Your prepared Pull Request:" style wording. The delivery therefore
   never read feedback, never wrote a title or description and never ran
   `gh pr ready`, so the pull request stayed a `[WIP]` draft (#1158, #1160).
   Fixed in `056b2cac3` by binding that wording. Pinned by
   `rust/tests/web/issue-1162-offline-hive-end-to-end.test.mjs`, which uses the prompt
   verbatim from `rust/tests/fixtures/hive-mind/formal-ai-prompt.txt`.
2. **One literal became two lines.** Every print literal was joined into the
   expected stdout, so a requirement that mentioned `Hello, World!` twice
   produced a program and a verify script that print it twice. A paraphrase
   using "outputs" produced no recipe at all. Fixed in `e2cc497df`: one line per
   distinct literal unless a count or "again" asks for a repeat, and "outputs"
   is an output verb (#1156).
3. **Several workflows, one under the wrong name.** A delivery added a new
   workflow next to one an earlier session had committed. Fixed in `718c340e9`:
   there is one workflow per delivery, named as the request suggests, and the
   earlier session's duplicate is retired.
4. **Non-idempotent completion.** A rerun committed again, failed on "nothing
   to commit", or re-edited an unchanged pull request. Fixed in `cc1f10b7c` and
   `718c340e9`: the porcelain check is exact, an unchanged tree is reported as
   success, and `gh pr edit` and `gh pr ready` run only when something differs
   (#1157).
5. **Build output and session evidence were committed.** `Main.jar`, the Rust
   binary and the session logs reached the branch. Fixed: staging is restricted
   to declared artifacts with literal pathspecs, build outputs are added to
   `.gitignore`, and Formal AI's own bookkeeping notices are not treated as
   requested work (#1157, #1160).
6. **A missing compiler ended the session.** Prerequisite recovery existed but
   the answer neither named the procedure a grant would run nor reported a
   recovery. Now the grant-required answer names the pinned publisher
   procedure, and a recovered transaction is reported with the program, the
   procedure, the re-probe and the retried command (#1159).

## 3. Acceptance items

| # | Item | Status | Evidence |
| --- | --- | --- | --- |
| 1 | Separate the evidence kinds | Implemented | `scripts/lib/hello-world-delivery-grade.mjs` reports exact output as `unobserved` unless stdout was observed independently; `rust/tests/web/issue-1162-delivery-obligations.test.mjs` |
| 2 | A full work item that does not start from a completed program | Implemented offline | `rust/tests/web/issue-1162-offline-hive-end-to-end.test.mjs`: an empty repository, a bare origin and a fake `gh`, driven by Hive Mind's prompts through the JS planner to a ready pull request. `rust/tests/unit/agentic-coding/issue_1162_delivery_commands.rs` covers the Rust side. |
| 3 | Check every requirement independently | Implemented | 13 obligations in `data/benchmarks/hello-world-delivery-obligations.json`. The 2026-11-10 snapshots fail on exactly their listed defects. |
| 4 | Keep `ci/test-hello-world.yml` (#1450) separate | Implemented | grader `workflowPath`; recipe workflow binding; tests in the obligations and variants files |
| 5 | Variants | Implemented | `rust/tests/web/issue-1162-delivery-variants.test.mjs` and `experiments/issue-1162/variant-matrix.mjs`; missing compiler in `rust/tests/web/observed-prerequisite-recovery.test.mjs` and `rust/tests/unit/issue_1159_toolchain_bootstrap.rs`; resume, feedback and no-op continuation in `recipe-resume-protocols`, `hive-own-session-feedback` and the offline e2e rerun |
| 6 | The grader rejects print-twice | Implemented | `exactOutputObligation` and the `newlineRule` policy |
| 7 | Three clients with no LLM, telling infrastructure failures from wrong code | Partial, live part **blocked** | Offline: Hive Mind's verbatim prompts (formal-ai, regular, restart) drive full deliveries; the historical logs and snapshots are graded per obligation. A live Claude/Agent/Codex run against a released version needs a release containing this change and Hive Mind credentials, and neither is available to the solver. |
| 8 | Hosted and external transport | **Blocked** | Needs repository and workflow credentials for the external test repositories. Not attempted, and no credit is claimed. |
| 9 | Repeated-run and held-out matrix | Partial, live part **blocked** | The matrix is predeclared in `experiments/issue-1162/variant-matrix.mjs` and pinned by tests; the offline e2e repeats the Rust delivery and requires the rerun to be a no-op. Repeated live runs are blocked for the same reasons as item 7. |

## 4. Sub-issues

| Sub-issue | State | This pull request |
| --- | --- | --- |
| #1154 codex `cmd` argument | Fixed on `main` | — |
| #1155 unauthenticated `gh` text accepted as the issue body | Fixed on `main` | — |
| #1156 doubled output | Open | Root cause 2. `rust/tests/unit/issue_1156_explicit_stdout_coreference.rs` |
| #1157 artifacts, idempotent commit | Open | Root causes 4 and 5. `rust/tests/unit/issue_1157_idempotent_commit.rs` |
| #1158 PR feedback, description, ready | Open | Root cause 1. `rust/tests/unit/issue_1158_pull_request_steps.rs` |
| #1159 toolchain bootstrap | Open | Root cause 6. `rust/tests/unit/issue_1159_toolchain_bootstrap.rs` |
| #1160 regular prompt | Open | Root causes 1 and 5. `rust/tests/unit/issue_1160_regular_prompt.rs` and the fixtures in `rust/tests/fixtures/hive-mind/` |
| #1161, #1484, #1485 | Closed | — |

Each open sub-issue also needs a manual end-to-end confirmation on a released
version. That check is **blocked** for the same reason as acceptance item 7, so
this pull request does not close #1154 to #1160.

## 5. Existing libraries considered

- **Test doubles for `gh`:** a fake `gh` script (`rust/tests/fixtures/hive-mind/fake-gh.cjs`) was chosen over HTTP mocking libraries such as `nock`. The delivery shells out to `gh`, so replacing the executable tests the real command lines.
- **Git:** real `git` against a temporary bare origin, not an in-memory implementation such as `isomorphic-git`, so that porcelain, pathspec and push behaviour are the real ones.
- **Workflow execution:** `act` was already used by runs 40377859689 and 40475832633 for same-repository evidence. Native hosted execution is acceptance item 8 and is blocked.

## 6. Raw data

- `raw-data/hive-mind-logs/`: the three 2026-09-27 session logs, gzipped, with e-mail addresses replaced by `<redacted-email>`.
- `raw-data/test-repositories/`: the 2026-11-10 pull request snapshots (title, body, checks, files, commits) replayed by the grader.

## 7. How to reproduce locally

```sh
node --test --test-concurrency=1 rust/tests/web/issue-1162-delivery-obligations.test.mjs
node --test --test-concurrency=1 rust/tests/web/issue-1162-offline-hive-end-to-end.test.mjs   # needs rustc, jq, python3; the Kotlin case needs kotlinc
cargo test --test unit -- issue_115 issue_1162
```
