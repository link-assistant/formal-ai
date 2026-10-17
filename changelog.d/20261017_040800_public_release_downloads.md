---
bump: patch
---

### Fixed

- Verify fresh anonymous public release downloads with exact byte counts, SHA-256 digests and complete advertised checksum coverage; include the independently published browser engine in the signed consolidated manifest.
- Cold-install the publicly downloaded browser package and verify real worker behavior, and test downloaded macOS DMG and ZIP packages using the existing account-free self-signing and actual renderer startup.
- Preserve bounded CI-only verification and retain text diagnostics without downloading or executing release binaries locally.
