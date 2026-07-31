# Issue 706: any-language protocol

This case study replaces the four-language coordination contract with one
registry: `data/seed/languages.lino`. It does not claim that every registered
language has complete conversational coverage.

## N → N+1

1. Put the candidate data in `data/language-additions/<code>.lino`.
2. Run the dry run:

   ```sh
   node scripts/language-protocol.mjs \
     --language ar \
     --candidate data/language-additions/ar.lino \
     --dry-run
   ```

3. Add the covered lexemes, response intents, detection metadata, and operation
   phrases to seed data. Register the language as `partial`; uncovered meanings
   must resolve to `language_gap`, never an English response presented as native
   output.
4. Generate and review the evidence:

   ```sh
   node scripts/language-protocol.mjs --language es --write
   node scripts/language-protocol.mjs --language es --check
   ```

5. Promote `status partial` to `status full` only after the complete importer
   catalog and response catalog pass. The bulk lexeme importer derives its full
   language list from the same ledger, so that promotion is a data edit.

The generated matrix contains each language→meta→same-language route and all
ordered source/target pairs. Rust tests execute the same registry-derived
matrix against the real translation pipeline using the seeded `apple` meaning.

## Fifth-language proof

Spanish was selected as the fifth language because a useful Latin-script
slice could be audited without confusing script detection with fluency. The
covered slice contains:

- greetings and identity responses;
- six meaning surfaces (`manzana`, `hola`, `gracias`, `sí`, `pan`, `agua`);
- an uppercase operation phrase;
- concept lookup, math wrapper, and translation round-trip specimens.

Spanish detection needs no Rust edit either. `src/language.rs` no longer
enumerates languages: it embeds `data/seed/language-detection.lino` and derives
the whole detector from it. A rule declares its `script`, Unicode range,
`markers`, and whether it is the `fallback`. Spanish shares the Latin script
with the fallback language, so its range only widens the Latin count to the
accented letters and the vote is carried by its markers (`¿`, `¡`, `ñ`, `qué`,
`hola`, …). `data/seed/languages.lino` therefore records `detection_mode
script_and_markers`, not the earlier `explicit_language_context` workaround.
The JS worker (`src/web/worker/formal_ai_worker_00.js`) and the `no_std` WASM
worker read the same registry, so all three surfaces agree by construction.

### Honest gaps instead of silent English

`seed::localized_response(intent, language)` implements the ledger's
`fallback_policy explicit_gap`: exact language → the seed's `language unknown`
"unsupported language" record → English as the last resort. Handlers that
previously wrote `response_for(intent, language).or_else(|| response_for(intent,
"en"))` answered a Spanish speaker in English without saying so; all 43 such
sites now route through the helper.

All six declared suites pass, recorded as 1000 permille in
`coverage-es.lino`. Meaning coverage remains `partial`, and the response
registry contains a Spanish `language_gap` message for uncovered meanings.
This is the honest boundary: the report measures the declared multilingual
suite, not general Spanish fluency.

## Sixth-language dry run

`data/language-additions/ar.lino` demonstrates the next addition without
source-code changes. Arabic has a distinct Unicode block and passes five of six
suites (833 permille); the missing math-wrapper suite emits `language_gap`.
The candidate is not registered or shipped as supported data.

## Auto-learning a new language's request frames

Registering a language makes it *detectable*; it does not teach the engine the
request frames that language phrases questions with. Issue #701 already built a
general adoption cycle for exactly that problem, and that cycle never mentions
Google Trends — it takes a frontier slug and a list of recorded prompts. So the
language half needed a second **recorded frontier**, not new learning logic.

1. The contributor adds a prompt corpus to `data/language-additions/<code>.lino`
   (`prompt` records with `rank`, `query`, `variation`, `prompt`). Every `query`
   must already be a committed surface in `data/seed/meanings-translation.lino`,
   so the corpus adds request frames only — it cannot smuggle in vocabulary.
2. `cargo run --example issue_706_language_frontier` runs the **live engine**
   over that corpus and records only the prompts it actually fails, into
   `data/meta/learning-frontier-language-gap.lino`. Nothing is asserted by hand;
   a prompt that already routes never reaches the frontier. A candidate language
   with no corpus is preserved as an explicit `frontier_gap` naming what is
   missing — the same honesty rule as `language_gap`.
3. `formal-ai learn cycle --frontier language-gap` replays the record through
   the *same* cycle the trends frontier uses. `--frontier` is an open registry
   (`learning_cycle::recorded_frontiers`), not a closed Rust enum, so a third
   frontier is a registration, not a refactor. Over the Spanish corpus it
   derived two frames by query deletion — `qué es …` and `cuéntame sobre …` —
   each supported by two prompts, each validated on the held-out remainder, and
   emitted them as human-gated promotion proposals.
4. Adoption is a seed edit: the two frames were written into
   `data/seed/learned-request-openers.lino`.
5. `data/meta/language-adoption-ledger.lino` pins the capability delta. It reads
   "before" from the frozen frontier record and produces "after" live through
   the production solver path: 7 of 7 prompts leave the unknown path and recover
   their term (`unknown_to_web_search`), 0 unadopted. Re-recording the frontier
   from the same corpus now yields `learning_frontier "0"` — the loop is closed,
   and that empty re-record is the proof rather than a claim.

The committed frontier record is deliberately frozen at its pre-adoption state,
because it is the "before" half of the ledger. `tests/unit/issue_706_any_language.rs`
enforces every step of this, including the byte equality of the pinned ledger.

## Self-hosting

The protocol invariant these leaves were reviewed against was authored by Formal
AI itself, driven by the real Agent CLI against the production-mode server; the
raw client and server traces are in
[`self-hosting-authorship/`](self-hosting-authorship/README.md).

## Scale path

The registry, generated matrix, CI coverage guard, CI change-parity guard, and
importer now share one authority. Adding a partial language increases the
generated pair count from N² to (N+1)² and makes the test guard discover it
automatically. Promotion to full additionally makes the importer require a
grounded label for every imported meaning. This keeps additions reviewable:
coverage can grow as data shards while runtime control flow stays unchanged.
