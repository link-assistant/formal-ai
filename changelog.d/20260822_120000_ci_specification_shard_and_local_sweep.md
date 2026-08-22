---
bump: patch
---

### Changed

- Stop running the specification shard twice per pipeline. The `full` test lane
  skipped only `data_files::` and `self_ast_census`, so it also ran the 1034
  `specification::` tests that the parallel `specification` lane was running at
  the same moment — a lane that needs 689 seconds on its own to do exactly
  that. Measured on run 32555911181: the `full` lane held 700.17 seconds in a
  single test binary, 87% of the job that set the pipeline's critical path.
  Compilation was not the cost — sccache reported a 79.57% hit rate with zero
  errors in that same job, and the test step logged no `Compiling` line at all.
  The lane that owns those tests still runs them, and a test pins that so the
  skip cannot quietly become a coverage hole.

- Sweep the build cache on every commit, not only Rust ones. Cargo never
  removes anything, so `target/` accumulates artifacts from every branch and
  dependency version until the disk fills. A docs-only commit used to leave the
  previous build's artifacts behind just the same.

- Prune with cargo-sweep when it is installed. It asks cargo which artifacts the
  current build actually references, so a dependency the next build still needs
  survives even when it was compiled weeks ago; the previous mtime comparison
  could not tell a stale artifact from a current one that simply did not need
  rebuilding, and deleted live dependencies the next build then recompiled. The
  mtime path remains as a fallback. `CARGO_TARGET_MAX_SIZE_MB` caps the tree
  locally (4GB by default) and is unset on CI, where the runner is billed for
  the rebuild rather than the disk.

- Run the `cargo-test` commit hook through `scripts/cargo-test.sh`. A bare
  `cargo test` starts one compile job and one test thread per core, pinning the
  whole machine for the length of a commit, and prunes nothing afterwards.
