## Issue #895 Coverage Publication And Ratchet

Issue [#895](https://github.com/link-assistant/formal-ai/issues/895) (child of
[#710](https://github.com/link-assistant/formal-ai/issues/710)) makes the
"double the tests toward 100%" requirement enforceable. CI generated an LCOV
file and uploaded it, but nothing read the numbers, so no threshold could fail
and no decrease could be detected. The gate lives in
`scripts/check-coverage-ratchet.rs`, the reviewed floors in
`coverage/baseline.json`, and the design rationale in
[`docs/design/coverage-ratchet.md`](../design/coverage-ratchet.md).

| ID | Requirement | Status / Evidence |
| --- | --- | --- |
| R895-1 | Publish coverage in both a human-readable and a machine-readable form. | Implemented: `coverage/summary-<name>.md` (per-metric table of covered/total counts, measured percentage, baseline, delta in percentage points, status; the ten least-covered files; the inventory result) is appended to `$GITHUB_STEP_SUMMARY` and uploaded, alongside `coverage/summary-<name>.json` carrying the same metrics plus per-file counts. Regressions are also emitted as `::error::` annotations. |
| R895-2 | Check a baseline threshold and reject decreases unless an explicit reviewed baseline update is included. | Implemented: `scripts/check-coverage-ratchet.rs` compares measured line and function percentages against `coverage/baseline.json` and exits `1` on a drop beyond `tolerance_percent`. `--update-baseline` raises a floor freely but refuses to lower one without `--justification "<reviewed reason>"`, which is recorded as `lowered_reason` so the decrease reaches review as a sentence in the diff. An empty denominator is a hard error rather than `0%`. |
| R895-3 | Cover Rust and browser production paths, or document separate honest denominators. | Implemented as two denominators, never averaged. `rust` measures the workspace via `cargo llvm-cov`. `browser` measures `src/web/` via `tests/web/`, which loads the unbundled page scripts and the 24-module worker mirror through `node:vm` under their real repository paths so V8 attributes coverage to the files the browser downloads, and boots the worker through `src/web/formal_ai_worker.js` with the canonical `data/seed/*.lino` corpus behind `fetch`. |
| R895-4 | The browser denominator must not be narrowed silently. | Implemented: every `src/web/**/*.{js,jsx}` file must be measured or listed in `coverage/browser-unmeasured.txt` as a `path<TAB>reason` row. Modeled on `scripts/hardcoded-language-allowlist.txt`, the gate fails on an undeclared, missing, stale or unexplained row, so the list can shrink but never grow silently. Generated bundles are excluded as build output and their sources measured instead. |
| R895-5 | Add regression tests for threshold enforcement and baseline updates. | Implemented: `scripts/check-coverage-ratchet-tests.rs` covers LCOV parsing, the threshold and tolerance band, the published Markdown and JSON reports, the refusal to lower without a justification, the recording and later clearing of `lowered_reason`, and the inventory gate; a final case validates the committed `coverage/baseline.json`. Run as `rust-script --test scripts/check-coverage-ratchet.rs` in the `lint` job. |
