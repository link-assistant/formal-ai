TASK (tag NOTATION): the links notation we own should be human readable, concise and deduplicated, with full English words and `-` preferred over `_`. Apply this by rule (substitution passes), not by hand.

Measured on 2026-10-08:
- `data/seed` holds 228 `.lino` files, about 5.3 MB together with `data/meta`.
- About 22,900 names use `_` and about 6,900 use `-`.
- The most frequent structural lines are `text` (17,464), `surface` (14,200), `lexeme` (5,175) and `language` (4,769).

Do:
1. **Measure first, as a ratchet:** `scripts/measure-notation.mjs` with `--check`. It reports, per directory:
   - names with `_`;
   - abbreviated words (a seeded list of abbreviation → full word, e.g. `pos` → `position`, `qid` stays as a proper term);
   - total characters;
   - duplicated subtrees, i.e. identical child blocks repeated across links.

   Record the values in `data/meta/notation-ratchet.lino` (direction down), and add a CI gate.
2. **`_` → `-` in names we own.** A name is a link name, key or slug in `data/seed` / `data/meta`, and the same name as an exact string literal in JS and Rust code that reads it: role constants, intent ids, slugs.
   - Write the substitution as a rule file (`data/meta/notation-rules.lino`), applied by `experiments/formal_ai_subagent/apply-notation-rules.mjs`, with `--check`.
   - It rewrites the `.lino` files, their `rust/embedded` and `js/seed` mirrors, and the matching quoted string literals in `js/`, `ts/` (via `translate-es --write`), `rust/src` and `rust/tests`.
   - Never touch identifiers that are code syntax (Rust/JS variable or function names follow their own language conventions), external formats (OpenAI-compatible JSON field names, Wikidata ids, HTTP headers), or quoted human text.
   - Generalize the parsers where needed, so that both spellings are accepted during the transition and one canonical spelling is written.
   - Go one seed family at a time (meanings, responses, handler rules, ledgers), each as its own rule application. After each family, run only the tests that read that family (grep for the converted names). CI runs the rest. LEAD commits between families.
3. **Concise form for the most repeated structure:**
   - The pattern `lexeme <lang>` / `surface` / `text <word>` costs about 37k lines. Design a shorter equivalent that stays readable, e.g. `<lang> "word" "other word"` under a `words` link.
   - Implement the reader in the JS seed parser first, then the Rust one, accepting both forms.
   - Convert by rule. Measure the characters saved, and make sure readability is not lost (show before and after samples in the report).
   - Deduplicate repeated subtrees by naming them once and referencing them, where it is clearly more readable.
4. **Full English words in names we own,** in code and notation: apply the seeded abbreviation list as a rule, in the same way as step 2. Start with the notation and the JS; the Rust changes follow the JS twin.
5. **Document the notation style** in `docs/links-notation-style.md`, or extend the existing doc if one exists: `-` over `_`, full words, concise forms, deduplication. Fix the docs that say otherwise.

Rules:
- **No cargo and no rust-script locally;** Rust is checked by rustfmt and CI only.
- JS first, then the Rust twin, with the same behaviour.
- Every family conversion must keep the relevant `local-gates.mjs --only` twins and the tests that read the family green. CI runs the rest (owner, 2026-10-08: minimal local tests).
- Ledger rows T390–T419. Use Formal AI for the rule files and the small edits.
- Stop and report after each family, so LEAD can commit.
- Do not commit.
