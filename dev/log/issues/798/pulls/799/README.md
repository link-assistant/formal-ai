# Issue 798 / pull 799 evidence index

This directory is the durable investigation record for issue #798. `github/`
contains issue, pull-request, review, event, run, check, and related-work API
responses. `ci-logs/` contains the complete logs for both cited failing runs plus
warning/error indexes. `templates/` is a commit-pinned snapshot of all three
pipeline templates and Hive Mind's CI/CD guidance. The root-level `*-before.log`
and `*-after.log` files preserve local reproductions and verification.

## Requirements and disposition

1. Download and inspect both cited CI runs: complete; no job was inferred from
   the GitHub status summary alone.
2. Eliminate real failures, false positives, false negatives, and warnings:
   macOS signing, WASM warning enforcement, Windows warning enforcement, VSIX
   bundling, and self-hosting evidence attribution are addressed.
3. Compare every workflow/script with Rust, JavaScript, Python templates and
   Hive Mind guidance: see `template-comparison.md` and the snapshots.
4. Reconstruct the sequence, identify root causes, research existing solutions,
   and cover fixes with reproducing tests: see `timeline-root-causes.md`,
   `online-research.md`, and local validation logs.
5. Apply repeated problems repository-wide: the desktop warning gate is global
   across its full six-platform matrix; bundled-browser signing is excluded at
   any depth for both macOS architectures; direct rustc warning enforcement is
   fixed at its single invocation boundary.

## Outcome classification

- **Real release failure:** ad-hoc re-signing descended into Playwright's
  already-structured Chrome framework and invalidated its framework seal.
- **False negatives:** direct `rustc` did not inherit Cargo's `RUSTFLAGS`, and
  desktop builds did not deny Rust warnings; both allowed diagnostics through.
- **Actionable packaging warning:** the VSIX shipped all of `node_modules`
  (9,015 files); dependency code is now bundled and `node_modules` excluded.
- **Lockfile false negative:** npm 11 detected missing optional musl package
  records; the regenerated lockfile passes a clean install on workflow Node 22.
- **Correct policy failure:** the self-hosting ratchet correctly rejected a
  decrease. It must not be weakened; this PR supplies valid Formal AI session
  evidence on every commit instead.
- **Secondary failure:** finalize correctly reported absent macOS artifacts
  after both macOS jobs failed. It requires no suppression.
