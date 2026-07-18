---
bump: minor
---

### Added
- `scripts/check-hardcoded-language.rs` (rust-script) gate that scans `src/` for user-facing prose string literals and fails the build on any literal missing from the committed allowlist, or on an allowlist row whose literal no longer occurs (issue #659, R379)
- `scripts/hardcoded-language-allowlist.txt`: sorted, tab-separated inventory of today's hardcoded natural-language debt, one `<path>\t<text>` row per literal, regenerated with `--write`
- "Check hardcoded natural language" step in the `release.yml` lint job and a matching local-checks entry in `CONTRIBUTING.md`

### Changed
- Migrated the duplicated English fallback answers in `src/engine_responses.rs` into the grounded `data/seed/multilingual-responses.lino` records, now read via `seed::response_for`, proving the R379 burn-down loop (allowlist shrinks as prose moves into meanings)
