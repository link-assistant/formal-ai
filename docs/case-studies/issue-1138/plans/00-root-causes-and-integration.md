# Plan 00 — Root causes and integration (all bottlenecks of #1138)

This plan answers one question the twelve bottleneck plans cannot answer
individually: why the same shape of gap keeps reappearing, and what the twelve
plans must share so that one pull request lands them in sync. Plans 01–12 hold
the per-bottleneck root causes, options and decisions; this plan holds the
root cause they have in common, the dependency graph between them, and the
contracts each of them must implement against.

## 1. Evidence summary (measured on `main` at `be8fd3174`, 2026-09-16; corrected 2026-09-16 against plans 01-13)

Rows marked **corrected** replaced a figure issue #1138 quoted from a stale
document with the value a plan measured in this worktree. Every correction names
the plan that measured it and the file the measurement was read from, so the
correction is auditable rather than asserted.

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
| **corrected** — the issue's "37 handler files / 48 entries" is `ROADMAP.md:362`'s pre-#1126 figure. Measured in this worktree: 58 precedence rows + 5 unledgered prelude methods; 16 migrated, 2 justified-native, **40 pending**; **39** `try_*` dispatch entries; handler files counted three incompatible ways (36 / 42 / 45); **19** hard-coded promotion predicates (the issue said 18); 1,286 allowlisted prose literals; 548 literal predicates against a stale ceiling of 549; the solver never reads the doublets store | plan 09 §Current state, run of `scripts/check-debt-ratchet.rs` and `scripts/check-minimal-core-boundary.rs`; `src/intent_formalization/prompt_relevants.rs:29-158`; #959 | B9 |
| #745 and #758 re-measured still-broken by the maintainer on 2026-07-25 at v0.303.0 | maintainer comment on #710 | B10 |
| **corrected** — the issue's "ladder 8/24" is that same 2026-07-25 number. The committed baseline is **24/24** (L1 3/3, L2 6/6, L3 7/7, L4 8/8); it runs only in `.github/workflows/task-ladder.yml`, covers en and ru only, and is 24 nodes rather than the 10-20 paraphrases per intent #745 asks for | `experiments/issue_840_task_ladder/results.json`; plan 10 §Current state | B10 |
| **corrected** — the issue's "no local-search counterpart" is stale: **11** `local_path_*` roles exist, with `data/seed/meanings-local-search.lino` (427 lines), `src/agentic_coding/local_search.rs` (709 lines) and a 56-case suite. What is still true is the cause: routing is decided by surface tokens, and the counterpart was added as another token family rather than as a structural decision | `src/seed/roles/intent.rs:455-475`; plan 10 §Current state | B10 |
| Traceability has no rows for R710-R*, R873, R919, R922, R924; ROADMAP still says 0/20; R67 describes a removed defect | `docs/requirements-traceability.md`; `ROADMAP.md:492,570`; `REQUIREMENTS.md:114` | B11 |
| **corrected** — the issue's "least action (#491) has no code" is wrong: `rank_passing_drafts` implements least-action ranking today and `data/meta/draft-portfolio-recipe.lino:61` declares its key order. It is **dormant**: it runs only inside `run_portfolio`, whose gate `SolverConfig::draft_count` defaults to 1. TRIZ (#901), 2-4-6 (#802) and the binary-splitting constraint (#453) genuinely have no code | `src/draft_portfolio.rs:332-346`, `src/solver.rs:183`; plan 12 §Current state | B12 |
| **corrected** — `data/benchmarks/equation-type-corpus.lino` records **10** `benchmark_limitation` rows, not the seven `docs/benchmarks.md:144-156` lists; the three the doc omits are the named-unknown routing gaps (`What is x if …`, `Calculate x for …`, `Find x: …`), and the last is a routing bug, not a math one | `data/benchmarks/equation-type-corpus.lino:891-980`; plan 08 §Current state; carry-over C63 | B8, B11 |
| **corrected** — the issue's R710-R8 citation for "end-to-end automatic source-cache reconstruction remains open" is wrong; that clause is **R710-R7**. R710-R8 is the Agent-CLI meaningful-work row | `docs/requirements/issue-0710-repository-and-retention-continuation.md:16-17`; plan 07 §Issues addressed | B7 |
| **corrected** — the issue's "self-hosting share 0.15 % (58 of 38,373)" is **prose, not a ledger row**. It occurs exactly once in the tree, as the output of a local `scripts/self-hosting-metric.rs` run over `origin/main..HEAD` for PR #888, and it predates the failed strict measurement recorded two files later. The newest ledger row is `tag "v0.350.0"`, `percentage_basis_points "171"`, `trailing_percentage_basis_points "389"`. No plan may cite 0.15 % as a ledger value | `docs/case-studies/issue-710/plans/04-…md:460-461` versus `data/meta/self-hosting-ledger.lino:1214-1230`; plans 03 and 06 §Risks | B3, B11 |

