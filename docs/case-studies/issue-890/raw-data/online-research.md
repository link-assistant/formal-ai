# Issue 890 Online Research

## Intermediate Representations

- LLVM, *LLVM Language Reference Manual*:
  <https://llvm.org/docs/LangRef.html>

  LLVM documents a common intermediate representation whose equivalent forms
  can be transformed independently of their source syntax. Formal AI uses the
  same architectural boundary at a much smaller scale: `FormalProof` is the
  semantic value, and Rust/Python programs are projections rather than the
  proof itself.

## Executable Assertions

- Rust standard library, `assert!` macro:
  <https://doc.rust-lang.org/stable/core/macro.assert.html>

  Rust's runtime assertion is always checked. The generated Rust proof uses it
  to validate that its witness satisfies the encoded bounds before printing.

- Python language reference, `assert` statement:
  <https://docs.python.org/3/reference/simple_stmts.html#the-assert-statement>

  Python defines `assert condition` as a conditional `AssertionError` in
  non-optimized execution. The test invokes normal `python3`, so the generated
  program checks the same witness invariant as the Rust program.

## Repository Prior Art

- Issue [#403](https://github.com/link-assistant/formal-ai/issues/403) and PR
  [#420](https://github.com/link-assistant/formal-ai/pull/420) introduced the
  Russian integer-interval solver but stopped at a localized prose proof.
- Issue [#526](https://github.com/link-assistant/formal-ai/issues/526) and PR
  [#635](https://github.com/link-assistant/formal-ai/pull/635) established the
  `source -> CodeMeaning -> target` rule and the `N` formalizers plus `N`
  renderers constraint used here.
- PR [#794](https://github.com/link-assistant/formal-ai/pull/794) established
  source-first multilingual routing and the native/browser parity precedent.

All external sources above are official primary documentation. They informed
the representation and executable-verification design; no external source code
or prose was copied into the implementation.
