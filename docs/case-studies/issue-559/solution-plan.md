# Issue 559 Solution Plan

This plan preserves the issue's first-session constraint: document and plan first, then implement after maintainer approval or changes.

## Recommended Architecture

Introduce a general `ProblemFrame` that every prompt passes through before any specialized behavior runs. The exact Rust type can evolve, but the frame should have these conceptual fields:

- `impulse`: original user message and conversation context.
- `mode`: chat, agentic coding, task execution, diagnostic, or offline-constrained mode.
- `language`: detected language and translation metadata.
- `needs`: questions, requirements, commands, constraints, preferences, safety constraints, fresh-data needs, and follow-up references.
- `evidence_policy`: whether local memory, source cache, web search, tools, tests, or user clarification are required.
- `decomposition`: task graph nodes, dependencies, and completion status.
- `candidate_methods`: candidate methods with evidence for why each applies.
- `selected_methods`: methods chosen for execution, preserving deterministic order.
- `validation_plan`: tests, checks, expected evidence, or answer-quality constraints.
- `observations`: tool outputs, test results, search results, and handler outputs.
- `composition`: how subresults are merged.
- `presentation`: final answer requirements and unmet need status.

Existing handlers should not disappear at first. They should become executable methods selected through the frame.

## Method Registry

Add or extend `.lino` data to describe methods:

- method id
- aliases and meaning references
- preconditions
- inputs and outputs
- required tools or permissions
- evidence policy
- validation policy
- current Rust or JS implementation hook
- compatibility precedence group
- related tests and benchmarks

This lets the system keep current behavior while moving the control plane from Rust handler ordering into reviewable data.

## General Algorithm

The general algorithm should be:

1. Capture the impulse and context.
2. Formalize into a `ProblemFrame`.
3. Detect all needs in the frame.
4. Decide evidence policy, including whether fresh data is needed.
5. Decompose when the frame is large, ambiguous, multi-requirement, or agentic.
6. Retrieve applicable methods from the method registry.
7. Generate candidates and score them against preconditions and constraints.
8. Execute selected methods and record observations.
9. Validate against the frame's validation plan.
10. Compose subresults and mark every need as satisfied, deferred, blocked, or rejected.
11. Simplify the final response.
12. Record traces, cache safe conclusions, and optionally propose method/rule improvements behind review gates.

## Phased Implementation

### Phase 0: Planning Artifact

Status: this PR step.

Deliverables:

- Case-study requirements.
- Architecture inventory.
- External research notes.
- Phased plan and risk assessment.

### Phase 1: Add ProblemFrame Without Behavior Changes

Goal: make the general frame observable before changing routing.

Tasks:

- Add a Rust `ProblemFrame` or equivalent internal structure.
- Populate it from existing formalization data.
- Record the frame in solver events for every prompt.
- Add tests proving every prompt path emits the frame.
- Add fixtures for multi-need prompts such as question plus requirement plus constraint.

Expected tests:

- New unit tests for frame construction.
- Existing reasoning-loop tests unchanged.
- Snapshot or structured assertions for needs, evidence policy, and selected compatibility route.

### Phase 2: Inventory Current Handlers As Methods

Goal: make every current specialized handler visible as data.

Tasks:

- Create a method registry in `.lino`.
- Add one registry entry for each `SPECIALIZED_HANDLERS` member.
- Preserve current precedence as explicit compatibility data.
- Add a grounding test that fails when the Rust handler table and registry diverge.

Expected tests:

- Registry covers every specialized handler.
- Every registry implementation hook resolves.
- Current dispatch order can be reconstructed from compatibility metadata.

### Phase 3: Move Cue Recognition Out Of Rust

Goal: reduce hardcoded natural-language trigger logic.

Tasks:

- Move recognizer cues from `append_prompt_relevants` into seed meanings or method preconditions.
- Replace Rust string lists with data lookup and structural predicates.
- Keep non-language structural checks in code only when they are truly syntax-level, such as numeric expression parsing.

Expected tests:

- No-hardcoded-natural-language guard updated to cover moved cues.
- Prompt variation tests for each migrated cue family.
- Backward compatibility tests for current supported prompts.

### Phase 4: Select Methods Through ProblemFrame

Goal: route by frame and method registry while preserving old behavior.

Tasks:

- Add a method-selection engine that reads method preconditions and frame needs.
- Run old dispatch and new selection in comparison mode first.
- Record differences as diagnostics without changing answers.
- Switch to registry selection only after parity is proven.

Expected tests:

