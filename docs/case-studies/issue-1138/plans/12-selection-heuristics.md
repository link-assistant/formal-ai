# Plan 12 — Selection heuristics (bottleneck B12 of #1138)

Status: design recorded before implementation. One of the four heuristics is partly
implemented today and three have no code at all; this plan says exactly which is which and
corrects the issue's own summary where it overstates the gap.

## Issues addressed

- **#1138 B12** — "least action (#491), TRIZ (#901), 2-4-6 hypothesis search (#802),
  moonshot task splitting (#453) have no code; these are the selection/splitting heuristics
  of the meta algorithm." This plan delivers all four as heuristic methods the recursive
  core calls.
- **#491 Principle of least action** — "when multiple solutions are generated, we can use
  least action or shortest path/steps to evaluate/score solutions", and "try from the start
  produce for each task split into 2 sub tasks … 1, 2, 4, 8 and so on". Tracked as
  R491-C1..C4 (`docs/requirements/issue-0491-least-action-continuation.md:10-13`), all
  Partial or Open. **This plan delivers R491-C1 and R491-C3 and advances R491-C2;
  R491-C4 ("include user satisfaction and requirement completeness when comparing
  candidate solutions") is recorded Open in the shard as a universal capability
  and this plan does not deliver it, so #491 is not in plan 13's `Closes` list —
  it stays open with that single named remainder (reconciled 2026-09-16; plan 13
  "Issues this PR will NOT close").** **Correction to the issue text: #491 is not
  "zero delivered code".**
  `src/draft_portfolio.rs:332-346` implements least-action ranking today, and
  `data/meta/draft-portfolio-recipe.lino:61` declares
  `least_action_order "cost_size, cost_steps, draft_index"`. What is missing is that it
  reaches exactly one code path, which is off by default. This plan delivers R491-C1 and
  R491-C3.
- **#901 TRIZ** *(this plan delivers the mechanism — contradictions as links with
  a 0-1 selection value, the separation principles as seed data, resolution at
  the ranking seam — but not the 20-task validation corpus #901 also asks for,
  which is named as the follow-up in risk 5; #901 therefore stays open with that
  remainder, reconciled 2026-09-16)* — "each such contradiction or union of different criteria/metrics of trade
  offs are links in our theory. Where we can assign value from 0 to 1 … to actually solve
  the binary contradiction by selecting 50% or 10% or 80%". Zero code:
  `grep -rni "triz" src/ data/` → 0. Delivered here as `ContradictionLink` with a 0–1
  selection value, plus the separation principles as seed data.
- **#802 2-4-6 game** — "generate list of hypothesis, and find shortest possible sequence
  of experiments to narrow the search … Each try should reduce number of possibilities
  ideally in half or less … first try to disprove the hypothesis." Zero code:
  `grep -rni "2-4-6\|wason\|hypothesis space" src/` → 0. The nearest existing thing is
  `src/fact_checking.rs:66` `RefutationStage`, which is statement refutation inside one
  handler, not experiment selection. Delivered here as `RefutationSearch`.
- **#453 Moonshot tasks** *(this plan delivers the binary-splitting constraint,
  R453-M1 to M3; R453-M4 — "combine all different approaches … for each duplicated
  idea, find the first source of it in the history" — is filed Open with its
  blocker named in risk 7, so #453 stays open with that remainder, reconciled
  2026-09-16)* — "at least we should be able to split each task into 2 parts.
  After that we will have enough data to split them again and again recursively … combine
  all different approaches (while removing duplicates, for each duplicated idea, we need to
  find the first source of it in the history)." Zero code for the binary constraint.
  `data/meta/task-decomposition-invariant.lino` already *declares* it; nothing enforces it.
- **#1073 R1073-5** — "Before leaning toward any conclusion, the system must actively
  search for refutations … If neither succeeds, the honest output is 'not confirmed and not
  refuted'." The reasoning-standard gate exists (`src/reasoning_standard/mod.rs`, the
  thirteenth recipe step at `data/meta/recursive-core-recipe.lino:100-107`). #802 is the
  *search strategy* that gate is currently checking for and never finds.
- **#559 R344** — the registry is the sole dispatch authority. This plan adds heuristics to
  that one registry rather than beside it, so no second authority appears.
- **#662 / budget search** — `src/solver_search.rs:157` `try_budget_search` is the only
  consumer of the draft portfolio, and it recognizes arithmetic reachability only
  (`parse_search_problem`, `:539`). Its sampling (`random_candidate` `:363`, `breed`
  `:383`) is exactly what refutation-first search replaces.

## Current state

### Least action exists, in one place, off by default

```rust
// src/draft_portfolio.rs:332-346
/// Rank the passing drafts by least action: smallest artifact first, then fewest
/// steps, then the lowest draft index as the final deterministic tie-break
/// (issue #491).
fn rank_passing_drafts<A>(drafts: &[DraftEvaluation<A>]) -> Vec<usize> {
    let mut ranked = drafts.iter().filter(|draft| draft.passed())
        .map(|draft| draft.index).collect::<Vec<_>>();
    ranked.sort_by_key(|index| {
        let draft = &drafts[*index];
        (draft.cost_size, draft.cost_steps, draft.index)
    });
    ranked
}
```

with the correctness-first ordering at `:268-279`:

```rust
fn is_better<A>(candidate: &DraftEvaluation<A>, current: &DraftEvaluation<A>) -> bool {
    (std::cmp::Reverse(candidate.passed_tests), candidate.cost_size, candidate.cost_steps)
        < (std::cmp::Reverse(current.passed_tests), current.cost_size, current.cost_steps)
}
```

The cost vocabulary is `DraftArtifact` (`src/draft_portfolio.rs:61-68`): `cost_steps: u32`
(evaluations, expansions) and `cost_size: usize` (rendered length), both documented as
"wall-clock independent so ranking cannot depend on machine speed" (`:56-60`).

Four facts bound it:

1. **It runs only inside `run_portfolio`** (`src/draft_portfolio.rs:194`), whose only two
   leaves are `src/solver_search/portfolio.rs:273` (arithmetic reachability) and
   `src/rule_synthesis_portfolio.rs:190` (two strategies only — `matches!(strategy, "reuse"
   | "rule_derivation")` at `:52-54`).
2. **It is off by default.** `SolverConfig::draft_count` defaults to `1`
   (`src/solver.rs:183`), and `try_portfolio_rule` bails at `:197` when
   `draft_count <= 1`. `src/solver.rs:439-446` documents this as "At the default
   `draft_count` of 1 this is a no-op".
3. **Nothing else ranks by cost.** `src/algorithm_discovery.rs:586-605` uses a completely
   different, independent heuristic — sort then drop subsumed candidates, with
   `fn subsumes(longer, shorter)` at `:900`. `src/solver_search.rs:334` scores by absolute
   distance to a target. `src/meta_frame.rs` has no scoring at all.
