---
bump: minor
---

### Added

- Stable Rust is now a gate rather than a convention.
  `nothing_in_the_tree_reaches_for_a_nightly_toolchain` refuses a toolchain
  file, a toolchain action asking for anything but stable, a per-invocation
  toolchain override, a bootstrap environment variable, and an unstable feature
  attribute -- across every tracked source, script and workflow, so a future
  dependency that only builds on nightly is caught at review time instead of
  quietly moving the toolchain.
  `the_crate_is_on_edition_2024_and_the_judge_compiles_the_same_edition` holds
  the manifest and the `rustc` that judges a self-authored version to the same
  edition.

### Fixed

- The issue-#1021 traceability gate counted its requirements with a literal
  `(1..=31)` and went stale the moment R1021-32 was written: it kept passing
  while checking one fewer requirement than the branch had. The IDs are now read
  from the shard that assigns them and asserted to run contiguously from 1, so a
  gap, a duplicate, or a row nobody wired up is a failure rather than a shorter
  loop.
