---
bump: minor
---

### Added

- The request is now formalized into obligations instead of matched phrases
  (issue #1166, E131). `rust/src/intent_formalization/obligations.rs` parses
  an issue body into an `ObligationGraph`: one ledger `ObligationNode` per
  enumerated clause, each classified against the seed lexicon (`print_stdout`,
  `ci_workflow_request`, the program-request roles, plus documented
  multilingual fallback tables for style, naming, and badge vocabulary), with
  quoted output literals anchored to their value and mentions of the same
  literal coreferenced to exactly one node.

### Fixed

- The Kotlin doubled-output root cause of #1156: the literal to print was
  collected once per mentioning clause, so an issue body naming
  `Hello, World!` twice (in "print exactly" and in the Expected Output block)
  printed it twice. `unique_output_literal` now returns the value once no
  matter how many clauses carry it.
- A prompt starting with a shell token is no longer classified as a terminal
  command when it formalizes into work obligations — a quoted output literal
  plus an authoring clause is a sentence about building something, including
  in languages the English-only marker words of the #1175 guard do not cover;
  `git status` and `echo "Hello, World!"` stay commands.