4. **There is no shared cost type.** `cost_size` + `cost_steps` in `DraftArtifact` is the
   only cross-domain cost vocabulary in the tree; R491-C3's "elapsed time, computational
   work and memory" has no representation anywhere
   (`docs/requirements/issue-0491-least-action-continuation.md:12`: "A shared measured-resource
   optimizer across all reasoning and execution paths remains open").

The `.lino` extension point already exists —
`data/meta/draft-portfolio-recipe.lino` carries a `record_type "meta_selector"` record with
`passing_gate`, `least_action_order`, `composition_backtrack` and `ordered_merge`. It is a
single record describing a single ordering.

### TRIZ: nothing, and three unrelated things named "contradiction"

`grep -rni "triz\|separation principle\|technical contradiction" src/ data/` → 0. The word
`contradiction` appears in three unrelated senses, none of them TRIZ:

- `src/requirement_contradiction.rs:11` — `RESPONSE_INTENT: &str = "requirement_contradiction"`,
  reporting a conflict between a prompt and a retained requirement.
- `src/relative_meta_logic.rs:352` — `pub contradiction: TruthValue`, a probabilistic
  evidence-against term.
- `src/how_to_guide.rs:211` — two sources disagreeing about a step, resolved by source tier.

`ROADMAP.md:303-304` mentions "contradiction warnings (R384)" for #559, again unrelated.
`GOALS.md:34` says "Reduce contradictions by splitting overloaded names into distinct
meanings" — that is meaning disambiguation, not trade-off resolution.

### 2-4-6: nothing, and a refutation vocabulary confined to fact-checking

`src/fact_checking.rs` has the closest analogue:

```rust
// src/fact_checking.rs:66
pub enum RefutationStage { DisproveStatement, DisproveNegation, Decompose, DepthBound }
// :79
pub enum RefutationOutcome { Refuted, Unrefuted, Inconclusive }
// :90
pub struct RefutationAttempt { … }
// :234
max_refutation_depth: u8,   // seeded from config.max_decomposition_depth at :243
```

This is *disprove-first applied to one statement*, which is half of #802. The other half —
maintain a set of live hypotheses and choose the experiment that halves it — does not
exist. `grep -rn -i "hypothesis" src/` returns 21 hits, all `StepKind::Hypothesis` in
`src/proof_engine/` (`types.rs:107`, with the four-language surfaces at `:120-123`): a
proof-step label, not a search space.

The reasoning standard already *checks* for refutation-first behaviour and already reports
the honest default. `data/meta/recursive-core-recipe.lino:105` says the verdict "defaults
to not_confirmed_not_refuted with its blockers named rather than to a lean". So the gate
is installed and there is no search behind it.

### Binary splitting: declared as an invariant, enforced by nothing

`data/meta/task-decomposition-invariant.lino` states it plainly:

```
task_decomposition_contract
  record_type meta_invariant
  atomic "require an observable completion contract and no pending children"
  binary "every non-leaf task is recursively split into exactly two children; 1, 2, 4, 8, 16, 32 and so on leaves are valid complete layers"
```

Neither splitter obeys it:

- `src/task_decomposition.rs:447` `split_once_checkable` → `:451-462`
  `split_once_checkable_with_ledger` returns `merged` whenever `merged.len() >= 2`. Three
  clauses produce three children.
- `src/meta_frame.rs:456` `decompose_once` returns `split_sentences(span)` when there is
  more than one sentence, else `split_clauses(span)` — n-ary in both branches.
  `WorkUnit::build` (`:320-344`) maps over every segment.

So the repository's own meta-invariant contradicts its two splitters, and
`tests/unit/specification/task_decomposition.rs` does not catch it because nothing asserts
arity. R491-C1's status is honest about this
(`docs/requirements/issue-0491-least-action-continuation.md:10`: "Partial: task-decomposition
and failure-driven recursive-execution tests cover binary structure and atomic leaves.
Complete decomposition of arbitrary natural-language obligations remains open").

The failure-driven side has the hook and no implementation:

```rust
// src/recursive_execution.rs:111
fn split(&mut self, _task: &RecursiveTask, _failure: &TaskAttempt, _split_depth: u8)
    -> Vec<RecursiveTask> { Vec::new() }
// :216
pub const DEFAULT_SPLIT_DEPTH_BOUND: u8 = 4;
```

`SplittingExecutor` (`src/task_decomposition/recursive.rs:55`) fills it by delegating to
`split_once_checkable`, "one level per split, deliberately" (`:1-13`) — and therefore
n-ary too.

### Trace of one concrete choice today

Prompt: *"Combine the numbers 3, 7 and 8 with plus and times to reach 59."*

1. `src/solver.rs` step 7 → `solver_search::try_budget_search` (`:157`).
2. `parse_search_problem` (`:539`) recognizes it; `MAX_OPERANDS = 6` (`:594`) admits it.
3. `config.draft_count` is `1`, so `:172-181` runs the single-draft path:
   `run_search(seed_from_prompt(prompt), …, config.compute_budget)` with budget `512`
   (`src/solver.rs:182`), half random (`random_budget = budget.div_ceil(2)`, `:236`), half
   evolutionary over a population of `POPULATION = 8` (`:305`).
4. Candidates are scored by `|value − target|` (`src/solver_search.rs:334-338`) and kept by
   `insert_population` sorting on that diff (`:346`).
5. **No selection heuristic runs at all.** `rank_passing_drafts` is never reached,
   `least_action` never appears in the trace, no contradiction is named, no hypothesis is
   refuted, and nothing was split.
6. If the prompt had instead been *"Write a strong AI"* (#453's third example), the path is
   `decompose_once` → one sentence, one clause → `AtomicityReason::SingleNeed` with no route
   → `NeedStatus::Blocked`. The moonshot is declared atomic and blocked, which is the exact
   opposite of "nothing is a hard task; split it in two".

## Root causes

1. **The one implemented heuristic is a private function inside one search.**
   `rank_passing_drafts` is `fn`, not `pub fn` (`src/draft_portfolio.rs:335`), reachable
   only through `run_portfolio`. *Mechanism:* #704 delivered selection *for the portfolio*,
   and the portfolio is the only thing that got it.
2. **The portfolio is off by default, so even that path is dormant.**
   `draft_count: 1` (`src/solver.rs:183`). *Mechanism:* the knob defaults to the cheapest
   setting, and nothing raises it.
3. **There is no cost type shared by the things that choose.** `DraftArtifact.cost_*`
   (`src/draft_portfolio.rs:61`), `solver_search::score` (`:334`) and
   `algorithm_discovery`'s subsumption sort (`:586-605`) are three unrelated notions of
   "better". *Mechanism:* each was written for its own module, none for the meta algorithm.
4. **A trade-off between two criteria has nowhere to be recorded.** The system can say a
   candidate is smaller or has fewer steps, but cannot say "this one is more complete and
   that one is shorter, and the requirement asks for 70 % completeness". *Mechanism:* every
   existing ranking is a total order over one lexicographic key; a contradiction is not a
   key, it is a pair of keys with a weight.
5. **Search is generate-and-test, never eliminate.** `random_candidate` (`:363`) and
   `breed` (`:383`) sample; `score` measures closeness. Nothing tracks what has been *ruled
   out*. *Mechanism:* the evolutionary shape was taken from the budget-search issue (#662),
   which is a confirmation search by construction.
6. **The binary invariant is data with no reader.**
   `data/meta/task-decomposition-invariant.lino`'s `binary` field names
   `source_reader "src/intent_formalization/requirements.rs"`, which reads the
   `source_integrity` field — not the `binary` one. *Mechanism:* the invariant was written
   as documentation and the grounding suite checks the file exists, not that each field is
   enforced.
7. **`TaskExecutor::split` ships a `Vec::new()` default** (`src/recursive_execution.rs:111`),
   so a task that fails with no children silently stops splitting unless someone wraps it in
   `SplittingExecutor`. *Mechanism:* a default that means "do nothing" is indistinguishable
   from "cannot split", and only the latter is honest.
8. **Nothing chooses among *discovered* algorithms, because discovery produces candidates
   nobody ranks.** `AlgorithmDiscoveryRun::validated_candidates` (`src/algorithm_discovery.rs:421`)
   is consulted only by `method_learning` and dreaming; there is no seam where the core says
   "I have three algorithms for this leaf, pick one." *Mechanism:* discovery was built to
   feed learning, not to feed solving.

## Solution options

### Option A — Hard-code each heuristic at its own call site

*Description.* Add least-action ranking to `algorithm_discovery`, a TRIZ tie-break to
`draft_portfolio`, a hypothesis loop to `solver_search`, and a binary fold to
`task_decomposition` — four independent edits.

*Pros.* Smallest diff per heuristic; no new abstraction; each lands independently.

*Cons.* Four more places where "how we choose" is a Rust branch, which is exactly the
memoization the doctrine forbids and exactly what #959's handler ratchet is trying to
reverse. A fifth heuristic would need a fifth edit. Nothing is discoverable, nothing is
forgettable, and the trace cannot say *which* heuristic decided.

*Doctrine fit.* Fails "associative stack only (seed data in `.lino`, registry methods) over
new specialized Rust handlers."

*Effort.* Small. *Risk.* Low today, high as debt.

### Option B — Heuristics as records in the one method registry, called at named seams

*Description.* `MethodRegistry` gains a third collection, `heuristics`, loaded from
`data/meta/selection-heuristics.lino` the way `learned_methods` is loaded from
`data/seed/learned-methods.lino`. Three roles — rank a candidate set, choose the next
experiment, split a task in two — each with a Rust trait and a seeded, ordered table of
implementations. The recursive core calls them at four named seams; the trace records which
heuristic ran and why it won.

*Architecture sketch.*

```
data/meta/selection-heuristics.lino
        │  (role, slug, order, applies_when, parameters)
        ▼
MethodRegistry { methods, learned_methods, heuristics }   ← one authority (R344)
        │
        ├─ role rank      → CandidateRanker   → draft_portfolio::rank_passing_drafts
        │                                     → algorithm_discovery candidate order
        ├─ role experiment→ ExperimentChooser → solver_search step 7
        └─ role split     → TaskSplitter      → meta_frame::WorkUnit::build
                                              → recursive_execution::TaskExecutor::split
```

*Pros.* One authority, so R344 stays true. Adding a heuristic is a `.lino` edit plus one
impl, and reordering them is a pure data edit. The trace names the deciding heuristic, so
"why did you pick that solution?" has a real answer (which `PortfolioSelection`'s
`comparison_artifact`, `src/draft_portfolio.rs:100-106`, already tries to give for one
path). TRIZ principles and separation axes become seed data discoverable from Wikipedia,
so they are forgettable and rediscoverable. Behaviour-preserving by construction: the
seeded `least_action` ranker reproduces `(cost_size, cost_steps, index)` exactly.

*Cons.* Three traits, not one — rank, experiment and split are genuinely different
operations, and pretending otherwise would produce a trait nobody can implement. Touches
`MethodRegistry`, which is pinned by R331 and by
`tests/unit/docs_requirements_issue_559.rs`.

*Doctrine fit.* Strong on every clause.

*Effort.* Large but incremental — the four heuristics land one at a time behind the same
registry.

*Risk.* Medium: changing how candidates are ordered changes answers wherever ordering was
load-bearing.

### Option C — One `SelectionHeuristic` trait over an abstract "choice"

*Description.* A single trait `fn choose(&self, situation: &Situation) -> Decision` with
`Situation`/`Decision` as sum types covering ranking, experiments and splitting.

*Pros.* One registry table, one dispatch, one event kind.

*Cons.* `Decision` becomes an enum every implementation must match exhaustively and mostly
return `Unsupported` from; the type says nothing about what a heuristic can actually do, so
the compiler stops helping. A ranker asked to split would have to fail at runtime where
Option B fails at compile time.

*Doctrine fit.* Neutral. *Effort.* Medium. *Risk.* Medium — a stringly-typed core.

### Option D — Push all four into the `.lino` recipe layer with no Rust traits

*Description.* Express each heuristic entirely as `meta_selector` records and let
`recipe_interpreter` execute them, the way `data/meta/draft-portfolio-recipe.lino:61`
already expresses `least_action_order`.

*Pros.* Maximum doctrine purity — no new Rust at all. Perfectly forgettable.

*Cons.* `least_action_order "cost_size, cost_steps, draft_index"` works as data because it
is a *field list over an existing struct*. Halving a hypothesis space and folding an n-ary
split into a balanced binary tree are computations, not field lists. Expressing them in
`.lino` means writing an interpreter for a general expression language, which is a much
larger project than this bottleneck and is not what `recipe_interpreter` is
(`src/recipe_interpreter.rs:269-354` dispatches recorder *names*, it does not evaluate).

*Doctrine fit.* Ideal in principle, undeliverable in this increment.

*Effort.* Very large. *Risk.* Very high.

## Decision

**Option B is selected**, with Option D's ordering discipline adopted where it genuinely
fits: every heuristic's *parameters and precedence* are `.lino` data, and only its
*computation* is Rust. Option A is rejected because four new Rust decision sites is the
handler pattern #959 is ratcheting down. Option C is rejected because a single
`choose` trait would make every heuristic return `Unsupported` for two of three roles,
moving a compile-time guarantee to runtime for no gain. Option D is rejected as the right
end state at the wrong increment: halving a hypothesis space is not a field list, and
building a general `.lino` evaluator is a larger project than B12.

Ordering within the plan: **least action first** (it makes the existing implementation
general and is behaviour-preserving), **binary splitting second** (it makes the declared
invariant true and unblocks moonshots), **refutation-first search third** (it needs the
ranker), **TRIZ last** (it needs a candidate set with more than one criterion to trade off,
which only exists after the first three).

## Architecture

### New module `src/selection_heuristics.rs`

Names verified free (`SelectionHeuristic`, `HeuristicRegistry`, `ActionCost`,
`CandidateScore`, `ContradictionLink`, `SeparationAxis`, `RefutationSearch`,
`HypothesisSpace`, `BinarySplit`, `balanced_split`, `selection_heuristics` → 0 hits each).
Deliberately avoided: `Candidate` (`src/solver_search.rs:79`), `AlgorithmCandidate`
(`src/algorithm_discovery.rs:128`), `DraftPlan`/`DraftArtifact`/`PortfolioLeaf`
(`src/draft_portfolio.rs:44`, `:61`, `:72`), `Hypothesis` (the `StepKind::Hypothesis`
variant at `src/proof_engine/types.rs:107` makes the bare name confusing —
`SearchHypothesis` is used instead), `RefutationStage`/`RefutationOutcome`/`RefutationAttempt`
(`src/fact_checking.rs:66`, `:79`, `:90`).

```rust
//! The heuristics the recursive core uses to choose among candidates, to choose
//! the next experiment, and to split a task it cannot solve directly
//! (#491, #901, #802, #453).

/// Which of the three selection jobs a heuristic performs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeuristicRole { Rank, Experiment, Split }

impl HeuristicRole {
    #[must_use] pub const fn slug(self) -> &'static str;
    #[must_use] pub fn from_slug(slug: &str) -> Option<Self>;
}

/// One heuristic as it appears in the registry: link data, not a branch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeuristicMethod {
    pub name: String,
    pub role: HeuristicRole,
    /// Declaration order in `data/meta/selection-heuristics.lino` is precedence.
    pub order: usize,
    /// The situation slugs this heuristic applies to; empty means always.
    pub applies_when: Vec<String>,
    /// Role-specific parameters, read as data (e.g. the least-action key order).
    pub parameters: Vec<(String, String)>,
}

impl HeuristicMethod {
    #[must_use] pub fn to_links_notation(&self) -> String;
}
```

`src/method_registry.rs` gains one field and two accessors, and keeps every existing
signature. **This declaration is authoritative for the whole plan set: plan 07
also grows this struct, by making `learned_methods` executable at last
precedence. One registry with three collections keeps R344's single dispatch
authority true; two plans growing it without a shared declaration is how a second
authority appears (plan 00 §9 R16).** (`method_for_route`, `ordered_method_names_for_relevants`, `to_links_notation`
are all pinned by R331 and `tests/unit/docs_requirements_issue_559.rs`):

```rust
pub struct MethodRegistry {
    pub methods: Vec<Method>,
    pub learned_methods: Vec<LearnedMethod>,
    /// Selection heuristics, loaded from `data/meta/selection-heuristics.lino`.
    /// A heuristic is never a route target: `method_for_route` never returns one.
    pub heuristics: Vec<HeuristicMethod>,
}

impl MethodRegistry {
    /// The heuristics for `role`, in declared precedence order, filtered by
    /// `situation`. Empty is a reportable state: the core then falls back to the
    /// deterministic identity ordering and says so in the trace.
    #[must_use]
    pub fn heuristics_for(&self, role: HeuristicRole, situation: &str) -> Vec<&HeuristicMethod>;
}
```

### Least action (#491): score candidate algorithms by steps, code, resources

```rust
/// The deterministic cost of producing or running one candidate.
///
/// Every field is wall-clock independent, because a ranking key that depends on
/// machine speed is not reproducible (`src/draft_portfolio.rs:56-60` already
/// states this for the two fields it has). R491-C3's "elapsed time" is reported
/// beside the ranking, never inside it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ActionCost {
    /// Elementary steps: evaluations, expansions, recorder invocations.
    pub steps: u32,
    /// Rendered size of the artifact, in characters.
    pub code_size: usize,
    /// Deterministic resource units: nodes visited, links touched, bytes read.
    pub resource_units: u64,
    /// Number of leaves the task was split into to reach this candidate (#453).
    pub leaf_count: u32,
}

/// Correctness first, cost second. Incomplete work is never a cheaper solution
/// (`docs/requirements/issue-0491-least-action-continuation.md:4-5`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandidateScore {
    pub candidate_id: String,
    /// Checks satisfied out of checks declared, counted from the `Evidence`
    /// rows plan 08's five self-checks emit for this candidate's
    /// `derivation_id`. `(0, 0)` is not a pass.
    ///
    /// **reconciled: was a bare `(usize, usize)` with no stated source; now
    /// counted from `Evidence`, so "it passed" is an observation rather than a
    /// number a ranker was handed (plan 00 §9 R17, R15).**
    pub checks: (usize, usize),
    pub cost: ActionCost,
}

impl CandidateScore {
    /// Whether every declared check passed and at least one was declared.
    #[must_use] pub const fn satisfies(&self) -> bool;
    #[must_use] pub fn to_links_notation(&self) -> String;
}

/// Rank a candidate set. The key order is read from the heuristic's parameters,
/// so reordering the dimensions is a `.lino` edit.
pub trait CandidateRanker {
    fn slug(&self) -> &'static str;
    /// Indices of `scores`, best first. Only candidates with `satisfies()` may
    /// be ranked; the rest are returned separately by the caller as refuted.
    fn rank(&self, scores: &[CandidateScore], parameters: &[(String, String)]) -> Vec<usize>;
}

/// The seeded default, byte-compatible with today's behaviour.
pub struct LeastActionRanker;
```

`src/draft_portfolio.rs:335` `rank_passing_drafts` is rewritten to build `CandidateScore`s
and delegate to the registry's `Rank` heuristics. With the seeded `least_action` heuristic
whose `key_order` parameter is `"code_size,steps,candidate_index"`, the output is identical
to today's `(cost_size, cost_steps, index)` — that identity is the migration test.

`src/algorithm_discovery.rs:586-605` keeps its subsumption filter (a *correctness* relation:
a longer validated candidate subsumes a shorter one) and hands the survivors to the same
ranker instead of its ad-hoc `sort_by`. This is the seam #1138 B12 names: "choosing among
discovered algorithms."

`.lino` schema, `data/meta/selection-heuristics.lino`:

```
selection_heuristics
  record_type "meta_selector_catalog"
  schema_version 1
heuristic_least_action
  record_type "selection_heuristic"
  role "rank"
  order "1"
  slug "least_action"
  issue "491"
  key_order "code_size,steps,candidate_index"
  correctness_first "true"
  rationale "Shorter reasoning or code must retain the full required behavior and input range (R491-C2); cost is compared only among candidates that already satisfy every declared check."
heuristic_resource_least_action
  record_type "selection_heuristic"
  role "rank"
  order "2"
  slug "least_resources"
  issue "491"
  applies_when "execution_measured"
  key_order "resource_units,steps,code_size,candidate_index"
  rationale "When an execution record supplied measured resource units (plan 05), rank by work actually done before ranking by artifact size."
```

### Balanced 1/2/4/8 splitting (#453, R491-C1)

```rust
/// Split a task into exactly two children, preserving every segment.
pub trait TaskSplitter {
    fn slug(&self) -> &'static str;
    /// `None` when the task cannot be split at all — which is the atomicity
    /// judgement, and is reported rather than silently returning the input.
    fn split(&self, task: &str, parameters: &[(String, String)]) -> Option<BinarySplit>;
}

/// Exactly two children, with the balance actually achieved recorded honestly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BinarySplit {
    pub left: String,
    pub right: String,
    /// Checkable segments on each side, so an unbalanced split is visible.
    pub left_weight: usize,
    pub right_weight: usize,
    /// Byte spans in the original task; the two must be contiguous and cover it.
    pub left_span: (usize, usize),
    pub right_span: (usize, usize),
}

impl BinarySplit {
    /// `|left_weight - right_weight|`. Zero is a perfect halving; the value is
    /// reported, never optimized away by dropping a segment.
    #[must_use] pub const fn imbalance(&self) -> usize;
    #[must_use] pub fn to_links_notation(&self) -> String;
}

/// Fold the n-ary output of `task_decomposition::split_once_checkable` into the
/// balanced binary tree `data/meta/task-decomposition-invariant.lino` declares,
/// by cutting at the boundary that minimizes `imbalance()`.
///
/// Every segment survives on exactly one side, so this is a regrouping, never a
/// discard (R710-R9). A task whose n-ary split has fewer than two checkable
/// segments yields `None`, unchanged from today.
#[must_use]
pub fn balanced_split(task: &str) -> Option<BinarySplit>;
```

Where the core calls it:

- `src/meta_frame.rs:320` — `WorkUnit::build` currently maps over every segment of
  `decompose_once`. It instead asks the registry's `Split` heuristics, so a three-clause
  span becomes a two-child node whose left child holds two clauses. `leaf_count()`
  (`:384`) then lands on 1, 2, 4, 8 for balanced inputs, which is #491's stated goal, and
  `unit_count()` (`:378`) grows by the interior nodes — reported, not hidden.
- `src/recursive_execution.rs:111` — `TaskExecutor::split`'s `Vec::new()` default becomes a
  call into the same heuristic, so a failing task with no children splits instead of
  silently stopping. `DEFAULT_SPLIT_DEPTH_BOUND` (`:216`) is unchanged; no new bound.

The moonshot case (#453's *"Напиши сильный искусственный интеллект"*) stops being a
`SingleNeed` leaf: it has one clause, so `balanced_split` returns `None` at the *text*
level, and the core then asks plan 01's concept lookup for the decomposition approaches
published for that goal, deduplicated to their first historical source — which is #453's
second half and is honestly recorded here as **dependent on plan 01**. Until plan 01 lands,
a single-clause moonshot is reported as `Underivable` (plan 05's obligation vocabulary)
with the named blocker, not as "atomic".

### TRIZ contradictions as links with a 0–1 selection value (#901)

```rust
/// A trade-off between two criteria, as a link with a value on the range.
///
/// #901: "each such contradiction or union of different criteria/metrics of
/// trade offs are links in our theory. Where we can assign value from 0 to 1 …
/// by selecting 50% or 10% or 80%."
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContradictionLink {
    /// `stable_id("contradiction", &format!("{criterion_a}:{criterion_b}:{situation}"))`.
    pub link_id: String,
    /// The two criteria in tension, as meaning slugs (e.g. `answer_completeness`).
    pub criterion_a: String,
    pub criterion_b: String,
    /// Where on the range the requirement itself places the answer, in basis
    /// points of criterion B: `0` is fully A, `10_000` is fully B. Integer, so
    /// the value is exactly comparable and hashable — never a float in an id.
    pub selection_basis_points: u16,
    /// How the value was derived; never guessed.
    pub derivation: ContradictionDerivation,
    pub resolution: TrizResolution,
    /// The requirement spans and source links the value was derived from.
    pub evidence: Vec<String>,
}

/// Where the selection value came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContradictionDerivation {
    /// Counted from the requirement's own clauses demanding each criterion.
    RequirementClauses { a_clauses: usize, b_clauses: usize },
    /// Read from a seeded or learned contradiction record for this situation.
    SeededRecord { record_id: String },
    /// Neither: the contradiction is named and left unresolved, honestly.
    Underivable { reason: String },
}

/// The resolution shapes TRIZ names, seeded as data, not as Rust branches.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrizResolution {
    /// Pick the point on the range (`selection_basis_points`).
    Range,
    /// Satisfy both by separating them along an axis.
    Separation { axis: SeparationAxis },
    /// Apply a named inventive principle from `data/seed/triz-principles.lino`.
    Principle { principle_id: String },
    /// Named and unresolved. Reported; never silently defaulted to 50 %.
    Unresolved { reason: String },
}

/// The four classical separation principles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeparationAxis { InTime, InSpace, OnCondition, BetweenPartAndWhole }

/// Detect the contradictions in a ranked candidate set: two candidates that each
/// win on a different cost dimension are a technical contradiction.
#[must_use]
pub fn contradictions_in(scores: &[CandidateScore], requirement: &str) -> Vec<ContradictionLink>;

/// A ranker that resolves detected contradictions by their selection value
/// instead of by the arbitrary index tie-break.
pub struct TrizRanker;
```

`data/seed/triz-principles.lino` holds the forty inventive principles and the four
separation principles as link data, each with `principle_id`, a canonical name, a
`resolves` criterion pair where one is known, and a `source` pointing at the Wikipedia
article the sources registry already declares. That is what makes it forgettable and
rediscoverable: delete the seed, rediscover it through plan 01's lookup from
`https://en.wikipedia.org/wiki/TRIZ`, replay offline, and the content hash matches.

Where the core calls it: `CandidateRanker` precedence. `least_action` ranks first; when it
reports a tie on its key order *and* `contradictions_in` finds the tied candidates optimize
different dimensions, `TrizRanker` (order 3) breaks the tie by the selection value rather
than by `candidate_index`. The trace records the `ContradictionLink` so "why did you pick
the longer one?" answers with the criterion pair and the basis points.

### Refutation-first 2-4-6 hypothesis search (#802)

```rust
/// One rule under test. `SearchHypothesis`, not `Hypothesis`, because
/// `StepKind::Hypothesis` (`src/proof_engine/types.rs:107`) already owns the word
/// in a different sense.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchHypothesis {
    pub hypothesis_id: String,
    /// The rule in the meta language, e.g. `ascending_by_two`.
    pub rule: String,
    pub alive: bool,
    /// The experiment id that killed it, when it is dead.
    pub refuted_by: Option<String>,
}

/// One probe and what each hypothesis predicts for it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Experiment {
    pub experiment_id: String,
    /// The probe as it will actually be run — a command, a prompt, a fetch.
    pub probe: String,
    /// Hypotheses that predict the probe succeeds.
    pub predicts_yes: Vec<String>,
    /// Hypotheses that predict it fails.
    pub predicts_no: Vec<String>,
}

impl Experiment {
    /// Live hypotheses surviving the worse of the two outcomes. Lower is better:
    /// this is the halving criterion (#802: "Each try should reduce number of
    /// possibilities ideally in half or less").
    #[must_use] pub fn worst_case_survivors(&self) -> usize;

    /// Whether this probe is a *disproof attempt* of the leading hypothesis —
    /// #802's "first try to disprove the hypothesis, and only if that is not
    /// possible find a proof of it".
    #[must_use] pub fn attempts_refutation_of(&self, leading: &str) -> bool;
}

/// The live hypothesis set and the experiments run against it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HypothesisSpace {
    pub space_id: String,
    pub hypotheses: Vec<SearchHypothesis>,
    pub experiments: Vec<Experiment>,
    /// Every observation applied so far, so the elimination is replayable.
    pub observations: Vec<Evidence>,   // plan 05, plan 00 §4.3
}

impl HypothesisSpace {
    #[must_use] pub fn alive(&self) -> Vec<&SearchHypothesis>;

    /// Apply one observation: kill every hypothesis whose prediction it
    /// contradicts, recording which experiment did it.
    pub fn observe(&mut self, experiment_id: &str, record: Evidence) -> usize;

    /// The honest verdict when the space stops shrinking: one survivor is a
    /// conclusion, zero survivors is a refuted frame, and more than one with no
    /// discriminating experiment left is `not_confirmed_not_refuted` with the
    /// survivors named — the same default `data/meta/recursive-core-recipe.lino:105`
    /// already requires of the reasoning standard.
    #[must_use] pub fn verdict(&self) -> SearchVerdict;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SearchVerdict {
    Concluded { hypothesis_id: String },
    AllRefuted,
    NotConfirmedNotRefuted { survivors: Vec<String>, blocker: String },
}

pub trait ExperimentChooser {
    fn slug(&self) -> &'static str;
    /// Pick the next probe: minimize `worst_case_survivors`, and among equals
    /// prefer one that attempts to refute the leading hypothesis.
    fn next_experiment<'a>(&self, space: &'a HypothesisSpace) -> Option<&'a Experiment>;
}

pub struct RefutationSearch;
```

Where the core calls it:

- **Step 7 of the universal loop** (`VISION.md:189`): before the budget search samples
  (`src/solver_search.rs:218` `run_search`), the candidate *rules* become a
  `HypothesisSpace` and `RefutationSearch` picks the probe. Elimination replaces sampling
  where a discriminating probe exists; sampling stays as the fallback when none does.
- **Discovery** (plans 01/02): "which source do I consult next?" is exactly experiment
  selection over competing interpretations.
- **The reasoning standard** (`src/reasoning_standard/mod.rs`) gains a real search behind
  the gate it already enforces for R1073-5, instead of reporting
  `not_confirmed_not_refuted` because nothing was tried.

Each observation is an `Evidence` record from plan 05 (**reconciled: was `ExecutionRecord` — plan 00 §9 R2**), so the elimination is evidence, not
assertion — which is also what makes the whole space replayable and the verdict honest.

### Failure and honesty behaviour

- `heuristics_for(role, situation)` empty → the core uses the deterministic identity
  ordering and emits `heuristic:none` naming the role and the situation. It never silently
  falls back.
- `balanced_split` returning `None` is an atomicity judgement, reported with the reason;
  it is never "the task is easy".
- `BinarySplit::imbalance()` is always reported. A 7-clause task splits 4/3 and says so.
- `ContradictionDerivation::Underivable` and `TrizResolution::Unresolved` name the
  contradiction and refuse to invent a value. There is no default 50 %.
- `SearchVerdict::NotConfirmedNotRefuted` names the survivors and the blocker, matching the
  standard already declared at `data/meta/recursive-core-recipe.lino:105`.
- No heuristic may change a *correctness* judgement. `CandidateScore::satisfies()` is
  computed before any ranker runs, and a ranker that reorders unsatisfying candidates above
  satisfying ones is a test failure.

## Tests first

### Held-out multilingual cases, with the actual prompt text

New file `tests/unit/issue_1138_selection_heuristics.rs`. The split cases are moonshot-shaped
(#453) and must produce exactly two children with a reported imbalance:

- **en**: `"Design an architecture that teaches itself to score steadily between 860 and 864 points in Atari Breakout, write a benchmark for it, and explain how the benchmark is run."`
- **ru**: `"Разработай архитектуру, которая сама научилась бы стабильно выбивать от 860 до 864 очков в Atari Breakout, напиши для неё бенчмарк и объясни, как этот бенчмарк запускается."`
- **hi**: `"एक ऐसी संरचना बनाओ जो खुद सीखकर Atari Breakout में लगातार 860 से 864 अंक ला सके, उसके लिए एक बेंचमार्क लिखो, और बताओ कि वह बेंचमार्क कैसे चलाया जाता है।"`
- **zh**: `"设计一个能够自己学会在 Atari Breakout 中稳定拿到 860 到 864 分的架构，为它写一个基准测试，并说明这个基准测试怎么运行。"`
- **es**: `"Diseña una arquitectura que aprenda por sí sola a sacar de forma estable entre 860 y 864 puntos en Atari Breakout, escribe un banco de pruebas para ella y explica cómo se ejecuta ese banco de pruebas."`

Held-out paraphrases (same three obligations, different wording; asserted to produce the
same two-child shape and the same imbalance):

- **en**: `"I want a self-teaching architecture that reliably lands 860-864 in Breakout. Also produce a benchmark. Also document the way to run it."`
- **ru**: `"Мне нужна самообучающаяся архитектура, надёжно набирающая 860-864 в Breakout. Ещё сделай бенчмарк. И опиши, как его запускать."`
- **hi**: `"मुझे एक स्वयं सीखने वाली संरचना चाहिए जो Breakout में भरोसे से 860-864 लाए। साथ ही एक बेंचमार्क भी बनाओ। और उसे चलाने का तरीका भी लिखो।"`
- **zh**: `"我要一个能自学的架构，在 Breakout 里稳定拿到 860 到 864 分。另外做一个基准测试。再写清楚运行它的方法。"`
- **es**: `"Quiero una arquitectura que aprenda sola y saque de forma fiable 860-864 en Breakout. Haz además un banco de pruebas. Y documenta cómo ejecutarlo."`

The TRIZ cases pose an explicit trade-off in the requirement, so the selection value is
*derived* from clause counts and never guessed:

- **en**: `"Give me the shortest answer that still covers every case, and if you must choose, favour completeness."`
- **ru**: `"Дай самый короткий ответ, который всё же покрывает каждый случай, а если придётся выбирать — отдай предпочтение полноте."`
- **hi**: `"सबसे छोटा उत्तर दो जो फिर भी हर मामले को कवर करे, और अगर चुनना पड़े तो पूर्णता को प्राथमिकता दो।"`
- **zh**: `"给我最短的答案，但仍然要覆盖每一种情况；如果必须取舍，优先保证完整。"`
- **es**: `"Dame la respuesta más corta que aun así cubra todos los casos y, si tienes que elegir, prioriza la exhaustividad."`

### Unit and specification tests

| Path | Name | Asserts |
| --- | --- | --- |
| `tests/unit/specification/selection_heuristics.rs` | `the_seeded_least_action_ranker_reproduces_the_previous_portfolio_order` | migration identity against `(cost_size, cost_steps, index)` |
| | `an_unsatisfying_candidate_never_outranks_a_satisfying_one` | correctness before cost (R491-C2) |
| | `a_zero_zero_check_count_is_not_a_pass` | `satisfies()` requires a declared check |
| | `no_ranking_key_depends_on_wall_clock` | `ActionCost` has no time field; the ranker reads only `ActionCost` |
| | `reordering_the_key_order_is_a_lino_edit_not_a_rust_edit` | swap `key_order` in the seed, observe the new order |
| | `an_empty_heuristic_table_falls_back_deterministically_and_says_so` | the `heuristic:none` event |
| `tests/unit/specification/task_decomposition.rs` (extend) | `every_non_leaf_unit_has_exactly_two_children` | enforces the declared `binary` invariant |
| | `a_binary_split_preserves_every_segment_and_its_byte_spans` | regrouping, never discarding (R710-R9) |
| | `the_imbalance_of_an_odd_split_is_reported_not_hidden` | 7 clauses → 4/3, `imbalance() == 1` |
| | `a_leaf_count_of_a_balanced_task_is_a_power_of_two` | 1, 2, 4, 8 (#491) |
| | `an_unsplittable_task_returns_none_with_a_reason` | never returns the input unchanged |
| `tests/unit/specification/refutation_search.rs` | `the_chosen_experiment_minimizes_worst_case_survivors` | the halving criterion |
| | `a_refuting_probe_is_preferred_over_a_confirming_one_at_equal_power` | disproof-first (#802) |
| | `observing_a_result_kills_every_contradicted_hypothesis_and_names_the_experiment` | |
| | `two_survivors_with_no_discriminating_probe_report_not_confirmed_not_refuted` | matches `data/meta/recursive-core-recipe.lino:105` |
| | `every_elimination_is_backed_by_an_evidence_record` | plan 05 join |
| | `the_same_space_and_seed_produce_the_same_experiment_sequence` | determinism |
| `tests/unit/specification/triz_contradictions.rs` | `a_tie_on_the_least_action_key_with_two_winning_dimensions_is_a_contradiction` | detection |
| | `the_selection_value_is_derived_from_requirement_clauses_not_guessed` | `RequirementClauses { a, b }` |
| | `an_underivable_contradiction_is_named_and_left_unresolved` | no default 50 % |
| | `the_selection_value_is_integer_basis_points_so_ids_stay_hashable` | no float in a content id |
| | `the_forty_principles_are_seed_data_and_survive_forget_and_rediscover` | delete the seed, rediscover, same content hash |
| `tests/unit/issue_1138_selection_heuristics.rs` | `a_moonshot_prompt_splits_into_exactly_two_children` | five languages |
| | `a_held_out_paraphrase_produces_the_same_split_shape_and_imbalance` | five languages |
| | `an_explicit_trade_off_requirement_moves_the_selection_value` | five languages; "favour completeness" ⇒ basis points toward completeness |
| | `a_single_clause_moonshot_is_reported_as_underivable_not_atomic` | the honest #453 limit until plan 01 lands |
| `tests/unit/specification/method_registry.rs` (extend) | `a_heuristic_is_never_returned_by_method_for_route` | heuristics never answer |
| | `the_registry_event_lists_every_heuristic_with_its_role_and_order` | one authority, one event |
| `tests/unit/docs_requirements/issue_1138.rs` (extend; **reconciled: was `tests/unit/docs_requirements_issue_1138.rs` — plan 00 §9 R11**) | `issue_1138_selection_heuristics_are_traceable` | grep-pins `pub struct HeuristicMethod`, `fn balanced_split`, `pub struct ContradictionLink`, `pub struct HypothesisSpace` |

### Gates and ratchets

- **Migration identity gate.** Before any behaviour change, a test asserts the seeded
  `least_action` heuristic reproduces `rank_passing_drafts`'s current output on the existing
  portfolio corpus, byte for byte. Only after that passes may a second ranker be seeded.
- **New ratchet** `data/meta/selection-heuristic-ratchet.lino`:
  `non_binary_work_unit_nodes` — the count of `WorkUnit` nodes with more than two children
  over the benchmark corpus. Measured honestly at its current (large) value, and strictly
  downward every release until it reaches 0. This is the only honest way to enforce the
  declared `binary` invariant across a live corpus without a flag day.
- `data/meta/task-decomposition-invariant.lino` gains a real `source_reader` for its
  `binary` field (it currently names `src/intent_formalization/requirements.rs`, which reads
  `source_integrity`), and `tests/unit/specification/task_decomposition.rs` checks each
  field has a reader that enforces it.
- The existing benchmark floors are untouched. Changing candidate ordering can change
  answers wherever ordering was load-bearing; the coding-modification, industry and
  equation suites are the ratchets that catch it.
- `cargo run --example regenerate_self_ast_census` after the new `src/` module.
- `rust-script scripts/check-hardcoded-language.rs` must not gain an allowlist entry: every
  TRIZ principle name and every criterion slug is seed data.

## Implementation leaves

- [x] Add `src/selection_heuristics.rs` with `HeuristicRole`, `HeuristicMethod`,
      `to_links_notation`; register in `src/lib.rs`.
- [x] Add `data/meta/selection-heuristics.lino` with the `least_action` record only; add
      `heuristics` and `heuristics_for` to `MethodRegistry`; extend its
      `to_links_notation` and the `method_registry` event; assert `method_for_route` never
      returns a heuristic.
- [x] Add `ActionCost`, `CandidateScore`, `satisfies`, `CandidateRanker`,
      `LeastActionRanker`; the key order reads from `parameters`.
- [x] Rewrite `src/draft_portfolio.rs:335` `rank_passing_drafts` over `CandidateScore` and
      the registry; land the migration identity test in the same commit.
- [x] Hand `src/algorithm_discovery.rs:586-605`'s surviving candidates to the same ranker,
      keeping `subsumes` (`:900`) as the correctness filter it is.
- [x] Seed `heuristic_resource_least_action` reading `resource_units` from plan 05's
      `Evidence` records; R491-C3.
- [x] Add `BinarySplit`, `imbalance`, `balanced_split`, `TaskSplitter`; seed the `split`
      role in `data/meta/selection-heuristics.lino`.
- [x] Measure `non_binary_work_unit_nodes` over the benchmark corpus; write
      `data/meta/selection-heuristic-ratchet.lino` at the measured value.
- [x] Switch `src/meta_frame.rs:320` `WorkUnit::build` to the `Split` heuristic; take one
      ratchet step down; re-measure. **This leaf lands after plan 05's obligation
      join test, which asserts every `WorkUnit` leaf span is covered by an
      obligation node span: this leaf changes the leaf set that join matches
      against, and the join must stay green through it (plan 00 §9 X14).**
- [x] Replace `TaskExecutor::split`'s `Vec::new()` default
      (`src/recursive_execution.rs:111`) with the same heuristic.
- [x] Fix `data/meta/task-decomposition-invariant.lino`'s `source_reader` for `binary`;
      add the per-field reader check.
- [x] Add `SearchHypothesis`, `Experiment`, `worst_case_survivors`,
      `attempts_refutation_of`, `HypothesisSpace`, `observe`, `SearchVerdict`,
      `ExperimentChooser`, `RefutationSearch`; seed the `experiment` role.
- [ ] Wire `RefutationSearch` into `src/solver_search.rs` step 7 ahead of `run_search`,
      falling back to sampling when no discriminating probe exists.
- [ ] Wire `HypothesisSpace::verdict` into `src/reasoning_standard/` so R1073-5's gate has
      a search behind it.
- [x] Add `ContradictionLink`, `ContradictionDerivation`, `TrizResolution`,
      `SeparationAxis`, `contradictions_in`, `TrizRanker`; seed the `rank` role at order 3.
- [x] Add `data/seed/triz-principles.lino` (40 inventive + 4 separation principles) with
      `source` links into the existing sources registry; add the forget-and-rediscover test.
- [x] Add `tests/unit/issue_1138_selection_heuristics.rs` (twenty prompts, five languages)
      and the four specification files; register each in `tests/unit/mod.rs`.
- [ ] Write `docs/requirements/issue-0901-triz-contradictions.md`,
      `docs/requirements/issue-0802-hypothesis-search.md`,
      `docs/requirements/issue-0453-moonshot-splitting.md`; regenerate `REQUIREMENTS.md`.
- [ ] Add the `changelog.d/` fragment and the traceability rows.

## Docs to update

The exact quoted statements and their replacement text moved to plan 11's
findings table on 2026-09-16, so there is one docs authority and no document
is described in two places (plan 00 §8). This plan's entries are rows
**D257-D271** of
[`11-docs-consistency-audit.md`](11-docs-consistency-audit.md) §"Issue #1138
plan doc replacements", and plan 11's leaves apply them after the ledger rows
they cite exist (plan 00 §7).

| row | document |
| --- | --- |
| D257 | `docs/requirements/issue-0491-least-action-continuation.md:10` |
| D258 | `docs/requirements/issue-0491-least-action-continuation.md:12` |
| D259 | `data/meta/draft-portfolio-recipe.lino:61` |
| D260 | `data/meta/task-decomposition-invariant.lino` |
| D261 | `docs/meta-algorithm.md:150-151` |
| D262 | `docs/meta-algorithm.md`, new section after the budget-search section (`:790-871`) |
| D263 | `VISION.md:189` |
| D264 | `VISION.md:99-100` |
| D265 | `VISION.md:192` |
| D266 | `ROADMAP.md` |
| D267 | New shard `docs/requirements/issue-0901-triz-contradictions.md` |
| D268 | New shard `docs/requirements/issue-0802-hypothesis-search.md` |
| D269 | New shard `docs/requirements/issue-0453-moonshot-splitting.md` |
| D270 | `docs/requirements-traceability.md` |
| D271 | `docs/requirements/issue-1138-bottleneck-audit.md` |

Any further document this plan's implementation touches is added as a new
plan 11 row, never as a second copy here.

## Risks and open questions

1. **Changing candidate ordering changes answers.** Everywhere ordering was load-bearing
   and untested, a reorder is a silent behaviour change. The mitigation is the migration
   identity gate — land the registry with an ordering byte-identical to today's, then change
   one thing at a time behind the ratchets. There is no way to make this risk zero.
2. **Binary splitting changes the work-unit tree for every prompt.** `unit_count()`
   (`src/meta_frame.rs:378`) grows by the interior nodes, `NeedLedger::resolve`'s
   `best_leaf_for` (`:734`) matches against a different leaf set, and every trace-pinning
   test sees a different tree. This is the single largest behavioural change in the plan and
   is why it is gated by a measured ratchet rather than a flag day.
3. **`ActionCost::resource_units` has no producer until plan 05 lands.** Until `Evidence`
   records carry measured units, the `least_resources` heuristic has nothing to rank by and
   must stay unseeded. Ordering dependency: plan 05 before this leaf.
4. **R491-C3 asks for elapsed time and memory; this plan deliberately excludes wall-clock
   from the key.** That is a defensible reading — a nondeterministic ranking key would break
   R13 — but it is a *reading*, and the shard text must say so explicitly rather than quietly
   satisfying a different requirement than the one written. Flagged for maintainer review.
5. **Deriving a TRIZ selection value from clause counts is crude.** "Favour completeness"
   is one clause; a requirement with five completeness clauses and one brevity clause yields
   8,333 basis points toward completeness, which is arithmetic rather than judgement. It is
   deterministic and inspectable, which is the bar; whether it is *right* is an open
   question that only the 20-task TRIZ corpus #901 asks for can answer. That corpus is not
   in this plan's scope and is named as the follow-up.
6. **#802's halving criterion assumes hypotheses are enumerable.** For a coding task the
   hypothesis space is unbounded, so `RefutationSearch` applies only where a finite
   candidate set already exists (rule synthesis, source selection, algorithm choice). Where
   it does not, sampling stays. This limit must be stated in the shard, not elided.
7. **#453's "first historical source" deduplication is not delivered here.** It needs plan
   01's live lookup and a provenance-tracing capability that does not exist. R453-M4 is filed
   as Open with its blocker named. Claiming it would be exactly the overstatement this issue
   exists to remove.
8. **Three traits mean three registry tables mean three failure modes.** A heuristic seeded
   with the wrong `role` is inert. The registry event lists every heuristic with its role and
   order so an inert one is visible, but nothing stops a typo; a per-role non-empty assertion
   over the seeded catalog is the cheapest guard and is in the test table.
