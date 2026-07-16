---
bump: patch
---

### Fixed
- Stop the auto-release job from failing with "cannot rebase: Your index contains
  uncommitted changes" when a concurrent release lands on `origin/main` mid-job.
  The release now rebases onto the remote while the tree is still clean, before
  the version bump is written and staged.
- Only rebase when `origin/<branch>` actually has commits the release job lacks.
  Being ahead of the remote no longer reports "Local branch is behind remote".
- Create the release tag only after the release commit reaches the remote, so a
  `pull --rebase` retry can no longer leave the tag on an orphaned commit.
