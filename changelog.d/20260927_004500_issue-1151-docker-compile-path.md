---
bump: patch
---

### Fixed

- The Auto Release job can publish the Docker image again: the final stage
  copies the binary from `/app/target/release/formal-ai`, a contract the
  prebuilt stage already satisfied but the compile stage never did after the
  Plan 16 L1 move put cargo's output in `rust/target/release` — run
  36278582429 (v0.352.0) failed at the GHCR push with
  `"/app/target/release/formal-ai": not found`, after the crate itself had
  already published to crates.io and passed its smoke test. The compile stage
  now stages its binary at the contract path, the contract is stated where
  both stages can see it, and no pull request could have caught this: the
  docker-build check pins `BINARY_SOURCE=prebuilt` and never runs on main
  (issue #1151). The same PR un-broke the next release in a second way the
  v0.352.0 run revealed: the release commit bumps the crate version and
  appends the self-hosting ledger row but never re-rendered the status
  surfaces that project them (README.md, docs/status.md), and because the
  bot's push triggers no workflow, nothing noticed until the next tree ran
  the `check_status_render` gate — the release script now regenerates and
  stages those surfaces as part of the release commit.
