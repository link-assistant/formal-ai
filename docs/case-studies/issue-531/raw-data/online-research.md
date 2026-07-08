# Issue 531 Online Research

This file records the online sources used in the first issue #531 research pass.
It is intentionally concise; raw GitHub API snapshots and upstream source
excerpts are stored as neighboring raw-data files.

## Data.Doublets.Sequences

- Source: <https://github.com/linksplatform/Data.Doublets.Sequences>
- Observed status: Unlicense repository; latest GitHub release reported by
  GitHub CLI was `csharp_0.6.5`, published 2025-01-26.
- Checked commit: `6a6a69fc3ce42b0bd3e421c17c810ec2f37cb12b`, dated
  2025-07-19, with commit message "Update target framework from net7 to net8."
- Relevant components:
  - `BalancedVariantConverter`
  - `OptimalVariantConverter`
  - `CompressingConverter`
  - `LinkFrequenciesCache`
  - `SequenceIndex`
  - `StringToUnicodeSequenceConverter`
- First-pass finding: the C# and C++ `CompressingConverter` copies disagree in
  the max-frequency comparison. The C++ version matches the expected "choose
  higher frequency" behavior. The C# copy should be verified before Rust porting.

## SEQUITUR

- Source: <https://arxiv.org/abs/cs/9709102>
- Relevance: SEQUITUR incrementally infers a hierarchical grammar from a
  sequence by replacing repeated phrases. It is a strong conceptual match for
  issue #531's "start from sequences and associative deduplication" request.
- Use in Formal AI: useful as prior art for hierarchical compression traces and
  exact expansion checks. It should inform tests and scoring, not replace the
  link-native data model.

## Re-Pair

- Source: <https://pmc.ncbi.nlm.nih.gov/articles/PMC12330530/>
- Relevance: Re-Pair style grammar compression repeatedly replaces frequent
  adjacent pairs. That is very close to `CompressingConverter` and to the issue's
  requested associative deduplication baseline.
- Use in Formal AI: useful for choosing repeated-pair fixtures, tie-breaking
  rules, and performance expectations for frequency caches.

## ARC-AGI

- Sources:
  - <https://arcprize.org/arc-agi>
  - <https://github.com/fchollet/arc-agi>
- Relevance: ARC-AGI tasks are small 2D colored-grid problems built around
  abstraction, transformation, symmetry, object movement, and analogy. They are
  a useful benchmark family for the 2D portion of issue #531.
- Use in Formal AI: start with tiny hand-curated ARC-style fixtures for
  rotation, reflection, translation, and color remapping before attempting a
  larger benchmark integration.

## Meta-Theory And Relative-Meta-Logic

- Sources:
  - <https://github.com/link-foundation/meta-theory>
  - <https://github.com/link-foundation/relative-meta-logic>
- Relevance: issue #531 explicitly asks to consider meta-theory and
  relative-meta-logic. The first pass preserved repository metadata so later
  implementation work can align sequence pattern inference with the broader
  Link Foundation theory vocabulary.

## Local Search Summary

- No existing Formal AI module currently implements link-native sequence
  compression or transformed pattern inference.
- Existing local components to reuse are `src/link_store.rs`,
  `src/substitution.rs`, `src/solver.rs`, `src/meta_core.rs`, and
  `src/solver_handlers/text_manipulation.rs`.
- The current text deduplication operation is a useful baseline but not a
  replacement for sequence-level compression.
