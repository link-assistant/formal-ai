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
  (issue #1151).
