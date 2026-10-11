---
bump: minor
---

### Added
- Repository history import records `Co-Authored-By:` trailers as `coauthor:<name>` evidence, and names the top-level items a commit added, removed or modified (`symbol:<path>#fn name`, `#impl Display for Answer`, `#function load`) from seeded item keywords in `data/seed/history-formalization.lino` (issue #1180).
- `formal-ai repository-history import` and `formal-ai repository-history query <SQL>` run the incremental importer and a read-only ANSI-SQL memory query over its store.
- `js/agentic/crate/history_context.mjs` is the JavaScript twin of the importer's pure core (rules seed, `git log` parser, trailer and co-author scans, named-item diff, commit formalization), with node tests in `rust/tests/web/agentic-history-context.test.mjs`.

### Fixed
- The history rules seed rows were never read: `HistoryRules::from_seed_text` did not descend under the `history_formalization` root, so the built-in defaults always applied.
