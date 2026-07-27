---
bump: minor
---

### Added
- Dialogue world model (issue #702): `src/world_model_atoms.rs` classifies each
  turn (fact / wish / confirmation / correction / state query) from the
  `world_state_*` cue sets in `data/meta/cue-lexicon.lino`, and
  `src/world_model_dialog.rs` maintains current and target contexts as links
  networks with provenance back to the turn that asserted each fact, a
  hash-chained append-only synchronization log, merge with conflict detection,
  split, relative-meta-logic recalculation of dependent statements, and
  `forecast` for action-consequence prediction.
- `world_state` chat handler: "what is left to do?" (en/ru/hi/zh) is answered
  from the current→target difference and backed by `world_state:*` evidence
  links, never from remembered prose.
- bAbI-style world-state tracking benchmark slice
  (`data/benchmarks/world-state-tracking-suite.lino`, 16 self-authored dialogues
  with held-out paraphrases) with a `minimum_pass_count` ratchet, catalogued in
  `docs/benchmarks.md`.
- Case study `docs/case-studies/issue-702/` and requirements R702-1 … R702-10 in
  `REQUIREMENTS.md`.
- Unlimited nested symbolic contexts with full, isolated, or conditional
  inheritance; lazy nearest-first reference resolution returns an inspectable
  local trace and requires explicit permission before an outside lookup.
- Review-gated issue-702 auto-learning report derived through the shared
  associative-memory pipeline and reproduced through the real Agent CLI.

### Changed
- `SolverConfig` gains `world_model_mode` (`WorldModelMode::{Off, Track}`, env
  override `FORMAL_AI_WORLD_MODEL_MODE`). It defaults to `Off`, so the feature is
  trace-only until opted in and existing behaviour is unchanged.
- Dialogue turns, solver coreference, and agentic research-topic recall now use
  the common context hierarchy. Rust and browser coding-language inheritance no
  longer stop after four scopes; both use cycle-safe visited sets.
- Promoting dialogue-local current state into shared general memory now fails
  closed unless `GeneralMemoryPermission::Allowed` is supplied explicitly.
