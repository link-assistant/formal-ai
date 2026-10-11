TASK (tag GENERALIZE): deliver R1188-U1 and the open part of R344. Generalize, don't specialize.

1. **The 23 prompts held verbatim in code.**
   - `node scripts/check-prompt-specialization.mjs --list` lists the prompts that tests send to Formal AI and that sit verbatim in code under `rust/src/` or `js/`. Examples: "What is your name?", "Что делаешь в свободное время?", "Tell me about yourself", "Show behavior rule unknown", "Write me hello world program in Rust".
   - For each one, read the code that holds it. Replace the specialization with the general rule. The words a request class is recognized by become seed lexemes of a meaning, in all five languages (en, ru, hi, zh, es). The answer becomes a seeded response row. The code reads the role.
   - A literal that is an example in a help text, rather than a branch, becomes a seeded example the help reads.
   - Change both roots together: the JS worker (`js/worker/*.js`) and Rust.
   - The pinned tests must still pass unchanged. A test may only gain cases, such as a paraphrase in another language that the general rule now answers.
   - Lower `specialization_ceiling` in `data/meta/prompt-specialization-ratchet.lino` to the measured count; aim for 0.
2. **The 14 pending handlers of `data/meta/handler-migration-ledger.lino`** (R344, issue #699).
   - Migrate as many as you can to seed rules, smallest first, the way the MIGRATE4 batches did: a `handler-rules.lino` rule set, a five-language response, the rule interpreter in both roots, and a browser twin.
   - Keep the `handler_files` and `try_dispatch_entries` ratchets falling. Update the ledger rows and the R344 row honestly.
3. **Update the requirement rows** R1188-U1 and R344 with the measured numbers. Then run the requirement pipeline:

   ```
   node scripts/assemble-requirements.mjs --write && node scripts/generate-requirement-status.mjs --write && node scripts/render-status.mjs --write && node scripts/check-requirement-status.mjs
   ```

Rules:
- **No cargo and no rust-script locally.** Check Rust with `rustfmt --edition 2024 --check` only, on files that declare no `mod`.
- **Size limits:** Rust ≤1000 lines and other files ≤1500. Worker line budgets fall or are raised only with a recorded reason. Use named consts instead of `contains("`/`starts_with("` literals (debt ratchet).
- **Seeds:**
  - Mirror every seed edit to `rust/embedded/data/seed/`, `js/seed/` and `packages/formal-ai-engine/assets/seed/`.
  - Regenerate `data/seed/roles.lino` with `python3 scripts/generate-role-registry.py`.
  - Keep `python3 scripts/check-closure-audit.py` at its reviewed value by grounding every new token in a meanings record.
- **JS:** run `node scripts/translate-es.mjs --write` after JS edits, and `node scripts/translate-js-rust.mjs --write --fetch <module>` for edited modules in the translation scope.
- **Local tests:**
  - Run only the tests next to your change.
  - Then run `node experiments/formal_ai_subagent/local-gates.mjs --only check_prompt_specialization,check_debt_ratchet,check_file_size,check_hardcoded_language,check_worker_line_budget,check_response_parity_debt,check_language_parity,check_closure_audit,check_readable_code`.
  - CI runs the rest.
- **Disk:** no repository copies.
- **Claims:** claim your files in `experiments/formal_ai_subagent/claims.md`. Other agents:
  - TWINS adds Rust twins of `js/agentic/` functions;
  - CIFIX-LOOP commits CI fixes;
  - RENAME renames files by rule, so re-read paths before editing.
- **Formal AI:** use it for single-line seed edits and ledger rows (`node experiments/js_dogfood/drive.mjs --dir . --steps 6 "<request>"`, payloads in «»). Log each delegated edit as a row in `docs/case-studies/pull-request-1188/formal-ai-dogfood.md` (rows T620–T659). Log each failure as a gap in `experiments/formal_ai_subagent/gaps.md` and fix it generally in both roots, with a regression test whose file cites the row id. Then run `node scripts/tally-formal-ai-dogfood.mjs --write`.
- **Do not commit;** LEAD integrates. Report the counts before and after, what moved to seed, and every file you changed.
