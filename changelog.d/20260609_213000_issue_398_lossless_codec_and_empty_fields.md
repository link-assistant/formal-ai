---
bump: patch
---

### Changed
- Made the JSON ↔ Links Notation cache codec (`formal_ai::json_lino`) losslessly round-trip the *entire* Wikidata/Wiktionary snapshot — `forms`, `senses`, `claims`, and every metadata key — instead of the previous lexeme-only projection. `data/cache/**/*.lino` were regenerated as full native snapshots (e.g. `L3412` 6 → 195 lines), empty arrays/objects/nulls are never emitted, and the Wiktionary source JSON is pretty-printed multi-line.
- Replaced the circular round-trip test (which compared the lino to the converter's own lossy output) with one that rebuilds the full original JSON from the lino and asserts key-for-key equality with the raw `.json` (`wikidata_lino_cache_rebuilds_full_json_losslessly`, `wiktionary_cache_is_pretty_printed_and_rebuilds_full_json`, and the `verify_cache_roundtrip` example).
- Migrated every meaning header in `data/seed/**/*.lino` from the YAML-style trailing-colon form (`monday:`) to native Links Notation nodes (`monday`), removing all 428 empty colon redefinition fields tree-wide. The transform is parse-equivalent (`parse_colon_definition` already mapped `monday:` to `(name = "monday", id = "")`) and regenerates the embedded browser-worker seed. `seed_lino_files_have_no_empty_redefinition_fields` now enforces the reviewer's exact `^\s*[\w-]+:\s*$` regex.

### Added
- Added `scripts/migrate-empty-redefinition-fields.rs`, a re-runnable whole-tree migration that strips trailing-colon redefinition headers and refreshes the `src/web/formal_ai_worker.js` embed.
