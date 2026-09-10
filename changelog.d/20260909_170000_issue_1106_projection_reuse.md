---
bump: patch
---

### Fixed
- Issue #1106: recording a chat exchange no longer rebuilds the entire native link-cli projection. Every completion persists the memory, and persisting replaced the whole graph in one transaction, so a store of 400 events rebuilt all 400 to append three — measured at 51.8 s of a 53 s request, growing with the square of the accumulated history. That was the slowness the previous fix in this release deliberately left open, having only stopped it from blocking unrelated callers. The projection now records how many events it holds, in a marker beside the database, and a completion appends only what is new: the same request costs 0.35 s. The graph is unchanged by this — a store built incrementally and a store rebuilt from scratch were compared byte for byte, and both the 67 MB database and its 1 736 recorded addresses are identical. A prefix that does not describe the database (a `.lino` replaced underneath it, a marker left by an interrupted write) is detected and rebuilds instead, so the reduction is an optimization and never a weakened guarantee.
