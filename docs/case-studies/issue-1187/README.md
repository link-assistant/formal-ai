# Issue #1187 (E151) — optional GitHub credentials in every workflow

## Problem

- Bot pull requests' checks waited for approval: every check run on #1124
  (7 workflows) ran only after a person acted, because a `pull_request`
  event created by `GITHUB_TOKEN` requires approval by GitHub's rule.
- `FORMAL_AI_BOT_TOKEN` was read by `self-authored-pull-request.yml` with a
  `github.token` fallback, but no such secret exists (`gh secret list`), so
  everything ran at the default layer anyway — an implied dependency on a
  credential nobody configured.
- Three `pull_request` workflows (`layered-ci.yml`, `evidence-check.yml`,
  `workflows.yml`) cannot be dispatched at all, so the approval wait had no
  escape hatch; `release.yml`'s dispatch defaults take release paths.
- Upcoming unattended workloads (#1169 daily updates, #1171 parity refresh,
  #1179 wrong-statement issues, #1182 cross-org findings, #1170 the
  self-coding ladder) all write to GitHub.

## Delivered

| Requirement | Artifact |
|---|---|
| R1 one resolver | `.github/actions/automation-token/action.yml`: GitHub App (`AUTOMATION_APP_ID` + `AUTOMATION_APP_PRIVATE_KEY`) → `AUTOMATION_TOKEN` → `github.token`; layer printed to log **and** job summary; `can-create-repositories` output true only at the App layer. Local mirror of link-foundation/.github `resolve-github-token` (their issue #1) under the same contract — when the shared action ships, this file delegates to it and no workflow changes. |
| R2 checks without approval | `.github/actions/dispatch-checks/action.yml`: dispatches every `pull_request` workflow on the bot PR's head sha; workflows lacking `workflow_dispatch` are `::warning::`-reported, not silently skipped. |
| R3 checks-only dispatch of release.yml | Deferred edit in `docs/integration-manifest.md` (`mode` input default `checks`, every release/publish/tag job requiring explicit mode **and** `refs/heads/main`) — GUARD-held with #1084's five build steps in the same file's neighborhoods. |
| R4 isolation without tokens | `.github/workflows/e2e-isolation.yml`: orphan branch `e2e/<purpose>/<run_id>/<name>` built with `git checkout --orphan` from only the named paths, `e2e-task` labelled tracking issue, PR based on the branch, cleanup closing and deleting both; a separate repository only when the resolver reports `can-create-repositories: true`. |
| R5 cross-repository degrades | Resolver summary text + the `e2e-isolation`/`cross-org-duplication` workloads write findings to reports and one tracking issue at layer `default`. |
| R6 documentation | CONTRIBUTING.md "GitHub credentials" section: the three layers, `AUTOMATION_TOKEN` as the only token name, nothing needs configuring. |
| Tests | `rust/tests/unit/issue_1187_credentials.rs`: the resolver implements the three layers and summary logging; the dispatcher dispatches; isolation uses an orphan branch and the resolver; CONTRIBUTING states the rules; **no workflow reads a token secret other than the two `AUTOMATION_*` names unless that file is recorded as a pending migration** in the manifest; every pending manifest row names a file and an issue. |

## Residuals (GUARD-tracked in docs/integration-manifest.md)

The existing-file migrations — resolver adoption in
`self-authored-pull-request.yml` and `release.yml`, `workflow_dispatch` +
`branches-ignore: e2e/**` in the three undispatchable workflows, release
mode guards — are exact-edit rows in the manifest, applied by main when the
GUARD lifts. The unit test keeps every row honest: a pending row must name
its file and issue, and a file still reading `FORMAL_AI_BOT_TOKEN` must be
pending or the build fails.

With no `AUTOMATION_*` configured (today's state) the acceptance run is:
label an issue `formal-ai-solve`; the bot PR appears; dispatch starts its
checks; the summary reads `layer default`.
