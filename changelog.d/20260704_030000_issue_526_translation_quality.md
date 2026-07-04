### Added
- Added issue #526 round-trip translation quality requirements, natural/code translation regression coverage, and case-study documentation.

### Changed
- Reworked code translation (`translate_program`) to route through a language-neutral code meta language (`CodeMeaning` / `formalize_code_meaning` / `render_code_meaning`) instead of direct `(source, target)` pairs, so it stays at `N` formalizers + `N` renderers and pairs like Python → JavaScript or Rust → Go translate through one shared meaning.
