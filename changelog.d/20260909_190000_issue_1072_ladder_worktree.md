---
bump: patch
---

### Fixed
- Issue #1072: the Agent CLI ladder no longer copies the repository once per node. Each node extracted a 966 MB `git archive` (most of it `dev/` and `docs/` evidence) into a fresh temporary directory, `git init`ed and committed it, then compiled there — a path cargo had never seen, so every leaf rebuilt the crate from scratch. Measured on run 34326451343: 110 minutes for 32 leaves, 25 of them Agent CLI work. One sparse worktree is created once and reset between nodes, so the second leaf's `cargo check` is incremental and the agent's snapshot store is created once instead of per node (115 stores, 31 GB, in six hours). Each node's Agent CLI turn is bounded (`LADDER_NODE_BUDGET`, default 240 s; leaves that produced no proof had run 236 s unbounded), one line per node reaches the job log while the step runs, and `verify-node.sh` prints its `cargo` timings.
- Issue #1109: opening a link-cli store removes replacement databases left by processes that no longer exist. A rebuild killed partway leaves a 67 MB `.<name>.database.<pid>.<n>.tmp` beside the store; 1.1 GB across seventeen dead process ids was found beside one store. Only files whose process id is dead are removed.