## 2. Root causes — the one every bottleneck shares

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

### Decision

**Option C.** Option A rejected because it recreates the root
cause. Option B rejected because the transition violates the doctrine and
would leave the branch red for the duration.

## 4. The five shared contracts

Names below are the contracts plans 01-12 implement against, **after** the
reconciliation of 2026-09-16 (section 9). Each contract names the plan that owns
its single definition and the plans that consume it. Where a plan proposed a
different name, section 9 records the rename and the plan is amended in place;
there is now exactly one of each name in the plan set.

| Contract | Rust home | Owner | Consumers |
| --- | --- | --- | --- |
| 4.1 `Need`, `NeedKind`, `NeedState` | `src/needs.rs` | **plan 00 leaf C1** (lands before plan 01) | 01, 03, 04, 05, 06, 08, 09 |
| 4.2 `SourceLookup`, `LookupBounds`, `LookupOutcome` | `src/source_walk.rs` + `src/concept_lookup.rs` | **plan 01** | 02, 04, 06, 08, 09, 10, 12 |
| 4.3 `Evidence`, `ObservationKind`, `EvidenceSource` | `src/execution_evidence.rs` | **plan 05** | 02, 03, 06, 07, 08, 12 |
| 4.4 `Workspace`, `WorkspaceSpec`, `Location`, `RunCommand`, `UnifiedDiff` | `src/repository_workspace/` | **plan 03** | 06 (recovery inside `run`), 08, 13 |
| 4.5 Status | `scripts/render-status.rs` into `docs/status.md` | **plan 11** | 02, 03, 08, 09, 10, 12 write ledger rows |

### 4.1 Need — the "I lack X" record

A need is the unit that connects every step. It is a link in the associative
store, not a Rust-only struct, so it can be remembered, forgotten and
rediscovered.

```
need <id>
  kind      concept | procedure | part | prerequisite | evidence | decision
  subject   "<surface text as written by the user or the failing tool>"
  language  en | ru | hi | zh | es | ...
  raised_by <obligation id | need id | tool result id>
  state     open | planned | satisfied | unsatisfiable
  satisfied_by <evidence id>          # only when state = satisfied
```

