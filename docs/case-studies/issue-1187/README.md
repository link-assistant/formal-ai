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
| R5 cross-repository degrades | Resolver summary text + `cross-org-duplication.yml` drops `--open` at layer `default` and uploads its findings record; the "one tracking issue in this repository" half is not built yet (see the status table below). |
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

## Raw data

- [`raw-data/issue-1187.json`](raw-data/issue-1187.json) — the issue (title,
  body, comments, creation time; it has no comments).
- [`raw-data/pr-1124-runs.json`](raw-data/pr-1124-runs.json),
  [`raw-data/pr-1124-checks.json`](raw-data/pr-1124-checks.json),
  [`raw-data/pr-1124-commits.json`](raw-data/pr-1124-commits.json) — the
  Actions runs, check rollup and commits of bot pull request #1124, fetched
  with `gh run list --branch formal-ai/issue-1123-34542617225` and
  `gh pr view 1124`.

## Timeline

| When (UTC) | Event |
| --- | --- |
| 2026-09-10 23:38:45 | `github-actions[bot]` pushes `b2e10ed7e` and opens #1124 (self-authored pull request for #1123). |
| 2026-09-10 23:38:50 | The bot pushes `9aad05591`, the actual fix. |
| 23:38:51 / 23:38:57 | 13 `pull_request` runs are created for those two commits, actor `github-actions[bot]`. Every one of them has **zero jobs** (`gh api …/runs/34543081478/jobs` → `total_count 0`): they waited for approval and never started. |
| 23:42:35 | `konard` pushes `55717e9c8` (rustfmt). |
| 23:42:47 | 7 `pull_request` runs start on `55717e9c8` with actor `konard`; all succeed. These are the checks the rollup shows. |
| 2026-09-11 00:09:55 | #1124 merges; the 13 bot-triggered runs are closed out as `failure` at the same second, still with no jobs. |
| 2026-09-29 17:14 | Issue #1187 opened from that evidence (the issue's "every check run on #1124 (7 workflows) … actor `konard`"). |
| 2026-09-30 11:12 | PR [#1188](https://github.com/link-assistant/formal-ai/pull/1188) opened for the bulk batch. |
| 2026-09-30 (+07:00) | `2f86f34b4` — resolver, dispatcher, `e2e-isolation.yml`, CONTRIBUTING section, `issue_1187_credentials.rs`, manifest rows. |
| 2026-10-06 (+07:00) | `df830dec7` — actionlint and zizmor findings on the new workflows fixed. |
| 2026-10-07 (+07:00) | `46d45c9e1` — timeouts, concurrency, `persist-credentials` and current action majors on the new workflows. |

## Requirement status

IDs follow the shard `docs/requirements/issue-1187-optional-github-credentials.md`
(the issue's R1–R6 become R1187-1 … R1187-6; its Tests section supplies
R1187-7 and R1187-8).

| ID | Issue | Status | What is missing |
| --- | --- | --- | --- |
| R1187-1 | R1 one resolver | partial | `self-authored-pull-request.yml` still reads `FORMAL_AI_BOT_TOKEN`; resolver is a local mirror of link-foundation/.github#1 |
| R1187-2 | R2 checks without approval | partial | no workflow calls `dispatch-checks`; three workflows lack `workflow_dispatch` |
| R1187-3 | R3 checks-only `release.yml` dispatch | pending | `release_mode` defaults to `instant`; no `main` guard |
| R1187-4 | R4 isolation without tokens | partial | no `branches-ignore: e2e/**`; no separate-repository path at the App layer |
| R1187-5 | R5 cross-repository degrades | partial | no tracking issue in this repository; untested |
| R1187-6 | R6 documentation | implemented | the section overstates the test (it checks only `FORMAL_AI_BOT_TOKEN`) |
| R1187-7 | Tests: automated | partial | `workflow_dispatch` and release-guard rules not asserted; file is `rust/tests/unit/issue_1187_credentials.rs`, not `ci_cd/` |
| R1187-8 | Tests: manual | pending | not run; needs R1187-1/-2 applied to the bot workflow |

Definition-of-done items: shard (this pass), traceability rows (drafted for
the maintainer), this case study (the "first bot pull request whose checks
started by dispatch" cannot be recorded until R1187-8 runs), a
`changelog.d/` fragment (not written), and a green
`run-ci-gates.rs --stage rust` (not re-run for this write-up).

## Root cause

GitHub, not this repository, decides that a `pull_request` run caused by a
`GITHUB_TOKEN` push needs approval. The repository's part was having no
second path: the only bot token (`FORMAL_AI_BOT_TOKEN`) was never
configured, three `pull_request` workflows could not be dispatched at all,
and dispatching `release.yml` would take its release path (`release_mode`
defaults to `instant`). So every bot pull
request waited for a person — #1124 waited until a human push, four minutes
later, started real checks.

## Prior art

- GitHub documentation, "Triggering a workflow from a workflow": events
  caused by `GITHUB_TOKEN` do not start new runs except `workflow_dispatch`
  and `repository_dispatch`, and `pull_request` runs it causes require
  approval — the reason R1187-2 dispatches.
- `actions/create-github-app-token` — the standard way to mint an App
  installation token in a job, which the resolver's App layer uses.
- link-foundation/.github#1 (`resolve-github-token`, `dispatch-checks`) and
  link-assistant/hive-mind#2323 / #2324 — the same rule for the rest of the
  organization; the local actions keep the same contract so they can
  delegate once those ship.
