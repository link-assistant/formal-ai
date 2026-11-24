# Isolated native regression admission checkpoint

Issue: https://github.com/link-assistant/formal-ai/issues/1580

Pull request: https://github.com/link-assistant/formal-ai/pull/1581

## Reproduction and correction

`nativeCompileInput()` excluded the complete `rust/tests/` prefix. Consequently,
a PR changing only an isolated native regression source or its embedded fixture
could skip native execution. The new JavaScript regression discovers the actual
workflow targets, resolves their Cargo source paths and embedded `include_str!`
fixtures, then checks admission for each input independently. Before the fix it
fails for `rust/tests/unit/calendar-currency.rs`; after the fix it passes.

The selector now includes these isolated test inputs, the native HTTP replay and
its support inputs, and the admission policy. Existing negative controls retain
unrelated tests, fixtures, scripts and documentation outside native execution.

The focused workflow also runs `synthesis_contracts`, an explicit Cargo target
pointing directly at the twelve existing synthesis specification tests. It reuses
their source and assertions without building the complete unit suite. Those
contracts include multilingual compound answers, derived algebra, filtered object
counts, varied remainder-sale inputs and deterministic repeated requests.

## Verification and limits

- The admission regression fails before the selector correction and passes after.
- The complete policy test file passed 22/22 on one unchanged retry. The first run
  hit its existing 100 ms VM guard in another test; both complete logs are retained.
  Neither timeouts nor assertions were changed.
- The two forced-function HTTP tests passed against the JavaScript CLI.
- Seven affected local gates passed: abbreviations, changelog, both file-size
  checks, CI speed, generated files and runner labels. Rustfmt and whitespace checks
  also passed. Complete logs and hash inventories are retained under
  `experiments/issue-1580/evidence/checks/native-test-admission/`.
- No local Cargo compilation or native execution occurred. The new synthesis
  target and additional native source-operand tests still require hosted CI.
- The first test draft referred to the called job by the wrong ID. Its complete
  error log is retained separately from the actual before-fix admission failure.
- The source commit remains local; the preceding published head passes 23/23 workflows. These
  supplied commands, edits and conclusions earn zero independent-authorship credit.

This checkpoint does not establish completion of issue #1580 or any currently
unverified synthesis behavior.

The Cargo target addition also invalidates the existing governed manifest witness.
The actual fixture-integrity file fails on the changed Cargo SHA before refresh.
The official reviewed writer updates exactly four Cargo fixture hashes; complete
case objects, unsupported objects and native source manifest are otherwise equal.
Both fixture-integrity tests and all 34 identifier tests pass afterward. Complete
before/after logs and the binary predecessor are under `native-test-admission/governed-refresh/`.
This source-fixture verification does not execute the native target.
