# Plans for issue #1138 — a truly general, self-coding meta algorithm

Written before the code, so the work can be resumed from any point if a session
is lost. Each plan is a checklist; a box is ticked in the same commit that lands
the step, and a step that turns out wrong is struck through with the reason,
never deleted. The convention is the one `docs/case-studies/issue-1085/plans/`
and `docs/case-studies/issue-710/plans/` established.

## Maintainer instructions this batch answers

Quoted, not paraphrased, so nothing in these plans substitutes other words for
the architect's. The first instruction (2026-09-14) is the one PR #888 answered
and is repeated in `docs/case-studies/issue-710/plans/README.md`; it still
governs every decision here. The second (2026-09-16) opened this batch:

> Now make pull request for https://github.com/link-assistant/formal-ai/issues/1138,
> include in this pull request as much issues as possible, do much more detailed
> plan on how to address all these than in the issue. Each item should have
> exact architecture decisions listed, so we carefully plan all implementation
> in sync, we plan to make it in a single pull request, so you need to create
> separate work tree for it, and start with as detailed analysis and plan as
> possible, for everything we find root causes, and propose multiple solution
> options, and select best of them. We should double check all our vision,
> requirements and all docs, no docs should be outdated, and our vision must be
> fully consistent with latest evolving requirements, all not yet fully done
> that from all previous merged pull request must be planned here to actually
> fully done it here.

The standing doctrine applies unchanged: associative stack only; generalization
over memoization with held-out paraphrases in en, ru, hi, zh and es;
deterministic and honest (0 % is a number, a fake floor is not); no deferral,
no budgets, no bypasses; nothing is a hard task, split it until each leaf is
directly solvable; every solution discovered dynamically from trusted sources
and rediscoverable after being forgotten; nothing hard-coded for any test
([`VISION.md`](../../../../VISION.md), "The Goal Is The Meta Algorithm").

## How to resume

Plan authoring and reconciliation are **complete**; no production code has
changed yet. Read [`00-root-causes-and-integration.md`](00-root-causes-and-integration.md)
first — its §4 holds the five shared contracts with the plan that owns each, and
its §9 is the reconciliation log — then
[`14-implementation-order.md`](14-implementation-order.md), which is the single
ordered leaf list. Start at wave T (write every named test, failing, in one
commit) and work down; a plan's own file is the detail behind its leaf rows.

```bash
# The PR branch is checked out in a worktree of the main clone.
cd /Users/konard/Code/Archive/link-assistant/formal-ai/.claude/worktrees/issue-1138
git status --short                 # inspect and preserve unfinished work
df -h /private/tmp                # check space before building
export RUSTUP_TOOLCHAIN=1.98.1     # the crate needs 1.98; the local default is older
export LINDERA_DICTIONARIES_PATH=/tmp/formal-ai-lindera-cache
export CARGO_TARGET_DIR=/Users/konard/Code/Archive/link-assistant/formal-ai/target

# Cheap gates first (no crate build), then the rust stage.
rust-script scripts/check-hardcoded-language.rs
rust-script scripts/check-minimal-core-boundary.rs
rust-script scripts/run-ci-gates.rs --stage rust

# After any src/ change and any docs/requirements/ change respectively.
cargo run --example regenerate_self_ast_census
rust-script scripts/assemble-requirements.rs --write

# Commit with a message file, never -m (backticks in -m are executed by the shell).
CARGO_TEST_NO_PRUNE=1 DOCKER_NO_PRUNE=1 git commit -F /tmp/msg.txt
```

Rules that bit earlier sessions: never `git stash` in this shared worktree
(use a WIP commit); never rewrite pushed history (branch protection rejects
non-fast-forward updates, use the append-only retraction the repository
provides); never lower a gate or loosen a policy to get CI green.

## Plans

