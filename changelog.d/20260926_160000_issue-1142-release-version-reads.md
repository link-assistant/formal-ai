---
bump: patch
---

### Fixed
- The release pipeline reads the formal-ai version through
  `rust-script scripts/get-version.rs` (rust-paths root discovery)
  everywhere it needs it for GitHub Pages deploys: the two
  "Resolve Pages deploy ref" steps (auto-release and changelog-pr) and
  the Pages deploy "Read formal-ai version" step previously ran inline
  `sed` reads against the repository-root `Cargo.toml`, which plan 16 L1
  (`70df98cf6`) moved to `rust/Cargo.toml`; on the first release-relevant
  main push after that merge the `if: always()` ref-resolution step
  failed with "Could not read formal-ai version from Cargo.toml" and
  took the whole Auto Release job red. `scripts/stamp-pages-artifact.sh`
  mirrors the same fallback ordering (root manifest first, then
  `rust/Cargo.toml`).
- The Pages deploy job installs rust-script through
  `scripts/install-rust-script.sh` (retry wrapper) before reading the
  version, and the `rust_script_install_steps_use_retry_wrapper`
  tripwire test counts the new occurrence (8 → 9) so the install step
  cannot silently regress to a bare `cargo install`.
