# Issue 531 Case Study: Pattern Inference

Status: research, proposal, and implementation pass for PR #642. The first
session collected evidence, fact-checked the upstream converters, listed
requirements, and proposed a staged plan; this PR then follows that plan
through to a working, tested runtime. It ships a link-native sequence
substrate (`src/sequences/`), associative deduplication, 1D and 2D pattern
inference, a `pattern_inference` solver handler wired into the dispatch table,
and the pattern-vocabulary ontology in the seed knowledge. The research and
proposal material below is retained for traceability; the "planned" phase
language in the requirement tables is annotated where a phase is now
implemented.

## Source Material

- Issue: [#531](https://github.com/link-assistant/formal-ai/issues/531).
- Prepared PR: [#642](https://github.com/link-assistant/formal-ai/pull/642).
- Raw issue, PR, upstream repository, and source excerpts are saved in
  `docs/case-studies/issue-531/raw-data/`.
- The upstream sequence reference is
  [linksplatform/Data.Doublets.Sequences](https://github.com/linksplatform/Data.Doublets.Sequences),
  checked at commit `6a6a69fc3ce42b0bd3e421c17c810ec2f37cb12b`.
- Related theory repositories captured in raw data:
  [link-foundation/meta-theory](https://github.com/link-foundation/meta-theory)
  and
  [link-foundation/relative-meta-logic](https://github.com/link-foundation/relative-meta-logic).

## Problem Statement

Formal AI already stores facts and events as link-like structures, but it does
not yet have a link-native sequence layer that can infer repeated structure,
compress repeated sub-sequences, compare transformed variants, or generalize the
same machinery from 1D text to 2D image grids. Issue #531 asks for that work to
start from associative deduplication and from the existing Doublets sequence
ecosystem instead of inventing an isolated algorithm.

## Findings

- `src/link_store.rs` is the current boundary for importing/exporting doublet
  records and selecting a native `doublets-rs` backend.
- `src/substitution.rs` already models data-driven link-pattern substitution,
  which is the closest local analogue to a future pattern matcher.
- `src/solver.rs` and `src/meta_core.rs` already describe simplification,
  recursive work units, method selection, and evidence recording. Pattern
  inference should plug into those methods rather than bypassing them.
- `src/solver_handlers/text_manipulation.rs` has ordinary line deduplication,
  but it is string-level only. It does not build link-native sequences,
  compression traces, or reusable structural patterns.
- The C# sequence package has three converter families worth porting or
  adapting: `BalancedVariantConverter`, `OptimalVariantConverter`, and
  `CompressingConverter`.
- The C# and C++ copies of `CompressingConverter` differ in the max-frequency
  selection condition. The C++ version selects larger frequencies as expected;
  the captured C# source appears to compare in the opposite direction in
  `UpdateMaxDoublet`. That discrepancy must be verified before porting.
- Prior art maps cleanly to the request: SEQUITUR infers hierarchical repeated
  phrases, Re-Pair repeatedly replaces frequent pairs, and ARC-AGI supplies
  small transformed 2D grids that exercise rotation, reflection, translation,
  and analogy-like operations.

## Requirements And Plans

The full decomposition is in `requirements.md`. The implementation direction is
in `solution-plan.md`, and the current-code/upstream inventory is in
`architecture-inventory.md`.

At a high level, the recommended path is:

1. Add a link-native sequence substrate with unique symbol initialization,
   sequence markers, sequence indexing, and round-trip tests.
2. Port the safe parts of Data.Doublets.Sequences converters, beginning with
   balanced sequence construction and then adding optimal and compressing
   variants after frequency-selection behavior is verified.
3. Implement associative deduplication as a compression trace that can expand
   back to the original sequence and can be stored in Links Notation.
4. Extend matching over transformations for 1D text and 2D grid projections:
   reverse, shift, substitution, rotation, reflection, translation, and
   center-relative symmetry.
5. Ground ontology terms such as sequence, pattern, repetition, compression,
   symmetry, rotation, translation, analogy, and invariant in seed data.
6. Add benchmarks and fixtures before wiring the method into broader solver
   behavior.

## Verification

- The raw research artifacts are preserved under `raw-data/`.
- `REQUIREMENTS.md` records issue #531 rows R396-R407.
- `tests/unit/docs_requirements_issue_531.rs` keeps the root requirements,
  case-study files, and raw-data evidence connected.
