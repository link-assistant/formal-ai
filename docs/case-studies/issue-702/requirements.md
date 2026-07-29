# Issue 702 — Requirements

Every requirement extracted from the issue body
([`raw-data/issue-702.json`](raw-data/issue-702.json)) and its comments
([`raw-data/issue-702-comments.json`](raw-data/issue-702-comments.json)), plus
the nested-context clarification on PR #818
([`raw-data/pr-818-comments.json`](raw-data/pr-818-comments.json)).
Issue
#702 (E60) is the **implementation** issue for the per-requirement plans issue
#649 drafted in
[`../issue-649/solution-plans.md`](../issue-649/solution-plans.md); #649's PR
[#675](https://github.com/link-assistant/formal-ai/pull/675) landed the
*substrate* ([`src/world_model.rs`](../../../src/world_model.rs)), and this issue
asks for the **dialogue layer** on top of it.

Status legend: **Done** — delivered by PR
[#818](https://github.com/link-assistant/formal-ai/pull/818); **Realized** —
already present and reused as-is.

## Feature requirements

| ID | Requirement | Status |
|---|---|---|
| **R702-1** | Seed and maintain the **current-state context** from the dialogue log, with **provenance** back to the turn that asserted each fact. | Done — `DialogueWorldModel::observe` + `record_provenance` in [`src/world_model_dialog.rs`](../../../src/world_model_dialog.rs); every asserted atom carries `provenance:turn:<n>` and `asserts:<atom>` statement links. |
| **R702-2** | Build the **target-state context** from intent formalization: "I want …" statements and imperative requests. | Done — the `world_state_target` cue set in [`data/meta/cue-lexicon.lino`](../../../data/meta/cue-lexicon.lino) routes an utterance to `assert_target`; extraction is `world_model_atoms::state_atom`. |
| **R702-3** | Expose `diff(current, target)` as a **links network**, queryable from chat ("what is left to do?"). | Done — `DialogueWorldModel::difference`/`remaining` over `Context::difference`, surfaced in chat by the `world_state` contextual handler ([`src/solver_handlers/world_state.rs`](../../../src/solver_handlers/world_state.rs)). |
| **R702-4** | A **synchronization loop**: the agent proposes target edits, the user confirms or corrects; every step is an **append-only event**. | Done — `propose_target` plus confirmation/correction turns routed by `observe`, each recorded as a `SyncEvent` in a content-addressed hash chain verified by `chain_is_intact()`. |
| **R702-5** | **Merge and split** as first-class operations: union with **conflict detection**; split by topic/subtree. | Done — `DialogueWorldModel::merge_from` (returns the conflicting links) and `split_current` over `Context::split_off`. |
| **R702-6** | **Dependent statements via relative-meta-logic**: changing one statement recalculates dependents, and the **trace names each recalculated link**. | Done — `depends_on` + `revise_statement` return a `RecalculationReport` naming every updated statement; the cascade is `Context::recalculate` on the RML decimal grid. |
| **R702-7** | **Action-consequence prediction**: `predict(current, action) -> hypothetical_state`, compared with the target, reporting **satisfied / violated** needs — deterministic and inspectable. | Done — `DialogueWorldModel::forecast` returns an `ActionForecast { satisfied, violated, remaining_before, remaining_after }` rendered as Links Notation. |
| **R702-8** | **Everything is links**: no embeddings, no graph/edge/vertex terminology; **all four languages** (en/ru/hi/zh); knobs in `SolverConfig`; **trace-only until opted in**. | Done — every artifact is Links Notation; the recognition vocabulary is four-language cue data; `SolverConfig::world_model_mode` (`FORMAL_AI_WORLD_MODEL_MODE`) defaults to `Off`. |
| **R702-9** | Acceptance coverage: scripted dialogue, RML recalculation trace, prediction safety, merge/split round-trip, and a state-tracking benchmark ratchet. | Done — behavior tests plus the 16-case `world-state-tracking-suite.lino` ratchet cover every acceptance bullet. |
| **R702-10** | Collect the issue data and analysis under `docs/case-studies/issue-702/`. | Done — this case study, its raw issue/PR snapshots, requirements, plans, implementation analysis, and verification matrix are protected by the documentation traceability test. |
| **R702-11** | Support **unlimited nested contexts** for conversation, dialogue, task, pull request, issue, repository, organization, and future scopes. | Done — [`ContextHierarchy`](../../../src/world_model_context.rs) uses an iterative child→parent walk with cycle rejection and no depth constant; tested through 320 scopes. |
| **R702-12** | Context inheritance is explicit: **full**, **fully isolated**, or **partial conditional** inheritance of any link-carried axiom, fact, requirement, or goal. | Done — `InheritancePolicy::{Full, Isolated, Conditional(Vec<LinkPattern>)}`; conditional boundaries accumulate so an ancestor link must satisfy every boundary crossed. |
| **R702-13** | **Lazy nearest-first reference resolution**: exact local match, then each inherited context outward; outside-world lookup only if explicitly allowed. | Done — `ContextHierarchy::resolve` stops at the nearest matching context and returns a Links Notation trace of `visited`, `resolved_in`, and depth; a miss distinguishes `Unresolved` from `ExternalLookupRequired` without performing network access. |
| **R702-14** | Apply the generalized context mechanism in all relevant touched paths and remove obsolete fixed-depth inheritance. | Done — dialogue turns, solver coreference, and agentic research-topic recall use the hierarchy; Rust/browser coding-idiom inheritance uses visited-set cycle protection instead of a four-level cap. |
| **R702-15** | Execute the same change task through **Formal AI auto-learning and the real Agent CLI**, retaining self-hosting evidence. | Done — the shared associative-memory learning renderer ranks issue-702 observations; a real external Agent CLI drove Formal AI through write → verify → final and retained a byte-reproducible report and both transcripts under [`dev/log/issues/702/pulls/818/agent-cli/`](../../../dev/log/issues/702/pulls/818/agent-cli/). |
| **R702-16** | Require an explicit permission decision before dialogue-local `current` state can enter shared `general` memory. | Done — `WorldModel::general()` is read-only and `commit_current_to_general(GeneralMemoryPermission)` returns `PermissionDenied` without mutation unless explicitly allowed. |

## Acceptance criteria

| ID | Criterion | Covering test |
|---|---|---|
| **R702-A** | A scripted multi-turn dialogue produces inspectable `current_state` / `target_state` / `state_diff` links and answers "what remains?". | `a_scripted_dialogue_tracks_state_answers_the_goal_question_and_forecasts`, `asking_what_is_left_answers_from_the_difference_in_every_language` |
| **R702-B** | Changing a premise recalculates dependent probabilities, and the trace names each recalculated link. | `revising_a_premise_recalculates_every_dependent_statement` |
| **R702-C** | `predict` on a bounded agent action shrinks the diff, and a destructive action is flagged as violating a target need **before** execution. | `a_helpful_action_shrinks_the_gap_and_a_destructive_one_is_flagged` |
| **R702-D** | Merge/split round-trip tests plus a **bAbI-style state-tracking benchmark slice with a ratchet**. | `contexts_merge_with_conflict_detection_and_split_round_trips`, `issue_702_world_state_suite_tracks_each_case` (`minimum_pass_count` = 16 in [`data/benchmarks/world-state-tracking-suite.lino`](../../../data/benchmarks/world-state-tracking-suite.lino)) |
| **R702-E** | Data collected to `docs/case-studies/issue-702/`. | `issue_702_world_model_documents_are_traceable` |
| **R702-F** | A 320-level scope chain resolves lazily, while full/isolated/conditional policies and cycles behave deterministically. | `reference_resolution_has_no_fixed_context_depth_limit_and_is_lazy`, `local_links_shadow_fully_inherited_links`, `isolated_contexts_stop_inheritance_and_gate_external_lookup`, `conditional_inheritance_filters_links_at_every_boundary`, `hierarchy_rejects_parent_cycles` |
| **R702-G** | Dialogue and live solver/agentic reference paths use the common substrate, with Rust/browser inheritance parity. | `dialogue_turns_are_real_nested_contexts_not_only_a_standalone_utility`, `runtime_coreference_searches_nearest_relevant_ancestor_turn`, `issue_687::learn_about`, `rust_and_browser_coding_inheritance_are_cycle_safe_not_depth_capped` |
| **R702-H** | A real Formal AI Agent CLI run authors and verifies an exact task artifact and retains its learning trace. | `dev/log/issues/702/pulls/818/analysis.md` and `issue_702_world_model_documents_are_traceable` |
| **R702-I** | Dialogue-local facts cannot enter shared general memory without explicit permission. | `dialogue_state_requires_explicit_permission_before_entering_general_memory` |
