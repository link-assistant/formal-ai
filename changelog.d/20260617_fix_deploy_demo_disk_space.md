---
bump: patch
---

### Fixed

- CI: the **Deploy Demo to GitHub Pages** job no longer crashes with
  `No space left on device`. The job stopped restoring the multi-gigabyte
  `target/` cache shared with the `lint`/`test` jobs (it now caches only the
  Cargo registry under a dedicated `*-cargo-docs-*` key) and proactively frees
  unused pre-installed SDKs from the runner before building the API docs. Disk
  usage is now logged with `df -h` around the cleanup for future diagnosis.
  (#523)
