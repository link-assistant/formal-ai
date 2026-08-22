---
bump: patch
---

### Fixed

- Install the build-cache sweep from `build.rs` instead of waiting for someone
  to install the `pre-commit` framework. The hook has been described in
  `.pre-commit-config.yaml` since the previous release and never ran: that
  config takes effect only after `pre-commit` is installed *and*
  `pre-commit install` has been run, and on a fresh clone neither is true. The
  machine it was written for reached 205MiB free of 460GiB with the config
  committed and inert. `build.rs` now points `core.hooksPath` at a tracked
  `.githooks/`, so an ordinary `cargo build` arms it — the one step every
  contributor takes without being told. Installation is best-effort and skipped
  on CI: a tarball with no `.git`, a sandbox with no `git`, or a read-only
  checkout must all still build, and an existing `core.hooksPath` is never
  overwritten.
