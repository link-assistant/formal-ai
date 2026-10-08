TASK (tag LEXEMES): close the meaning-lexeme language debt: every meaning that owns lexemes has them in en, ru, hi, zh and es.

## State (2026-10-08)
- `data/meta/language-parity-debt.lino` lists 910 `uncovered_behavior` rows:
  - 758 rows miss only `es`;
  - 138 rows miss `ru,hi,zh,es`;
  - a few rows miss `en` or other languages.
- The debt ratchet (`language_parity_gaps` in `data/meta/debt-ratchet.lino`) holds it at 910.
- The checker `scripts/check-language-parity.rs` (with `scripts/language-parity-lib.rs`) is Rust only, so it cannot run locally.

## Do
1. **JS twin of the checker.** Write `scripts/check-language-parity.mjs`, a byte-for-byte twin of `check-language-parity.rs`: same output, same exit code, and a `--write` mode that regenerates the ledger if the Rust script has one.
   - Add a parity gate `data/meta/ci-gates/check-language-parity-js-twin.lino`, following the other `*-js-twin` gates and `scripts/lib/checks-twin-parity.mjs`; it runs the Rust half only in CI.
   - Register the twin in `experiments/formal_ai_subagent/local-gates.mjs` if it needs a mapping.
2. **Add the missing lexemes.**
   - Write each meaning's missing language as a `lexeme <language>` block with `surface` / `text` entries, after the meaning's last lexeme.
   - Follow the wording the existing languages use: the same sense, natural forms, several common surfaces where the other languages list several.
   - Batch by seed file, with a small rule script `experiments/formal_ai_subagent/add-lexemes.mjs <seed.lino> <language> <table.tsv>` (the twin of `add-translations.mjs`). Keep the tables under `experiments/formal_ai_subagent/lexemes/`.
   - Mirror every changed seed file byte for byte to `rust/embedded/data/seed/`, `js/seed/` and `packages/formal-ai-engine/assets/seed/` wherever the file exists.
   - Regenerate the ledger and lower the `language_parity_gaps` ceiling to the measured value after each batch.
3. **Careful areas.**
   - A new surface can change routing: a Spanish word that equals a word of another language, or a very short word, may trigger a role on unrelated prompts.
   - Avoid one- and two-letter Latin surfaces unless the meaning really is that word.
   - After each batch, run the routing probe tests next to the changed roles (find them with grep for the role name), for example `rust/tests/web/issue-1175-claim-routing.test.mjs` and the multilingual intent coverage tests. CI runs the rest.
4. Entries that genuinely have no translation, such as a proper name or a language-specific opener, stay on the ledger with a reason, if the ledger format allows one; otherwise report them.
5. Update the language coverage status (Spanish is `partial` in `docs/status.md`), and any requirement row about Spanish parity, honestly.

## Rules
- **No cargo and no rust-script locally.** Run only the tests next to the change. Mind the disk; no repository copies.
- **Size:** files at most 1500 lines. Split a seed file by meaning category if it would pass that, register the new file in `data/meta/seed-registry.lino`, and run `node scripts/generate-seed-registry.mjs --write`.
- **Gates:** `local-gates.mjs --only check_debt_ratchet_js_twin,check_file_size_js_twin,check_seed_registry_js_twin,check_hardcoded_language_js_twin` plus your new twin, and `python3 scripts/check-closure-audit.py`.
- **Formal AI:** use it for small edits; ledger rows T540–T579; failures go in `gaps.md`.
- **Claims:** claim the seed files in `claims.md`.
- **Others:** REQ-ROUTE appends to `data/seed/meanings-requirement-extraction.lino`, and TEXT-CAPABILITY owns `data/seed/text-formalization.lino`; leave both out.
- **Do not commit.** Report after every 300 rows closed, so LEAD can commit.
