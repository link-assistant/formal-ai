# Plan 00 — Root causes and integration (all bottlenecks of #1138)

This plan answers one question the twelve bottleneck plans cannot answer
individually: why the same shape of gap keeps reappearing, and what the twelve
plans must share so that one pull request lands them in sync. Plans 01–12 hold
the per-bottleneck root causes, options and decisions; this plan holds the
root cause they have in common, the dependency graph between them, and the
contracts each of them must implement against.

## 1. Evidence summary (measured on `main` at `be8fd3174`, 2026-09-16)

| Observation | Where | Bottleneck |
| --- | --- | --- |
| `UnknownConceptLookup` has one implementation, `NoLookup`; production `discover()` calls it | `src/coding/concept_discovery.rs:87`, `:189` | B1 |
| The universal solver's external-search step logs `policy:no_fetch_capability` and returns | `src/solver.rs:874-884` | B1 |
| 13 trusted source kinds with licenses, APIs, cache paths and settings opt-outs already exist, used live only by the how-to handler | `data/seed/sources-registry.lino`; #991 | B1, B4 |
| Composition draws on 151 named per-language template programs (12 tasks, 14 languages), 82 seeded structural meanings, 40 hand-written composition arms and 7 embedded oracle snapshots (the issue said 25; plan 02 counted) | `src/coding/catalog/templates_*.rs`, `blueprint_programs.rs`, `data/seed/meanings-coding-structure.lino`, `src/knowledge.rs` `CodingOracle` | B2 |
| HumanEval and MBPP are 20/20 on the first 20 tasks only; tasks 21–164 and 21–500 have never been run | `data/benchmarks/external-results.lino` | B2 |
| A SWE-bench instance becomes one prompt asking for a diff; no clone, no checkout, no base commit anywhere in the harness | `src/external_benchmarks/cases.rs:152-166` | B3 |
| An open-ended refactor of this repository produced a byte-identical file; a regression-authoring request read a nonexistent file and stopped | `docs/case-studies/issue-710/plans/07-*.md` "Live source-edit probe", `06-*.md` | B3 |
| Memory-contract probes yield sentences, "zero concepts or procedures", 2 of 9 protocol primitives | `docs/case-studies/issue-710/plans/06-*.md`, `07-*.md` "Observed limits" | B4 |
| A selected method is "planned, not satisfied"; runtime per-need verification feedback is open | `REQUIREMENTS.md` R340–R344 | B5 |
| `kotlinc: command not found`, `scalac: not found`; automatic setup discovery is the open row of plan 07 | `docs/case-studies/issue-710/plans/07-*.md` | B6 |
| Self-improvement gated off, method learning withholds, promotion never pushes, dreaming PR stale | `docs/meta-algorithm.md:137-141`; #922; #656; PR #887 | B7 |
| GSM8K 2/20, MATH 0/20, object counting 0/20, CoEdIT 0/20, unchanged since v0.347.0 | `data/benchmarks/external-results.lino` | B8 |
| 37 handler files / 48 entries pending migration; 1,286 allowlisted prose literals; the solver never reads the doublets store | #959; `scripts/check-hardcoded-language.rs`; `VISION.md` "Current Direction" | B9 |
| #745 and #758 re-measured still-broken, ladder 8/24 | maintainer comment on #710, 2026-07-24 | B10 |
| Traceability has no rows for R710-R*, R873, R919, R922, R924; ROADMAP still says 0/20; R67 describes a removed defect | `docs/requirements-traceability.md`; `ROADMAP.md:492,570`; `REQUIREMENTS.md:114` | B11 |
| Least action, TRIZ, 2-4-6, moonshot splitting: no code | #491, #901, #802, #453 | B12 |

## 2. The common root cause

Every bottleneck above is an instance of one architectural fact: **the meta
algorithm's steps exist as separate, hand-instantiated pipelines per topic,
and the step that should connect them, "I lack X, go get X", is a stub.**

Concretely:

1. **Retrieval is per-handler, not per-need.** The how-to handler fetches
   wikiHow and Stack Exchange; the coding catalogs fetch Python docs,
   Wikifunctions, OEIS and Rosetta; the research handler fetches search
   results. Each was built for one topic, with its own client, cache and
   parser. The universal loop has no retrieval step at all
   (`policy:no_fetch_capability`), and the coding path's lookup trait was
   declared but never implemented. So the question "what does this word mean"
   has an answer only where someone wrote a topic-specific fetcher (B1, B4,
   B8, B10).
2. **Knowledge is stored in the shape the fetcher returned, not as concepts
   and procedures.** Sentences are preserved, structural meanings are seeded
   by hand, templates are written per task family. A composer can only
   compose what has been reduced to parts, so the parts had to be seeded
   (B2, B4, B9).
3. **Completion is decided by routing, not by execution.** A need is marked
   satisfied when a method is selected, so a pipeline that never ran the
   program, never cloned the repository, and never installed the compiler
   still reports success. Without execution evidence there is nothing for a
   learning loop to learn from, so learning loops were made inert instead
   (B3, B5, B6, B7).
