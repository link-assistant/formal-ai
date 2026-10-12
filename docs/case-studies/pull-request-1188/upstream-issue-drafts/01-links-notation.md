<!-- repo: link-foundation/links-notation -->
<!-- title: Formatting a reference that holds both `'` and `"` writes a backslash escape that the parser cannot read, in JavaScript and in Rust -->

### Problem

The grammar escapes a delimiter inside a quoted reference by doubling it. It also has the n-quote form. There is no backslash escape (see `rust/links-notation/src/quotes.rs`: "a run of 2N inside it is that delimiter escaped"). The formatter's `escapeReference` / `escape_reference` takes another path when a reference contains both quote kinds. It writes `\'`:

- JS: [`js/src/Link.js#L114-L117`](https://github.com/link-foundation/links-notation/blob/5b106e6317da374c04309226e38b4d787e7eb99c/js/src/Link.js#L114-L117): `` return `'${reference.replace(/'/g, "\\'")}'`; ``
- Rust: [`rust/links-notation/src/lib.rs#L1174-L1177`](https://github.com/link-foundation/links-notation/blob/5b106e6317da374c04309226e38b4d787e7eb99c/rust/links-notation/src/lib.rs#L1174-L1177): `return format!("'{}'", reference.replace('\'', "\\'"));`

As a result, `formatLinks` / `format_links` produces documents that the library's own parser reads back as different links.

### Reproduction (JS, `main`)

```js
import { Parser, Link, formatLinks } from 'links-notation';
const text = formatLinks([new Link('r', [new Link(`it's "x"`)])]);
console.log(text);                                             // (r: 'it\'s "x"')
console.log(new Parser().parse(text)[0].values.map(v => v.id)); // [ 'it\\', 's', 'x', "'" ]   <- 4 refs, value lost
console.log(new Parser().parse(`(r: 'it''s "x"')`)[0].values.map(v => v.id)); // [ `it's "x"` ]  <- doubling works
```

Rust has the same branch, so `format_links(&parse_lino_to_links(doc)?)` corrupts any reference that contains both quote kinds.

### Proposal

Use the grammar's own escape in both languages. One option is to double the chosen delimiter. Another is to open an n-quote run longer than the longest inner run. lino-objects-codec already implements the n-quote option as `readable::quote` (Rust `rust/src/readable.rs#L364`). Add a property test in both languages: for any string `s`, `parse(format(Link(r, [Link(s)])))` yields exactly `s`.

### Benefit for consumers

Every consumer that writes free text, such as code fragments, prose or chat logs, through `format_links` gets lossless output. Today it would need its own quoting layer. link-assistant/formal-ai keeps one for this reason: [`rust/src/links_format.rs#L23-L52`](https://github.com/link-assistant/formal-ai/blob/7f44ef2bc7440fda778dd2cc8d7f404eeaaa0a0c/rust/src/links_format.rs#L23-L52). Its comment notes that "renderers that hand-rolled a C-style one therefore emitted documents a reader ends early or rejects outright".

---
Found while deduplicating general-purpose code in [link-assistant/formal-ai#1188](https://github.com/link-assistant/formal-ai/pull/1188).
