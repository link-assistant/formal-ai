---
bump: minor
---

### Added
- Settings panel can reset each setting to its default individually, or all of
  them at once (issue #386).
- Conversations list can copy the whole dialog as Markdown; with diagnostics
  mode on, reasoning steps are folded in after each AI message (issue #386).

### Changed
- Prompt recognition references *meanings*, not hardcoded word lists. A new
  canonical lexicon (`data/seed/meanings.lino`) defines language-independent,
  self-describing meanings — each `defined_by` other meanings (a closed graph in
  the spirit of relative-meta-logic), grounded in real lexical data
  (`wiktionary`), tagged with the semantic `role`s it plays, and lexicalised in
  every supported language. The program-artifact follow-up gate
  (`src/program_coreference.rs` and its `formal_ai_worker.js` mirror) no longer
  enumerates ~100 per-language words; it asks the lexicon which surface words
  evidence a `program_artifact` and a `program_modification`, so the words live
  once in data while the code understands the concepts (issue #386).
- The prefilled "Report issue" body omits settings already at their shipped
  default (Mode, Status, Diagnostics, Theme, Guess/Follow-up probability,
  Temperature, inference-only Location), folds the worker into the version line
  (`<version> (wasm)`), shortens the attach-memory section to a docs pointer, and
  drops the Reasoning Trace when the dialog was trimmed to fit GitHub's URL cap
  (issue #386).
- Documented the issue #386 case study (`docs/case-studies/issue-386/`) with raw
  data, a reconstructed timeline, the full requirements list, a corrected
  root-cause analysis of the "Отмени сортировку" refusal, and the implemented
  inverse-derivation fix.

### Fixed
- The follow-up "Отмени сортировку" ("cancel the sorting") no longer returns
  `intent: unknown`. Operations now declare their inverse in the seed
  (`cancel_reverse_sort` carries `inverse "reverse_sort"`), and the subtractive
  substitution rules are *derived at runtime* by mirroring the additive ones, so
  a "cancel X" follow-up lowers the accumulated program back through "X" —
  restoring the ascending sort while keeping earlier edits such as the path
  argument. Adding a new cancellable operation is now pure seed data with no new
  control flow, and the behavior is covered across English, Russian, Hindi, and
  Chinese in both the Rust solver and the web worker (issue #386).
