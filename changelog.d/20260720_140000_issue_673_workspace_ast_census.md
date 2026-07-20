---
bump: minor
---

### Added
- A workspace-wide self-AST census (issue #673): `data/meta/self-ast/` now holds one census document per `src/` module plus an index, replacing the single pinned module as the algorithm's view of its own source. Modules under `src/agentic_coding/` are censused at full AST fidelity through the meta-language links network; the rest carry a signature-level census (items, symbols, spans). Every document records its fidelity marker.
- `formal_ai::self_ast_census` with `WorkspaceCensus::compile`/`resolve`, per-module `ModuleCensus`, and a pure `drift_report` so a stale, missing, or orphaned census document is detected without touching the filesystem.
- `cargo run --example regenerate_self_ast_census` regenerates the census incrementally — only the documents whose modules changed are rewritten — and `cargo run --example dump_self_ast_census [reference]` prints the index or a resolved module.

### Changed
- The general planner resolves edit targets through the census index (`resolve_census_target`), so a request naming a bare module file (`method_registry.rs`) is planned against its real workspace path instead of a hardcoded one.
