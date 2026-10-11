<!-- repo: link-foundation/meta-language -->
<!-- title: Self-translation types an unannotated parameter from one item's body, so the Rust signature can contradict every caller -->

### Problem

When a function has no JSDoc `@param` type, the portable core infers the parameter's type from how the item's own body uses it (`js/src/translation/javascript-infer.js`, main at `679a3b3c`). A parameter that nothing in the body constrains becomes a Number. A parameter the body iterates with `for…of` or indexes becomes an array of strings. Self-translation checks each item alone, so the callers elsewhere in the module, which do constrain the type, are never seen. The Rust signature is then a guess, and the item is still reported as `translated`.

From link-assistant/formal-ai's modules, as they were before JSDoc types were added:

```js
function pluralize(count, singular, plural) {
  return count === 1 ? singular : plural;
}
// called as pluralize(n, 'file', 'files')
```

```rust
pub fn pluralize(count: f64, singular: f64, plural: f64) -> f64 { … }
```

```js
function escape(text) {
  let out = '';
  for (const character of text) { … }
  return out;
}
// called with a string
```

```rust
pub fn escape(text: Vec<String>) -> String { … }
```

Both compile. Neither can be called the way the JavaScript is called, and nothing in the item list or the output says the types were guessed.

### Consumer need

link-assistant/formal-ai counts translated items as progress and ratchets the count ([link-assistant/formal-ai#1188](https://github.com/link-assistant/formal-ai/pull/1188), `data/meta/js-rust-translation.lino`). Its first measurement found 17 translated items whose signatures were inferred. 9 of them contradicted their callers: six took string parameters typed `f64` (one compares two kind names with `<`), and three took a string typed `Vec<String>`. With JSDoc types added, 5 of the 9 translate with `String` parameters, and the other 4 are carried (string indexing, iteration and ordering are outside the core). That is the honest result. The consumer now flags translated items that lack `@param` types, but it has to detect them heuristically from the source lines.

### Request

1. Refuse a parameter whose type nothing constrains (`type: declare the type of <name> with JSDoc`), instead of choosing Number. Or emit it generically.
2. When self-translating a module, infer a parameter's type from the module's call sites as well, or refuse when the item's own body is the only evidence.
3. Record in the item list (and in the translated marker) which parameters had inferred types, so a consumer can require declared types without parsing the source.

Acceptance: the two functions above, unannotated, are either carried with a "declare the type" reason or translated with `&str`/`String` parameters, and a corpus case pins it.

---
Found while wiring the js -> rust leg in [link-assistant/formal-ai#1188](https://github.com/link-assistant/formal-ai/pull/1188) (`docs/case-studies/pull-request-1188/js-rust-translation.md`).
