# Issue 531 Online Research

Research refreshed 2026-08-02 for the PR review asking for algorithm discovery
from logs/events/guides and integration with auto-learning. Repository snapshots
and upstream excerpts live beside this file.

## Link Sequences And Grammar Compression

### Data.Doublets.Sequences

- Source: <https://github.com/linksplatform/Data.Doublets.Sequences>
- Checked commit: `6a6a69fc3ce42b0bd3e421c17c810ec2f37cb12b`
  (2025-07-19).
- Relevant components are `BalancedVariantConverter`,
  `OptimalVariantConverter`, `CompressingConverter`, `LinkFrequenciesCache`,
  `SequenceIndex`, and `StringToUnicodeSequenceConverter`.
- The captured C# and C++ `CompressingConverter` implementations disagree in
  the maximum-frequency comparison. The Rust implementation follows the
  behavior supported by explicit frequency tests and lossless expansion rather
  than copying the disputed condition.

### SEQUITUR and Re-Pair

- SEQUITUR paper: <https://arxiv.org/abs/cs/9709102>
- Re-Pair analysis: <https://arxiv.org/abs/1811.01472>
- Re-Pair implementation analysis: <https://arxiv.org/abs/2202.08447>

Both families replace repeated sequence structure with reusable grammar
symbols. SEQUITUR emphasizes hierarchical grammar constraints; Re-Pair
repeatedly replaces a most-frequent adjacent pair. Formal AI adopts the latter
as its small deterministic compression proof, including non-overlapping
replacement and exact expansion. Compression is evidence that a routine is
structurally reusable; it is not, by itself, evidence that executing the routine
is safe or correct.

## Event Logs, Discovery, And Conformance

### XES event data

- IEEE 1849-2023 standard page:
  <https://standards.ieee.org/ieee/1849/10907/>
- Process-mining community XES overview:
  <https://www.tf-pm.org/resources/xes-standard/for-researchers/first-xes>

XES establishes an extensible event-log/stream interchange model. Formal AI
does not add an XES dependency in this PR: its portable `demo_memory` already
has case (`conversation_id`), activity (`tool`/`kind`), input, output, and event
identity fields. The adopted design decision is nevertheless the same: normalize
ordered sources at an adapter boundary before applying discovery.

### Workflow/process discovery

- Workflow-mining paper: <https://ieeexplore.ieee.org/document/1316839/>
- Inductive Miner: <https://doi.org/10.1007/978-3-642-38697-8_17>
- Process-model quality dimensions:
  <https://doi.org/10.1155/2013/507984>
- Process Mining Manifesto:
  <https://processmining.org/old-version/files/mao-process-mining.pdf>

Process mining distinguishes model discovery from conformance checking and
evaluates more than observed frequency: fitness, precision, generalization, and
simplicity are recurring quality dimensions. This directly changed the
implementation. Two occurrences may infer a candidate, but at least one
excluded observation must test it. Contradictory value or structure is retained
as a failed candidate. Maximal-window filtering provides a small simplicity
pressure. Precision beyond argument consistency, richer process structure, and
statistical generalization scores are not claimed here.

### Learning models from long traces

- Paper: <https://arxiv.org/abs/2001.05230>

This work combines trace segmentation and program synthesis to obtain concise
models from long execution traces. It supports treating event boundaries and
episode segmentation as first-class rather than flattening unrelated cases.
Formal AI uses unique trace-boundary links for the compression proof and mines
within each case only. Its first model class is intentionally narrower:
contiguous sequences with argument data flow, not arbitrary automata.

## Reusable Program Libraries And Auto-Learning

### DreamCoder

- Paper: <https://arxiv.org/abs/2006.08381>

DreamCoder's wake/sleep loop learns reusable symbolic abstractions that improve
future program search, including replay or imagined tasks. The relevant lesson
is architectural: retained experience should yield reviewable library
candidates. Formal AI's idle dreaming pass now creates
`algorithm_learning_candidate` memory events. It does not adopt neural
recognition, probabilistic search, or automatic promotion.

### LAPS

- Paper: <https://arxiv.org/abs/2106.11053>

LAPS combines language with learned program libraries. This reinforces using
compiled guides and demonstrations as inputs to one learner. Formal AI adapts
`CompiledProcedure` into the same `ExecutionTrace` used for runtime logs, while
keeping operation semantics in its reviewed seed/host vocabulary.

### ReGAL

- Paper: <https://arxiv.org/abs/2401.16467>

ReGAL refactors groups of programs into shared abstractions and uses execution
to verify/refine them. Formal AI adopts the separation between abstraction
proposal and executable verification: it infers constants/parameters, writes a
portable candidate, integrity-checks it, and offers side-effect-free
conformance. Actual host execution additionally requires held-out success,
automated gates, and named human approval.

## 2D Transformation Benchmark Context

- ARC-AGI project: <https://arcprize.org/arc-agi>
- ARC dataset repository: <https://github.com/fchollet/ARC-AGI>

ARC-style colored grids remain the benchmark context for the first-pass 2D
rotation, reflection, translation, symmetry, and analogy implementation. That
transform matcher and the new event-algorithm learner share link-native sequence
primitives but solve different model classes; the event learner does not claim
to infer an arbitrary ARC program from demonstrations.

## Link Foundation Theory Context

- Meta-theory: <https://github.com/link-foundation/meta-theory>
- Relative meta-logic: <https://github.com/link-foundation/relative-meta-logic>

Issue #531 explicitly requested both. Repository metadata is preserved in the
neighboring JSON files. The practical alignment in this PR is representation:
operations, boundaries, compressed pairs, roots, schemas, and evidence all have
link identities. No theorem-proving capability from those repositories is
claimed or copied.

## Project-Local Fact Check

The design was compared with `VISION.md`, `ROADMAP.md`, `CONTRIBUTING.md`, and
existing implementations rather than added as an isolated subsystem:

- `EventLog` is already the universal solver's ordered record.
- `MemoryEvent`/`MemoryStore` is the cross-interface persistent observation
  format and feeds default-on dreaming.
- `skill_procedure` already compiles freely phrased guides into canonical steps.
- `agentic_coding::driver` already retains actual requested tools and inputs.
- `learning_cycle`, procedure learning, execution learning, and search-fusion
  learning already enforce proposal/held-out/human-review boundaries.

The new core therefore normalizes those sources, reuses `src/sequences/`, and
extends dreaming and the public/Agent CLI surfaces. It does not introduce a
second memory store, an operation-specific generated Rust branch, a network
dependency, or a hidden path around promotion.

## Conclusions Applied In Code

1. Repetition proposes; held-out conformance decides whether the proposal is
   currently supported.
2. Trace/case boundaries are semantic and may not be compressed into invented
   cross-case routines.
3. Argument vectors carry useful invariants and data-flow relations that an
   operation-name-only grammar loses.
4. Counterexamples must remain visible, including truncated continuations.
5. Learned libraries can improve later behavior only through an auditable,
   integrity-checked, human-gated path.
6. Branching, concurrency, recursion, temporal logic, and new operation
   semantics remain distinct model classes; claiming them from contiguous
   frequency alone would not be supported by this evidence.
