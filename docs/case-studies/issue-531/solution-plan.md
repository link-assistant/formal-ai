# Issue 531 Solution Plan

This plan separates the broad issue into staged work that can be reviewed and
tested independently. Phases 0-7 describe the original research and pattern
implementation. Phases 8-10 record the implemented August 2026 review
follow-up from pattern recognition to safe algorithm learning.

## Phase 0: Research Contract

Delivered by this PR:

- preserve issue, PR, upstream, and online research evidence;
- inventory current Formal AI integration points;
- decompose requirements;
- propose implementation phases and risks;
- add a traceability test so the research remains connected to the root
  requirements file.

Acceptance gate: `tests/unit/docs_requirements_issue_531.rs` passes and raw
evidence files are present.

## Phase 1: Link-Native Sequence Substrate

Add a Rust module for sequence symbols and sequence roots on top of the existing
link store abstraction.

Minimum scope:

- unique symbols for scalar sequence elements and unicode code points;
- sequence markers for typed sequences;
- empty, single, pair, and multi-element sequence construction;
- sequence expansion back to the original element IDs;
- optional Links Notation export/import fixtures.

Acceptance gate: unit tests prove stable round trips across the in-memory link
store and the native doublets backend when the feature is enabled.

## Phase 2: Data.Doublets.Sequences Converter Ports

Port converter behavior in increasing risk order:

1. `BalancedVariantConverter`, because it is deterministic and easy to verify.
2. `SequenceIndex`, because adjacent-pair lookup is needed by compression.
3. `LinkFrequenciesCache`, with explicit tests for existing-link counters.
4. `OptimalVariantConverter`, after local-level behavior is pinned.
5. `CompressingConverter`, after the C# vs C++ max-frequency discrepancy is
   resolved by source comparison or fixtures.

Acceptance gate: small fixtures demonstrate the same root structure or
compression choice as the verified upstream behavior.

## Phase 3: Associative Deduplication

Implement repeated-pair and repeated-sub-sequence compression as data:

- detect adjacent pairs and their frequencies;
- choose a replacement pair deterministically;
- replace all non-overlapping usages;
- record each compression step as a trace;
- expand the final tree back to the original sequence;
- expose the trace through tests and optional diagnostics.

Acceptance gate: fixtures such as `A B A B C A B` compress repeated structure
and expand exactly, with no information loss.

## Phase 4: Transformed Pattern Matching

Generalize matching from exact repeated sequences to transformed candidates.

1D transforms:

- reverse;
- shift/translation in sequence index space;
- symbol substitution/permutation;
- repeated interval or rhythm detection.

2D transforms:

- rows, columns, diagonals, borders, and connected components projected into
  sequences;
- rotations and reflections;
- translations over relative coordinates;
- center-relative and axis-relative symmetry;
- color or symbol remapping.

Acceptance gate: ARC-style mini fixtures prove that the matcher can explain
simple rotation, reflection, translation, and repeated-object analogies without
hardcoding each answer.

## Phase 5: Ontology And Seed Meanings

Add seed meanings for the vocabulary the engine needs to explain itself:

- sequence;
- pattern;
- repetition;
- compression;
- deduplication;
- transformation;
- symmetry;
- rotation;
- reflection;
- translation;
- analogy;
- invariant.

Acceptance gate: generated explanations reference grounded seed meanings rather
than ad hoc English strings.

## Phase 6: Solver And Meta-Core Integration

Expose sequence pattern inference as a bounded method:

- add a method entry usable by the universal solver loop;
- record candidate patterns, rejected candidates, compression scores, and
  expansion checks as evidence;
- keep diagnostics default-off but available for issue reproduction;
- route text tasks through string/unicode sequence conversion only when pattern
  inference is relevant.

Acceptance gate: a solver test shows a pattern-inference method selected for a
small structured input and records auditable evidence in the meta-core.

## Phase 7: Benchmarks

Delivered with small, deterministic fixtures before broad benchmarks:

- text repeated phrase examples;
- symbolic sequences with nested repetition;
- event streams from portable memory and existing link records;
- ARC-AGI inspired grid examples;
- requirements-to-solution fact-checking examples where repeated missing
  obligations should be detected.

