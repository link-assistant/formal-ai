---
bump: patch
---

### Changed
- Replaced every pipe-packed multi-value in `data/seed/*.lino` with the canonical
  reference-list form `keyword ("a" "b c" d)`, so multi-values are real links
  instead of in-string separators (issue #398, defect #4). Covers
  `supported_languages`, `tasks`, `languages`, `inputs`, `outputs`, aliases, and
  every other former `"a|b|c"` field; `code` listings remain the sole field that
  may legitimately contain `|`.
- The LiNo reference-list tokenizer now decodes quoted scalars (which may contain
  spaces) across all four parsers (Rust `seed::parser`, `src/web/seed_loader.js`,
  the e2e `lino-seed-parser.mjs`, and migration tooling).

### Added
- `formal_ai::supported_languages()` accessor that reads the declared languages
  from the `agent-info.lino` reference list, replacing ad-hoc `split('|')` parsing
  scattered across the test suite.
- A comprehensive CI guard (`seed_lino_values_never_pipe_pack_multi_values`) that
  fails on *any* `|` in a seed value except the exempt `code` field, so pipe
  packing can never silently return.