- Old and new selection agree for all benchmark and prompt-variation fixtures.
- Handler precedence edge cases remain covered.
- Unknown prompts still produce reasoning traces instead of silent failures.

### Phase 5: Expand Class-Level Coverage

Goal: test task classes, not only examples.

Tasks:

- Build representative prompt families for every historical handler group.
- Include multi-need prompts, follow-ups, multilingual prompts, and ambiguous prompts.
- Add negative examples where a method should not fire.

Expected tests:

- Prompt variation suite covers each method family.
- No regressions in benchmark ratchets.
- Requirement-tracking tests map new coverage to issue 559 requirements.

### Phase 6: Generalize Agentic And Chat Policies

Goal: make big-task planning and fresh-data answers part of the general algorithm.

Tasks:

- Add a task-graph policy for large agentic requests.
- Add an evidence policy for fresh internet data when the prompt is time-sensitive or source-sensitive.
- Integrate source cache freshness and citations into frame validation.
- Ensure chat mode can answer directly when no external data is required.

Expected tests:

- Agentic big-task prompts instantiate a todo/task graph.
- Chat fresh-data prompts require source evidence.
- Offline mode reports evidence limitations clearly.

### Phase 7: Gate Self-Modification

Goal: let the algorithm reason about improvements without silently changing itself.

Tasks:

- Reuse `docs/design/self-improvement-loop.md`.
- Convert unknown traces into proposed method or rule patches.
- Require tests and benchmark deltas before accepting a proposal.
- Keep human review as the final gate.

Expected tests:

- Unknown trace can produce a proposed method entry.
- Proposal is not applied without verification.
- Bad proposals are rejected by tests or benchmarks.

### Phase 8: Retire Direct Specialized Dispatch

Goal: make the method registry the control plane.

Tasks:

- Remove the Rust handler table only after registry selection is proven.
- Keep implementation hooks for existing Rust handlers.
- Update docs and architecture diagrams.

Expected tests:

- Full local CI.
- Browser worker parity.
- Benchmarks and prompt variations.
- Changelog and documentation checks.

## Solution Options Considered

Option A: Planning only now.

- Recommended for this first session because the issue explicitly asks for planning before execution.

Option B: Data registry plus compatibility dispatch.

- Recommended implementation path.
- Lowers risk by preserving existing handlers and tests.
- Moves control metadata into `.lino` incrementally.

Option C: Full Links Notation method interpreter immediately.

- Good long-term target.
- Too risky as the first implementation step because current behavior depends on many specialized handlers and precedence cases.

Option D: Adopt an external orchestrator such as LangGraph, AutoGen, Microsoft Agent Framework, or DSPy.

- Not recommended as the core implementation.
- Useful design references, but the repo already has Rust, browser worker, Links Notation, offline, cache, and no-hardcoded-natural-language constraints.

## Requirement Mapping

- R5, R6, R7, R8: addressed by `ProblemFrame`.
- R9: addressed by Phase 6 task graph policy.
- R10: addressed by Phase 6 evidence policy.
- R11: addressed by method registry and general algorithm.
- R12: addressed by Phase 7 gated self-modification.
- R13, R14, R16: addressed by compatibility and parity tests.
- R15: addressed by keeping `.lino`, cache, meanings, overrides, and source cache as first-class architecture.
- R17: addressed by phase-sized commits in the existing PR.

## Risks

1. Handler precedence regressions.
   - Mitigation: reconstruct current precedence from registry data and run old/new comparison tests before switching.

2. Hardcoded language logic moves but does not become more general.
   - Mitigation: store meanings and preconditions as data, then test multilingual and variation prompts.

3. Task graph becomes overbuilt for simple chat.
   - Mitigation: frame can select direct-answer mode when decomposition is unnecessary.

4. Fresh-data policy creates flaky tests.
   - Mitigation: separate live web behavior from deterministic source-cache tests.

5. Rust and JS worker drift.
   - Mitigation: require mirror tests or generated fixtures for every migrated method.

6. Self-modification becomes unsafe.
   - Mitigation: proposal-only by default, with tests, benchmarks, and human review gates.

## Verification Plan

Before switching behavior:

- `cargo fmt`
- `cargo clippy --all-targets --all-features`
- `rust-script scripts/check-file-size.rs`
- `cargo test`
- existing benchmark or prompt-variation commands used by this repo

For each phase:

- Add focused unit tests first.
- Add fixture prompts for the class being migrated.
- Compare old and new behavior until parity is proven.
- Update PR description with phase status and tests.
