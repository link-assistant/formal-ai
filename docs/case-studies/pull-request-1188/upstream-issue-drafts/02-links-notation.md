<!-- repo: link-foundation/links-notation -->
<!-- title: Rust: `Display for LiNo` writes references unquoted, and `escape_reference` is private. The JS `Link.toString()` quotes, and the JS `Link.escapeReference` is public. -->

### Problem

In Rust, `impl Display for LiNo<T>` writes a `Ref` verbatim ([`rust/links-notation/src/lib.rs#L649-L660`](https://github.com/link-foundation/links-notation/blob/5b106e6317da374c04309226e38b4d787e7eb99c/rust/links-notation/src/lib.rs#L649-L660)). Because of that, `LiNo::reference("two words").to_string()` is `two words`, which parses back as two references. The quoting function [`fn escape_reference`](https://github.com/link-foundation/links-notation/blob/5b106e6317da374c04309226e38b4d787e7eb99c/rust/links-notation/src/lib.rs#L1149) is private, so a caller cannot quote one scalar without building a `LiNo` and calling `format_with_config`.

The JS package behaves differently. `new Link('two words').toString()` returns `('two words')` because `toString()` calls `format()`, and `Link.escapeReference` is a public static method. The same program therefore writes different documents in each language.

### Where a consumer works around it

link-assistant/formal-ai, [`rust/src/lino_adapters/links_notation.rs#L74-L89`](https://github.com/link-assistant/formal-ai/blob/7f44ef2bc7440fda778dd2cc8d7f404eeaaa0a0c/rust/src/lino_adapters/links_notation.rs#L74-L89):

```rust
/// Render indentation with the crate's scalar encoder. Calling `LiNo`'s Display
/// would omit escaping, and flattening children would erase the seed hierarchy.
out.push_str(&LiNo::reference(node.name.clone()).format_with_config(config));
```

### Proposal

1. `pub fn escape_reference(reference: &str) -> String`, re-exported at the crate root. This matches JS `Link.escapeReference`. It should be fixed to use the grammar's escape first (see Draft 1).
2. Make `Display` for `LiNo::Ref` go through `escape_reference`, as JS `toString()` does. Alternatively, document that `Display` is a debug form and add `LiNo::to_lino()` as the round-trippable form.
3. Add a test that `parse_lino(&x.to_string())` equals `x` for refs that contain spaces, `:`, `(`, a leading `#`, or are empty.

### Benefit

Rust and JS have the same API. Consumers no longer need a wrap-and-format workaround, and they cannot silently write unquoted multi-word references through `to_string()`/`format!("{}")`.

---
Found while deduplicating general-purpose code in [link-assistant/formal-ai#1188](https://github.com/link-assistant/formal-ai/pull/1188).