| # | Plan | Bottleneck | Closes / delivers | Status |
| --- | --- | --- | --- | --- |
| 00 | [Root causes and integration](00-root-causes-and-integration.md) | all | the one architectural root cause behind B1–B12, the dependency graph between plans, the five shared contracts with their single owners, and the reconciliation log | planned, reconciled |
| 01 | [Live concept lookup](01-live-concept-lookup.md) | B1 | one `UnknownConceptLookup` over the sources registry, used by the universal loop and the coding path | planned, reconciled |
| 02 | [Composition from sources](02-composition-from-sources.md) | B2 | composer over retrieved parts; idiom catalog becomes a deletable bootstrap; full 164/500 upstream runs | planned, reconciled |
| 03 | [Repository workspace protocol](03-repository-workspace-protocol.md) | B3 | one clone/locate/edit/test/diff protocol shared by SWE-bench, the #848 ladder and self-coding; `solve --model formal-ai` as an attributed authoring path | planned, reconciled |
| 04 | [Formalization depth](04-formalization-depth.md) | B4 | formalizer that emits needs and stores concepts and procedures, recursing through plan 01 | planned, reconciled |
| 05 | [Obligation execution evidence](05-obligation-execution-evidence.md) | B5 | every obligation node carries an execution record before `Satisfied`; unsatisfied nodes decompose | planned, reconciled |
| 06 | [Prerequisite discovery](06-prerequisite-discovery.md) | B6 | "command not found" becomes a requirement solved through plans 01/04; docker execution for #930/#937; browser runtime honesty | planned, reconciled |
| 07 | [Learning loops that change behavior](07-learning-loops-that-change-behavior.md) | B7 | learned items change the next answer across the registry; PR #887 decision; automatic source-cache reconstruction | planned, reconciled |
| 08 | [Verifiable task routing](08-verifiable-task-routing.md) | B8 | any task with a verifiable expectation enters the discovery path; GSM8K/MATH/CoEdIT/object counting re-measured | planned, reconciled |
| 09 | [Handler migration ratchet](09-handler-migration-ratchet.md) | B9 | pending handlers migrated to meta-methods; promotion predicates in seed; doublets store read by the solver; strict-downward ratchets | planned, reconciled |
| 10 | [Intent routing](10-intent-routing.md) | B10 | capability routing on held-out paraphrases with zero cross-tool misroutes; #745/#758; frontier queue #1087 | planned, reconciled |
| 11 | [Docs consistency audit](11-docs-consistency-audit.md) | B11 | every outdated or contradictory statement across VISION, ROADMAP, REQUIREMENTS shards, traceability, benchmarks, meta-algorithm, with replacements; single generated status table | planned, reconciled |
| 12 | [Selection heuristics](12-selection-heuristics.md) | B12 | least action, TRIZ, 2-4-6 refutation-first search, balanced task splitting as registry methods | planned, reconciled |
| 13 | [Merged-PR carry-over and issue coverage](13-merged-pr-carryover-and-issue-coverage.md) | all | every partial delivery declared in a merged PR mapped to a plan leaf; the open issues this PR closes and does not close, with reasons | planned, reconciled |
| 14 | [Implementation order](14-implementation-order.md) | all | the single ordered leaf list across plans 01–12 plus plan 13's new leaves and plan 11's doc leaves — 340 leaves in waves T, I1–I9, F and D, each with its deliverable, files, test, gate and dependencies; plus the local resource rules | planned, reconciled |

## Order of work

1. Plan 00 — the shared contracts are fixed before any plan is implemented,
   so the twelve plans stay in sync in one pull request.
2. Plans 11 and 13 — the docs audit and the carry-over ledger are inputs to
   every other plan's scope; they are finished before code changes.
3. Plans 01 → 04 → 02 → 05 → 06 → 03 → 07 → 08, in that order, because each
   consumes the previous one (lookup feeds formalization, formalization feeds
   composition, composition feeds execution evidence, evidence feeds
   prerequisite recovery, all of it feeds the workspace protocol, and only then
   can learning loops and non-coding suites be measured on the real pipeline).
4. Plans 09, 10, 12 as ratchets and registry methods alongside, never as new
   handlers.
5. Plan 11's replacements are applied last, after the numbers exist, so no
   document states a number that was not measured.

Plan 14 turns that order into 340 leaves in eleven waves and is the document to
work from; this list is the reasoning behind it.

## Log

- 2026-09-16: issue #1138 filed from three parallel audits (plans of #710,
  requirements and traceability and benchmarks, GitHub history) plus direct
  source inspection on `main` at `be8fd3174`. Worktree `issue-1138` created
  on branch `worktree-issue-1138` from that commit. Plan directory created;
  seven Opus 5 agents assigned one plan area each; plan 00 and this index
  written by the coordinating session.
