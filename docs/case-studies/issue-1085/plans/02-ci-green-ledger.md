# Plan 02 -- a check that was green on identical input is not run twice

Closes the remainder of #1107; answers "if the previous commit is fine, skip
the same check twice".

## Why the current gating is not enough

`detect-changes` compares the **complete PR range** (`base..head`) on every
push. A PR that once touched `src/` re-runs every Rust job on every later
push, including a docs-only one. The ladder ran (and was cancelled) four times
today on pushes that changed workflows and tests only.

## Design: content-addressed green markers

A composite action `.github/actions/green-ledger` with inputs `check` (a
name) and `paths` (the inputs that decide the check's result). It computes
`digest = git ls-files -s -- <paths> | git hash-object --stdin` (the same key
`formal-ai-binary` already uses) and looks up `actions/cache` key
`green-<check>-<os>-<digest>`.

- **hit** -> output `already-green=true`, and the job prints
  `::notice::<check> was green for identical inputs at <sha>` and writes the
  same to `$GITHUB_STEP_SUMMARY`. The job's remaining steps carry
  `if: steps.ledger.outputs.already-green != 'true'`. The job still
  *completes* (green), so `pipeline-status` and branch protection see a result
  -- but the log says why nothing ran. This is a reported skip, not a silent
  one (the #1107 constraint).
- **miss** -> the job runs; a final step `if: success()` saves the marker
  (a one-line file with the sha and run URL).

Markers are per-branch caches plus `main`'s (GitHub's cache scoping), so a
branch inherits `main`'s green for anything it did not change, and a re-push
inherits its own.

## Which checks, and their input paths

| Check | Job / workflow | paths |
| --- | --- | --- |
| unit-tests | `test` (ubuntu, macos-intel) | src tests data Cargo.toml Cargo.lock build.rs examples |
| macos-core | `macos-core-tests.yml` | same |
| docker | `docker-build` | src data Cargo.* Dockerfile docker/ scripts/ |
| box-projects | `box-language-projects` | src data experiments/box_* Cargo.* |
| e2e-local | `test-e2e-local` | src src/web tests/e2e data Cargo.* package*.json |
| agent-cli-e2e | `agent-cli-e2e.yml` | src data experiments/agent_cli_e2e Cargo.* .github/workflows/agent-cli-e2e.yml |
| ladder-1028 | `issue-1028-agent-ladder.yml` | src data experiments/issue_1028_agent_cli_ladder data/meta/ladder-ratchet.lino Cargo.* |
| agentic-matrix | `agentic-cli-matrix.yml` | src data experiments/agentic_cli_matrix Cargo.* |

Workflow files of the check itself are always in `paths` (a change to the
check re-runs it).

## Steps

- [ ] 1. Write `.github/actions/green-ledger/action.yml` (+ `scripts/`
      digest helper reused from `formal-ai-binary`).
- [ ] 2. Wire the eight checks above. Each: `ledger` step after checkout,
      `if:` on the body steps, `save` step at the end.
- [ ] 3. `main` never skips: the action takes `enabled` =
      `github.ref != 'refs/heads/main'`, so every merge re-verifies.
- [ ] 4. Contract test in `tests/unit/ci-cd/`: every job that uses the ledger
      has (a) a save step gated on `success()`, (b) the notice line, (c) its own
      workflow file in `paths`. Register in the gate registry
      (`data/meta/ci-gates/`), retiring nothing -- but note in
      `check-ci-gate-registry.lino` the defect it addresses (#1107).
- [ ] 5. Record the shrink-only pull-request wall-clock ceiling (#1089 item 4)
      in `data/meta/kernel-ratchet.lino` style: `data/meta/ci-wall-clock.lino`
      with the measured longest-path minutes; `check-ci-wall-clock.rs` reads
      the last run via the API only on `main` (advisory on branches).
- [ ] 6. CONTRIBUTING.md: one paragraph under "A branch pays for what it
      changed" describing the ledger and how to force a re-run
      (`workflow_dispatch`, or bump the check's workflow file).

## Verification

- Push a docs-only commit after this lands: every ledgered job shows the
  notice and finishes in under a minute.
- Push a `src/` change: all ledgered jobs run.
- `main` run after merge: all run (enabled=false there).
