---
bump: patch
---

### Fixed
- Default-branch CI is green again: the total-closure regression test now
  passes `cargo fmt --check`, unknown-opener browser tests cannot be intercepted
  by live search providers, and permission-replay tests wait for the worker
  response before reading queued-task state. This removes one deterministic
  failure and one retry-masked false positive from run 31186108359.
  ([#980](https://github.com/link-assistant/formal-ai/issues/980))
