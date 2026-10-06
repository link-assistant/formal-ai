## Standing Doctrine: JavaScript First, Full Parity, Then Translate (2026-10-06)

Stated by the project owner while continuing PR #1188 (quoted in
[the 2026-10-06 architect note](../architect-notes/2026-10-06-javascript-first-full-parity.md)).
It sharpens the 2026-09-24 three-roots doctrine (R992-R996): JavaScript is not only a full root
but the **first** one. Requirements are implemented and tested in JavaScript
first, and Rust follows by translation through the meta language — or, until
the translator covers a construct, by a port that names its JavaScript
original. The stated reason is the same iteration speed, plus local resources:
Rust builds run in CI on push, not on the developer machine.

| ID | Requirement | Status / Evidence |
| --- | --- | --- |
| R997 | Every requirement of the coding, QA, reasoning and text-transform issues is implemented in the JavaScript worker before (or together with) its Rust twin; the browser handler registry carries no `null` ("native surface only") row for a handler the Rust engine has. | Delivered 2026-10-07: every handler row the Rust engine has is a JavaScript twin; `scripts/check-js-parity.mjs` measures 0 native-only rows against `js-parity-ratchet.lino` ceiling 0. The server surface follows under R1013-R1015. |
| R998 | The number of `null` rows in the browser handler registry (`workerHandlerRegistryDefinition()` in `js/worker/formal_ai_worker_20.js`) only falls: `scripts/check-js-parity.mjs` fails when it exceeds the ceiling in `data/meta/js-parity-ratchet.lino`, and asks for the ceiling to be lowered when it is below. | Delivered 2026-10-06; runs in the js tier of `.github/workflows/layered-ci.yml`. |
| R999 | R995 is amended: a worker module's line ceiling in `data/meta/worker-line-budget/` may rise when the growth is a JavaScript twin of a native handler, and the shard's rationale names the handler keys. The budget still forbids unexplained growth. | Doctrine row, 2026-10-06. |
| R1000 | The js → ts leg of translation runs in JavaScript (`scripts/translate-es.mjs`, a byte-for-byte twin of `rust/src/es_tokenizer.rs` + `es_meta.rs` `write_source`), so `ts/` regenerates without compiling the crate; CI proves the two translators render identical bytes. New JavaScript handler modules are written in the portable subset link-foundation/meta-language's JavaScript → Rust translator accepts (top-level functions with JSDoc types, `const`/`let`, tagged unions dispatched by `switch`, no classes, generators or destructuring) so the js → rust leg becomes mechanical when that translator is released. | js → ts twin delivered 2026-10-06 (`rust/tests/web/translate-es.test.mjs`, layered-ci ts and rust tiers). js → rust awaits [meta-language#196](https://github.com/link-foundation/meta-language/pull/196) (open draft). |
