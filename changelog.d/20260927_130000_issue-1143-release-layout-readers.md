---
bump: patch
---

### Fixed
- Changelog fragments and the collected CHANGELOG.md are discovered where
  they live instead of being derived from the rust root: the shared
  `rust-paths.rs` helpers (and `get-bump-type.rs`, `version-and-commit.rs`,
  `create-changelog-fragment.rs`, `collect-changelog.rs`, which now delegate
  to them) resolve repository-root `changelog.d/` first, then
  `{rust-root}/changelog.d/`, defaulting to the root. Plan 16 L1
  (`70df98cf6`) moved `Cargo.toml` to `rust/` while the fragments stayed at
  the root, so the bump-type step reported `fragment_count=0` /
  `has_fragments=false` and `check-release-needed.rs` skipped the cut with
  ~24 pending fragments (issue #1143). `workspace_manifest_resolution`
  pins the discovery contract hermetically instead of the old derivation.
- The Pages deploy job generates the Rust API docs through
  `scripts/build-rust-api-docs.sh`, which now discovers the crate manifest
  (root `Cargo.toml`, else `rust/Cargo.toml`) and pins `--target-dir
  target`, so the build works from either layout and the artifact
  assembly keeps copying repository-root `target/doc/`. The bare
  `cargo doc` invocation failed with exit 101 "could not find Cargo.toml"
  on main run 36255580824 after the same manifest move (issue #1143).
