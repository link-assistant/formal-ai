---
bump: patch
---

### Fixed

- Cover all active Cargo projects and tracked npm manifests in dependency maintenance. Reject incomplete inventories and unclassified manifests, preserve historical reproducers explicitly, and resolve each active Rust lockfile with its owning manifest.
