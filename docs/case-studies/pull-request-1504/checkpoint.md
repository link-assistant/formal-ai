# PR #1504: current-main qualification follow-up

PR #1506 merged as 0261ddcb70e3875f6fc25499c8a770ae19a3b488 after green PR checks. Its [complete main pipeline](https://github.com/link-assistant/formal-ai/actions/runs/41340262533) passed, including native compilation/tests, specification checks, all lint lanes, fresh-binary server parity, both agent CLI E2E jobs, Pages and native full/slim container publication. [Release v0.355.28](https://github.com/link-assistant/formal-ai/releases/tag/v0.355.28) is published from df939f759123335d81b4d6e56ce59b0fa06b5597, one release-metadata commit after the tested merge. The crate was published, audited against the verified package and smoke-tested in CI. The earlier failures and successive fixes below remain historical evidence.

## Requirements and drafted fixes

- Restore the explicit issue/PR reference required by the CI gate registry. Enable the existing R1085-16 JavaScript guard in the fast PR profile. The missing reference explains the lint-lane failures and many native metadata assertions.
- Preserve established local filename/folder discovery before the new file-command content-search fallback. Keep the original seven #1501 cases and held-out operands covered.
- Reuse the established Read argument builder for source-custody requests, preserving client path aliases.
- Keep native tests outside production source. Move the two shared-vector tests to external test modules, retaining every assertion and fixture.
- Check the actual shared Clippy script for its target scope. The previously documented temporary style deferral remains unchanged; compiler and bug/performance warning policy remains fatal.
- Apply the existing third-party download-action deprecation suppression only to affected action steps.
- Retain server identity, real exit/signal state, response-byte count and bounded raw stdout/stderr tails on request failures before cleanup. The JavaScript client sent TCP EOF before asynchronous handlers could reply. A JavaScript-only probe received zero bytes with half-close and HTTP 200 while retaining the write side. The parity client now writes the request and lets the response close the connection. The original corpus request is retained in a focused transport regression.

## CI-only diagnostic replay

The explicit Layered CI dispatch reuses formal-ai-release-binary from a successful Build binaries and tests job. It requires a matching immutable main push source and the canonical producer workflow, with bounded job/artifact queries. Normal PR jobs remain unchanged; the diagnostic dispatch skips the ordinary tier jobs and executes no build or release. Its historical native binary does not qualify the current PR head.

After the branch is pushed, reproduce the original server corpus using:

```sh
gh workflow run layered-ci.yml --ref fix/post-1503-main -f diagnostic-native-run=41328792781 -f diagnostic-native-source=1980a31f47d4afdc33092a1baa83484fd73e631f
```

