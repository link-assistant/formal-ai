---
bump: patch
---

### Fixed
- Collapsed every `facet <kind>` wrapper whose child was an empty-bodied colon redefinition (`word_surface:`, `lexical_sense:`, ...) into native Links Notation `subject predicate` lines (`notation word_surface`, `denotation lexical_sense`). This removes the valueless `concept:` shape the review banned, across the whole `data/seed` tree.

### Changed
- Taught the Rust seed consumer (`parse_semantic_facets`) to read the direct `<kind> <target>` subject-predicate form in addition to the legacy `facet <kind>` wrapper, de-duplicating targets so both forms project identical facets.
- Added `scripts/migrate-empty-facet-fields.rs`, a std-only re-runnable migration that performs the collapse tree-wide and regenerates the embedded browser worker fallback (`src/web/formal_ai_worker.js`).

### Added
- Added a tree-walking CI guard (`seed_lino_files_have_no_empty_redefinition_fields`) that fails when any `data/seed/**/*.lino` line is an empty-bodied colon field with no deeper-indented child, with no hard-coded filename.
