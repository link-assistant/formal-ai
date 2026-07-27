# Issue 702 — Solution Plans

One plan per requirement of [`requirements.md`](requirements.md). Each plan
names what it **reuses** (issue #702 is an assembly issue: the substrate landed
with #649/PR #675) and the **prior art** the shape is borrowed from.

## R702-1 — Current state from the dialogue, with provenance

**Plan.** Fold the conversation with `DialogueWorldModel::from_turns`: every user
turn is classified (`world_model_atoms::classify`), and a declarative turn's
`state_atom` is asserted into the `current` context together with two statement
links — `asserts:<atom>` and `provenance:turn:<n>` — so any fact can be traced
back to the turn that said it.
**Reuses.** `Context`/`Statement` from [`src/world_model.rs`](../../../src/world_model.rs),
`SubstitutionLink` doublets, the `EventLog`.
**Prior art.** STRIPS world states; dialogue-state tracking (DST) belief states.

## R702-2 — Target state from "I want …" and imperatives

**Plan.** Keep the recognition vocabulary out of Rust: `world_state_target` in
[`data/meta/cue-lexicon.lino`](../../../data/meta/cue-lexicon.lino) lists the
wish/imperative cues in en/ru/hi/zh, and `classify` consults it through
`cue_lexicon::matches`. A matching turn contributes its `state_atom` to the
`target` context instead of `current`.
**Reuses.** The cue-lexicon data path (#553) and the four-language convention.
**Prior art.** Goal/target states in STRIPS; user-goal slots in task-oriented
dialogue.

## R702-3 — `diff(current, target)` as links, askable from chat

**Plan.** `difference()` returns a `ContextDiff` of links; `remaining()` returns
the target links not yet satisfied. The chat surface is a **contextual** handler
([`src/solver_handlers/world_state.rs`](../../../src/solver_handlers/world_state.rs))
— contextual handlers are the only ones that receive both `history` and
`SolverConfig`, which this one needs. It answers with a seed-localized template
(`world_state_remaining` / `world_state_reached`) and appends
`world_state:remaining`, `world_state:difference` and `world_state:sync_events`
evidence links, so the answer is the difference, not prose about it.
**Reuses.** `Context::difference`, `seed::response_for` localization, the
contextual dispatch table.
**Prior art.** `git diff` between two trees; STRIPS goal-delta.

## R702-4 — Synchronization loop with append-only events

**Plan.** `propose_target` records a `SyncEvent`; the user's next turn, routed by
`observe`, either confirms it (`UtteranceKind::Confirmation`) or corrects it
(`UtteranceKind::Correction`, which supersedes the proposal instead of deleting
it, event kinds `target_confirmed` / `target_corrected`). Each `derived_id` hashes
the previous event's id, so `chain_is_intact()` detects any rewrite of history.
**Reuses.** `stable_id` (FNV-1a) content addressing, the `EventLog` append-only
discipline.
**Prior art.** Event sourcing / hash-chained logs; mixed-initiative confirmation
subdialogues.

## R702-5 — Merge and split as first-class operations

**Plan.** `merge_from` unions another model's current context and **returns the
conflicting links** (same `from`, different `to`) rather than silently picking a
winner; `split_current` moves a named set of statement ids into a child
`Context` via `Context::split_off`, so merge∘split round-trips.
**Reuses.** `Context::merge`/`split_off`.
**Prior art.** Three-way merge with conflict markers; ATMS environments.

## R702-6 — Dependent statements via relative-meta-logic

**Plan.** `depends_on(consequent, premise)` records a `Dependency`; changing a
premise with `revise_statement` runs `Context::recalculate` over the RML decimal
truth grid and returns a `RecalculationReport` that **names every statement it
touched**, so the trace is inspectable rather than a summary count.
**Reuses.** The RML truth values and dependency cascade from #649.
**Prior art.** JTMS/ATMS belief revision; spreadsheet recalculation order.

## R702-7 — Action-consequence prediction

**Plan.** `forecast(action)` applies the action's add/delete links to a *copy* of
the current context, compares the hypothetical state with the target, and returns
`ActionForecast { satisfied, violated, remaining_before, remaining_after }`
rendered as Links Notation. A destructive action shows up as a non-empty
`violated` set (`violates_target()`) **before** anything executes; nothing is
sampled, so the same action always yields the same forecast.
**Reuses.** `Action`/`Prediction` from `src/world_model.rs`.
**Prior art.** STRIPS progression planning; "simulate before commit" in
transactional systems.

## R702-8 — Links-only, four languages, opt-in knob

**Plan.** Every artifact is Links Notation and every recognition list is data;
no embeddings or graph/edge/vertex vocabulary anywhere in the feature. The knob
is `SolverConfig::world_model_mode` (`WorldModelMode::{Off, Track}`, env override
`FORMAL_AI_WORLD_MODEL_MODE`), defaulting to `Off`: with the knob off the handler
declines and no `world_state:` link is ever emitted, which is what the inertness
test asserts.
**Reuses.** The trace-only-until-opted-in enum-knob pattern used by the other
mode knobs in `SolverConfig`.
**Prior art.** Feature flags with a shadow/trace mode before enforcement.

## R702-9 — Acceptance suite and benchmark ratchet

**Plan.** Cover the scripted dialogue, dependency recalculation, safe and
destructive predictions, and merge/split round-trip as direct behavior tests.
Replay sixteen self-authored bAbI-shaped dialogues across en/ru/hi/zh and reject
any pass count below the committed `minimum_pass_count`.
**Reuses.** Existing benchmark fixture and ratchet conventions.
**Prior art.** bAbI state tracking and monotonic quality gates.

## R702-10 — Case-study data and analysis

**Plan.** Retain authenticated issue/PR snapshots, a requirement matrix,
per-requirement implementation plans, root-cause analysis, and a verification
map under `docs/case-studies/issue-702/`, then pin their presence and key claims
with an automated documentation test.
**Reuses.** The repository's prior case-study layout.
**Prior art.** Requirements traceability matrices.

## R702-11 — Unlimited nested contexts

**Plan.** Represent every scope with the existing `Context` and keep the
child→parent relation in `ContextHierarchy`. Walk iteratively with a visited set,
not recursion or a configured depth constant, so a conversation/dialog/task/PR/
issue/repository/organization chain and future scopes use the same mechanism.
Reject missing parents, self-parenting, duplicates, and cycles at mutation time.
**Reuses.** `Context`, deterministic `BTreeMap` ordering, Links Notation.
**Prior art.** Lexical environments and dynamically nested configuration scopes.

## R702-12 — Full, isolated, and conditional inheritance

**Plan.** Put an `InheritancePolicy` on each child→parent boundary. `Full` passes
all ancestor links, `Isolated` stops lookup, and `Conditional` carries
`LinkPattern`s. Conditions accumulate across boundaries so a distant link is
visible only if it satisfies at least one pattern from every conditional scope
it crosses.
**Reuses.** `LinkPattern`; the new public `LinkPattern::matches` applies the same
variable-binding semantics as substitution rules.
**Prior art.** Module export lists, capability attenuation, and lexical shadowing.

## R702-13 — Lazy nearest-first reference resolution

**Plan.** `resolve(start, reference, external_policy)` checks exact `from`
matches in the start context and walks outward only until the first visible
match. The result names every inspected context, the resolving context and
depth, and matched links. A miss is either `Unresolved` or
`ExternalLookupRequired`; the resolver never performs network work itself.
**Reuses.** Context links and deterministic Links Notation traces.
**Prior art.** Lexical name resolution and local-first cache lookup.

## R702-14 — Generalized runtime adoption

**Plan.** Make each dialogue turn a full-inheritance child of the previous turn;
replace immediate-last-user coreference with the common hierarchy and record its
resolution trace; replace the agentic research path's private reverse scan; and
remove the four-level coding-language inheritance cap in both Rust and browser
implementations, retaining visited-set cycle termination.
**Reuses.** Existing coreference seeds, research topic extraction, and
Rust/browser code-idiom catalogs.
**Prior art.** Scope-chain lookup in interpreters; cycle-safe prototype chains.

## R702-15 — Formal AI Agent CLI self-hosting and learning

**Plan.** Persist the hierarchy failure observations as links, register a
`LearningReport` descriptor so the shared `AssociativeMemory` adapter performs
ranking and multi-hop recall, then run that exact issue-702 learning task through
the repository's OpenAI-compatible Formal AI server from the real external Agent
CLI. Preserve the exact report, both transcripts, and session provenance; assert
the retained bytes equal the production renderer.
**Reuses.** Shared auto-learning renderer, agentic CLI harness, and
`CONTRIBUTING.md` evidence bundle convention.
**Prior art.** Compiler bootstrapping and golden artifact comparison.

## R702-16 — Permission-gated general memory

**Plan.** Remove public mutation access to `WorldModel::general` and make
`commit_current_to_general` accept an explicit `GeneralMemoryPermission`.
`Denied` returns an error before mutation; `Allowed` performs the existing
context merge. Keep a read-only accessor for inspection.
**Reuses.** `Context::merge_from` and fail-closed enum gates.
**Prior art.** Capability boundaries and explicit transaction authorization.

## R702-D — bAbI-style tracking slice with a ratchet

**Plan.** [`data/benchmarks/world-state-tracking-suite.lino`](../../../data/benchmarks/world-state-tracking-suite.lino)
holds sixteen self-authored dialogues in the *shape* of bAbI tasks 1/2/6 plus
everyday goal dialogues, across en/ru/hi/zh, each with held-out paraphrase
variants. `tests/unit/specification/world_state_benchmarks.rs` replays each
dialogue through the solver and asserts `passed >= minimum_pass_count` (16).
**Reuses.** The fixture + self-contained parser + ratchet convention of the
existing suites (#304/#317, #444).
**Prior art.** bAbI 20 QA tasks; anti-memorization held-out splits.
