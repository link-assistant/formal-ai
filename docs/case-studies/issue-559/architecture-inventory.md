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
- Follow-up PR feedback on <https://github.com/link-assistant/formal-ai/pull/560#issuecomment-4783154352>
- `docs/case-studies/issue-433/README.md`
- `docs/case-studies/issue-468/README.md`

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

Issue #433 is also directly relevant. It audits each specialized handler recognizer as fixed enumeration, hybrid, or compositional, then demonstrates how `numeric_list` can be reconstructed from the issue #423 meta-algorithm primitives. That makes the current missing piece clearer: the repository has examples of class-level construction recipes, but no single registry where every method advertises its preconditions, required evidence, validation policy, and executable hook.

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

## Follow-Up Feedback Applied To Architecture

The 2026-06-23 PR feedback adds architectural constraints that are not fully captured by the initial first-session plan:

1. **Recursive solving is not just optional decomposition.** The current solver can decompose in specific places, but there is no universal recursive work-unit model that keeps splitting until a unit is directly solvable by a method, library call, standard function, or reviewed skill.
2. **Skill accumulation is not a first-class control plane.** Existing handlers, examples, generated source, seed rules, and standard-library functions can behave like reusable skills, but the repository does not yet index them with applicability, proof status, examples, negative examples, validation cost, and safe reuse boundaries.
3. **Fresh-data research is handler-specific.** The desktop and web-search paths exist, but there is no general evidence pipeline that can expand a prompt into terms, phrases, sentences, and questions; search multiple providers; rerank; crawl; extract; compare; and feed hypothesis gaps back into the same recursive loop.
4. **Link-native modeling needs to be explicit.** `meta-language` and Links Notation already use links, but future docs should avoid making a separate non-link ontology the primary model. Dependency, task, method, evidence, and sequence relations should be represented as links.
5. **Dependency readiness is not gated.** The repo uses several organization-owned crates and packages, but the implementation plan did not previously say which upstream capability must exist before each phase can begin.

## Architecture Gaps To Address

1. Method selection is not yet a first-class, data-described reasoning step.
2. Specialized handler precedence still lives in Rust ordering.
3. Some natural-language cue recognition still lives in Rust code.
4. Big-task planning is documented for agentic coding but not represented as a general recursive task-link network for every large request.
5. Chat-mode fresh-data policy is not a unified evidence policy in the solver frame.
6. Existing tests cover many behaviors, but they do not yet prove class-level parity for every historical handler family.
7. Meta recipes exist per capability; the project needs a general recipe that can call those methods as data-described submethods.
8. There is no `ProblemFrame` type, event, or `.lino` schema that every solver response must emit.
9. There is no `WorkUnit` or equivalent recursive unit that records parent/child links, atomicity, required evidence, selected skill, validation result, and composition result.
10. There is no method/skill registry covering all `SPECIALIZED_HANDLERS`, contextual overrides, standard library operations, repo helper functions, generated examples, and future learned rules.
11. There is no old-vs-new comparison mode for registry selection before replacing direct dispatch.
12. There is no uniform need-satisfaction ledger that forces the final answer to mark every detected question, requirement, constraint, and deferred item.
13. There is no general search/rerank/crawl/extract/evaluate loop for fresh-data chat questions.
14. There is no upstream dependency gate document that says when implementation must pause for `meta-language`, `links-notation`, `lino-objects-codec`, `doublets`, `link-calculator`, or `agent-commander`.

## Upstream Dependency Snapshot

The related organization dependency audit is captured in [upstream-dependency-audit.md](upstream-dependency-audit.md). The important architecture conclusion is:

- No upstream blocker prevents the next behavior-preserving phases: `ProblemFrame`, recursive work-unit tracing, method registry inventory, and old/new selection comparison.
- `meta-language` already advertises mutable link networks, source spans, lossless parse/reconstruction, generated-source rendering, snapshots, structural query/replace, substitutions, LiNo parsing, and cross-language reconstruction. Those capabilities are enough for the planned algorithm-as-data representation.
- `links-notation` has an existing streaming-parser issue, <https://github.com/link-foundation/links-notation/issues/197>. It matters for very large future frame exports but is not a blocker for the initial registry and frame data.
- `link-calculator`, `doublets`, `platform-mem`, `lino-objects-codec`, `lino-arguments`, `lino-i18n`, and `agent-commander` do not currently block the next implementation phases.

## Compatibility Constraints

- Preserve existing user-visible behavior unless a test and requirement explicitly approve a change.
- Preserve `.lino`, cache, override, meanings, and source-cache architecture.
- Keep current specialized handlers callable during migration.
- Keep Rust and browser worker mirrors aligned.
- Add tests before changing routing behavior.
- Avoid direct adoption of a large external orchestration runtime as the core engine unless it can satisfy Rust, offline, deterministic, data-driven, and parity constraints.
