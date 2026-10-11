TASK (tag TWINS): deliver R1188-U16. Every function a module under `js/agentic/` exports has a Rust twin.

`node scripts/check-planner-twins.mjs --list` lists the 88 exported functions that still have no twin. The gate is `check_planner_twins`, and the ceiling lives in `data/meta/planner-twin-ratchet.lino`. Bring the count to 0, or as close as an honest reading allows.

For each listed function, read the JavaScript and find what the Rust root does for the same behaviour. Then take exactly one of these four actions:

1. **The Rust root has the behaviour under another name.** Add a doc comment line `Mirrors \`<rust symbol>\` in rust/src/<path>.rs.` above the JavaScript function. Run `node scripts/check-twin-citations.mjs`, which fails on a citation whose definition does not exist.
2. **The Rust root has the behaviour inline,** inside a larger function. Extract it into a named `fn` with the snake_case name, in the Rust module the JavaScript module mirrors, and call it from where the inline code was. Do not change the behaviour.
3. **The function stands for a Rust language or standard library feature,** so Rust has no `fn` of its own:
   - `str::char_indices`, byte slicing;
   - `#[derive(Debug)]` formatting, `Clone`, `Default`;
   - `serde_json` parsing, an `Option` combinator.

   Add the doc comment line `Rust built-in \`<feature>\`.` above it.
4. **The behaviour is missing from Rust.** Write the Rust twin, with a doc comment naming the JavaScript module, and add a Rust unit test next to the existing tests for that module. JavaScript first means the Rust root must do what the JavaScript does.

When you finish, lower `no_twin_ceiling` to the measured count. Then update the R1188-U16 row in `docs/requirements/issue-1188-user-requirements.md` honestly: Implemented only if the count is 0. Then run the requirement pipeline:

```
node scripts/assemble-requirements.mjs --write && node scripts/generate-requirement-status.mjs --write && node scripts/render-status.mjs --write && node scripts/check-requirement-status.mjs
```

Rules:
- **No cargo and no rust-script locally.** Check Rust with `rustfmt --edition 2024 --check <file>` only, and only on files that declare no `mod`. rustfmt recurses into submodules, so check `git status` afterwards. CI compiles. Write Rust carefully: match the types of the surrounding code and the crate's existing helpers, keep every new `fn` used (or `pub`) so clippy's dead-code lint passes, and add `#[must_use]` on pure `pub fn`s as the neighbours do.
- **Size limits:**
  - Rust files stay at most 1000 lines and other files at most 1500.
  - `rust/src/agentic_coding/planner.rs` is at its limit: do not grow it.
  - The debt ratchet counts `contains("` and `starts_with("` literals in `rust/src`. Use named consts instead.
- **Regenerate after editing JavaScript:**
  - run `node scripts/translate-es.mjs --write` after any JavaScript edit;
  - run `node scripts/translate-js-rust.mjs --write --fetch <module>` for each edited module in the translation scope;
  - finish with `node scripts/translate-js-rust.mjs --verify`.
- **Local tests:** run only the tests next to the JavaScript you change, plus `rust/tests/web/planner-twins.test.mjs` and `node scripts/check-twin-citations.mjs`. Run `node experiments/formal_ai_subagent/local-gates.mjs --only check_debt_ratchet,check_file_size,check_planner_twins,check_readable_code` at the end.
- **Disk:** no repository copies, and delete scratch files.
- **Claims:** claim your files in `experiments/formal_ai_subagent/claims.md`. Other agents:
  - LEXEMES appends lexemes to `data/seed/*.lino`;
  - CIFIX-LOOP commits CI fixes;
  - RENAME may start later and rename files.
- **Formal AI:** use it for the single-line doc-comment insertions (`node experiments/js_dogfood/drive.mjs --dir . --steps 6 "<request>"`, payloads in «»). Log each delegated edit as a row in `docs/case-studies/pull-request-1188/formal-ai-dogfood.md` (rows T580–T619). Log each failure as a gap in `experiments/formal_ai_subagent/gaps.md` and fix it generally in both roots, with a regression test.
- **Do not commit;** LEAD integrates. Report the final count, which action you took for each function (grouped), and every file you changed.
