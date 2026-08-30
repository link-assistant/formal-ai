---
bump: minor
---

### Fixed

- Report what a tool found instead of answering the same request a second time
  without it. The route that answers a question about how a task decomposes
  plans no tool call, so its answer is the same on every turn, and it sat ahead
  of the route that reports a tool result: a request to look at the repository
  was correctly planned as a search, and the turn that existed to report the
  search reported a decomposition of the instructions instead. An answer reached
  without looking has no standing to overrule one reached by looking, so the
  route now stands aside once a tool has run — the same rule the routes on
  either side of it already follow (#1066).
