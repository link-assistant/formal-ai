# Public release download verification

This follow-up repairs the public-download verification gap under the existing [desktop download requirements](https://github.com/link-assistant/formal-ai/issues/479). The broader issue and subissue scope remains in [PR #1221](https://github.com/link-assistant/formal-ai/pull/1221); its 27 full umbrella acceptance rows receive no completion credit from this repair.

macOS self-signing was already implemented and merged. Both v0.354.0 architectures produced valid ad-hoc signatures, sealed resources and successful deep-strict verification. It needs no Apple Developer account. Apple Developer signing and notarization are separate capabilities. The download consumer must preserve this existing behavior without re-signing packages or deleting quarantine metadata.

## Required production evidence

- Select one published stable release and authenticate its exact tag, source tree and immutable public asset metadata.
- Download every public asset anonymously on hosted runners, stream its bytes through SHA-256, require exact official size and digest, and discard large files before the next download.
- Require all consolidated checksum rows to match the actual downloads and all source-advertised assets to have checksum coverage. The engine archive and its durable worker/protocol receipts must not be silently omitted.
- Independently download both macOS DMG and ZIP packages on matching fresh macOS runners. Check bundle/version, sealed resources, actual signatures, authenticated post-sign native executable bytes, and real loaded renderer/preload status. Observe Gatekeeper and quarantine without silently changing either.
- Cold-install the independently downloaded engine archive into a new package-manager cache/profile, verify the selected-source worker bytes against the original authenticated producer, and run genuine calculation, seed/memory and disposal checks in a browser worker.
- Keep only bounded JSON/text observations and failure logs as CI artifacts. Publication commands and producer-local packages are insufficient evidence of public consumption.

## Resource and acceptance limits

The v0.354.0 public assets total approximately 8.13 GiB. Verification is hosted only, at most 100 assets, two GiB per asset and twelve GiB per selected release. Local checks use synthetic unit-test bytes; no local Rust compilation, dependency installation or product-native/WASM/Electron execution is allowed.

The v0.354.0 engine archive was published with an official GitHub digest but absent from its Desktop consolidated manifest. The strict verifier must reject that missing row. Completion requires a real production release with the corrected manifest plus successful hosted public-byte, macOS and engine consumers; local fixture success alone does not satisfy it. Record exact CI and release evidence in the repair PR description. Unconfigured npm or extension registries remain explicit outcomes and receive no publication credit.