- 2026-09-16: **plans 01–13 reconciled against plan 00 §4 and against each
  other**, executing plan 00 §8's reconciliation leaf. All four of its boxes are
  now ticked and the work is recorded in plan 00 §9.

  **Renames (19).** One name per contract, with both plans amended in place and a
  one-line `reconciled: was X, now Y because Z` note at each amended spot. The
  load-bearing ones: `ExecutionRecord` → `Evidence` (plans 05, 07, 12 adopt the
  contract name plans 00, 03 and 06 already used); `Need`/`NeedKind`/`NeedState`
  given one home in `src/needs.rs` instead of two (plan 01's
  `src/seed/sources.rs` and plan 04's `src/formalization/needs.rs`), landed by a
  new plan-00 leaf C1 before either consumer; plan 02's
  `src/coding/source_lookup.rs` withdrawn so plan 01 owns the one `SourceLookup`
  implementation, and plan 09's M2 family renamed `source_lookup` →
  `retrieval_method` for the same reason; plan 03's `Command` → `RunCommand` and
  `VerifyBackend` → plan 06's `ExecutionBackend`; plan 02's second registry axis
  `coding_role` withdrawn in favour of `need_kinds`; `GuideBounds` →
  `LookupBounds`; two `procedure_text` modules → one `src/procedure_text.rs`
  owned by plan 04; two `docs_requirements_issue_1138` test paths → one; plan
  08's `VerifiedAnswer::checks` from `Vec<String>` to `Vec<Evidence>`.

  **Conflicts resolved (14).** Two plans deleting the same event
  (`policy:no_fetch_capability`); plan 02 wiring a trait plan 01 deletes; plan 09
  adding a JavaScript seed parser while closing the issue that asks for it to be
  deleted; plan 08 adding a 60th handler while plan 09 makes the ratchet strict
  (resolved by moving plan 08's L17 ahead of L8 so the slot is freed before it is
  filled); four plans adding measures to an at-or-below checker; two plans
  rewriting one `docs/meta-algorithm.md` paragraph; two trace-changing leaves
  sharing one R343 parity run; plan 12 changing the leaf set plan 05's join test
  matches against; and five more, each with its plan-00 §9 row.

  **Issues moved (5).** Plan 13's `Closes` list fell from 37 to **32**. #453,
  #491, #901, #954 and #1090 moved to "will not close", each because a plan's own
  text says the ask is not fully delivered — R453-M4 and R491-C4 are filed Open,
  #901's validation corpus is out of scope, no leaf reorganizes `src/`, and
  #1090's resolution is a maintainer decision plan 11 records rather than makes.
  Three issues stayed only because the missing leaf was **added** rather than the
  row quietly downgraded: #950 (plan 09 leaf 42), #949 (plan 11 L75), #1089
  (plan 11 L76). Forty-two issue references, covering twenty-five distinct issues, were added to plans 01–12 so every
  plan's "Issues addressed" list and plan 13's coverage table now agree in both
  directions.

  **Docs rows (120).** Every "Docs to update" body of plans 01–10 and 12 moved
  verbatim into plan 11 as rows **D156–D275**; each plan now carries a pointer to
  its row ids and no document is described in two places. Plan 11 also gained the
  three sections it was the only plan to lack — "Issues addressed", "Root causes"
  and "Tests first" — and its five in-document generated regions became one
  generated `docs/status.md`, which is what plan 00 §4.5 asked for and what plan
  11's own risk 3 asked about.

  **Evidence corrected (8).** Plan 00 §1 now records, with the file each was read
  from: the handler census is 58 precedence rows plus 5 prelude methods with 40
  pending and 39 `try_*` entries, not "37 files / 48 entries"; there are **19**
  promotion predicates, not 18; the #840 ladder is **24/24**, not 8/24 (8/24 is
  the maintainer's 2026-07-25 number); **11** `local_path_*` roles exist, so the
  "no local-search counterpart" line is stale; the least-action ranker **exists
  and is dormant** behind a `draft_count` of 1, so #491 is not "no code"; the
  equation corpus records **10** loud-failure rows, not the seven
  `docs/benchmarks.md` lists; the source-cache-reconstruction clause is
  **R710-R7**, not R710-R8; and **0.15 % is prose, not a ledger row** — the
  newest ledger row is 171 basis points for v0.350.0, and no plan may cite the
  0.15 % figure.

  **Plan 14 written.** 340 leaves across plans 01–12, plan 13's fourteen new
  leaves and plan 11's seventy-six doc leaves, in one order respecting plan 00
  §5, grouped into wave T (tests first, 63 files, one commit), waves I1–I9
  (implementation in bulk per dependency stage), wave F (the Formal AI self-use
  loop, every failure recorded as a test) and wave D (documents and the generated
  status surface). Each leaf carries its deliverable, the files it touches, the
  test that proves it, the gate it must keep green and its dependencies. Three
  orderings are left explicitly open because fixing them would be a decision no
  plan made.