Acceptance gate: the existing 1D/2D fixtures plus
`data/benchmarks/issue-531-algorithm-traces.lino` are versioned and
deterministic. Discovery tests report both held-out-validated cases and rejected
constant/operation/missing-step counterexamples. The runnable examples are
compiled by `cargo check --examples --all-features`.

## Phase 8: Trace-To-Algorithm Generalization

Delivered:

- normalize runtime events, portable memory conversations, compiled guides,
  and Agent transcripts into one `ExecutionTrace` model;
- intern operations as link addresses and preserve unique trace boundaries;
- find maximal non-overlapping repeated episodes, including loop bodies inside
  a single long trace;
- bound exhaustive mining to 4,096 observed steps and 32 steps per candidate,
  failing closed without learning from a partial prefix;
- infer invariant constants, varying parameters, and reused parameters that
  preserve cross-step data flow;
- reserve two occurrences for support and use later exact or same-entry traces
  only as held-out tests;
- preserve failed value and structural candidates instead of discarding the
  evidence.

Acceptance gate: `tests/unit/issue_531_algorithm_discovery.rs` proves the
link-native compression, parameterization, loop case, constant drift, changed
operation, missing step, and source-adapter contracts.

## Phase 9: Safe Auto-Learning And Execution Boundary

Delivered:

- content-address candidate schemas and their complete evidence independently;
- parse and integrity-check portable discovery artifacts;
- materialize bindings through side-effect-free conformance;
- retain validated proposals in default-on dreaming as
  `algorithm_learning_candidate` events;
- require a green named automated gate plus named human approval before an
  `ApprovedAlgorithm` can call an explicit host;
- keep failed proposals, artifact tampering, missing bindings, failed gates,
  declined approval, and unnamed reviewers fail-closed.

Acceptance gate: artifact, promotion, generic-host, dreaming, and public CLI
tests pass; no discovery or conformance path can reach the host.

## Phase 10: Formal AI And Real Agent CLI Replay

Delivered:

- add `formal-ai learn algorithms` and `formal-ai algorithm conformance`;
- teach the general Agent planner to recognize actual embedded `demo_memory`
  observations instead of one English request phrase;
- make the planned session write observations, invoke learning, read and parse
  the artifact back, and invoke conformance over the same candidate;
- replay the whole task through the deterministic in-repo Agent driver;
- add `experiments/agent_cli_e2e/run_issue_531.sh` to the mandatory CI job so
  the real `@link-assistant/agent` client drives `formal-ai serve`, executes the
  public binary commands, and leaves the validated artifact in its workspace.

Acceptance gate: the in-repo whole-task test and real-client script both reach
`conformance_passed`, show `side_effects "false"`, and leave the candidate
human-gated.

## Further Model Classes, Not Hidden Deferrals

The implemented learner exhaustively searches its bounded model class:
contiguous sequential routines up to 32 steps, including repeated loop bodies,
from runs of at most 4,096 observed steps, with string constants and shared
parameters. Larger inputs fail closed with no partial proposal. Branch
predicates, concurrent partial orders, recursion, temporal constraints,
semantic postconditions, and new host operations need different typed
observations and counterexamples. They are not silently approximated or claimed
by this PR because frequency alone cannot establish their semantics or safety.
The adapters and candidate/gate boundary are reusable when such evidence types
are introduced.

## Risks

- The upstream C# and C++ `CompressingConverter` sources differ in pair
  selection logic. Do not port the C# condition blindly.
- Transformation search can grow combinatorially. Every phase must bound search
  by candidate count, depth, score, or input size.
- Pattern inference should remain a solver method, not an all-purpose hidden
  replacement for the reasoning loop.
- Compression without exact expansion is data loss. Expansion tests are
  mandatory for every converter and deduplication step.
- 2D grids need relative-coordinate semantics; flattening alone will miss
  spatial invariants.
- A common first operation can join otherwise different routines. Treating
  same-entry traces as held-out counterexamples is intentionally conservative:
  ambiguity blocks promotion instead of selecting a convenient cluster.
- Artifact text is untrusted input. Both schema and evidence identities must be
  recomputed before conformance or promotion eligibility is considered.
