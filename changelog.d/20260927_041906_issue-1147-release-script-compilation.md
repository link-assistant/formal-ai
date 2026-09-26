---
bump: patch
---

### Fixed

- The Auto Release job can compile its scripts again: the unused
  `PreparedRelease` import that `RUSTFLAGS=-Dwarnings` turned into a hard
  error in `scripts/version-and-commit.rs` (run 36269287140, latent since
  the PR #1139 bulk state) is gone, and `scripts/create-changelog-fragment.rs`
  now declares the `regex` dependency its `rust-paths.rs` include has needed
  since manifest parsing moved there — every other rust-paths includer
  already declared it. Both scripts run only in push-to-main release jobs,
  so no PR-side job had ever compiled them; both are now verified locally
  under `-Dwarnings` via side-effect-free early exits. The import had a
  second consumer: the PR-side `check_release_changelog_collection` gate
  runs `rust-script --test` over the script, whose tests module constructs
  `super::PreparedRelease` — so the name is now imported under
  `#[cfg(test)]`, present exactly when the tests compile it and absent
  from the release build that must stay warning-free (issue #1147).