4. **Status is written by hand in several places.** Because the pipeline
   does not produce evidence uniformly, humans wrote status into VISION,
   ROADMAP, REQUIREMENTS, traceability and plans separately, and they drifted
   (B11).
5. **Choice among alternatives has no scoring function.** When several
   procedures are retrieved, or a task is too large, nothing ranks or splits
   them, so the seeded single answer wins by default (B12).

The fix is therefore not twelve features. It is one loop with five shared
contracts, and twelve places where that loop replaces a hand-built pipeline.

## 3. Options for the integration shape

### Option A — twelve independent fixes, one per bottleneck

Each plan implements its own lookup, cache, evidence record and status entry.

- Pros: plans can be implemented in any order; least coordination.
- Cons: reproduces the root cause (per-handler pipelines); twelve caches and
  twelve evidence formats; the docs audit has to be redone per plan; violates
  "associative stack only" because each fix arrives as another handler.
- Doctrine fit: poor.

### Option B — one new "discovery runtime" crate that every handler calls

A new module owns lookup, formalization, composition, execution and memory;
handlers are rewritten to call into it.

- Pros: single implementation of each contract; the migration ratchet (B9)
  and this runtime are the same work.
- Cons: a big-bang rewrite of 37 handlers in one step is exactly the "hard
  task" the doctrine says to split; risks a long red period on the branch.
- Doctrine fit: good in the end state, poor in the transition.

### Option C — five shared contracts fixed first, then each plan migrates its
area onto them, in dependency order, behind the existing gates (selected)

The contracts are small traits and `.lino` schemas that already-existing code
can adopt one call site at a time. The universal loop gains the retrieval step
first, so every later plan is measured on the real pipeline. Handlers migrate
as the plans reach them; the B9 ratchet records progress.

- Pros: every commit keeps CI green; each leaf is directly solvable; the
  order guarantees each plan consumes the previous one's output rather than a
  new seed; the same evidence format feeds the status table, so B11 is solved
  structurally rather than by editing prose.
- Cons: the contracts must be right before the plans start, so this plan
  must be reviewed first; some plans (03, 06) cannot be measured until 01, 04,
  02 and 05 land.
- Doctrine fit: matches "split until each leaf is directly solvable" and
  "generalization over memoization".

**Decision: Option C.** Option A rejected because it recreates the root
cause. Option B rejected because the transition violates the doctrine and
would leave the branch red for the duration.

## 4. The five shared contracts

Names below are the contracts plans 01–12 implement against. Where a plan
proposes a different name for the same thing, the reconciliation leaf in
plan 14 renames to the one recorded here, so there is exactly one of each.

### 4.1 Need — the "I lack X" record

A need is the unit that connects every step. It is a link in the associative
store, not a Rust-only struct, so it can be remembered, forgotten and
rediscovered.

```
need <id>
  kind      concept | procedure | part | prerequisite | evidence | decision
  subject   "<surface text as written by the user or the failing tool>"
  language  en | ru | hi | zh | es | …
  raised_by <obligation id | need id | tool result id>
  state     open | planned | satisfied | unsatisfiable
  satisfied_by <evidence id>          # only when state = satisfied
```

Rules: a need is `planned` when a method is selected and `satisfied` only
when an `evidence` record (4.3) references it. `unsatisfiable` is an honest
terminal state that the answer must report; it is never silently dropped
(this is the B5 fix). A need of kind `prerequisite` is raised by a tool
failure such as `command not found` (B6) and is solved by the same lookup and
formalization path as a `concept` need (B1, B4).

### 4.2 Source lookup — one client over the sources registry

```rust
pub trait SourceLookup {
    /// Resolve a need against the sources registry, in registry order for the
    /// need's kind, honoring settings opt-outs and licenses. Returns evidence
    /// or an honest `NotFound` with the sources consulted.
    fn lookup(&mut self, need: &Need, bounds: &LookupBounds) -> LookupOutcome;
}
```

- One implementation, backed by the existing `CachedSourceClient` and
  `CurlSourceTransport` (`src/coding/function_catalog/`), extended to every
  source in `data/seed/sources-registry.lino`. The how-to handler's live
  wikiHow and Stack Exchange calls migrate onto it (plan 09 leaf).
- `LookupBounds` are depth and evidence bounds, never a time or token budget.
- The content-addressed cache is the only persistence; offline replay of a
  cached lookup is byte-identical and recorded as `cache_hit`. `forget`
  deletes by content id; `rediscover` re-fetches and must reproduce the id.
- The universal loop (`src/solver.rs`) calls this where it now logs
  `policy:no_fetch_capability`; the coding path's `UnknownConceptLookup`
  becomes an adapter over it (plan 01 decides whether the coding trait stays
  as a thin adapter or is replaced).
- Registry order and trust are data: the registry row's `kind`,
  `source_tier` and `primacy` fields already exist; plan 01 adds the
  `need_kinds` a source may answer.

