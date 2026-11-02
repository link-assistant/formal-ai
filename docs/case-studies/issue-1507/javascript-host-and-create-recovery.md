# JavaScript host and small development-task recovery

## Scope and requirements

Issue #1507 must allow a trusted JavaScript-only host to perform basic tasks without initializing Rust grammar WASM. The production default and immutable per-worker grammar policy remain intact. Request text cannot grant or change host capabilities. Unsupported Rust source analysis must report its named missing-frontend gap.

All development actions, including repository reads, commands, tests and verification, go through JavaScript Formal AI. Start with small useful tasks; record the request, tool effects and independent checks before increasing difficulty. Supplied patches, literal replacement bodies, generators and temporary recovery loaders receive zero autonomous synthesis credit.

## Verified changes

- The trusted server setting FORMAL_AI_SOURCE_USAGE_RUST_GRAMMAR accepts only true or false. False skips Rust syntax initialization; explicit host options retain precedence and conflicting installation policies fail closed. The new regression reproduced the old failure with WASM execution trapped, then passed without any WASM attempt. Captured released-client first-step behavior remains covered.
- A complete literal file request using the following exact content previously failed before tools. Generic seed content-lead forms now bind the entire quoted body. The target-only owner writes only that target and requires a complete independent read after the latest write. No auxiliary plan file or invented receipt is emitted. Held-out verbs, nested paths, Unicode/whitespace, repeats and failed/truncated/mismatched readbacks are checked. Rust has the paired owner and CI-only native regression; TypeScript is generated from JavaScript.
- Failed command envelopes previously discarded stdout whenever stderr existed. JavaScript, generated TypeScript and Rust now retain both labeled streams once without converting a failing exit or signal into success. The original exit-2 sentinel replay shows both streams.
- A tiny edit through a Bash-only client previously fell through to an unrelated memory-program answer. The edit owner now reports workspace-edit-capability-gap/v1 with actual missing capabilities, complete false and verified false. The JavaScript server preserves the resolved Gap for logging; the native edit owner returns the same structured gap content. Supported read/edit behavior and genuine memory gaps remain tested.

During implementation, a duplicate import prevented the planner from loading. A temporary loader removed precisely that import in memory under an exact source hash, allowing actual Formal AI Read/Edit to remove the physical import. Normal loading and syntax checking then passed. The creation regression imports this owner and catches such load failures. The loader is not a production dependency.

## Real Start probe and current boundary

Released Start 0.41.0 was tested against the locally modified JavaScript checkout based on 7cb102cbd5aacd8c2d2e2b132f444bdea0f99e95, not a claimed new published Formal AI version. Its isolated server environment requires applying the opt-out through the trusted launcher. A listing request passed with one ls call and no file changes.

The exact edit request then produced the explicit missing Read/Edit gap with zero tools and unchanged fixture files. Start still exited zero and called that an answer. Its advertised Bash tool and permitted write programs cannot perform a guarded text replacement. [Start #258](https://github.com/link-foundation/start/issues/258) records both limitations; [#257](https://github.com/link-foundation/start/issues/257) separately records missing bounded 5xx diagnostics. Keep Start probes in isolated, explicitly owned workspaces. Continue using the direct JavaScript host for maintained edits until the client contract is repaired; never let a client inventory or hash protected workspace artifacts. Never bypass its tool policy with an arbitrary interpreter.

## Verification and remaining acceptance

Focused JavaScript suites cover host policy/released requests, generic creation, existing creation-provider cases, edit capability gaps and failed command streams. Only one necessary JavaScript suite runs locally at a time with bounded memory/deadlines. No Rust compilation/tests, product/Rust-grammar WASM, Docker or application builds run locally; native verification belongs to CI.

Review also reproduced stale certification from a target read before a write followed by an unrelated read. Shared Progress accessors now accept only a complete target-bound read after the latest write and any later potentially mutating tool. Failed, truncated or missing receipts cannot fall back to earlier source bytes. Eleven focused cases cover creation and additive writes, including later Edit/MultiEdit/opaque commands and unrelated writes.

The abstract natural-language instruction to repair command-envelope normalization returned no-plan. Its behavioral repair was externally specified and executed through Formal AI; general source-repair synthesis is not claimed. Quoted transport-shaped blocks inside literal source previously caused a create candidate to be cropped and rejected. Shared parsing now preserves closed quoted/fenced payloads verbatim and applies wrapper removal and directory declarations only at unquoted marker offsets. Genuine outside wrappers and quoted directory values remain supported. The exact failed test-source creation request now writes and independently reads the entire captured UTF-8 source unchanged; fake directory facts inside the literal grant no workspace scope. The captured source is readable declarative fixture data, with its content hash pinned in the regression.

PR #1509 merged as 7cb102cbd5aacd8c2d2e2b132f444bdea0f99e95. Its main tests passed and its release is being qualified separately from this draft. Original #1483 hosted semantic acceptance is still incomplete: the independent oracle has unresolved API/integration obligations and frozen fixtures omit required Rust module roots. Do not spend the twelve-cell hosted matrix expecting complete acceptance, count closed issue state as delivery, or start the next #1170 task before the current acceptance is independently established.

Only synthetic observations and bounded public CI evidence are reported. Raw local logs, private environment, personal files and protected root artifacts remain excluded.
