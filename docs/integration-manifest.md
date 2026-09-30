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
  `build-push-action` steps' credentials come from
  `actions/automation-token` (replace `FORMAL_AI_BOT_TOKEN` reads);
  #1084: the five steps (lines ~151, 715, 760, 920, 963 as of e4193a615)
  pass `platforms: linux/amd64,linux/arm64` or delegate to
  `container-images.yml` via `workflow_call`; #1187 R3: a `mode` input
  (default `checks`) guards every release/publish/tag job with
  `github.ref == 'refs/heads/main'` (the `workflow_dispatch` jobs at lines
  ~193, 263, 295, 383, 580 currently run on any dispatch).
- `pending:` `.github/workflows/layered-ci.yml` — #1187 R2: add
  `workflow_dispatch` with `mode: checks` + `pull-request` inputs; add
  `branches-ignore: e2e/**` to the `pull_request` trigger (R4).
- `pending:` `.github/workflows/evidence-check.yml` — same as layered-ci.
- `pending:` `.github/workflows/workflows.yml` — same as layered-ci.
- `pending:` `.github/workflows/self-authored-pull-request.yml` — #1187 R1:
  `FORMAL_AI_BOT_TOKEN` → the resolver (`AUTOMATION_TOKEN` fallback chain);
  after opening/pushing a PR at layer `default`, call
  `actions/dispatch-checks` (R2).
- `pending:` `.github/workflows/cross-org-duplication.yml` (this branch adds
  it, new) — after creation no pending edit; listed because its first
  scheduled run must confirm the resolver degrades (R5) rather than files
  cross-repository issues at layer `default`.
- `pending:` `rust/Cargo.toml` — the version-currency fork (J) owns it
  while its commits land; no wave-3A edit is pending on it.

## Applied

- `applied:` `.github/actions/automation-token/action.yml` (new, this
  branch) — the resolver itself.
- `applied:` `.github/actions/dispatch-checks/action.yml` (new, this
  branch) — the R2 dispatcher.
- `applied:` `.github/workflows/container-images.yml` (new, this branch) —
  multi-arch builds (#1084) + slim publishing and size budget (#1153).
- `applied:` `.github/workflows/e2e-isolation.yml` (new, this branch) — R4
  orphan-branch isolation.
