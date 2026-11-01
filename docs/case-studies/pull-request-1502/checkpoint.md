# PR #1502: file-command routing and main qualification

## Requirements

Fix all seven reported cases in [#1501](https://github.com/link-assistant/formal-ai/issues/1501): preserve ordinary listing; normalize an unambiguous typo in the directory adjective without rewriting quoted paths; report unsupported bulk deletion without listing or false completion; search literal content recursively; return an observed bounded file count; honor the requested line limit; and refuse unowned compound shell input. Preserve existing dedicated-tool priority. Commands require real tool outcomes; unsupported inputs grant no effects.

Use a shared finite grammar and operand binding rather than exact issue-request lookup. Keep JavaScript, TypeScript and Rust behavior paired, with shared original and held-out vectors. Local verification uses focused, sequential JavaScript tests only. Rust compilation, native tests, application and Docker builds belong in CI.

Attempt small development tasks in English through JavaScript Formal AI. If unsupported, improve its reusable JavaScript logic, retry the unchanged request, then maintain native parity. Supplied engine/bootstrap scripts receive zero autonomous synthesis credit. Do not generalize one successful edit into a success-rate or token-cost claim.

## Delivered draft

The file-command grammar binds typed operands for listing, search, line-limited reads and file counts. Count traversal stops at 4096 entries, closes its iterator and reports a gap on invalid status/output. Bulk deletion is explicitly unsupported. Shell operators and substitutions in bare command input are refused. Existing dedicated listing/search tools retain priority. All original requests and held-out variants have shared JavaScript/native vectors.

A new bounded named-import operation uses the complete readable source representation, derives one insertion from observed syntax, reparses the whole source, checks the exact inverse, preserves unrelated comments/whitespace and performs fresh readback. The unchanged English request initially made zero tool calls; after the engine improvement it performed Read → Edit → Read and inserted the requested import. The defect came from a coordinator bootstrap in this branch. This is a real derived edit, not a claimed new live-main defect repair. The accompanying proof separates its 31 useful bytes from externally authored engine/bootstrap changes. Arbitrary Rust argument wrapping remains unsupported; general self-coding and #1483 acceptance are not claimed.

## Main failure fixes

The merged source 116e86043f5b58bfebdf586a66e52f73abd05eba failed native compilation at the source-repair optional preimage and an unused import. Both are corrected. Supported maintenance and function-expectation routes now precede generic repair intake; incidental examples cannot authorize unknown repair reads. Implicit Node-host setup reuses an already installed explicit grammar policy; explicit policy conflicts still fail.

Protocol lexical markers have one shared data owner consumed by both languages; incomplete searches use explicit state rather than diagnostic-prefix interpretation. The literal-predicate ceiling is not raised. The pinned JavaScript translator regenerated the stale engine-answer projection without compiling Rust. The progressive-plan projection was regenerated rather than hand-editing its counters.

Public failure references: [native compilation](https://github.com/link-assistant/formal-ai/actions/runs/41317019270), [captured JavaScript regressions](https://github.com/link-assistant/formal-ai/actions/runs/41317019346), [literal predicate gate](https://github.com/link-assistant/formal-ai/actions/runs/41317019358).

## Verification and remaining qualification

Focused checks passed: 18 file-command, 12 import-edit, 15 repair-intake, 4 maintenance, 8 selected expectation/listing and 1 original-repair ownership cases. Earlier focused lexical/manifest/discovery checks also passed. Native behavior is paired in source and shared vectors but is not qualified locally. PR checks, merged-main build, release publication and current-source Docker digests remain pending until observed green. The last observed release is v0.355.27 and does not contain these changes. No release or full-delivery claim follows from local tests.

No subagents remain active. No local Rust compilation/tests, Docker build or application build occurred. Protected root files and personal session exports are excluded.
