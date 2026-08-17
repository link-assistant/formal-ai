# Pull Request 1005 Case Study

Pull request [#1005](https://github.com/link-assistant/formal-ai/pull/1005)
implements issue #922 on branch `issue-922-d8d07e0db7e9` without merging
directly to `main`.

## Review Scope

Review covers the complete event-log method lifecycle: production trace
normalization, inherited held-out validation, proposal-only projection,
canonical promotion, confirmed materialization, registry loading, durable
rejection evidence, recursive recipe/source parity, and regression floors. It
also covers the static seed, release metadata, case-study evidence, and CI
replay of the real Agent CLI path.

This is behavioral infrastructure and data work, not a visual UI change. The
issue and review contain no screenshot to preserve, and before/after visual
evidence is not applicable.

## Review Channels

At the implementation baseline, the PR had no conversation comments, inline
review comments, submitted reviews, or requested changes. All three GitHub
feedback endpoints are preserved independently under `raw-data/github/` so a
later review cannot be mistaken for an empty `gh pr view --json comments`
result.

## CI History

The prepared branch SHA `931033b2c76d5715d47ce26c7b570d023c154a5e`
completed its initial CI/CD, Coverage, External Benchmarks, Security, and Stock
Rust Install runs successfully on 2026-08-13. That baseline predates the
implementation. `raw-data/github/initial-ci-runs.json` records run IDs,
timestamps, SHA, conclusions, and URLs. Fresh runs for the implementation SHA
are checked after the final push; any non-passing logs are downloaded before
the PR is marked ready.

## Decisions

- Reuse issue #531 discovery instead of adding `stitch_core`; the repository
  already has deterministic link-native compression and integrity evidence.
- Learn event kinds, not payloads, to prevent prompt-specific and
  self-referential method identities.
- Keep learned registry entries separate from compiled handlers; adoption must
  not invent executable code or alter dispatch precedence.
- Let promotion own gates and observations. Learner-produced documents cannot
  claim their own evidence.
- Start from an empty tracked seed and promote the exact generated method edit;
  policy prose remains in documentation so seed reference closure stays total.
- Add the reproducible Agent CLI promotion script to CI because benchmark-gated
  adoption is an acceptance criterion, not merely historical evidence.

## Verification

The issue case study contains raw production and Agent CLI evidence. Local
review runs focused behavior, recipe/source parity, formatting, Clippy,
examples, repository policy scripts, all Rust tests, doc tests, and browser unit
tests before push. The final PR description records exact commands and results
and closes the issue with the full GitHub URL.
