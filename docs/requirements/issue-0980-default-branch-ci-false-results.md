## Issue #980 Default-Branch CI False Results

Issue [#980](https://github.com/link-assistant/formal-ai/issues/980) audits the
seven default-branch workflows named in the issue, compares the full Rust, JS,
and Python pipeline-template trees, and fixes every actionable error found. The
complete run logs, template snapshots, timeline, requirements ledger, and root
cause analysis live in `dev/log/issues/980/pulls/981/`.

| ID | Requirement | Status / Evidence |
| --- | --- | --- |
| R980-1 | Download and inspect every referenced workflow run, including warnings and retry-masked flakes. | Implemented: all seven run records and complete logs are preserved under `dev/log/issues/980/pulls/981/ci-logs/`; run 31186108359 is the only failed workflow and its findings name the deterministic formatter error, external-search interception, and permission-state race. |
| R980-2 | Fix every actionable failure without hiding real failures behind retries. | Implemented: the rejected Rust line is formatted, `issue-282.spec.js` blocks cross-origin providers during local WASM parity checks, and `issue-541-permissions-cold-start.spec.js` waits for the observable pending-task state rather than returning after the user-message append. |
| R980-3 | Compare all CI/workflow/script files with the Rust, JS, and Python templates and Hive Mind practices. | Implemented: complete template tree snapshots at Rust `c867f78`, JS `7b70923`, and Python `98d6dca`, plus searchable control indexes and the Hive Mind guide, are preserved in the evidence bundle. Applicable workflow controls were already adopted by PRs 809 and 971; no new template-owned defect was found and therefore no duplicate upstream issue was filed. |
| R980-4 | Prevent recurrence with per-defect and composed verification. | Implemented: `tests/unit/ci-cd/issue_980.rs` pins all three source invariants; the actual browser specs exercise the external-network boundary and pending-task boundary end to end. Focused repetition passed 12/12 opener cases and 9/9 permission cases. |
