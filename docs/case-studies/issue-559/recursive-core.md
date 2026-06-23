# Issue 559 Recursive Core

This document specifies the recursive, bidirectional core of the general meta
algorithm. It answers the PR feedback's request that "the core algorithm be fully
recursive" (R19) and run "in both directions at the same time" (R20). It is
grounded in the existing solver flow so the recursion is an *extension* of what
runs today, not a parallel engine.

The vocabulary follows the canonical mapping in [alignment.md](alignment.md):
`ProblemFrame` = formalized impulse, `WorkUnit` = recursively-formalized
sub-impulse, depth/atomicity = `SolverConfig` knobs.

## Where This Plugs Into Today's Loop

The current entry chain is `solve` (`src/solver.rs:373`) →
`solve_with_history` (`:382`) →
`solve_with_history_and_probability_store` (`:396`) →
`solve_with_history_probability_store_and_intent_cache` (`:411`). The real body
(`:411-653`) already does, in this order:

1. formalize and cache an `IntentFormalization` (`:411`+);
2. local search `search:local` (`:478`);
3. shallow decomposition (`:480-483`);
4. write-program rescue (`:525`) before synthesis (`:536`);
5. specialized handlers via `handle_specialized_pattern` (`:551` skips them for
   concrete `WriteProgram`);
6. policy/unknown fallbacks and event emission.

