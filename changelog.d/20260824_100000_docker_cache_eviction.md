---
bump: patch
---

### Fixed

- Stop Docker layers evicting the compiler cache. The GitHub Actions cache is
  one 10GB pool shared by every workflow, and it reached 10.01GB — buildkit
  blobs holding 5.26GB against sccache's 2.44GB. GitHub then evicted
  compilation entries to make room for layers: the macOS specification lane's
  Rust hit rate fell from 48% to 27% between consecutive runs and the lane was
  killed at its 1400-second budget with no test having started, because those
  budgets were sized for a cache that hits. Two writers were paying for layers
  nobody reads — the pull-request image check, which since the previous release
  copies a prebuilt binary and so compiles nothing worth keeping, and the Docker
  Hub publish steps, which export the same layers the GHCR step in the same job
  just exported. Both now read the cache without writing to it; the from-source
  publish still exports, so a release does not rebuild every layer.
