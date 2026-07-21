# Issue 702 Case Study

> **Status:** implemented by PR [#818](https://github.com/link-assistant/formal-ai/pull/818).
> The world model is **trace-only until opted in**: `SolverConfig::world_model_mode`
> defaults to `Off`, and with it off the solver behaves exactly as before.

Issue [#702](https://github.com/link-assistant/formal-ai/issues/702) (E60, child
of [#651](https://github.com/link-assistant/formal-ai/issues/651)) asks for the
**dialogue layer** of the symbolic world model: the assistant should keep a
current state and a target state as links networks built from the conversation,
answer "what is left to do?" from their difference, keep the two in sync through
confirmations and corrections, recalculate dependent statements when a premise
changes, and predict what an action would do *before* it runs.

* Requirements: [`requirements.md`](requirements.md)
* Per-requirement plans: [`solution-plans.md`](solution-plans.md)
* Raw issue/PR data: [`raw-data/`](raw-data/)
* Parent design study: [`../issue-649/`](../issue-649/)

## 1. Collected Data

| File | Contents |
| --- | --- |
| [`raw-data/issue-702.json`](raw-data/issue-702.json) | Issue body: the eight numbered plans and the acceptance criteria. |
| [`raw-data/issue-702-comments.json`](raw-data/issue-702-comments.json) | Issue discussion. |
| [`raw-data/issue-651-parent.json`](raw-data/issue-651-parent.json) | Parent epic that groups the world-model work. |
| [`raw-data/pr-675.json`](raw-data/pr-675.json) | The PR that landed the #649 substrate this issue builds on. |

The issue's eight plans map one-to-one onto the per-requirement plans issue #649
recorded in [`../issue-649/solution-plans.md`](../issue-649/solution-plans.md)
(R649-01 … R649-08); #702 is the issue where those plans become reachable from a
conversation.

## 2. Requirements

R702-1 … R702-8 (features) and R702-A … R702-E (acceptance criteria) are stated
verbatim with their status and covering test in
[`requirements.md`](requirements.md), and mirrored in the repository-level
[`REQUIREMENTS.md`](../../../REQUIREMENTS.md).

## 3. Root Cause

PR #675 landed `src/world_model.rs` — contexts, statements, RML truth values,
dependency recalculation, STRIPS-style actions and predictions — but nothing
*fed* it. The world model was reachable only from Rust tests: no conversation
turn ever wrote a statement into a context, no cue vocabulary told a wish
("I want the door open") apart from a fact ("the door is closed"), and no chat
surface could answer a question about the gap between them. The gap was
therefore not in the reasoning substrate but in the **absence of a dialogue→links
ingestion path and of a handler that answers from the difference**. That is what
this issue closes.

## 4. Implemented Design

| Layer | Path | Role |
| --- | --- | --- |
| Utterance classification | [`src/world_model_atoms.rs`](../../../src/world_model_atoms.rs) | `classify` (fact / wish / confirmation / correction / state query) and `state_atom` (`text → SubstitutionLink`), driven entirely by cue data. |
| Recognition vocabulary | [`data/meta/cue-lexicon.lino`](../../../data/meta/cue-lexicon.lino) | `world_state_target`, `world_state_query`, `world_state_confirm`, `world_state_correct`, `world_state_because`, `world_state_separator`, `world_state_filler` — en/ru/hi/zh, no Rust phrase tables. |
| Dialogue model | [`src/world_model_dialog.rs`](../../../src/world_model_dialog.rs) | `DialogueWorldModel`: current/target contexts with provenance, hash-chained `SyncEvent` log, `difference`/`remaining`, `depends_on`/`revise_statement`, `forecast`, `merge_from`/`split_current`, `WorldModelMode`, `record_world_model` trace artifact. |
| Chat surface | [`src/solver_handlers/world_state.rs`](../../../src/solver_handlers/world_state.rs) | Contextual handler: rebuilds the model from `history`, answers `world_state_remaining` / `world_state_reached` from the difference, emits `world_state:*` evidence links. |
| Dispatch | [`src/solver_dispatch.rs`](../../../src/solver_dispatch.rs) | `"world_state"` in `CONTEXTUAL_HANDLER_NAMES` (contextual handlers receive both history and `SolverConfig`). |
| Localization | [`data/seed/multilingual-responses.lino`](../../../data/seed/multilingual-responses.lino) | `world_state_remaining` / `world_state_reached` templates in en/ru/hi/zh with `{count}`, `{remaining}`, `{diff_id}` placeholders. |
| Knob | [`src/solver.rs`](../../../src/solver.rs), [`src/solver_helpers/mod.rs`](../../../src/solver_helpers/mod.rs) | `SolverConfig::world_model_mode` (`Off` by default) with the `FORMAL_AI_WORLD_MODEL_MODE` env override. |

The answer is **computed, not remembered**: the handler rebuilds the model from
the conversation on every question, so replaying the same dialogue always yields
the same answer and no state is cached between turns.

## 5. Prior Art And Existing Components

* **Reused in-repo:** `Context`, `Statement`, `Dependency`, `Action`,
  `Prediction`, `RecalculationReport` (#649 / PR #675); `SubstitutionLink`
  doublets and `stable_id` content addressing; the cue-lexicon data path (#553);
  the seed localization path; the benchmark fixture + ratchet convention
  (#304/#317, #444); the trace-only-until-opted-in knob pattern.
* **External shapes:** STRIPS state/goal deltas and progression planning;
  JTMS/ATMS belief revision for the dependency cascade; event sourcing with
  hash-chained logs for the synchronization loop; three-way merge with conflict
  reporting; the bAbI state-tracking tasks for the benchmark slice's *shape*
  (self-authored dialogues — no upstream text is redistributed).

## 6. Verification

| Check | Where |
| --- | --- |
| Per-requirement behaviour (provenance, target extraction, difference, sync loop, merge/split, recalculation trace, forecast, opt-in) | [`tests/unit/issue_702_world_model_dialog.rs`](../../../tests/unit/issue_702_world_model_dialog.rs) |
| Chat surface in all four languages, inertness while off, deterministic replay, links-only vocabulary | [`tests/unit/issue_702_world_state_chat.rs`](../../../tests/unit/issue_702_world_state_chat.rs) |
| bAbI-style tracking slice with the pass-count ratchet (`minimum_pass_count` = 16) | [`tests/unit/specification/world_state_benchmarks.rs`](../../../tests/unit/specification/world_state_benchmarks.rs), [`data/benchmarks/world-state-tracking-suite.lino`](../../../data/benchmarks/world-state-tracking-suite.lino) |
| Cue sets consulted by Rust all exist in the lexicon data | [`tests/unit/specification/cue_lexicon.rs`](../../../tests/unit/specification/cue_lexicon.rs) |
| Contextual handler roster stays declared | [`tests/unit/specification/method_registry.rs`](../../../tests/unit/specification/method_registry.rs) |
| Documentation traceability | [`tests/unit/docs_requirements_issue_702.rs`](../../../tests/unit/docs_requirements_issue_702.rs) |

Run them with:

```bash
cargo test --test unit issue_702
cargo test --test unit issue_702_world_state_suite_tracks_each_case -- --nocapture
```
