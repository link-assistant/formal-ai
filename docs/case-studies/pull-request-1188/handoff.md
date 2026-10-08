# Handoff: finishing pull request #1188

Pull request [#1188](https://github.com/link-assistant/formal-ai/pull/1188), "Bulk QA, reasoning and coding fixes; recursive meta reasoner; JS server parity", branch `qa-reasoning-coding-bulk-fixes` into `main`.

This document is for the next agent. It covers what the owner asked for, how the work is done here, the state of the branch at the handoff, and what is left. Everything it names is committed on the branch.

## Current arrangement (2026-10-09)

The owner resumed this work and requested up to four GPT-6.1-sol subagents, subject to the four-active-slot environment limit including the coordinator. Keep slots busy, one subagent continuously monitoring CI/CD, and one delivering requirements by asking Formal AI, repairing it after failures and retrying. The coordinator also works continuously. Follow [the recommended multi-agent workflow](multi-agent-workflow.md), including its bulk drafting, small commits, coordinated push barriers and honest capability evidence. It supersedes the older agent and immediate-push instructions below.

## 1. Where the requirements live

The owner's requirements are recorded in three places, from the most literal to the most structured:

1. **[user-messages.md](user-messages.md)**: every message the owner sent in the Claude Code sessions of this repository, verbatim and deduplicated (R1188-U15). Regenerate it with `node experiments/formal_ai_subagent/collect-user-messages.mjs --write`.
2. **[docs/requirements/issue-1188-user-requirements.md](../../requirements/issue-1188-user-requirements.md)**: rows R1188-U1 to R1188-U30, one per requirement. Each row states the requirement, its status, what delivers it, the test that pins it and what is missing. **This is the work list.**
3. **The standing doctrine**: [docs/requirements/assembled/standing-doctrine.md](../../requirements/assembled/standing-doctrine.md) and the `doctrine-*.md` shards, for example R994 (three roots, full parity through the meta-language) and R1012 (the recursive meta algorithm). The latest vision is recorded verbatim in [docs/architect-notes/2026-10-08-architecture-naming-ci-speed-and-text-understanding.md](../../architect-notes/2026-10-08-architecture-naming-ci-speed-and-text-understanding.md) and folded into `VISION.md` (R1188-U14).

### 1.1 The owner's standing requirements, summarized

All of these are still in force.

**Delivery and code**
- JavaScript first: JS is the source, and full parity with Rust is guaranteed by CI (R1188-U16, U29).
- Translation between JS, TS, Rust and the meta-language is fully automated, through link-foundation/meta-language (R1188-U30, R994, R1000). Where it cannot translate yet, a recorded temporary workaround does, linked to the upstream issue that retires it. "It also must be recorded in our documents."
- Bulk changes are made as rules (substitution passes, generators with `--check`), never by hand (U8).
- No hardcoding. Generalize, don't specialize: a failing prompt is fixed by the smallest, more universal rule that covers its class, never by a case for one prompt (U1).
- Files are at most 1500 lines, Rust at most 1000. Code is readable and multi-line (U23).
- Follow [link-foundation/code-architecture-principles](https://github.com/link-foundation/code-architecture-principles) (U2).
- Names are full English words with no abbreviations (U4). File and directory names say what they hold (U5). The links notation we own prefers `-` over `_` (U6), is readable and deduplicated (U7), and is lossless one to one with the source (U3).

**CI**
- No CI job or step runs longer than 15 to 30 minutes (U9).
- Long jobs and tests start first (U10); CI is parallel at job and test level (U11); a gate enforces this automatically (U12).
- Dependencies are kept at their latest versions (gate `check-dependencies-latest`).
- The pull request lands as a valid, working, testable release with every workflow green (U27).

**Working style**
- Don't ask the owner questions; decide and record the decision.
- Use Formal AI from JavaScript for every small, well-specified step, log each use, and fix Formal AI when it fails (U13, U29).
- Use the `experiments/` folders for scripts and probes.
- **Never run `cargo`, `rustc` or `rust-script` locally.** Rust is compiled only in CI. Standalone `rustfmt --edition 2024 --check <file>` is fine.
- Be careful with disk (U25): no Rust builds and no full repository copies; delete scratch copies as soon as they are done.
- Keep local tests minimal (U24): run only the tests closest to the change, and let CI run the full suites.
- Collect requirements and draft them in bulk; every undrafted requirement must be drafted.
- Use meaningful file names. Keep docs and vision in sync with the code.

**Method**
- Apply the progressive JPEG method (U28, [docs/progressive-delivery.md](../../progressive-delivery.md)): deliver the whole at low resolution first, then sharpen. Raise the lowest level of the plan before refining higher ones.
- Make the JavaScript requirements pass first (U29).

**Agents (when the owner allows them)**
- At most three subagents at once, and never stop one early (U22).
- One of them is the CI fixer, which commits and pushes fixes as soon as each CI run reports (U26).
- Don't wait idle: work yourself while agents and CI run. "You don't pause, you do everything until is done."
- At this handoff the owner asked that **no new subagents are started**.

**Permissions the owner gave**
- "I EXPLICTLY ALLOW EVERYTHING."
- "I EXPLICITLY ALLOW SECRET SCAN."
- "Try again what autoclassifier blocks": retry a blocked action; after two or three failures, leave it for the owner.

## 2. Methodology: how development is done here

### 2.1 The loop for one change

1. **Find the general cause.** Reproduce the failure, then fix the class by rule: seed vocabulary in all five languages (en, ru, hi, zh, es) or a general mechanism. Never fix only the prompt.
2. **JavaScript first.** Change `js/…` first, then write the Rust twin by hand in `rust/src/…`, carefully, so it compiles in CI. A twin names its counterpart (`Mirrors \`fn x\``); gate `check-twin-citations` checks those names.
3. **Regenerate.**
   - `node scripts/translate-es.mjs --write` writes the `ts/` twins.
   - `bash scripts/sync-seed.sh` mirrors seed data. A seed edit also needs its `rust/embedded/data/seed/` copy.
   - `node scripts/generate-worker-crate-modules.mjs --write` regenerates the worker's copies of `js/agentic/crate` modules.
   - `node scripts/translate-js-rust.mjs --write --fetch <module>` re-projects a crate module to Rust.
4. **Test locally, minimally.** Run `node --test rust/tests/web/<nearest>.test.mjs`, then the gates: `node experiments/formal_ai_subagent/local-gates.mjs [--only a,b]`, which runs about 125 gates in JavaScript and skips the Rust-only ones.
5. **Record.**
   - Update the requirement row.
   - Run the requirement pipeline:
     ```
     node scripts/assemble-requirements.mjs --write && node scripts/generate-requirement-status.mjs --write && node scripts/render-status.mjs --write && node scripts/check-requirement-status.mjs
     node scripts/render-progressive-plan.mjs --write
     node scripts/tally-formal-ai-dogfood.mjs --write
     node experiments/formal_ai_subagent/requirement-coverage.mjs --write
     ```
   - Add a changelog fragment under `changelog.d/`.

### 2.2 Formal AI on its own development (dogfooding)

- **Running it.** `node experiments/js_dogfood/drive.mjs --dir <sandbox> '<request>'` runs the JavaScript planner on a sandbox directory.
  - Copy the file into a sandbox, ask, check the diff, and copy it back only if the diff is exactly the expected lines. Other agents may have edited the same file in the meantime.
  - Requests that work well:
    - `In f replace «A» with «B»`
    - `In f insert the contents of rows.md after the line that starts with «| Tn |»`
- **Logging.** Every use is a row of [formal-ai-dogfood.md](formal-ai-dogfood.md). Rows are numbered by agent tag in reserved ranges: LEAD T780–T809, CHAT-ROUTES T786–T799, TRANSLATE T750–T779, SPEC-PARITY T810–T839, FIX-GAPS T840–T869. The next free ranges are T870–T899 for repository runners and T900–T919 for test areas.
- **Failures.** A failure becomes a gap `Gnn` in [experiments/formal_ai_subagent/gaps.md](../../../experiments/formal_ai_subagent/gaps.md). It is fixed by rule in both roots and pinned by a regression test, and its row's After cell then reads "**Fixed** later (Gnn …). Was: …".
- **Tally.** [formal-ai-tally.md](formal-ai-tally.md) counts the rows; gate `check-formal-ai-tally` fails on a stale tally or a failure with no resolution.

### 2.3 Measures and ratchets

Every requirement that can be measured has a script and a ratchet file that only moves in the improving direction. When a measure improves, lower (or raise) its ratchet in the same commit.

| Measure | Script | Ratchet |
| --- | --- | --- |
| Notation: `_` names, repeated fields | `scripts/measure-notation.mjs` | `data/meta/notation-ratchet.lino` |
| Abbreviations | `scripts/measure-abbreviations.mjs` | `data/meta/abbreviation-ratchet.lino` |
| Progressive plan levels | `scripts/render-progressive-plan.mjs` | `data/meta/progressive-plan-ratchet.lino` |
| Rust spec suite in the browser worker | `scripts/check-specification-in-javascript.mjs` | `data/meta/specification-javascript-gaps.lino` |
| js → rust translation | `scripts/translate-js-rust.mjs --check --fetch` | `data/meta/js-rust-translation.lino` |
| Requirement extraction | `scripts/measure-requirement-extraction.mjs` | `data/meta/text-capability-ratchet.lino` |
| Debt (literal predicates, …) | `scripts/check-debt-ratchet.mjs` | `data/meta/debt-ratchet.lino` |
| CI job and step durations | `scripts/check-ci-speed.mjs` | `data/meta/ci-speed.lino`, `data/meta/test-durations.lino` |

### 2.4 Renames and notation by rule

- **Files.** Add a `rename` to a tree of `data/meta/rename-map.lino`, then run `node experiments/formal_ai_subagent/rename-by-rule.mjs --tree <name> --git-mv`. It moves the companions (ts twin, embedded mirror, worker line-budget shard, self-AST census) and rewrites every reference. Gate `check_rename_map` keeps old names from coming back.
- **Notation.** `node experiments/formal_ai_subagent/apply-notation-rules.mjs --family <name> --write` applies one family of `data/meta/notation-rules.lino`. Every new seed name must use `-` (for example `response-page-formalization-<kind>-<lang>`); `check-notation` fails on a new `_` name.

### 2.5 Committing in a shared working tree

Several agents share one working tree, so commits are made with a private index and only the files you own:

```bash
export GIT_INDEX_FILE=$SCRATCH/index; rm -f $GIT_INDEX_FILE
git read-tree HEAD
git add <your paths>
# A file someone else is also editing: stage a patched blob instead.
#   blob=$(git hash-object -w patched-copy); git update-index --add --cacheinfo 100644,$blob,<path>
git commit -F message.txt
unset GIT_INDEX_FILE; git reset -q   # main index back to HEAD; the working tree is untouched
```

Rules:
- Never `git stash` and never `git reset --hard` in the shared tree.
- Use bash scripts for long file lists, because zsh does not split words.
- After pushing, cancel the superseded runs of older heads, because the Actions runners saturate:
  ```
  gh run list --branch qa-reasoning-coding-bulk-fixes --json databaseId,headSha,status
  gh run cancel <id>
  ```
  A run that stays `queued` after a cancel needs `gh api -X POST repos/link-assistant/formal-ai/actions/runs/<id>/force-cancel`.
- End commit messages with the `Co-Authored-By` line the harness gives.

### 2.6 Reading CI

- `gh run view <id> --json jobs` lists the jobs.
- `gh api repos/link-assistant/formal-ai/actions/jobs/<job>/logs` fetches a job log; failing gates print `##[error]gate \`name\` failed`.
- Gates are data: one file per gate in `data/meta/ci-gates/*.lino`, with stage (`web`, `rust`, …), lane and run command.
- The release test job runs the prebuilt executables in shards (full lane 5, specification lane 4), planned longest-first from `data/meta/test-durations.lino`.

## 3. State at the handoff

Measured on the final integration commit of 2026-10-09 (see `git log` for its hash). CI on that commit is described in §4.1.

| Measure | Value |
| --- | --- |
| Requirements assembled | 1369. Progressive plan: 1 recorded, 2 measured, 46 partial; 921 pinned only by a Rust test ([progressive-plan.md](../../progressive-plan.md)) |
| Local gates | 124 of 124 pass (`node experiments/formal_ai_subagent/local-gates.mjs`) |
| JS web suite | 2326 tests: 2321 pass, 4 skipped, 1 todo, 0 fail (after the self-AST census from CI run 37830767346 was committed) |
| Rust specification suite carried to the browser worker | 136 of 1169 tests carried; **122 pass** (was 102 at the start of round 20); 14 gaps listed in `data/meta/specification-javascript-gaps.lino` |
| js → rust translation | 284 items by meta-language plus 31 by recorded workarounds; 2214 carried; 94 of 135 modules translate ([data/meta/js-rust-translation.lino](../../../data/meta/js-rust-translation.lino)) |
| Formal AI dogfooding | 427 tasks, 243 passed, 165 failed, of which 160 fixed, 5 open and 5 not reproduced ([formal-ai-tally.md](formal-ai-tally.md)) |
| Requirement extraction | recall 0.419, precision 0.407 on 138 issues |
| Architecture principles with a gate | 41 of 51 ([docs/architecture/principles.md](../../architecture/principles.md)) |
| CI | 83 of 94 jobs meet the 30-minute limit; the release test job is capped at 30 minutes |

### 3.1 What round 20 delivered

The changelog fragment [changelog.d/20261009_090000_formal-ai-subagent-round-twenty.md](../../../changelog.d/20261009_090000_formal-ai-subagent-round-twenty.md) lists it. In short:
- the release test job runs in 30 minutes on prebuilt executables;
- requirements cite JavaScript tests first;
- the Rust specification suite runs in the browser worker (102 → 122 passing);
- js → rust runs through meta-language with recorded workarounds;
- chat routes for summarize, formalize-a-page and round-trip translation choice;
- definition-of-done cues for requirement extraction;
- browser evidence links, policy gates and routing precedence;
- six Formal AI edit gaps fixed (G90, G99, G102, G104, G106, G107) plus the G91 guard regression that broke the Issue 1028 ladder;
- the E2E local web fix: the service-worker precache delayed the worker past the tests' wait;
- clippy 1.99 fixes;
- `audit-seed-metadata` reading files with a comment header;
- the `check-architecture-contents` gate;
- the last issue-numbered test files renamed.


## 4. What is left

The order follows the progressive method: make CI green first (R1188-U27), then raise the lowest plan level, then refine.

### 4.1 CI: make the head green (R1188-U27)

1. **Self-AST census: done.** The hand-edited Rust files changed their census documents. The census regenerated by CI run 37830767346 (artifact `self-ast-census`) is committed. Next time: `gh run download <run> -n self-ast-census` from the "Regenerate self-AST census" workflow, then copy it over the repository with `rsync -a`.
2. **The first CI compile of this round's hand-written Rust.** These were never compiled locally:
   - `rust/src/event_log.rs` (`event_log_evidence_links`);
   - `rust/src/solver_handlers/policy_gates.rs`, `rust/src/solver_helpers/mod.rs`, `rust/src/seed/roles/reasoning.rs`;
   - `rust/src/agentic_coding/{replace_list,request_sequence,planner,write_request,capability_router,evidence_record,quote_nesting,modules}.rs`;
   - `rust/src/translation/{round_trip,pipeline,free_sentence}.rs`, `rust/src/solver_handlers/{summarization_request,web_requests}.rs`, `rust/src/summarization/dependency.rs`;
   - `scripts/audit-seed-metadata.rs`;
   - the tests `pull_request_1188_fix_gaps.rs`, `page_formalization_route.rs`, `issue_1188_round_trip_translation.rs`, `pull_request_1188_requirement_extraction.rs`.
   Expect clippy and compile fixes. The Lint lanes, the `Test (ubuntu-latest / …)` shards and Layered CI's js → rust leg report them.
3. **E2E local web, `rust/tests/e2e/tests/repository-traffic-prompt.spec.js:44`.** The Russian visitor-visibility answer no longer contains the `docs.github.com/en/rest/metrics/traffic` link. It has failed since at least 3b6aad196 and is not investigated: compare the browser route with the native one.
4. **Issue 1028 ladder.** It fell from the recorded 16 of 32 leaves to 7 on 3b6aad196 because of the G91 guard. FIX-GAPS made the guard clause-scoped (`editRegion`/`edit_region`); confirm the ladder is back to 16 or more. Leaf L02 (`experiments/issue_1028_agent_cli_ladder/leaves.tsv`) asks to add `sensor_fusion` to `PLAIN_STEPS`, which needs five-language seed prose (`thinking_step_sensor_fusion`). Change the leaf to ask for that prose too, or pick an already translated step.
5. **Agent CLI E2E** `self_coding_session_replays` should be green after the refreshed `docs/case-studies/issue-651/self-coding-run/session.json`; verify. The macOS legs were not watched this round.
6. **The upstream fix for meta-language PR #218** (needed for R1188-U23) is in [experiments/formal_ai_subagent/upstream-patches/meta-language-218-rust-layout-tests-outside-source-directory.patch](../../../experiments/formal_ai_subagent/upstream-patches/meta-language-218-rust-layout-tests-outside-source-directory.patch). It moves the `rust_layout` tests out of `src/`, the only failing check of that PR (`check-no-src-tests`). Pushing it to the PR branch `issue-217-wrap-long-rust-lines` was blocked by the session's permission classifier, so it needs a push by the owner or an agent allowed to. Then merge #218, bump the pin and regenerate the projections (`node scripts/translate-js-rust.mjs --write --fetch`), and remove the projection exemption (U23).

### 4.2 Requirements still partial

Read each row's "Missing" clause in [issue-1188-user-requirements.md](../../requirements/issue-1188-user-requirements.md). The remaining work:

| Row | What is left |
| --- | --- |
| U1 | A specialization that does not repeat a prompt verbatim is still caught only in review. |
| U2 | 10 of 51 principles have no gate: Least Surprise, Clear Naming and Validation at Boundaries wait on U4 and U17; Prefer Immutability, Explicit Side Effects, Stateless Processes, Value Objects, Aggregates and Logs as Streams are conventions; High Cohesion waits on U5. |
| U3 | The lossless `.lino` ↔ source mapping for every merge. |
| U4 | 100 abbreviated file names (`rust/src/cli_env.rs`, `solver_config.rs`, `coding/task_spec.rs`, `client_integrations/{caller_args,tool_args,global_config}.rs`, `js/app/app-*`, scripts …) and the abbreviated bindings (729 under `js/`, 913 under `rust/src/`). Rename by rule (§2.4); Rust renames need CI to compile them. |
| U5 | Group the 408 top-level `rust/tests/unit/*.rs` files into area folders: the task is written up in [tasks/test-areas.md](../../../experiments/formal_ai_subagent/tasks/test-areas.md). |
| U6, U7, U8 | The other notation families (meanings, responses, ledgers) still use `_`. `apply-notation-rules.mjs` covers ratchets, gate names, handler rules and concise lexemes; extend it with a family per name kind. The seed-lexemes exceptions in `data/meta/notation-rules.lino` say when each excepted file can be converted. |
| U9 | 11 of 94 jobs still exceed 30 minutes; `node scripts/check-ci-speed.mjs` lists them with their exceptions. |
| U13 | Keep delegating every small step to Formal AI. Open gaps: G108 to G111 (from SPEC-PARITY) in `gaps.md`. |
| U18, U19, U21 | Formalization, translation and summarization quality: see each row's measured scores and "Missing". |
| U20 | Requirement extraction scores 0.419 / 0.407. About 514 of the 1354 gold rows are paraphrases that word overlap cannot match, so the ceiling of this measure is about 0.62. A better extractor needs a semantic match, not more cue words. |
| U23 | Waits on meta-language #218 (§4.1, item 6). |
| U27 | A fully green head (§4.1). |
| U28 | Keep the progressive plan ratcheted. Level 1 (R1021-14) and level 2 (R1187-8, R994) wait on post-merge events or upstream work. |
| U29 | 14 specification gaps are left, with SPEC-PARITY's analysis: six are browser-surface choices; skill_gap needs a JS ProcedureLearningProposal; the unknown-language fallback; concept introspection (bind `network_query` to the `concept_introspection` rule set); three need live data. The repository protocol runners have no JS twin yet (R1138-3-5): [tasks/repository-runners.md](../../../experiments/formal_ai_subagent/tasks/repository-runners.md). 921 requirements are pinned only by a Rust test, and that number must only fall. |
| U30, R994 | The remaining js → rust blockers are upstream: carried siblings (74 items), regex `.test()` (49) and untyped object fields (47). They need meta-language #204, #206, #207 and #211. |

### 4.3 Requirements that only happen after the merge

- **R1021-14 and R1021-22:** a real `solve` run of the self-authored workflow lands a reviewed pull request.
- **R1187-8:** labelling an issue `formal-ai-solve` produces a bot pull request whose checks start without approval. An `issues: labeled` event always runs the default branch's workflow, so this cannot be tested from the branch.

### 4.4 Ready task files for agents

Each file in `experiments/formal_ai_subagent/tasks/` is a self-contained brief; the shared rules are in `generalize.md`.

| Task file | Covers |
| --- | --- |
| `repository-runners.md` | R1138-3-5 |
| `test-areas.md` | U5 |
| `specification-parity.md` | U29; classes 1 to 3 are done, the 14 gaps above remain |


## 5. Lessons that cost time

- **Copy-backs lose others' edits.** A sandbox copy copied back over a file another agent edited in the meantime drops their lines; T475 was lost once this way. Copy back only when the diff is exactly your expected lines.
- **New seed names with `_` fail check-notation.** So do new abbreviated names: `src`, `config`, `args`, `docs` and `env` count as abbreviations.
- **Status surfaces count files on disk.** `docs/status.md` counts every file under `data/meta/worker-line-budget/`. Rendering it while untracked files from another agent exist, then committing without them, makes CI's Rust `render-status` see a stale page.
- **Clippy updates bring new lints.** 1.99 added `doc_markdown` for bare underscore paths in doc comments: write paths in backticks.
- **A late browser worker is not a dead one.** The E2E "composer disabled" failures were a worker boot slowed by the service-worker precache. Measure before assuming a crash.
- **Guards must be clause-scoped.** The several-files guard (G91) counted a path in a later sentence as a second edit target and broke the Issue 1028 ladder. A guard must read only the clause that holds the edit.
