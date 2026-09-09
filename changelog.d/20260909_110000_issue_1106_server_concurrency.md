---
bump: patch
---

### Fixed
- Issue #1106: `formal-ai serve` no longer stops answering every request while one is being solved. The accept loop handled each connection inline, so a single slow chat completion blocked unrelated callers — including a `GET /v1/models` that had answered a second earlier — which is why the failure was reported as a permanent wedge. Each connection is served on its own thread now. The underlying slowness is separate: every chat completion opens the memory store and hands its events to the solver, so the cost grows with accumulated history (measured: 21 ms for `formal-ai solve` against a store that takes the server over 60 s). This change stops it from being an outage for every other caller; the slowness itself is fixed separately in this same release.
