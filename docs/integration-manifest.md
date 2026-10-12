# Integration manifest — deferred existing-file edits

The wave-3 fleet drafts new files freely, but edits to files another fork
owns (or that the integration GUARD holds until the `feat(issue-1168)` and
`feat(issue-1169)` commits land) are recorded here instead, with exact edit
plans. The main session applies and ticks them during integration; the
`issue_1187_credentials` unit test reads this file so a deferred edit can
never be forgotten silently — every `pending:` row is asserted to name a
file, a section, and an edit.

## Pending

- `pending:` `.github/workflows/release.yml` — #1187 R1: the five
  `build-push-action` steps' registry credentials come from
  `actions/automation-token` where they push to GitHub (release.yml reads
  no `FORMAL_AI_BOT_TOKEN`; its GitHub writes use `secrets.GITHUB_TOKEN`);
  #1084: the five steps (lines ~151, 715, 760, 920, 963 as of e4193a615)
  pass `platforms: linux/amd64,linux/arm64` or delegate to
  `container-images.yml` via `workflow_call`.
- `applied:` `.github/workflows/release.yml` — #1187 R3: `workflow_dispatch`
  declares the checks-mode `mode` input and a `release_mode` that defaults to
  `checks`; `manual-release`, `changelog-pr`, the Pages deploy and the release
  preflight run only for an explicit `instant` or `changelog-pr` mode on
  `refs/heads/main`. `actions/dispatch-checks` may now dispatch release.yml,
  which then runs its lint and test jobs only.
- `pending:` `.github/workflows/cross-org-duplication.yml` — #1187 R5 (this
  branch adds it, new): after creation no pending edit; listed because its first
  scheduled run must confirm the resolver degrades (R5) rather than files
  cross-repository issues at layer `default`.
- `pending:` `.github/workflows/layered-ci.yml` — #1088: add a step running
  `rust-script scripts/check-evidence-lines.rs --base origin/main` and a
  step running `rust-script scripts/check-evidence-lines.rs --files` (the
  gate arms itself once `docs/evidence/index.lino` carries no `pending-move`
  URL), so the 2,000-line budget and the tracked-file limit are enforced on
  every pull request.

## Main-session actions

Not workflow or manifest edits, so not `pending:` rows (the unit test reads
those as deferred `.yml`/`.toml` edits), but still owed:

- `action:` `data/meta/duplicate-functions-baseline.lino` — #1182: created
  by `rust-script scripts/check-duplicate-functions.rs --write-baseline`
  (the file is forbidden to hand-edit); first run is a main-session action
  because it must follow the #1182 adapter landing.
- `note:` `rust/Cargo.toml` — the version-currency fork (J, #1169) owns it
  while its commits land; no wave-3A edit is pending on it.

## Applied

- `applied:` `rust/tests/unit/mod.rs` — the wave-3A test files are
  registered: `issue_1187_credentials`, `issue_1090_manual_column_retired`,
  `issue_954_module_map`, `issue_1088_evidence_index`, and the #1182 pair
  (`issue_1182_links_notation_conformance`, `issue_1182_duplicate_gate`).
- `applied:` `.github/actions/automation-token/action.yml` (new, this
  branch) — the resolver itself.
- `applied:` `.github/actions/dispatch-checks/action.yml` (new, this
  branch) — the R2 dispatcher.
- `applied:` `.github/workflows/container-images.yml` (new, this branch) —
  multi-arch builds (#1084) + slim publishing and size budget (#1153).
- `applied:` `.github/workflows/e2e-isolation.yml` (new, this branch) — R4
  orphan-branch isolation.
- `applied:` `.github/workflows/self-authored-pull-request.yml` — #1187
  R1: the author job takes `GH_TOKEN` from `actions/automation-token`
  (no `secrets.FORMAL_AI_BOT_TOKEN`; the action's `FORMAL_AI_BOT_TOKEN`
  variable is set only at layer `app`/`token`); R2: at layer `default` it
  calls `actions/dispatch-checks` after the bot push, with `actions: write`.
- `applied:` `.github/workflows/layered-ci.yml`, `evidence-check.yml`,
  `workflows.yml`, `web-ui-boundary.yml` — #1187 R2: a checks-mode
  `workflow_dispatch` (`mode: checks`, `pull-request`), the inputs
  `actions/dispatch-checks` passes; `evidence-check.yml` falls back to the
  default branch when a dispatched run has no `github.base_ref`.
- `applied:` every `pull_request`-triggered workflow — #1187 R4:
  `branches-ignore: ['e2e/**']`, so an isolated orphan-branch pull request
  never triggers this repository's own pipeline.