The recursive core generalizes **step 3**. Today `UniversalSolver::decompose`
(R74) only splits conjunctions (`and`, `with tests`, `with benchmarks`) bounded
by `SolverConfig::max_decomposition_depth`. The general version turns that single
shallow split into a bounded recursion of `WorkUnit`s, while steps 2, 4, 5, and 6
become the **leaf solvers** a unit can resolve to. Crucially, the loop *shape*
does not change (honoring `GOALS.md` "the shape of the loop should not branch by
domain"); recursion happens inside the existing decomposition step.

## Data Model (link-native)

Every structure below is a doublet link (`VISION.md:44`) serialized with
`format_lino_record` (R311). A `WorkUnit` links to:

- `parent` — the unit or frame that produced it;
- `children` — sub-units produced by the downward pass;
- `need` — the requirement this unit satisfies (R158);
- `candidates` — methods/skills proposed by the upward pass;
- `selected` — the chosen method/skill (or `none` while open);
- `evidence` — evidence links gathered under the evidence policy (R67);
- `validation` — validation result (satisfied / failed / blocked);
- `composition` — how children's results combine into this unit's result.

A `ProblemFrame` links to the impulse, the detected needs, the root `WorkUnit`,
the evidence policy, and the need-satisfaction ledger.

## Atomicity Predicate

A unit is **atomic** (a recursion leaf) when any of these holds:

1. **Direct method match** — the method registry has an entry whose preconditions
   the unit satisfies and whose validation cost is within policy (Decision 2 in
   [options-comparison.md](options-comparison.md)). This includes the 50
   `SPECIALIZED_HANDLERS`, the five contextual-override handlers, and registered
   skills.
2. **Single library/stdlib call** — the unit maps to one Rust standard-library or
   `link-calculator`/repo function call.
3. **Single reviewed skill** — a skill-registry entry (from `skill_compiler.rs`
   output or a learned, gated rule) applies directly.
4. **Depth bound reached** — `depth >= SolverConfig::max_decomposition_depth`, in
   which case the unit is forced to a leaf and solved by the best available
   method or recorded as `blocked` (never an unbounded loop — `NON-GOALS.md`).

The predicate is configured by a new `SolverConfig` knob, `atomicity_policy`,
added to the config **first** (per `NON-GOALS.md`: "new knobs are added to the
config first"). It selects how aggressively to stop splitting (e.g.
`prefer_compose` stops as soon as the upward pass can satisfy a unit;
`prefer_split` keeps splitting to the depth bound). The default reproduces
today's behavior.

## Downward Pass (decomposition-first)

```
fn solve_unit(unit, frame, cfg, depth):
    emit event work_unit:enter(unit, depth)
    if is_atomic(unit, cfg) or depth >= cfg.max_decomposition_depth:
        return solve_leaf(unit, frame, cfg)          # see "Leaf Resolution"
    needs = detect_needs(unit)                        # R7, R158
    children = []
    for need in needs:
        child = make_work_unit(parent=unit, need=need)
        children.push(child)
    # upward pass may already satisfy some children (see "Upward Pass")
    results = [ solve_unit(c, frame, cfg, depth+1) for c in children ]
    composed = compose(unit, results)                 # combination step
    unit.validation = validate(unit, composed, frame) # TDD/validation step
    emit event work_unit:exit(unit, unit.validation)
    return composed
```

`detect_needs` generalizes today's conjunction split: instead of only splitting
on `and`/`with tests`, it reads the formalized frame's detected requirements
(R158) so one prompt can yield several typed needs (R7). When `detect_needs`
finds a single need equal to the unit itself, the unit is atomic and goes to
`solve_leaf` — this is the base case that prevents infinite recursion even before
the depth bound.

## Upward Pass (construction-first)

The upward pass runs *interleaved* with the downward pass (R20: "both directions
at the same time"). Before splitting a unit, the engine asks the registry and
caches whether the unit can be satisfied by composing existing parts:

```
fn try_construct(unit, frame, cfg):
    parts = registry.candidates_for(unit)            # methods, skills, stdlib, examples
    parts += cache.facts_for(unit)                   # data/cache + memory (R67)
    for combo in feasible_compositions(parts, cfg):  # bounded by cfg
        candidate = compose_candidate(unit, combo)
        if validate(unit, candidate, frame).satisfied:
            unit.selected = combo
            return Some(candidate)                    # short-circuit: no split needed
    return None
```

The combined engine prefers whichever direction satisfies a unit first, governed
by `atomicity_policy` and `recursion_mode` (the comparison knob from Decision 3:
`down` / `up` / `both`). In `both` mode, `try_construct` runs before the downward
split; if it succeeds the unit becomes a leaf, otherwise the downward pass splits
it. This is the meeting-point design: construction handles "a ready composition
exists," decomposition handles "this is novel, break it down."

## Leaf Resolution (mapping to existing solvers)

`solve_leaf` does **not** introduce new behavior; it routes the atomic unit to one
of today's solvers, in the existing precedence (CR7 — order matters):

1. local search (`search:local`, today at `solver.rs:478`);
2. registry-selected specialized method (the generalization of
   `handle_specialized_pattern`, `:655-767`);
3. write-program rescue (`:525`) / synthesis (`:536`) for constructive units;
4. policy/unknown fallback with provenance.

Because leaves reuse the existing solvers, a single-unit (atomic) frame behaves
exactly like today's flow — which is how backward compatibility is preserved
(R13) and proven by widening
`specialized_handlers_still_publish_loop_events`
(`tests/unit/specification/reasoning_loop.rs:44`).

## Need-Satisfaction Ledger

After the root unit returns, the frame's ledger marks every detected need (R8):

- `satisfied` — a unit produced a validated result;
- `deferred` — intentionally postponed (e.g. out of scope this turn);
- `blocked` — no method/evidence available (recorded, not hidden);
- `rejected` — deliberately not done, with a reason.

The answer projection (`ARCHITECTURE.md` §12: "the answer is a projection")
renders satisfied needs and explicitly reports deferred/blocked/rejected ones, so
"address every detected need" (R8) is enforced structurally rather than by
prose.

## Voyager Mapping (deterministic, no neural runtime)

Per R18 / NG5, Voyager is a design reference only. The mapping onto this core:

| Voyager idea | Deterministic formal-ai mechanism |
| --- | --- |
| Automatic curriculum | Coverage backlog over unknown traces, benchmark gaps, dependency gaps, unhandled prompt families (drives which units/skills to build next) |
| Skill library (executable, growing) | Link-native method/skill registry (Decision 6) with applicability, examples, negative examples, validation cost, reuse boundary |
| Execution feedback / errors | `validate(unit, …)` results + tool observations + test outcomes |
| Iterative program improvement | `execute → observe → validate → patch candidate → retry within budget`, bounded by `SolverConfig` (no unbounded loop) |
| Self-verification | A critic method that checks a unit's validation plan and marks it satisfied / blocked / needs-another-split |
| Storing behaviors as code | Skills stored as data (`.lino` / generated source) under the §9 five-rule ladder; adoption gated by the self-improvement loop |

No GPT-4, embeddings, or embodied environment is introduced; every Voyager
mechanism maps to a deterministic, testable, data-described counterpart.

## SolverConfig Knobs (all added to config first)

| Knob | Purpose | Default (compat) |
| --- | --- | --- |
| `max_decomposition_depth` (existing) | Hard recursion bound | unchanged |
| `atomicity_policy` (new) | When a unit stops splitting (`prefer_compose` / `prefer_split`) | reproduces today |
| `recursion_mode` (new, comparison) | `down` / `up` / `both` | `down` until `both` proven |
| `selection_mode` (new, comparison) | `legacy` / `registry` / `compare` | `legacy` until parity proven |
| `evidence_policy` (extends existing source/offline policy) | When fresh data is required | offline-safe default |

Every knob ships disabled-by-default or set to the compatible value, so merging
the core changes no observable behavior until a knob is flipped — and each flip is
gated by the comparison harnesses in [options-comparison.md](options-comparison.md).

## Termination And Safety

- Recursion is bounded by `max_decomposition_depth` and the atomicity base case;
  there is no unbounded loop (`NON-GOALS.md`).
- Construction compositions are bounded by `feasible_compositions(…, cfg)`; the
  search space is capped, not exhaustive.
- Retry-within-budget for leaf repair is bounded by a budget knob; on exhaustion
  the unit is `blocked`, not retried forever.
- Self-modification of methods/skills never happens inline; it is a proposal
  routed through `docs/design/self-improvement-loop.md` (C3).

## Grounding And Tests

- The recursive core ships described as a `data/meta/*-recipe.lino` recipe,
  grounded by the parameterized recipe test (resolves C5).
- `specialized_handlers_still_publish_loop_events` is widened beyond arithmetic
  to prove each handler family still emits candidate/validation/simplification
  events when reached as a leaf.
- New prompt-variation cases (R129, 5–10 per family) assert that multi-need
  prompts produce multiple needs and a complete ledger.
- Snapshot tests assert the `.lino` frame/work-unit trace is stable, so
  representation refactors (Decision 1) are provably behavior-neutral.
