---
bump: patch
---

### Fixed
- `scripts/version-and-commit.rs` now updates the workspace-package entry in `Cargo.lock` in the same release commit that bumps `Cargo.toml`. Previously every release left `Cargo.lock` stale, forcing follow-up "sync Cargo.lock" commits and producing avoidable merge conflicts on every open PR.
