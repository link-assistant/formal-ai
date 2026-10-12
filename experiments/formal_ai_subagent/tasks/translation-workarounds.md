TASK (tag TRANSLATE): deliver R1188-U30. Translation between JavaScript, TypeScript, Rust and the meta language is automated. Where relative meta logic (RML) or the meta language (ML) cannot translate yet, a recorded temporary workaround in this repository does it, and it is retired when upstream catches up.

Read first:
- `docs/progressive-delivery.md` (the method and the translation table);
- the 2026-10-08 progressive JPEG architect note;
- `scripts/translate-js-rust.mjs` (the js → rust leg);
- `data/meta/js-rust-translation.lino` (272 of 2501 items translated; its refusal census: `import { … }` 416, JSDoc types 203, `null` 177, unknown names 117, arrow functions 111, function values 110, `new` 83, untagged objects 82, regular expressions 61, and the array methods);
- `scripts/translate-es.mjs` (js → ts).

Work progressively, the whole first and then sharper.

1. **The record.** Create `data/meta/translation-workarounds.lino`: one `workaround` per construct, with the refusal it lifts, the upstream issue in link-foundation/meta-language (or relative-meta-logic) that will retire it, and how it works. Open the upstream issues with `gh issue create` when none exists, and link them.
2. **The mechanism.** Add a pre-translation pass in `scripts/translate-js-rust.mjs` (or `scripts/lib/`). It rewrites a refused construct into an equivalent form the pinned meta-language translates, or completes the emitted Rust after it, for the constructs of the record. Keep it a rule table, not a special case per module. Each rewrite must keep the source and the projection reviewable: the projection header names the workarounds applied.
3. **Biggest first.** Start with `import { … }` and unknown names: sibling items and imports that resolve to a known Rust path through the twin citations of `scripts/check-planner-twins.mjs`. Then `null`/nullish handling, then the array methods with direct Rust iterator forms. After each one, run `--write --fetch` and check that `translated_items` rises and that nothing regresses.
4. **Verify.** Run `node scripts/translate-js-rust.mjs --verify` and the `--check --fetch` path. Projections must still compile in CI (`--compile` runs there, never locally). Pin the workaround table with a test, `rust/tests/web/translation-workarounds.test.mjs`.
5. **Measure the other directions.** Record in R1188-U30 which directions are automated (js → ts, js → rust) and which are not (rust → js, ts → rust). Add a gate that holds `translated_items` to a ratchet if none does yet.
6. **Update the rows.** Update R1188-U30 and R994 in the requirement shards, then run the requirement pipeline and `node scripts/render-progressive-plan.mjs --write`.

Rules: the same as `tasks/generalize.md`.
- No cargo, rust-script or rustc locally.
- Run local tests only next to your change.
- Claim files in `claims.md`.
- Use Formal AI from JavaScript (`experiments/js_dogfood/drive.mjs`) for every single-line edit, and log each one as a ledger row T740–T779. Append your row to the "Edits by hand" table of `docs/case-studies/pull-request-1188/formal-ai-dogfood.md` when you report.
- Don't commit. LEAD integrates.