### 4.3 Evidence — the execution record every satisfaction needs

```
evidence <id>
  for_need    <need id>
  produced_by <method id>
  command     "<exact command or api call>"
  exit        <integer or http status>
  output_hash <sha256 of observed output>
  source_ids  (<content ids consulted>)
  recorded_at <utc timestamp>
```

No obligation, need or work unit may reach `satisfied` without one. The
self-hosting metric, the benchmark ledger and the status table (4.5) read
this record instead of prose. Plan 05 owns the schema; plans 02, 03, 06 and
08 produce it; plan 07 consumes it to decide what was learned.

### 4.4 Workspace — one protocol for any repository

```rust
pub trait Workspace {
    fn open(spec: &WorkspaceSpec) -> Result<Self, WorkspaceError>;   // clone at base commit, or the own repo
    fn locate(&self, need: &Need) -> Vec<Location>;                   // self-AST census for own repo, search otherwise
    fn read(&self, loc: &Location) -> Result<String, WorkspaceError>;
    fn edit(&mut self, change: &Change) -> Result<Evidence, WorkspaceError>;
    fn run(&mut self, cmd: &Command) -> Evidence;                     // named tests, build, program
    fn diff(&self) -> Result<UnifiedDiff, WorkspaceError>;
}
```

SWE-bench, the #848 ladder and self-coding use the same implementation, so a
fix in one is a fix in all. Plan 03 owns it; plan 06 adds prerequisite
recovery inside `run`; the default-deny command policy in
`src/agentic_coding/shell_command_policy.rs` is reused, not duplicated.

### 4.5 Status — one generated table

Status of every requirement, benchmark and gate is generated from
`data/meta` ledgers and the evidence records, by one script, into one file
that VISION, ROADMAP and REQUIREMENTS link to instead of restating. Plan 11
owns the audit and the script; plans 02, 03, 08 write ledger rows; the doc
pin tests shrink to the count #1089 asks for.

## 5. Dependency graph between plans

```
01 lookup ──► 04 formalization ──► 02 composition ──► 05 evidence ──► 06 prerequisites ──► 03 workspace
                                                        │                                     │
                                                        └──► 08 verifiable routing ◄──────────┘
                                                        └──► 07 learning loops (needs evidence + workspace)
09 handler ratchet, 10 routing, 12 heuristics: run alongside, each leaf consumes 01/04 where it needs meaning
11 docs, 13 carry-over: inputs first (scope), outputs last (numbers)
```

Reading the arrows: nothing in 02 may add a seeded idiom that 01 and 04 could
have retrieved; nothing in 03 may report a diff that 05 has no evidence for;
nothing in 07 may learn from a run that produced no evidence; nothing in 08
may score a suite before the pipeline it measures exists.

## 6. Invariants every plan must keep

1. **No benchmark identifier, expected value, or per-task body in production
   data or code.** The no-memorization gate from plan 03 L11 of #710 stays and
   widens to new directories as plans add them.
2. **Every new capability lands as seed data plus a registry method, never as
   a `try_*` arm.** The B9 ratchet counts must not rise in any commit.
3. **Every held-out proof is in en, ru, hi, zh and es**, with prompts that
   share no seeded surface form with the training case.
4. **Every number in a document is a ledger row.** A plan may say "measure
   and record whatever it is"; it may not promise a score.
5. **Forget-and-rediscover is the acceptance test for anything retrieved**:
   delete by content id, rediscover online, replay offline, same id.
6. **Honest terminal states are answers.** `unsatisfiable`, `NotFound`,
   `command not found after recovery` are reported to the user with the
   sources consulted, never converted into completion prose.
7. **Gates only tighten.** No ceiling is raised, no floor lowered, no policy
   loosened to get green; a red gate is a leaf to solve.

## 7. What "in sync in one pull request" means operationally

- Plan 14 lists every leaf of plans 01–12 in one order that respects §5. Each
  leaf names the test that proves it and the gate it must keep green.
- A leaf is committed alone. The commit message names the plan and leaf. The
  plan's box is ticked in the same commit.
- Contract changes (this plan, §4) are their own leaves and land before any
  consumer.
- Documentation replacements from plan 11 land in the last leaves, after the
  ledger rows they cite exist.
- The PR description is regenerated from plans 13 and 14 (closes list, leaf
  status) at each push, so it never states a status the tree does not have.

## 8. Reconciliation leaf

Plans 01–12 were authored in parallel against this plan's §4. Before
implementation starts:

- [ ] Every trait, struct, `.lino` record and file path proposed in plans
      01–12 is checked against §4; duplicates are renamed to the contract
      name and the plan is amended in place with the reason.
- [ ] Every plan's "Issues addressed" list is cross-checked with plan 13's
      coverage table; a mismatch is fixed in both.
- [ ] Every plan's "Docs to update" list is merged into plan 11's findings
      table; the plan keeps only a pointer.
- [ ] Plan 14 is generated from the leaves of plans 01–12 in the §5 order.
