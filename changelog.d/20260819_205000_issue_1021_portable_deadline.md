---
bump: patch
---

### Fixed

- Bound a CI command with a deadline both runner families have. The apt retry
  wrapper used GNU `timeout`, which macOS does not ship, so the macOS core
  slices failed the tests that drive it while its own Linux job was green;
  `scripts/run-with-deadline.sh` keeps `timeout`'s 124-on-expiry contract on
  every runner, and a new gate holds the rule for every tracked script and
  workflow. The replacement is held to the promise it replaces: it never expires
  a deadline early, which measurement — not assertion — is what caught.
