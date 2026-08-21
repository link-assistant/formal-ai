---
bump: minor
---

### Added

- Added verified mutating filesystem actions (#824, #944): a request to move or
  copy a file is carried out as the ordered recipe its seed intent declares --
  the source exists, the destination is free, the destination's parent is
  created, the action runs, and the result is checked -- with each step observed
  before the next is planned, so a deep target path works and a destination that
  is already taken stops the recipe before anything changes.
- Added the mutating rungs `824.L1`-`824.L5` to the issue #916 write-effect
  ladder together with the sandbox-reset semantics #944 asks for: every rung
  declares the filesystem state it starts from, that state is materialized and
  read back off disk before the rung runs, and a rung may now require the steps
  that carried its action out and not only the effect they left behind.

### Changed

- A blocked action reports the check that stopped it and the status it exited
  with instead of claiming a completion the workspace would contradict.
