# Issue 559 Architecture Inventory

This inventory documents the current architecture before implementation planning, as requested in the issue.

## Documents Read

- `README.md`
- `ARCHITECTURE.md`
- `REQUIREMENTS.md`
- `VISION.md`
- `GOALS.md`
- `NON-GOALS.md`
- `ROADMAP.md`
- `docs/meta-algorithm.md`
- `docs/design/no-hardcoded-natural-language.md`
- `docs/design/self-improvement-loop.md`
- `docs/design/rule-synthesis.md`
- `CONTRIBUTING.md`
- Existing case studies, especially `docs/case-studies/issue-554` and `docs/case-studies/issue-468`

## Current Universal Solver Loop

`src/solver.rs` is the main runtime entry point. The core `UniversalSolver` flow already performs these broad steps:

1. Record the prompt and runtime context.
2. Detect language.
3. Apply probability replay when enabled.
4. Produce formalization candidates.
5. Select and cache an intent formalization.
6. Run local search and decomposition paths.
7. Try write-program and synthesis paths.
8. Run specialized handlers.
9. Apply policy and unknown-reasoning fallbacks.
10. Record candidate, validation, simplification, and answer projection events.

This means the repo already has a traceable universal envelope. Issue 559 should generalize what happens inside that envelope, especially method selection.

## Meta Algorithm Assets

`docs/meta-algorithm.md` currently describes two grounded recipes:

- Procedural how-to recipe: `data/meta/procedural-howto-recipe.lino`, tested by `tests/unit/specification/meta_algorithm.rs`.
- Agentic coding recipe: `data/meta/agentic-coding-recipe.lino`, tested by `tests/unit/specification/agentic_meta_algorithm.rs`.

Both are useful precedents because they describe behavior in data and then ground it through tests. The gap is that they are still specific algorithms, not one general problem-frame algorithm.

## Formalization And Routing

Relevant files:

- `src/concepts.rs`
- `src/translation/formalization.rs`
- `src/intent_formalization.rs`
- `data/seed/*.lino`
- `data/meanings.lino`

Current formalization records every prompt and produces intent candidates. `IntentFormalization` can carry relevant labels such as handler names, evidence, and slots. This is the natural place to evolve toward a general `ProblemFrame`.

Remaining hardcoded parts:

- `append_prompt_relevants` in `src/intent_formalization.rs` still derives relevants from Rust string predicates.
- `looks_like_text_manipulation` and related helper checks still embed English operation cues.
- `route_for_prompt` has a seed route path, but it is not yet the only source of routing behavior.

## Specialized Handler Dispatch

Relevant file:

- `src/solver_dispatch.rs`

`SPECIALIZED_HANDLERS` is an ordered table. The comment explicitly says the table is the single source of truth for handler precedence and that the first matching handler wins. Handlers include web search, fetch, procedural how-to, conversation memory, project follow-up, text manipulation, fact lookup, translation, calendar, arithmetic, proof, setup, UI, source cache, and many issue-specific capabilities.

Relevant runtime:

- `UniversalSolver::handle_specialized_pattern` in `src/solver.rs`
- `ordered_handler_names` in `src/intent_formalization.rs`

The current system already lets intent formalization influence handler order by placing relevant handler names first. However, the executable control plane is still the handler table plus Rust predicates. Issue 559 targets this gap.

## Data, Cache, Overrides, And Meanings

The issue requires preserving this architecture. Relevant assets include:

- `.lino` seed and meta files under `data/`
- meaning and synonym records
- source cache paths
- intent formalization cache
- contextual overrides in solver routing
- event log and trace recording
- benchmark and requirement-tracking data

The plan must migrate control data into these surfaces instead of replacing them.

## Testing Surfaces

Relevant tests:

- `tests/unit/specification/reasoning_loop.rs`
- `tests/unit/specification/meta_algorithm.rs`
- `tests/unit/specification/agentic_meta_algorithm.rs`
- prompt variation and benchmark tests
- no-hardcoded-natural-language and total-closure guards
- Rust and browser worker parity tests

Important current compatibility guard:

- `specialized_handlers_still_publish_loop_events` checks that specialized handler answers still publish candidate, validation, and simplification evidence. This implies a migration can keep handlers but must route them through the universal evidence model.

## Existing Self-Improvement Design

`docs/design/self-improvement-loop.md` already defines the intended safe path:

- Unknown traces can suggest new Links Notation rules.
- Suggestions are not silently applied.
- Generated changes must pass verification and benchmarks.
- Human review remains part of the loop.

Issue 559 can reuse this as the later self-modification gate for changing the general meta algorithm.

## Architecture Gaps To Address

1. Method selection is not yet a first-class, data-described reasoning step.
2. Specialized handler precedence still lives in Rust ordering.
3. Some natural-language cue recognition still lives in Rust code.
4. Big-task planning is documented for agentic coding but not represented as a general task graph for every large request.
5. Chat-mode fresh-data policy is not a unified evidence policy in the solver frame.
6. Existing tests cover many behaviors, but they do not yet prove class-level parity for every historical handler family.
7. Meta recipes exist per capability; the project needs a general recipe that can call those methods as data-described submethods.

## Compatibility Constraints

- Preserve existing user-visible behavior unless a test and requirement explicitly approve a change.
- Preserve `.lino`, cache, override, meanings, and source-cache architecture.
- Keep current specialized handlers callable during migration.
- Keep Rust and browser worker mirrors aligned.
- Add tests before changing routing behavior.
- Avoid direct adoption of a large external orchestration runtime as the core engine unless it can satisfy Rust, offline, deterministic, data-driven, and parity constraints.
