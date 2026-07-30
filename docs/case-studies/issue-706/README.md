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
slice could be audited without confusing script detection with fluency. Its
detection claim is intentionally limited to an explicit `es` language context;
plain Latin text remains ambiguous. The covered slice contains:

- greetings and identity responses;
- six meaning surfaces (`manzana`, `hola`, `gracias`, `sí`, `pan`, `agua`);
- an uppercase operation phrase;
- concept lookup, math wrapper, and translation round-trip specimens.

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

## Scale path

The registry, generated matrix, CI coverage guard, CI change-parity guard, and
importer now share one authority. Adding a partial language increases the
generated pair count from N² to (N+1)² and makes the test guard discover it
automatically. Promotion to full additionally makes the importer require a
grounded label for every imported meaning. This keeps additions reviewable:
coverage can grow as data shards while runtime control flow stays unchanged.
