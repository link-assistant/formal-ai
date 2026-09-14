# Plans for finishing PR #888 (issue #710) — coding and benchmarks by discovery

Written before the code, so the work can be resumed from any point if a session
is lost. Each plan is a checklist; a box is ticked in the same commit that lands
the step, and a step that turns out wrong is struck through with the reason,
never deleted. The convention is the one `docs/case-studies/issue-1085/plans/`
established.

## Maintainer instruction this batch answers (2026-09-14)

Quoted, not paraphrased, so nothing in these plans substitutes other words for
the architect's:

> Make everything work in https://github.com/link-assistant/formal-ai/pull/888
> as we expected, collect all issues requirements from all previous issues,
> check requirments document, and double check what is fully done, and what is
> done partially focus with coding related issues and benchamarks so we have
> much more capability out the box, prefer solution by generalization, ideally
> each solution should be discovered dynamically using all our best tools, so
> it is always easy to forget all discoverable experience, and rediscover it.
> We should not hard code solutions for any of tests, instead we should
> according to our best vision about meta algorithm make this algorithm to
> discover enough knowledge in the internet to understand each word/concept and
> reconstruct from the formalized knowledge collected in the internet the step
> by step guide/algorithm. Formalization itself may need recursive knowledge
> collection from trusted sources. Please try to reproduce how previously
> humans did manual programming/research and so on relying mostly on the web
> search, if they knew nothing about domain or topic, searched for parts of
> ready algorithms and so on, search the info on how to do each part, or what
> each part means and so on. We need to have dynamic discovery algorithms, that
> use primarely trusted sources. So the goal is not to know everything in
> advance, the goal to know how to get know anything when it is needed.
> Everything must be done in 888 PR, you have unlimited time, do as needed,
> make sure to carefully and deeply analyze and plan in markdown documents
> before any actions, so if work interrupted at any stage we will always be
> able to continue.

The standing doctrine still applies unchanged: associative stack only;
generalization over memoization with held-out paraphrases in en, ru, hi, zh and
es; deterministic and honest (0 % is a number, a fake floor is not); no
deferral, no budgets, no bypasses; nothing is a hard task — split it until each
leaf is directly solvable ([`VISION.md`](../../../../VISION.md), "The Goal Is
The Meta Algorithm").

## How to resume

```bash
# The PR branch is checked out in a worktree of the main clone.
cd /tmp/wt888                      # branch merge-888-into-main → origin/issue-710-14da90b08a12
git status --short                 # must be clean before starting a leaf
export RUSTUP_TOOLCHAIN=1.98.1     # the crate needs 1.98; the local default is older

# Cheap gates first (no crate build), then the rust stage.
rust-script scripts/check-hardcoded-language.rs
rust-script scripts/check-minimal-core-boundary.rs
rust-script scripts/run-ci-gates.rs --stage rust

# Focused suites named in each plan, e.g.
cargo test --test unit issue_710 -- --nocapture
cargo run --example regenerate_self_ast_census    # after any src/ change
rust-script scripts/assemble-requirements.rs --write   # after any docs/requirements/ change

# Commit with a message file, never -m (backticks in -m are executed by the shell).
git commit -F /tmp/msg.txt
```

Where a step left state on disk that the next step needs, the plan names the
path. Nothing in these plans depends on the shell history of the session that
wrote it.

## Plans

| # | Plan | Closes / delivers | Status |
| --- | --- | --- | --- |
| 00 | [PR #888 CI recovery](00-pr-888-ci-recovery.md) | the three red checks on `cde14085d`: web bundle drift, the E2E slowdown, the two commits without `Formal-AI-Model` | sections 1-2 done, section 3 next |
| 01 | [Requirements audit: coding and benchmarks](01-requirements-audit-coding-and-benchmarks.md) | every coding/benchmark requirement from #1–#1137 classified done / partial / not done with evidence, and what this PR does about each | drafted |
| 02 | [Dynamic discovery design](02-dynamic-discovery-design.md) | the meta algorithm applied to a coding task: understand each word, search trusted sources for ready parts, reconstruct the algorithm, verify, remember, forget and rediscover | drafted |
| 03 | [Implementation leaves](03-implementation-leaves.md) | the ordered, individually verifiable leaves that deliver plan 02 and the audit's not-done rows inside PR #888 | drafted |

## Order of work

1. Plan 00 — the branch must be green before anything is added to it.
2. Plan 01 — the audit is the input to plan 03's scope; finish it before coding.
3. Plan 03 leaves in the listed order; plan 02 is the design they implement.
4. Final preparation: requirement rows, traceability rows, changelog, PR body,
   one CI wait.

## Log

- 2026-09-14: plans written. State at that moment: `cde14085d` pushed; CI red
  on Lint and Format Check (`src/web/app.js` rebuilt locally with bun 1.2.20
  while CI builds with the pinned 1.4.0), on the E2E job (the held-out
  computer-use generalization step took 85 s on `main` run 34810804701 and
  more than 540 s on this branch, run 34815480961), and on the Self-Hosting
  Evidence Check (commits `8a2054245` and `ae1194e7c` carry
  `Formal-AI-Session` and `Formal-AI-Evidence` but no `Formal-AI-Model`).
  The scheduled External Benchmarks run of the same day scored HumanEval 1/20
  and the only pass came from a per-task body in
  `src/solver_handlers/program_synthesis.rs`; see plan 01 row B1.

- 2026-09-14, later: plan 00 sections 1 and 2 are done and committed.

  The web bundle was rebuilt with the pinned bun 1.4.0; the other three
  bundles came out byte identical to `main`, so only `app.js` had drifted.
  `CHANGELOG.md` and the fragment release map were regenerated by the script
  that owns them, which the branch was behind on after merging `main`.

  The E2E slowdown was measured rather than argued, and the three solver
  suspects were wrong. A real captured 55 KB agentic request solves in 30 ms
  on this branch and 29 ms on `main`; the whole script run locally was 336 s
  on the branch against 324 s on `main`. `sample` put 3789 of 4649 samples in
  `fcntl`, and the cause is `synchronize_memory_events` rebuilding a
  projection with one `fsync` per doublet, which is superlinear in the
  history the suite accumulates. Staging the replacement without per-append
  sync and flushing once before publishing takes a 112-event rebuild from
  15.8 s to 0.2 s and the whole script from 10 min 41 s to 39 s.
  `rebuilding_a_projection_costs_what_appending_to_it_costs` pins it as a
  ratio; it was confirmed to fail without the fix.

  Plan 00 section 3 correction: the two commits' evidence logs contain no
  version string at all, so the `formal-ai/0.317.0` value the plan first
  proposed would have failed the gate it was meant to satisfy, which reads
  the trailer value back out of the committed evidence. The trailer value
  will be the bare `formal-ai`, which the evidence does contain.

  Every gate in stage `rust` passes. One integration test,
  `with_formal_ai::standalone_with_formal_ai_binary_uses_same_wrapper`, fails
  identically on `main` at `de88ca251`, so it is pre-existing and is not
  this branch's to fix here.

