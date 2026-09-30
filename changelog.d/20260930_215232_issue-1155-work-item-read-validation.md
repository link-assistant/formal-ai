---
bump: patch
issue: 1155
---

### Fixed

- An unauthenticated `gh issue view` prints its how-to-authenticate banner,
  and the Agent CLI echoes that output with the exit status dropped — the
  banner was then accepted as the issue body, the run wrote its plan record,
  and answered `planned_not_executed` without ever trying the `webfetch`
  tool the client offered (2026-09-27 Scala run). Every planned read now
  prints its own exit status as a sentinel line, the output must still have
  the title-and-body shape of a work item, and a failed read walks the
  fallbacks in order — the client's fetch tool, the credential-free REST
  read, `gh api`, then the prepared pull request. An exhausted retrieval
  closes by listing every read tried with its result, and the only write is
  no longer spent on a plan record for an issue the session never read.
