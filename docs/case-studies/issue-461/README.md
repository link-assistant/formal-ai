# Issue 461 - Russian PHP Hello World Follow-Up

## Timeline

- 2026-06-13: A Russian web user asked `Что ты умеешь делать?`; the assistant
  advertised Hello World generation, then answered `unknown` to
  `На php не получится написать?`.
- 2026-06-26: The issue was reproduced with a native integration test that
  replays the capabilities answer as conversation history.

## Requirements

- The follow-up must not fall through to `unknown`.
- The prompt must inherit the Hello World task from the preceding capabilities
  context.
- PHP must be answered through the existing cached coding oracle, preserving
  source attribution to the Hello World Collection.
- Rust and the browser worker recognizers must remain in parity.

## Root Cause

The write-program formalizer only routed a prompt when the current turn named a
task such as `hello world`, or combined a program-kind noun with a program-request
verb. The reported follow-up named a known oracle language (`php`) and a request
verb (`написать`), but no program noun or task. Because the turn never became an
unsupported `write_program` request, context recovery could not inherit the
Hello World task from the capabilities answer and the oracle was never tried.

## Prior Work

Issue 412 added the cached coding oracle for languages outside the verified
catalog, including PHP. Issue 324 added context recovery for incomplete
write-program follow-ups. This fix connects those existing pieces by allowing a
request verb plus a catalog/oracle-known language to enter the same recovery
path.

## Fix

- Rust `write_program_parameters` now accepts prompts that mention a
  `program_request` verb and a language known by either the verified catalog or
  the cached coding oracle.
- `src/web/formal_ai_worker.js` mirrors the same predicate with the worker's
  local catalog and oracle cache.
- A regression test covers the exact Russian dialog shape and asserts the PHP
  oracle answer.

## Verification

- `cargo test --test integration issue_461_php_followup -- --nocapture`
- `cargo test --test integration issue_412_oracle_languages`
- `cargo test code_generation`
- `node --check src/web/formal_ai_worker.js`
- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features`
- `rust-script scripts/check-file-size.rs`
- `cargo test --all-features --verbose`
- `cargo test --doc --verbose`
