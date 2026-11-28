# DRAFT (not filed): PR workflows have no ready-for-review tier, so heavy suites run on every push or never

Status: **not filed.** This is a divergence from a best-practice rule, not a
template defect with an observed false positive or negative. hive-mind already
ships a patch for each template in
`link-assistant/hive-mind/examples/ci-cd-pull-request-tiers/{js,rust,python,php,csharp,java,cpp,go}.patch`,
so separate issues would duplicate its rollout. File this only if the patches
are not applied within the next template update round.

Duplicate search (2026-11-25): `gh issue list --state all --search "ready_for_review"`
and `--search "tier"` in all 8 `link-foundation/<lang>-ai-driven-development-pipeline-template`
repositories returned no matches.

Context: link-assistant/formal-ai#1639 (CI/CD audit). formal-ai runs its full
test matrix only on `main`. On 2026-11-25, PR #1638 merged green and then broke
9 test shards, the lint lane and an E2E on `main`.

## Reproduction (current `main` of each template)

| Template | Head | Trigger |
|---|---|---|
| js | `8e72538` | `.github/workflows/release.yml:8` `types: [opened, synchronize, reopened]` |
| rust | `1d29f55` | `.github/workflows/release.yml:8` same |
| python | `7a05d05` | `.github/workflows/release.yml:8` same |
| php | `e12a25d` | `.github/workflows/release.yml:8` same |
| csharp | `10edf2e` | `.github/workflows/release.yml:8` same |
| java | `dd803a2` | `.github/workflows/release.yml:7` same |
| cpp | `c1c60fb` | `.github/workflows/release.yml:18` same |
| go | `785292e` | `.github/workflows/release.yml:7` same |

None of these workflows reacts to `ready_for_review`, and none splits a
commit tier from a ready tier.

## Expected (CI-CD-BEST-PRACTICES.md §1, "Commit and ready-for-review checks"; §10)

- On `synchronize`, run lint, types and fast unit tests, including on drafts.
- On `ready_for_review`, run the heavy unit, integration and E2E suites.
- Use a per-event concurrency key, so a ready run is not cancelled by the next
  `synchronize`.

## Impact

The templates run every suite on every push, which is correct but slow. Downstream
repositories whose suites outgrow that budget move them to `main` only, as
formal-ai did, and then learn about regressions after merge. A ready tier in the
templates gives those repositories a supported middle ground.

## Workaround

Add `ready_for_review` to `types` and allow it in each heavy job's `if:`.

## Suggested fix

Apply `examples/ci-cd-pull-request-tiers/<lang>.patch` from hive-mind to each template.
