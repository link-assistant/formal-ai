---
bump: minor
---

### Changed

- Generalized the installation-conversion command recognizer
  (`installation_conversion::looks_like_command`) from a fixed tool-prefix
  whitelist to a provenance-aware structural rule: any well-formed command line
  is accepted regardless of which tool it invokes, while prose lines are rejected
  even when they mention a tool (issue #433).
- Replaced the install-step description table (`describe_command`) with verb/object
  intent inference, so unseen but recognizable tools (`bun install`, `pdm install`,
  `just build`, `zig build`) get accurate step descriptions without extending a
  table.
- Mirrored the structural recognizer and verb/object describer into the browser
  worker (`src/web/formal_ai_worker.js`) for cross-runtime parity.

### Added

- Case study `docs/case-studies/issue-433/` with (1) an audit classifying every
  specialized handler recognizer as fixed-enumeration vs compositional, (2) the
  installation-conversion generalization, and (3) a documented reconstruction of
  the `numeric_list` coding handler from the meta-algorithm rule primitives.
- False-positive (prose) and unlisted-tool regression coverage for the
  installation-conversion recognizer across the Rust unit suite, the
  source-mirror private-function suite, and the browser-worker experiment.
