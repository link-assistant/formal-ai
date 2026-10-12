---
bump: patch
---

### Security
- The VS Code extension now packages with `@vscode/vsce` 4.0.0. The 3.x line pulled in `braces` (GHSA-vfj7-8cjw-p6xm), which has no patched release (issue #1169).
- Desktop packaging overrides `global-agent` to ^4.1.3, so `electron-builder` 26.17.0 no longer pulls `roarr` > `sprintf-js` (GHSA-hp3w-g68c-fv3c, no patched release).
- Overrides `proxy-addr` to ^2.0.8 in the root `bun.lock` (GHSA-jqcg-44mw-7w3h) and `compression` to ^1.8.2 in the end-to-end harness. Every lockfile now audits clean at `--audit-level=moderate`.

### Added
- `formal-ai benchmark run --offset N` and the `shard_size` input of `external-benchmarks.yml` run one upstream suite as concurrent graded windows whose counts add up. A shard cannot append to the ledger (issue #1177, R1177-12).