Its single Rust definition, **owned by this plan as leaf C1** so that both plan
01 (which needs `NeedKind` as the registry's selection axis) and plan 04 (which
needs `Need` as the formalizer's output) consume one type rather than declaring
two:

```rust
// src/needs.rs -- the contract module. No behaviour, only the record.

/// What a need lacks, and therefore which sources may answer it.
/// One enum serves the registry (`SourceRecord::need_kinds`), the need record
/// and the lookup.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum NeedKind {
    Concept, Procedure, Part, Prerequisite, Evidence, Decision,
    #[default] None,
}

impl NeedKind {
    #[must_use] pub const fn slug(self) -> &'static str;
    #[must_use] pub fn from_seed(value: &str) -> Self;
}

/// The need lifecycle. This is the single need-status vocabulary in the tree;
/// `meta_frame::NeedStatus` maps onto it (Pending->Open, Planned->Planned,
/// Satisfied->Satisfied, Blocked->Unsatisfiable) and its producerless
/// `Deferred` / `Rejected` variants are removed with their own test.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NeedState { Open, Planned, Satisfied, Unsatisfiable }

/// One thing the system does not know.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Need {
    pub need_id: String,
    pub kind: NeedKind,
    pub subject: String,
    pub language: String,
    pub raised_by: String,
    /// "<doc_id>@<start>:<end>", exact byte span in the source text.
    pub source_span: String,
    pub depth: usize,
    pub state: NeedState,
    /// `Evidence::evidence_id` (4.3). Set only by an observation.
    pub satisfied_by: Option<String>,
}

impl Need {
    #[must_use] pub fn to_links_notation(&self) -> String;
}
```

Rules: a need is `planned` when a method is selected and `satisfied` only when
an `Evidence` record (4.3) references it. `unsatisfiable` is an honest terminal
state that the answer must report; it is never silently dropped (this is the B5
fix). A need of kind `prerequisite` is raised by a tool failure such as
`command not found` (B6) and is solved by the same lookup and formalization path
as a `concept` need (B1, B4).

**What each consumer adds beside the record, never inside it.** Plan 04 owns
`NeedOrigin` (`UnresolvedSurface` / `UnresolvedRelation` / `UnresolvedProcedure`
/ `RecursiveGloss`) and the retrieved `Vec<ConceptSense>`, carried in
`src/formalization/needs.rs` alongside the ledger row rather than inside it, so
`recipe_interpreter`'s event-for-event parity obligation (R343) is untouched.
Plan 06 owns `PrerequisiteNeed`, the in-memory projection of a `Need` whose
`kind` is `Prerequisite`. `src/coding/concept_discovery.rs::ConceptNeed` becomes
a re-export of `Need`; there is one definition.

### 4.2 Source lookup — one client over the sources registry

```rust
// src/source_walk.rs -- the trait and its bounds.
pub trait SourceLookup {
    /// Resolve a need against the sources registry, in registry order for the
    /// need's kind, honoring settings opt-outs and licenses. Returns evidence
    /// or an honest `NotFound` with the sources consulted.
    fn lookup(&mut self, need: &Need, bounds: &LookupBounds) -> LookupOutcome;
}

/// Declared bounds. Depth and evidence bounds, never a time or token budget.
/// `how_to_guide::GuideBounds` becomes an alias of this type and is deleted by
/// plan 01 L2; no plan may name `GuideBounds` in a new signature.
pub struct LookupBounds {
    pub max_depth: usize,
    pub max_pages_per_service: usize,
    pub max_services: usize,
    pub max_items: usize,
    pub max_capture_age_seconds: u64,
}

pub enum LookupOutcome {
    Found(Vec<ConceptSense>),
    NotFound { consulted: Vec<WalkSourceOutcome> },
}
```

- **Exactly one implementation in the tree**: `RegistrySourceLookup`
  (`src/concept_lookup.rs`, plan 01), backed by the existing
  `CachedSourceClient` and `CurlSourceTransport`, extended to every source in
  `data/seed/sources-registry.lino`. `RegistryConceptLookup` is a thin adapter
  over it that keeps `UnknownConceptLookup` compiling until plan 01 L18 deletes
  the trait. **No other plan may add a second implementation**: plan 02's
  proposed `src/coding/source_lookup.rs` is withdrawn (section 9, R3) and plan
  09's M2 family is renamed `retrieval_method` so a family interpreter is never
  confused with the contract trait (section 9, R4).
- The content-addressed cache is the only persistence; offline replay of a
  cached lookup is byte-identical and recorded as `cache_hit`. `forget` deletes
  by content id; `rediscover` re-fetches and must reproduce the id.
- The universal loop (`src/solver.rs:874-886`) calls this where it now logs
  `policy:no_fetch_capability`; **plan 01 L11 owns that deletion** and every
  other plan cites it rather than repeating it.
- **One selection axis.** Registry order and trust are data: the registry row's
  `kind`, `source_tier` and `primacy` fields already exist; plan 01 adds
  `need_kinds`, `extractor` and `api_language`. Plan 02's proposed `coding_role`
  axis is withdrawn (section 9, R5): a source's usefulness for composition is
  `need_kinds (part procedure)` plus the tier its `primacy` chain already
  derives, so there is one axis, not two.

### 4.3 Evidence — the execution record every satisfaction needs

```
evidence <id>
  for_need    <need id>
  produced_by <method id>
  command     "<exact command or api call>"
  argv        (<split arguments>)
  exit        <integer, http status, or the explicit token `none`>
  output_hash <sha256 of observed output>
  output_len  <bytes observed, so empty is distinguishable from missing>
  source_ids  (<content ids consulted>)
  kind        command_exit | file_bytes | tool_result | symbolic_check
  source      harness | local_process | engine
  recorded_at <utc timestamp>
```

```rust
// src/execution_evidence.rs -- one definition, owned by plan 05.

/// One executed observation, content-addressed and deterministic.
///
/// `evidence_id = stable_id("evidence", command | argv | exit | output_hash |
/// output_len | kind | source)`. No wall clock, pid or machine identity enters
/// the fingerprint, so the id is stable across runs and machines;
/// `recorded_at` is an event-log field carried beside the record and excluded
/// from the id.
pub struct Evidence {
    pub evidence_id: String,
    pub for_need: String,
    pub produced_by: String,
    pub command: String,
    pub argv: Vec<String>,
    /// `None` is honest; it is never rendered as zero.
    pub exit: Option<i64>,
    pub output_hash: String,
    pub output_len: usize,
    pub source_ids: Vec<String>,
    pub kind: ObservationKind,
    pub source: EvidenceSource,
    pub detail: EvidenceDetail,
    pub recorded_at: Option<String>,
}

/// Deterministic, hashable per-kind detail. Raw stdout and stderr are hashed,
/// never stored in the ledger; a caller that must show them receives the
/// non-persisted sibling `ObservedOutput { stdout, stderr }` from the same call.
pub enum EvidenceDetail {
    None,
    Tests { passed: Vec<String>, failed: Vec<String>, timed_out: bool },
}
```

No obligation, need or work unit may reach `satisfied` without one. The
self-hosting metric, the benchmark ledger and the status surface (4.5) read this
record instead of prose. **Plan 05 owns the schema**; plans 02, 03, 06 and 08
produce it; plan 07 consumes it to decide what was learned; plan 12 consumes it
as `ActionCost::resource_units` and as the observation that eliminates a
hypothesis. The name `ExecutionRecord` used by earlier drafts of plans 05, 07
and 12 is withdrawn (section 9, R2).

### 4.4 Workspace — one protocol for any repository

```rust
pub trait Workspace {
    fn open(spec: &WorkspaceSpec) -> Result<Self, WorkspaceError>;   // clone at base commit, or the own repo
    fn locate(&self, need: &Need) -> Vec<Location>;                   // self-AST census for own repo, search otherwise
    fn read(&self, loc: &Location) -> Result<String, WorkspaceError>;
    fn edit(&mut self, change: &Change) -> Result<Evidence, WorkspaceError>;
    fn run(&mut self, cmd: &RunCommand, backend: &ExecutionBackend) -> Evidence;
    fn diff(&self) -> Result<UnifiedDiff, WorkspaceError>;
}
```

SWE-bench, the #848 ladder and self-coding use the same implementation
(`RepositoryWorkspace`), so a fix in one is a fix in all. **Plan 03 owns it.**
Two renames were forced by collisions and are recorded in section 9:

- `RunCommand` (was `Command`, section 9, R6) — `Command` is already taken twice,
  by `std::process::Command` and by clap's `Command` enum in `src/main.rs`, and
  plan 03's own draft used `line` in the struct and `command` at the call site.
- `ExecutionBackend` (was plan 03's `VerifyBackend`, section 9, R7) — **plan 06
  owns** the enum in `src/execution_box/mod.rs`, with variants `HostSandbox`,
  `Box { image }`, `SweBenchImage { instance_id }`,
  `Conversation { conversation_id }` and `BrowserRuntime { runtime }`. Plan 06
  lands before plan 03 (section 5), so plan 03 consumes it rather than declaring
  a second "where does this run" vocabulary.

Plan 06 adds prerequisite recovery **inside** `run`; the default-deny command
policy in `src/agentic_coding/shell_command_policy.rs` and the allowlist in
`src/agent.rs:642-657` are reused, not duplicated, and widen only through
`data/seed/repository-command-allowlist.lino`.

### 4.5 Status — one generated surface

Status of every requirement, benchmark and gate is generated from `data/meta`
ledgers and the `Evidence` records by **one script**, `scripts/render-status.rs`,
into **one file**, `docs/status.md`. VISION, ROADMAP, ARCHITECTURE, GOALS and
the REQUIREMENTS shards link to it instead of restating a number.

- `data/meta/requirement-status-ledger.lino` is the per-requirement ledger
  (`id`, `shard`, `verdict`, `delivered`, `pull_request`, `tracker`,
  `automated_test`, `manual`); `docs/requirements-traceability.md` becomes a
  generated projection of it. **Plan 11 owns both.**
- The only in-place status-begin comment regions are the two that existing
  pin tests require the number to stand in — `docs/benchmarks.md` and
  `README.md` — and they are fed by the same script. This resolves plan 11's own
  open question (its risk 3: a generated region inside hand-written prose is a
  merge-conflict surface) in favour of one file plus two pinned regions, which
  is what this section asked for and what `CONTRIBUTING.md:970-982` requires.
- Plans 02, 03, 08, 09, 10 and 12 write ledger rows; none of them edits a status
  number in prose. The doc pin tests shrink to the count #1089 asks for through
  plan 11 L76.

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
   a `try_*` arm.** The B9 ratchet counts must not rise in any commit. Two
   consequences the reconciliation makes explicit. First, plan 09's leaves 1-5
   repair the ratchet before any plan adds a capability to it, so every later
   measure is added through one strict two-sided checker; the only rises the
   plan set permits are plan 09 leaf 1's two corrected undercounts
   (`handler_files` 42 to 46, `handler_migration_pending` 40 to 45), which land
   in a commit that changes no `src/` behaviour and are labelled
   "corrected undercount" in the ledger's `note`. Second, plan 08's
   `verifiable_task` is a **generic interpreter**, registered as such in
   `data/meta/core-boundary-ledger.lino`, and it takes the dispatch slot
   `pattern_inference` vacates in the same commit (section 9, R12), so
   `try_dispatch_entries` never rises.
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
   loosened to get green; a red gate is a leaf to solve. `store_read_share`
   (plan 09) and the `intents_measured` / `languages_measured` /
   `paraphrases_per_intent_per_language` / `capability_routing_cases_passing`
   measures (plan 10) are the declared exceptions whose strict direction is
   **upward**; each states its direction in its own `how` field, and a value
   that beats its ceiling in either direction fails the checker with
   "lower the reviewed ceiling", the rule
   `scripts/check-minimal-core-boundary.rs:286-290` already implements.

8. **A number in a plan is a measurement or a promise to measure.** No plan may
   state a score it has not run. Where a plan quotes issue #1138's own figures,
   section 1's corrected rows are authoritative; eight of them were wrong when
   the issue was filed.

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

## 8. Reconciliation leaf — executed 2026-09-16

Plans 01-12 were authored in parallel against this plan's section 4. The
reconciliation ran before implementation and is recorded in section 9.

- [x] Every trait, struct, `.lino` record and file path proposed in plans
      01-12 is checked against section 4; duplicates are renamed to the contract
      name and the plan is amended in place with the reason. **19 renames,
      recorded in section 9 as R1-R19; each amended spot carries a one-line
      `reconciled: was X, now Y because Z` note.**
- [x] Every plan's "Issues addressed" list is cross-checked with plan 13's
      coverage table; a mismatch is fixed in both. **42 issue references added to plans 01-12, covering 25
      distinct issues; 5 issues moved out of plan 13's `Closes` list into
      "will not close" with the reason; 2 new leaves added to plan 11 so two
      issues could stay in it.**
- [x] Every plan's "Docs to update" list is merged into plan 11's findings
      table; the plan keeps only a pointer. **120 rows added as D156-D275;
      each plan's "Docs to update" body is now a pointer to its row ids.**
- [x] Plan 14 is generated from the leaves of plans 01-12 in the section 5
      order, grouped into waves T, I1-I7, F and D.

### Contract leaf C1 — the one leaf this plan owns

- [x] **C1.** Add `src/needs.rs` with `NeedKind`, `NeedState`, `Need` and
      `Need::to_links_notation`, and the `need` record shape in
      `data/meta/need-contract.lino`. No behaviour, no caller. Test:
      `tests/unit/specification/needs.rs::the_need_record_round_trips_through_links_notation`
      and `::need_kind_slugs_match_the_seed_vocabulary`. This leaf lands before
      plan 01 L3, because `src/seed/sources.rs::SourceRecord::need_kinds`
      consumes `NeedKind` and plan 04's `src/formalization/needs.rs` consumes
      `Need`; without it the two plans would declare the type twice.

## 9. Reconciliation log (2026-09-16)

Every disagreement found between plans 00-13, the decision taken, and the plans
amended. A row exists for every rename and for every contradiction; nothing was
resolved silently. Both plans in each row carry the one-line
`reconciled: was X, now Y because Z` note at the amended spot.

### 9.1 Contract renames (R1-R19)

| id | was | now | why | plans amended |
| --- | --- | --- | --- | --- |
| R1 | `NeedKind` declared in `src/seed/sources.rs` (plan 01) and `Need`/`NeedKind`/`NeedState` declared in `src/formalization/needs.rs` (plan 04) | one definition in `src/needs.rs`, this plan's leaf C1 | two definitions of the record every step connects through would reproduce the root cause this plan names; plan 01 lands before plan 04, so neither could own it | 00, 01, 04 |
| R2 | `ExecutionRecord` (plans 05, 07, 12) | `Evidence` (section 4.3) | plan 00 fixed the contract name and plans 03 and 06 already used it; three-to-two in favour of `Evidence`, and the contract is authoritative | 00, 03, 05, 06, 07, 12 |
| R3 | `src/coding/source_lookup.rs::RegistryConceptLookup` (plan 02) | withdrawn; plan 02 consumes plan 01's `RegistrySourceLookup` | two implementations of the one trait B1's "fixed means" says must have one | 02 |
| R4 | plan 09's M2 family interpreter named `source_lookup` | `retrieval_method` (`src/retrieval_method.rs`) | a family interpreter named after the contract trait is unreadable; the family *calls* `SourceLookup`, it is not one | 09 |
| R5 | plan 02's registry field `coding_role primary/secondary/none` | withdrawn; `need_kinds` plus the derived `source_tier` | two selection axes over one registry is the defect plan 01 root-cause 5 names; the primary/secondary split is exactly what `PrimacyChain::derive_tier` already computes | 02 |
| R6 | `Command { line, names }` (plan 03) | `RunCommand { line, names }` | `Command` is taken by `std::process::Command` and by clap's enum in `src/main.rs`; plan 03 flagged the collision itself and its own draft used two different field names | 00, 03 |
| R7 | `VerifyBackend` (plan 03) | `ExecutionBackend` (plan 06, `src/execution_box/mod.rs`) | two enums for "where does this run"; plan 06 lands first in the section 5 order and its variant set is the superset | 00, 03, 06 |
| R8 | plan 02's `GuideBounds` in every new signature | `LookupBounds` | plan 01 L2 makes `GuideBounds` an alias and deletes it; a new signature naming the alias would re-open the divergence R991-1 exists to prevent | 02 |
| R9 | plan 02's `how_to_guide::ServicePreferences` in new signatures | `source_walk::ServicePreferences` | the type moves to the shared kernel in plan 01 L2 | 02 |
| R10 | `src/coding/procedure_text.rs` (plan 02) and plan 04's `procedure_from_steps(&[GuideStep])` | `src/procedure_text.rs`, declared by plan 04 L9 and extended by plan 02 L10; `ExtractedProcedure::from_step_records(&[ProcedureStepRecord])` | plan 04 lands before plan 02 in the section 5 order and both need the same "ordered step with provenance" record; the module is not coding-specific | 02, 04 |
| R11 | plans 05/07/12 test path `tests/unit/docs_requirements_issue_1138.rs`; plans 01/03/06 test path `tests/unit/docs_requirements/issue_1138.rs` | `tests/unit/docs_requirements/issue_1138.rs` | one file, in the directory form `docs_requirements/benchmarks.rs` already establishes | 05, 07, 12 |
| R12 | plan 08 registering `verifiable_task` as a 60th handler | `verifiable_task` takes the slot `pattern_inference` vacates; plan 08's leaf order becomes L1-L7, L17', L8, L9- | the B9 ratchet may not rise, and `INTENT_MARKERS` is exactly the recognizer `recognise_verifiable` replaces | 08, 09 |
| R13 | plan 01 `tests/fixtures/issue-1138/` | `tests/fixtures/issue-1138-b1/` | plan 04 already uses `issue-1138-b4/`; an unsuffixed directory would collide with plans 03, 06 and 08 | 01 |
| R14 | plan 08 `VerifiedAnswer::checks: Vec<String>` | `Vec<Evidence>` | a self-check that is a string is not an observation; invariant 6 requires the evidence | 08 |
| R15 | plan 05's open question "what does `SymbolicCheck { check_id }` name?" | `check_id = "<VerifiedAnswer::derivation_id>:<check slug>"`, owned by plan 08 | plan 05 risk 2 asked for this to be settled jointly with plan 12 before either lands; it is settled here | 05, 08, 12 |
| R16 | `MethodRegistry` extended independently by plan 07 (`learned_methods` execution) and plan 12 (`heuristics`) | one struct with three collections; plan 12's declaration is authoritative and cites plan 07 | R344 requires one dispatch authority; two plans growing the same struct without a shared declaration is how two authorities appear | 07, 12 |
| R17 | plan 12 `CandidateScore::checks: (usize, usize)` counted from an unnamed source | counted from satisfied `Evidence` rows | same reason as R14 | 12 |
| R18 | plan 11's status-begin comment regions in five narrative documents | one generated `docs/status.md` plus two pinned regions | section 4.5 asks for one file; plan 11's own risk 3 asks the same question and `CONTRIBUTING.md:970-982` answers it | 11 |
| R19 | plan 04's `ConceptNeed` "moved and left behind as a re-export" and plan 05's two `Obligation` types | `ConceptNeed` re-exports `needs::Need`; `agentic_coding::evidence_record::Obligation` is deleted and `task_obligations::Obligation` becomes a projection of `ObligationNode` | three names for one record | 04, 05 |

### 9.2 Cross-plan conflicts resolved (X1-X14)

| id | conflict | resolution | plans amended |
| --- | --- | --- | --- |
| X1 | Plan 01 L11 and plan 09 leaf 19 both delete `policy:no_fetch_capability` (`src/solver.rs:874-884`) | plan 01 L11 owns the deletion; plan 09 leaf 19 cites it as already done and migrates only the handler bodies | 09 |
| X2 | Plan 02 L14 wires `UnknownConceptLookup` while plan 01 L18 deletes the trait | plan 01 L18 is the last leaf of the whole set and is gated on plans 02 and 04 naming `SourceLookup` directly; plan 02 L14 becomes "take the lookup from the caller as `&mut dyn SourceLookup`" and never constructs a second implementation | 01, 02 |
| X3 | Plan 02 deletes seven algorithm-shaped runtime templates while plan 08's curated industry slice depends on one of them | the deletion (plan 02 L15) is gated on the slice-20 control **and** the 13/13 curated control staying green in the same commit; if either falls the honest response is to record the fall, not to restore the template | 02, 08 |
| X4 | Plan 08 L16 deletes `OBJECT_CATEGORIES`, which the curated 13/13 industry slice currently answers | plan 08's own L15 (retrieved category membership) must land before L16 and the two are verified together; the plan already flagged this as risk 7 and it is now an ordering constraint in plan 14 | 08 |
| X5 | Plan 09 leaf 13 adds a JavaScript `parseHandlerPrecedence` while #952 (which plan 13 lists as fully closed by plan 09) asks for the JS seed parser to be **deleted** in favour of the WASM parser | plan 09 leaf 13 routes the browser's precedence read through the WASM seed parser (`src/web/wasm-worker/`), not through a new JS parser; the leaf is renamed accordingly | 09 |
| X6 | Plans 03, 06, 10 and 12 each add ratchet measures to `data/meta/debt-ratchet.lino`, whose checker is at-or-below rather than strict | plan 09 leaves 1-5 repair the checker first; every other plan's debt measure is added through it afterwards. Capability ratchets (`capability-routing-ratchet`, `selection-heuristic-ratchet`, `adoption-effect-ratchet`, `obligation-evidence-ratchet`, `toolchain-ledger`) keep their own files | 03, 06, 09, 10, 12 |
| X7 | Plan 03 raises `swebench_slice` from 1 to 23 (leaf 16) while invariant 7 forbids raising a gate | a slice is a measurement width, not a ceiling; raising it can only lower the recorded score, and the ratchet groups by `(suite, slice)` so the 0/1 floor is untouched. Plan 03 leaf 16 states this explicitly | 03 |
| X8 | Plan 02 L22 adds a second floor (`full_slice`) per benchmark suite; plan 08 re-measures the same suites at slice 20 | the two floors are independent series and neither may be derived from the other; plan 02 L22 lands before plan 08 L22 so the re-measurement writes into a ledger that already knows about two slices | 02, 08 |
| X9 | Plan 02 and plan 04 both rewrite `docs/meta-algorithm.md:214-218` (the agentic recipe's three pinned constants) | plan 04 L12 owns the rewrite, because it owns `src/agentic_coding/formalization_recipe.rs`; plans 02 and 08 cite plan 11 row D181 rather than restating it | 02, 04, 08 |
| X10 | Plan 05 amends `tests/unit/specification/recipe_interpreter.rs:229-231`, which pins `satisfied "0"`, while invariant 7 forbids weakening a gate | the pin stays and a strictly stronger sibling is added about the *executed* ledger; plan 05 states that this is the one place it touches an existing assertion and that it strictly adds | 05 |
| X11 | Plan 07 flips `SelfImprovementMode` default from `Off` to `Propose`, which changes every trace and can break R343 parity; plan 05 adds recipe step 14, which also changes every trace | the two flips land in separate commits, each with its own parity run; plan 14 orders plan 05's step-14 leaf before plan 07's default flip so a parity failure has one cause | 05, 07 |
| X12 | Plan 09 batch M5 migrates `how_it_works` and `execution_failure` while plan 10 leaf 13 re-renders a previous turn for the non-understanding class through the same family | plan 09 leaf 33 (`dialogue_state_query`) lands before plan 10 leaf 13, which then adds one operation to the family rather than a handler | 09, 10 |
| X13 | Plan 10 deletes `data/seed/intent-routing.lino:400-402` (the memorized `我不明白` rows) while plan 09 batch M1 owns `clarification`, which reads them | plan 10 leaf 13 owns the deletion and lands after plan 09 leaf 36; the `clarification` rule set keeps its five-language role surfaces and loses only the three memorized literals | 09, 10 |
| X14 | Plan 12 changes `WorkUnit::build` to a binary tree, which changes the leaf set `NeedLedger::resolve` matches against, while plan 05 joins obligations to that same leaf set | plan 05's join test (every `WorkUnit` leaf span is covered by an obligation node span) is written before plan 12's leaf and must stay green through it; plan 14 orders plan 05 before plan 12's `WorkUnit` leaf | 05, 12 |

### 9.3 Counts

| measure | value |
| --- | --- |
| Contract renames (R1-R19) | 19 |
| Cross-plan conflicts resolved (X1-X14) | 14 |
| Issue references added to plans 01-12 | 42, covering 25 distinct issues |
| Issues moved out of plan 13's `Closes` list | 5 |
| New leaves added so an issue could stay in the `Closes` list | 3 (plan 09 leaf 42, plan 11 L75, plan 11 L76) |
| Docs rows merged into plan 11 | 120 (D156-D275) |
| Plan 00 evidence rows corrected | 8 |
