<!-- repo: link-foundation/meta-language -->
<!-- title: Self-translated Rust should pass clippy pedantic: #[must_use], no redundant parentheses, a minimal allow line instead of `unused` -->

### Problem

Self-translation (PR #196, `js/src/self-translation.js` at `ddcde32`) writes Rust that a consumer building with clippy's `pedantic` and `nursery` groups cannot commit as it is. The corpus's own expected output, `parity/self-translation/expected/arithmetic-to-rust.rs`, shows it:

```rust
// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

pub fn add(a: f64, b: f64) -> f64 {
    (a + b)
}

pub fn is_rust(name: String) -> bool {
    (name == "Rust")
}
```

- `clippy::must_use_candidate` (pedantic) asks for `#[must_use]` on a public function that returns a value and has no mutable argument, so it applies to both functions.
- `clippy::missing_const_for_fn` (nursery) applies to `add`, because floating-point arithmetic is allowed in `const fn` since Rust 1.82.
- The parenthesized tails are rustc's `unused_parens` ("unnecessary parentheses around block return value"). They pass only because the prelude allows the whole `unused` group. That allow also hides `dead_code`, `unused_variables` and `unused_imports` for every item in the translated module.

The Rust side of the corpus (`rust/tests/unit/self_translation.rs`) runs clippy at `-D warnings` with the default lint set, so these never fail upstream.

### Consumer need

link-assistant/formal-ai builds with `pedantic` and `nursery` at `-D warnings`. It copied the self-translation format (header, translated and carried blocks, hash-restored round trips, the `cases.lino` corpus) in [link-assistant/formal-ai#1188](https://github.com/link-assistant/formal-ai/pull/1188), `scripts/self-translation/`. To make the Rust committable, it emits:

- `#[must_use]` on every function;
- `&str` for string parameters, `String` for results;
- parentheses only where precedence needs them;
- an allow line computed from the constructs emitted, for example `#![allow(clippy::float_cmp, clippy::missing_const_for_fn)]`.

The emitted Rust is compiled as a test module there, under the same clippy and rustfmt gates as hand-written code. Any consumer with a stricter lint profile than the default needs the same changes before it can use the upstream translator instead of a copy.

### Request

Make the Rust emitter write lint-clean code by default, or behind an option:

1. `#[must_use]` on a value-returning `pub fn`.
2. No parentheses around a tail expression or an operand that precedence already binds.
3. `const fn` where the body allows it, or `clippy::missing_const_for_fn` named in the prelude.
4. A prelude allow line that lists only the lints the emitted constructs trigger, never the `unused` group.

Acceptance: `parity/self-translation/expected/*.rs` passes `cargo clippy -- -W clippy::pedantic -W clippy::nursery -D warnings`, and the round trips still restore their sources byte for byte.

---
Found while adopting meta-language's self-translation practices in [link-assistant/formal-ai#1188](https://github.com/link-assistant/formal-ai/pull/1188) (`docs/case-studies/pull-request-1188/conversion-best-practices.md`).
