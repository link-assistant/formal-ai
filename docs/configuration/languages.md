# Languages and phrasing parity

English, Russian, Chinese, and Hindi are supported language peers. A behavior
is complete only when equivalent intents, tool routing, output rendering, and
UI labels work in every supported language; English is not a privileged
production fallback for a missing language entry.

New phrasings and languages are **data-only** changes under `data/seed/`:

- meanings/roles files carry phrases and their semantic roles;
- `intent-routing.lino` connects meanings to intent slugs;
- `multilingual-responses.lino` contains localized response templates;
- `languages.lino` and environment seed data declare support;
- generated closure files and the web seed mirror are rebuilt from the seed,
  never hand-maintained as an independent behavior table.

Add several naturally different phrasings per intent and language rather than
literal branches in Rust or JavaScript. Then run the hardcoded-language guard,
multilingual intent coverage, language-change parity, web localization, and
the relevant Rust intent tests. The parity checks intentionally fail when a
new route or response exists in only one language.

```bash
rust-script scripts/check-hardcoded-language.rs
node tests/e2e/scripts/check-multilingual-intent-coverage.mjs
node tests/e2e/scripts/check-language-change-parity.mjs
```
