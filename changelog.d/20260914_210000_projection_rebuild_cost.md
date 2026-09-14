---
bump: patch
---

### Fixed
- Rebuilding the native projection of a memory store no longer pays one `fsync` per doublet. The replacement database is scratch until a `rename` publishes it, so its transitions log is now staged without per-append syncing and flushed once before publishing; what reaches the served path is exactly as durable as before. A 112-event rebuild fell from 15.8 s to 0.2 s and the cost is flat instead of superlinear, which is what pushed the held-out computer-use generalization suite past its 600 s budget in one run while the same code finished in 85 s in another (issues #1106, #710).
