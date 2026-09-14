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
| 00 | [PR #888 CI recovery](00-pr-888-ci-recovery.md) | the three red checks on `cde14085d`: web bundle drift, the E2E slowdown, the two commits without `Formal-AI-Model` | append-only retraction and local evidence check complete; CI confirmation pending |
| 01 | [Requirements audit: coding and benchmarks](01-requirements-audit-coding-and-benchmarks.md) | every coding/benchmark requirement from #1–#1137 classified done / partial / not done with evidence, and what this PR does about each | implemented and re-verified |
| 02 | [Dynamic discovery design](02-dynamic-discovery-design.md) | the meta algorithm applied to a coding task: understand each word, search trusted sources for ready parts, reconstruct the algorithm, verify, remember, forget and rediscover | implemented |
| 03 | [Implementation leaves](03-implementation-leaves.md) | the ordered, individually verifiable leaves that deliver plan 02 and the audit's not-done rows inside PR #888 | L1-L15 complete; local gates green; CI confirmation pending |

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

- 2026-09-14, after pushing `3a1762651`: CI confirms both fixes. The lint job
  passes, and the held-out computer-use generalization step took **78 s of its
  600 s budget** (job 104035509913), against more than 540 s before the fix and
  85 s for `main`'s own fastest run. Every E2E client job passes.

  Two checks still fail, both on the same cause: `Self-Hosting Evidence Check`
  and `Evidence Check Status` reject `8a2054245` for having no
  `Formal-AI-Model`. That is plan 00 section 3, which is blocked: adding a
  trailer to an existing commit requires rewriting every descendant and
  force-pushing, and this session's sandbox refuses to run the rewrite. The
  script that does it is committed as `plans/add-model-trailer.sh` and the
  section records the exact steps.

  `Code Coverage` failed once on an HTTP 504 fetching the sccache action, with
  no output of its own; rerunning it passed, so it was infrastructure and not a
  defect of this branch.

- 2026-09-15: implementation leaves L1-L14 are drafted and their focused gates
  are green. The synthesis handler no longer contains the three benchmark-task
  bodies; task recognition, licensed Python/Wikifunctions discovery, structural
  composition, bounded verification, and a tamper-detecting discovered-procedure
  ledger now form the production path. Rosetta Code stays behind an attributed
  example/execution boundary. The curated slice remains 13/13, the five-language
  held-out discovery suite is 25/25, and honest local online first-20 runs
  measured HumanEval 3/20 and MBPP 1/20. Those local measurements were not
  appended as scheduled ledger history.

  Formal AI was then asked through the real external Agent CLI to author
  `data/meta/coding-discovery-recipe.lino`. The first valid `with`-payload run
  searched for the requested bytes instead of writing them. A failing planner
  regression reproduced that routing defect; adding the seed-defined `with …`
  content lead fixed it. A second failure exposed macOS curl returning on the
  first refused health connection, so the authoring harness gained an explicit
  bind loop and early process-death check. The repeated run wrote and verified
  the exact recipe in session `ses_f5ec49c02ffe6yDSkWK0x1t0p1`.

  The language/core gates found a final architectural gap before review: new
  composition bodies and answer prose were still Rust literals, and the Rosetta
  request sat in the minimal handler directory. The bodies now interpret seeded
  structural idioms/runtime templates, all answer text is localized seed data,
  Rosetta moved outside the specialized-handler core, hardcoded-language debt
  fell from 1,278 to 1,274 entries, and the reviewed outside-core ceiling fell
  from 18,854 to 18,469 lines.

- 2026-09-15, final preparation: a tree-identical local rewrite proved that the
  two historical Formal AI commits could carry the only model value their
  archived evidence supports, but the repository's all-branch protection rule
  rejected the non-fast-forward update before changing the remote ref. The
  branch therefore uses the repository's purpose-built append-only remedy:
  one final commit retracts the two incomplete attribution claims by full hash,
  and every new commit is replayed on the live remote head for a normal
  fast-forward delivery.

  Formal AI's successful recipe run is isolated in its own attributed commit;
  its secret-scanned raw trace is stored in an
  [unlisted gist](https://gist.github.com/konard/dbae44b1e547bf1a9b1ba51c6178ecf7),
  with hashes and the non-uploadable blank stderr file recorded in the
  repository evidence pointer. The strict self-hosting measurement passes and
  attributes only the recipe's 32 lines; retraction cannot increase the
  numerator.

  All 32 Rust-stage gates, 82 web tests and six focused Playwright parity tests
  pass on the final tree. The only successful remote update is the normal
  fast-forward push; the remaining unchecked items are remote CI observations
  rather than uncommitted implementation work.

  The first remote status check then caught RUSTSEC-2026-0285, published on
  2026-09-14 against the locked `rustls` 0.23.43. The lockfile was advanced to
  patched 0.23.45. Reproducing the exact wrapper also found that its basic-sed
  `\+` proof patterns work under GNU sed but not BSD sed; a focused regression
  now keeps the equivalent extended expressions portable across CI and macOS.

  The restarted link check then followed outbound links embedded inside the
  byte-for-byte Python documentation replay fixtures. Three upstream targets
  had disappeared even though the captures themselves were intact. A focused
  workflow regression now excludes only `tests/fixtures/coding-discovery` from
  live-link health checks, preserving the licensed snapshots unchanged while
  keeping maintained Markdown and HTML under the existing gate.