The full original corpus passed in [diagnostic run 41334639147](https://github.com/link-assistant/formal-ai/actions/runs/41334639147) at 37a61cfbb0fa078adb8d6c5b7d22f0d7beffa0a3 against the unchanged original binary. All PR checks passed; PR #1504 merged as 851200b7cfb2b3fe264feb0f4d904ad2691fcda9. This historical diagnostic does not establish production delivery.

Fresh main [run 41334848898](https://github.com/link-assistant/formal-ai/actions/runs/41334848898) passed lint lanes 2–4 and default-feature documentation, but the native build and lint lane 1 failed. The follow-up fixes the relocated native test’s remaining embedded include path, checks that every relocated shared-vector input resolves in fast JavaScript CI, follows the committed github-script v9 snapshot, includes the actually invoked shared Clippy profile in disk-policy evidence, and keeps compiler-only explanation prose in a comment with a machine-readable lint reason. All changes were applied through JavaScript Formal AI from English; literal supplied changes receive zero synthesis credit. The focused file-command suite passes 21 checks. Rust verification and the new main/release/Docker cycle remain pending in CI; no local Rust execution is permitted.

A bare quoted replacement of a relative include path was not recognized and made no edit. Supplying the full include_str expression selected the supported literal edit route. This is a prompt-shape limitation, not general synthesis success; improve its reusable grammar after the release blocker is resolved.

## Local verification and development provenance

Focused sequential JavaScript checks passed: 111 existing registry/timing cases, 20 file-command cases, 7 file-IO cases, 4 retained-diagnostics/transport cases and 6 existing default-branch build-policy cases. No local Rust compilation/tests, product WASM, Docker or application builds occurred. Native text formatting was the only local native-tool operation.

The citation replacement was applied by JavaScript Formal AI from English, producing Read and Edit. Its terminal answer correctly reported the missing verification Run capability; independent exact-diff inspection established the requested literal effect. Supplied literals and externally authored engine/native bootstrap payloads receive zero autonomous synthesis credit. No general autonomous coding success rate, cost reduction or amplification claim is made.

## Fresh-main native assertion follow-up

PR #1505 merged as e92bb80f6e422d78df68617267e36b3879c531d9. Its [main pipeline](https://github.com/link-assistant/formal-ai/actions/runs/41336669362) passed native compilation, all lint lanes, fresh-binary server parity, every specification shard and both agent CLI E2E jobs. All other workflows on that source finished with passing or intentional skipped conclusions. Release remained blocked by three native assertions: the original invalid-documentation fixture expected the obsolete evidence-only route and guide-first order; a second workflow test still expected inline Clippy commands; and the explicit diagnostic lacked its own concurrency group.

The next batch retains every original invalid-evidence read and honesty assertion, adds explicit no-candidate/no-check/no-effect/unverified assertions for the current controller, checks the invoked shared Clippy owner including fatal bug/performance categories, and gives the read-only diagnostic a bounded cancellation group. No test is skipped or removed.

While drafting this batch through English, JavaScript Formal AI misclassified documentation examples inside a quoted test replacement as new instructions. The JavaScript planner now defers explicit literal edits to the established edit owner after prerequisite/read guards. Retrying the identical English request produced Read then Edit. Rust has the paired routing boundary, and TypeScript was regenerated with the maintained JavaScript translator. Supplied literal edits and bootstrap changes receive zero autonomous synthesis credit. The terminal verification-tool gap remains explicit; independent checks verified the applied diff.

Focused sequential JavaScript checks pass: seven evidence/quoted-payload regressions, fifteen repair-intake cases and three repository-documentation cases, including actual useful partial edits, preserved unrelated files and verified no-write repeats. Native compilation/tests and release/container qualification remained CI-only. The successful main pipeline and published v0.355.28 described above now include this batch.

## Observed publication and remaining assets

Both final GHCR tags are anonymously readable multi-architecture indexes containing exactly linux/amd64 and linux/arm64. Independent metadata-only JavaScript verification used the maintained manifest verifier; no image layers or native executables were downloaded locally.

- ghcr.io/link-assistant/formal-ai:0.355.28: sha256:e99f824221d5d25d039b981dfdac23faa2251989f5ae813fe863874d554d3651.
- ghcr.io/link-assistant/formal-ai:0.355.28-slim: sha256:8531eca0efb2d59591ffc7458b35f35f972634561be36057de39a4c09eef4ddb.

The initial full-image and Docker Hub mirror receipts bind the executed binary to the same df939f759 release source and version. Browser-engine and source-network publication passed. [Desktop Release 41350844802](https://github.com/link-assistant/formal-ai/actions/runs/41350844802) passed all native executable builds, CLI archives, VS Code packaging and desktop bundles, including Intel macOS artifact smoke tests. [Public-download verification 41355315055](https://github.com/link-assistant/formal-ai/actions/runs/41355315055) passed every public asset byte/hash check, both macOS package/renderer checks and the browser-package check. [VS Code extension publication 41355307133](https://github.com/link-assistant/formal-ai/actions/runs/41355307133) passed, including authoritative Marketplace/Open VSX publication and clean-profile installation checks.

## Released start integration probe

The user requested testing link-foundation/start#256 once released, adopting it for local Formal AI work only if the test works, otherwise reporting the failure and continuing the previous method. Published start-command 0.41.0 has npm source b12b2e7e70aefa56f680601959170ce12c2ce830, verified as a descendant of the requested PR merge.

A tiny isolated listing test used Node 24.18.0, Formal AI 0.355.28 at df939f759, a 384 MiB server heap, a 20-second task budget and zero retries. Under the explicit local prohibition on Rust/product and Rust-grammar WASM, the HTTP agent route attempted Rust Tree-sitter initialization and returned HTTP 500. start exited 70 before running any tool; independent fixture checks found no changed files. A direct replay exposed the LocalProductWasmForbidden cause that start omitted from its final 5xx evidence. No edit test was attempted after the first failure. This does not establish a failure with an unrestricted runtime.

Sanitized reports: [Formal AI #1507](https://github.com/link-assistant/formal-ai/issues/1507) owns eager Rust-frontend initialization; [start #257](https://github.com/link-foundation/start/issues/257) owns missing bounded 5xx diagnostics. Continue the previous direct JavaScript host with sourceUsageRustGrammar false. No raw local logs, environment, personal files or host paths were uploaded. No local Rust compilation/tests or Docker/application build occurred.

The automatic issue-triggered Hive draft attempt for #1507 used the current built Formal AI source but produced [PR #1508](https://github.com/link-assistant/formal-ai/pull/1508) without a verified implementation. Its gap-no-implementation rejection is separate from the green main release pipeline. Do not merge or count that draft as a solution or claim universal issue synthesis from the supported literal-edit subset.

All agents are finished; the coordinator works alone. Protected root files and personal session exports remain excluded. #1483 original hosted three-client acceptance remains pending. [Qualification 41355652376](https://github.com/link-assistant/formal-ai/actions/runs/41355652376) refused admission before any fixture effects because the unrelated failed issue-specific draft 41346476878 was classified as release CI. The follow-up excludes only the known hive-mind-draft workflow for issues/workflow_dispatch events; push, PR, workflow_run, unknown workflows, missing publishers and foreign sources remain fail-closed. A focused regression replays the observed failed draft and verifies those boundaries. No universal issue-coding capability is claimed. The next ordered #1170 task, #1484, waits for hosted qualification.
