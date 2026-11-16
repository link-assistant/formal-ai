---
bump: patch
---

### Changed
- A missing compiler without a workspace grant now stops with a "grant required" answer that names the program and the pinned publisher procedure a grant would run, the re-probe and the one retry (#1159), for example `scalac` and `https://api.github.com/repos/scala/scala3/releases/latest`.
- A recovered missing compiler is now reported in the answer: the program, the pinned publisher procedure that ran, the re-probe and the retried command (#1159). A failed transaction is never reported as a recovery.
- The delivered Scala workflow installs `scalac` from the same release feed as the recovery publisher record, pinned by a Rust test (#1159).
