# PR #1504: current-main qualification follow-up

PR #1502 merged as 6d370dd9300f68b100478b03e186c3f73b0cef3c; PR #1503 merged as 1980a31f47d4afdc33092a1baa83484fd73e631f after green PR checks. The current main source has built its native binaries and test executables. Default-feature documentation checks passed. Main runtime and release qualification are not yet green. The latest observed published release remains v0.355.27, preceding these changes.

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

Replay the fixed transport against the original binary, then finish PR checks, merge and monitor fresh main/release/Docker qualification. Do not claim production delivery from the historical diagnostic or the green PR checks.

## Local verification and development provenance

Focused sequential JavaScript checks passed: 111 existing registry/timing cases, 20 file-command cases, 7 file-IO cases, 4 retained-diagnostics/transport cases and 6 existing default-branch build-policy cases. No local Rust compilation/tests, product WASM, Docker or application builds occurred. Native text formatting was the only local native-tool operation.

The citation replacement was applied by JavaScript Formal AI from English, producing Read and Edit. Its terminal answer correctly reported the missing verification Run capability; independent exact-diff inspection established the requested literal effect. Supplied literals and externally authored engine/native bootstrap payloads receive zero autonomous synthesis credit. No general autonomous coding success rate, cost reduction or amplification claim is made.

All agents are finished; the coordinator works alone. Protected root files and personal session exports remain excluded. #1483 original hosted three-client acceptance remains pending. The next ordered #1170 task, #1484, waits for this main/release cycle to qualify.
