<!-- repo: link-foundation/lino-objects-codec -->
<!-- title: `format::escape_reference` and `unescape_reference` are not inverses, and `escape_reference` leaves a leading `#` and the empty string unquoted (Rust and JS) -->

### Problem

`format::escape_reference` / `escapeReference` ([Rust `rust/src/format.rs#L40`](https://github.com/link-foundation/lino-objects-codec/blob/3d52c62c94bfbfe00b42c7a795327f6af0189eba/rust/src/format.rs#L40), [JS `js/src/format.js#L35`](https://github.com/link-foundation/lino-objects-codec/blob/3d52c62c94bfbfe00b42c7a795327f6af0189eba/js/src/format.js#L35)) and `unescape_reference` / `unescapeReference` ([Rust `#L94`](https://github.com/link-foundation/lino-objects-codec/blob/3d52c62c94bfbfe00b42c7a795327f6af0189eba/rust/src/format.rs#L94), [JS `#L91`](https://github.com/link-foundation/lino-objects-codec/blob/3d52c62c94bfbfe00b42c7a795327f6af0189eba/js/src/format.js#L91)) are documented as a pair ("Reverses the escaping done by escape_reference"). They have these defects:

1. `unescape_reference` does not strip the delimiters it is given, and it replaces both `""` and `''` no matter which delimiter was used. Doubled quotes of the other kind are therefore corrupted.
2. `escape_reference` does not quote a value that starts with `#`. Since links-notation 0.23 that reads as a comment. `(r: #tag)` fails to parse with "Syntax error at line 1, column 10".
3. `escape_reference("")` returns an empty string, so the value disappears from the document. links-notation writes `""` for this case.

### Reproduction (JS `main`, `js/src/format.js`)

```js
import { escapeReference, unescapeReference } from 'lino-objects-codec';
for (const v of ['say ""hi""', `a''b"`, "it's"]) {
  const e = escapeReference({ value: v });
  console.log(JSON.stringify(v), '->', e, '->', JSON.stringify(unescapeReference({ str: e })));
}
// "say \"\"hi\"\"" -> 'say ""hi""' -> "'say \"hi\"'"   MISMATCH (quotes kept, "" collapsed)
// "a''b\""         -> "a''b"""     -> "\"a'b\"\""      MISMATCH ('' collapsed though "" was the delimiter)
// "it's"           -> "it's"       -> "\"it's\""       MISMATCH
escapeReference({ value: '#tag' }); // '#tag'  -> reads as a comment
escapeReference({ value: '' });     // ''      -> value disappears
```

The Rust bodies are identical, so they give the same results.

### Proposal

The crate now has three quoting rules: `format::escape_reference`, the private `format::format_indented_value` / `quoteReference`, and `readable::quote` (from PR #66). links-notation has a fourth. Make `format::escape_reference` quote exactly when links-notation would, which includes a leading `#`, the empty string and `\t`/`\r`, and have it quote through `readable::quote`. Make `unescape_reference` strip the outer delimiter and undouble only that delimiter, or deprecate it in favour of `decode_line`. Add a property test in both languages: `unescape_reference(escape_reference(s)) == s` and `parse(escape_reference(s))` is one reference equal to `s`.

### Benefit

Consumers that already call `format::escape_reference` for record headers and code fields get correct output without changes. In link-assistant/formal-ai these are [`rust/src/agentic_coding/code_artifact.rs#L355-L362`](https://github.com/link-assistant/formal-ai/blob/7f44ef2bc7440fda778dd2cc8d7f404eeaaa0a0c/rust/src/agentic_coding/code_artifact.rs#L355-L362), which writes code fragments and so meets Python `# ...` lines and empty strings, and [`rust/src/intent_formalization.rs#L111-L116`](https://github.com/link-assistant/formal-ai/blob/7f44ef2bc7440fda778dd2cc8d7f404eeaaa0a0c/rust/src/intent_formalization.rs#L111-L116). There is also its JS mirror at [`js/agentic/code_artifact.mjs#L29`](https://github.com/link-assistant/formal-ai/blob/7f44ef2bc7440fda778dd2cc8d7f404eeaaa0a0c/js/agentic/code_artifact.mjs#L29).

---
Found while deduplicating general-purpose code in [link-assistant/formal-ai#1188](https://github.com/link-assistant/formal-ai/pull/1188).
