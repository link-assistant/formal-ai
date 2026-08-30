---
bump: minor
---

### Fixed

- Keep the whole payload when a written file's text contains a semicolon. The
  bound on a literal write was read with the sentence splitter written for shell
  routing, where `build; deploy` is two commands to judge one at a time. Prose
  does not read a semicolon that way, so a file whose text was
  "… or a host surface; domain knowledge and policy belong in data." was written
  ending at *host surface;*, with the clause saying where domain knowledge goes
  thrown away. The two readings are now two named splitters over one
  implementation, and the payload keeps its second half (#1066).
