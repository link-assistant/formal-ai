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

Current work is **plan authoring**: the twelve bottleneck plans, the docs
audit and the carry-over ledger are being written; no production code has
changed yet. Read [`00-root-causes-and-integration.md`](00-root-causes-and-integration.md)
first, then the plan whose row below is not yet "implemented".

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
| 00 | [Root causes and integration](00-root-causes-and-integration.md) | all | the one architectural root cause behind B1–B12, the dependency graph between plans, and the shared contracts every plan must implement against | drafted |
| 01 | [Live concept lookup](01-live-concept-lookup.md) | B1 | one `UnknownConceptLookup` over the sources registry, used by the universal loop and the coding path | drafting |
| 02 | [Composition from sources](02-composition-from-sources.md) | B2 | composer over retrieved parts; idiom catalog becomes a deletable bootstrap; full 164/500 upstream runs | drafting |
| 03 | [Repository workspace protocol](03-repository-workspace-protocol.md) | B3 | one clone/locate/edit/test/diff protocol shared by SWE-bench, the #848 ladder and self-coding; `solve --model formal-ai` as an attributed authoring path | drafting |
| 04 | [Formalization depth](04-formalization-depth.md) | B4 | formalizer that emits needs and stores concepts and procedures, recursing through plan 01 | drafting |
| 05 | [Obligation execution evidence](05-obligation-execution-evidence.md) | B5 | every obligation node carries an execution record before `Satisfied`; unsatisfied nodes decompose | drafting |
| 06 | [Prerequisite discovery](06-prerequisite-discovery.md) | B6 | "command not found" becomes a requirement solved through plans 01/04; docker execution for #930/#937; browser runtime honesty | drafting |
| 07 | [Learning loops that change behavior](07-learning-loops-that-change-behavior.md) | B7 | learned items change the next answer across the registry; PR #887 decision; automatic source-cache reconstruction | drafting |
| 08 | [Verifiable task routing](08-verifiable-task-routing.md) | B8 | any task with a verifiable expectation enters the discovery path; GSM8K/MATH/CoEdIT/object counting re-measured | drafting |
| 09 | [Handler migration ratchet](09-handler-migration-ratchet.md) | B9 | pending handlers migrated to meta-methods; promotion predicates in seed; doublets store read by the solver; strict-downward ratchets | drafting |
| 10 | [Intent routing](10-intent-routing.md) | B10 | capability routing on held-out paraphrases with zero cross-tool misroutes; #745/#758; frontier queue #1087 | drafting |
| 11 | [Docs consistency audit](11-docs-consistency-audit.md) | B11 | every outdated or contradictory statement across VISION, ROADMAP, REQUIREMENTS shards, traceability, benchmarks, meta-algorithm, with replacements; single generated status table | drafting |
| 12 | [Selection heuristics](12-selection-heuristics.md) | B12 | least action, TRIZ, 2-4-6 refutation-first search, balanced task splitting as registry methods | drafting |
| 13 | [Merged-PR carry-over and issue coverage](13-merged-pr-carryover-and-issue-coverage.md) | all | every partial delivery declared in a merged PR mapped to a plan leaf; the open issues this PR closes and does not close, with reasons | drafting |
| 14 | [Implementation order](14-implementation-order.md) | all | the single ordered leaf list across plans 01–12, with the gate that proves each leaf | pending |

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

## Log

- 2026-09-16: issue #1138 filed from three parallel audits (plans of #710,
  requirements and traceability and benchmarks, GitHub history) plus direct
  source inspection on `main` at `be8fd3174`. Worktree `issue-1138` created
  on branch `worktree-issue-1138` from that commit. Plan directory created;
  seven Opus 5 agents assigned one plan area each; plan 00 and this index
  written by the coordinating session.
