---
bump: patch
---

### Fixed

- CI no longer runs the Rust test suite on changes that touch no code (issue #442). The `test` job in `release.yml` previously ran whenever the `changelog` job was *skipped* — but `changelog` is skipped precisely when there are no code changes, so docs-only / `.gitkeep` / changelog-fragment-only commits triggered the full `cargo test` matrix. The `test` job now gates on the `detect-changes` outputs (`any-code-changed` / `rs-changed` / `toml-changed` / `workflow-changed`), the same way `lint` and `coverage` already do.
