# Online Research For Issue 558

Captured 2026-07-04. The goal is not to import another agent framework, but to
identify proven components and design constraints for Formal AI's own
review-gated auto-learning loop.

## Software-engineering agents

### SWE-agent

- Source: <https://arxiv.org/abs/2405.15793>
- Project: <https://github.com/swe-agent/swe-agent>
- Finding: SWE-agent frames automated software engineering as an
  agent-computer-interface problem. The important reusable idea is not a
  specific model prompt, but the interface shape: repository navigation,
  file-edit commands, shell execution, and tests are first-class tools whose
  outputs feed the next step.
- Formal AI relevance: issue #538/PR #601 already moved in this direction with
  the Agent CLI and bounded recipes. Issue #558 needs to generalize that from
  four recipe axes into a repair loop that can open an arbitrary failing trace,
  derive a patch, run tests, and preserve the learning artifact.

### OpenHands

- Source: <https://github.com/OpenHands/openhands>
- Documentation: <https://docs.openhands.dev/overview/introduction>
- Finding: OpenHands is a software-development agent platform with local,
  remote, and cloud backends, plus sandboxing and GitHub-oriented workflows.
- Formal AI relevance: the useful pattern is a separated control plane: the
  agent UI/backends can be swapped while execution stays sandboxed. Formal AI
  should keep its own link-native reasoning core, but the self-healing executor
  should use the same boundary: isolated workspace, explicit permissions,
  generated patch, tests, and human review.

## Learning from failures without weight updates

### Reflexion

- Source: <https://arxiv.org/abs/2303.11366>
- Finding: Reflexion improves future decisions by recording verbal reflections
  over task feedback in memory, instead of updating model weights.
- Formal AI relevance: this maps cleanly to Formal AI's append-only memory and
  Links Notation data model. A failed answer should emit a `repair_case` with
  trace, hypothesis, fix attempt, validation result, and approved lesson. The
  next run should query those lessons before answering the same class of input.

### DSPy

- Source: <https://dspy.ai/>
- Project: <https://github.com/stanfordnlp/dspy>
- Finding: DSPy describes AI systems as modular programs with structured
  signatures and optimizers, rather than hand-maintained prompt strings.
- Formal AI relevance: Formal AI should not embed opaque prompt tweaks as
  "learning". The analogous link-native path is to represent solver methods,
  input/output signatures, examples, metrics, and accepted rewrites as data,
  then optimize or replace a method only when tests and review gates pass.

## Source code as data

### Tree-sitter

- Source: <https://tree-sitter.github.io/tree-sitter/>
- Finding: Tree-sitter builds concrete syntax trees and supports incremental,
  robust parsing across many languages, including Rust, JavaScript, TypeScript,
  HTML, CSS, JSON, and more.
- Formal AI relevance: `meta-language` already brings tree-sitter grammars into
  this repository. Issue #558 should keep using that existing CST engine for
  source-to-links, especially for UI and JS/TS glue where rustdoc JSON does not
  apply.

### rustdoc JSON

- Source: <https://rust-lang.github.io/rfcs/2963-rustdoc-json.html>
- Finding: rustdoc JSON is a structured output for Rust crates and exposes API
  items, docs, visibility, paths, and source spans.
- Formal AI relevance: rustdoc JSON is a better input for "what public API
  exists and how does it relate" than a raw token tree. It should complement,
  not replace, the CST projection: use CST for lossless code shape, rustdoc JSON
  for semantic API inventory, and Links Notation for the repository knowledge
  graph.

### syn

- Source: <https://docs.rs/syn/latest/syn/>
- Finding: `syn` parses Rust tokens into a syntax tree and can print syntax
  trees back into Rust token streams. It is heavily used by procedural macros.
- Formal AI relevance: `syn` is useful for Rust-only transformation and
  round-trip experiments, but should not become a parallel source-of-truth
  ontology. If used, it should feed the same link-native source model and be
  checked against the existing `meta-language` CST path.

### rowan

- Source: <https://github.com/rust-analyzer/rowan>
- Finding: `rowan` is a lossless syntax tree library used by rust-analyzer.
- Formal AI relevance: rowan's green/red tree design is relevant if Formal AI
  needs editable, incremental, lossless source trees. It is a candidate for the
  Links-to-source editor layer after the initial full-file round trip is proven.

## Design conclusion

The smallest credible issue #558 architecture is a human-gated repair loop:

1. Record a failing trace and classify it as a `repair_case`.
2. Map the failing case to source, data, tests, and prior lessons.
3. Generate or modify a link-native method, source artifact, or seed record in a
   throwaway workspace.
4. Rebuild and run acceptance tests.
5. Convert the accepted change into a PR and an approved learning record.

This intentionally avoids autonomous self-modification of the installed binary.
The learning loop can be dynamic, but the recompile/reattach step must remain
observable, testable, reversible, and human-approved.
