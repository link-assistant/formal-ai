# Issue 531 Solution Plan

This plan separates the broad issue into staged work that can be reviewed and
tested independently.

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

Use small, deterministic fixtures before broad benchmarks:

- text repeated phrase examples;
- symbolic sequences with nested repetition;
- event streams from existing link records;
- ARC-AGI inspired grid examples;
- requirements-to-solution fact-checking examples where repeated missing
  obligations should be detected.

Acceptance gate: benchmark fixtures are versioned, deterministic, and report
both solved cases and rejected candidates.

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
