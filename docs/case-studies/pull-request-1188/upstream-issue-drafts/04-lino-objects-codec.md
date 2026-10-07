<!-- repo: link-foundation/lino-objects-codec -->
<!-- title: `format_indented` (Rust) and `formatIndented` (JS) write different documents for the same record, and JS has no `format_indented_ordered` -->

### Problem

For the input `id = "rec"`, `{ status: "ok", note: "two words" }`:

| | output |
|---|---|
| Rust `format::format_indented_ordered("rec", &[("status","ok"),("note","two words")], "  ")` ([`rust/src/format.rs#L194`](https://github.com/link-foundation/lino-objects-codec/blob/3d52c62c94bfbfe00b42c7a795327f6af0189eba/rust/src/format.rs#L194)) | `rec` / `  status "ok"` / `  note "two words"` (no colon, values always double-quoted via the private `format_indented_value`) |
| JS `formatIndented({ id: 'rec', obj: { status: 'ok', note: 'two words' } })` ([`js/src/format.js#L528`](https://github.com/link-foundation/lino-objects-codec/blob/3d52c62c94bfbfe00b42c7a795327f6af0189eba/js/src/format.js#L528)) | `rec:` / `  status ok` / `  note 'two words'` (header colon, values bare unless needed, single quotes) |

JS also has no ordered-pairs variant. A plain object cannot keep caller order, because integer-like keys always move first: `formatIndented({ id: 'rec', obj: { name: 'x', 2024: 'y' } })` gives `rec:\n  2024 y\n  name x`.

### Where a consumer works around it

link-assistant/formal-ai needs byte-identical records from its Rust and JS runtimes, so it re-implements the Rust `escape_reference`, the private `format_indented_value` and `format_indented_ordered` in JS instead of using the npm package: [`js/agentic/crate/links_format.mjs#L1-L45`](https://github.com/link-assistant/formal-ai/blob/7f44ef2bc7440fda778dd2cc8d7f404eeaaa0a0c/js/agentic/crate/links_format.mjs#L1-L45). This mirrors [`rust/src/links_format.rs#L1-L21`](https://github.com/link-assistant/formal-ai/blob/7f44ef2bc7440fda778dd2cc8d7f404eeaaa0a0c/rust/src/links_format.rs#L1-L21).

### Proposal

- Add `formatIndentedOrdered({ id, pairs, indent })` to JS, taking `Array<[key, value]>`, with output identical to the Rust function.
- Pick one canonical flat-record shape (header colon or not, which quoting rule) and use it in both languages. Ideally it uses the corrected quoting from Draft 3. If JS `formatIndented` keeps its #35 recursive behaviour, give the flat form its own name in both languages.
- Add a shared fixture test (`fixtures/` already exists) that runs the same input through both implementations and requires identical bytes.

### Benefit

The codec could serve as the single cross-runtime record writer. Consumers with Rust and JS/WASM runtimes would stop keeping ports of private codec functions, which in formal-ai is about 45 lines in JS plus the Rust shim.

---
Found while deduplicating general-purpose code in [link-assistant/formal-ai#1188](https://github.com/link-assistant/formal-ai/pull/1188).
