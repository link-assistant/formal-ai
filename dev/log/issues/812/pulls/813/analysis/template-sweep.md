# Template sweep: CI/CD file-by-file comparison against the three upstream pipeline templates

Issue: https://github.com/link-assistant/formal-ai/issues/812 (requirement 7)

## 0. Scope and method

Compared the full CI/CD surface of this repo against three upstream templates and the
shared best-practices document:

| Source | Local copy |
| --- | --- |
| `link-foundation/rust-ai-driven-development-pipeline-template` | `/tmp/templates/rust`, archived at `dev/log/issues/812/pulls/813/templates/rust-ai-driven-development-pipeline-template/` |
| `link-foundation/js-ai-driven-development-pipeline-template` | `/tmp/templates/js`, archived at `dev/log/issues/812/pulls/813/templates/js-ai-driven-development-pipeline-template/` |
| `link-foundation/python-ai-driven-development-pipeline-template` | `/tmp/templates/python`, archived at `dev/log/issues/812/pulls/813/templates/python-ai-driven-development-pipeline-template/` |
| `link-assistant/hive-mind` `docs/CI-CD-BEST-PRACTICES.md` | `dev/log/issues/812/pulls/813/templates/CI-CD-BEST-PRACTICES.md` |

Surfaces compared: `.github/workflows/**`, `.github/actions/**`, `scripts/**`,
dependabot config, CI-relevant issue templates.

Template CI/CD trees (complete):

```
rust:   .github/workflows/release.yml
        .github/actions/setup-buildx-resilient/action.yml
js:     .github/workflows/release.yml, links.yml, example-app.yml
        .github/actions/setup-buildx-resilient/action.yml, publish-dockerhub/action.yml
python: .github/workflows/release.yml, docs.yml
```

This repo: `.github/workflows/release.yml` (1794 lines), `.github/workflows/desktop-release.yml`
(704 lines), `.github/actions/setup-buildx-resilient/action.yml`, `scripts/**` (55 files).

None of the three templates ship a `dependabot.yml` or CI-relevant issue templates, so
there is nothing to adopt there; this repo matches them by omission (class c).

### Relationship to the previous sweep

`dev/log/issues/810/pulls/811/analysis.md` section 7 already covered and closed:
desktop-release PR `paths` filter (now includes `src/**`, `Cargo.toml`, `Cargo.lock`,
`.github/workflows/desktop-release.yml:50-65`), `!cancelled()` adoption on the main
release gate jobs, `setup-buildx-resilient` parity, `install-rust-script.sh` parity
(this repo's copy is a superset with retry knobs), and per-job `timeout-minutes`
(verified: **all 17 release.yml jobs, all 4 desktop-release.yml jobs, and all template
jobs have `timeout-minutes`** — no gap). Those are not re-reported below.

Its two "still open" items were re-verified and **both are still open** — see F1 and F2.

### Verification notes (things that look like defects but are not)

* Every `scripts/*.sh` in this repo does set strict mode; an earlier `head -5` heuristic
  gave false negatives because of long header comments. Confirmed by whole-file grep:
  `check-secrets.sh:27`, `desktop-release-resolve.sh:52`, `free-runner-disk.sh:16`,
  `reproduce-issue-538.sh:26`, `simulate-fresh-merge.sh:21`, `sync-seed.sh:14` all use
  `set -euo pipefail`. Only `scripts/install.sh:30` uses `set -eu` (see F11).
* `desktop-release.yml:432-434` `[ -f "$f" ] && files+=("$f")` inside a `for` loop does
  **not** trip `set -e` (verified empirically with bash 5): the `&&` exemption applies and
  the loop exits 0. Not a defect.
* `desktop-release.yml:435-438` and `:621-624` both guard against empty arrays; the
  "collect" path fails loudly rather than silently producing an empty manifest. Good.
* All actions are pinned by major tag (`@v6`/`@v7`/`@v8`/`@stable`), identical to all
  three templates. Not a divergence (class c); SHA pinning would be a repo-wide policy
  change, not a template-parity item.
* Artifact `retention-days` is set on every non-coverage upload. Templates set it too.

## 1. Findings table

Class: **(a)** real defect in this repo, **(b)** best practice worth adopting,
**(c)** intentional difference, **(d)** defect in a template to report upstream.

| ID | file:line | Class | Severity | Evidence | Proposed fix |
| --- | --- | --- | --- | --- | --- |
| F1 | `scripts/check-file-size.rs:1` (whole file) vs `/tmp/templates/js/scripts/check-file-line-limits.sh:1` | a (false negative) | High | The size gate covers only `.rs` (1000/900), `.lino` (1500/1400) and worker `.js` (1500/1400). Neither `.md` nor workflow YAML is checked. `.github/workflows/release.yml` is **1794 lines** — 294 over the js template's 1500 ceiling — and CI is green. The js template explicitly walks `.github/workflows/release.yml` with the hint "Move inline scripts to the ./scripts/ folder to reduce file size." | Add a `.yml`-under-`.github/workflows/` rule (limit 1500, warn 1350) and a `.md` rule to `check-file-size.rs`; then extract the largest inline `run:` blocks of `release.yml` into `scripts/`. Prior sweep item (i), still open. |
| F2 | `.github/workflows/desktop-release.yml:143-497` (no invocation of `scripts/simulate-fresh-merge.sh`) | a (false negative) | High | `grep -rn simulate-fresh-merge .github/workflows` returns nothing for desktop-release. The packaging dry run builds the **PR head commit**, not the merge result, so a desktop packaging break introduced by a concurrent main commit is invisible until after merge. Both the rust and js templates run their fresh-merge simulation before the heavy jobs. | Add a `fresh-merge` step (or job) to `desktop-release.yml` for `pull_request` events that runs `scripts/simulate-fresh-merge.sh` before `build`. Prior sweep item (ii), still open. |
| F3 | `.github/workflows/release.yml:641-642`, `:677-681` | a (false negative) | High | The release gate is `build` ← `[lint, test]` and `auto-release` ← `[lint, test, build]`. `test-e2e-local` (`:1199`), `test-agent-cli-e2e` (`:1279`), `secrets-scan` (`:213`) and `docker-build` (`:159`) are **not** in the gate, so a push to main can publish a crate + GitHub release while the E2E suites and the secret scan are red. | Add `test-e2e-local`, `test-agent-cli-e2e` and `secrets-scan` to `auto-release.needs` with `needs.<job>.result == 'success'` guards, matching the `!cancelled() && needs.*.result` style already used at `:642`. |
| F4 | `.github/workflows/desktop-release.yml:588-591` | a (error waiting to happen) | High | `if: ${{ always() && ... }}`. `always()` also fires when the run is **cancelled**, so cancelling a desktop release still downloads whatever fragments exist and runs `gh release upload SHA256SUMS.txt BUILD-PROVENANCE.txt --clobber` (`:696`), overwriting a good manifest with a partial one. The js template calls this out explicitly: "Use `!cancelled()` instead of `always()` so cancellation propagates correctly (hive-mind issue #1278)" (`/tmp/templates/js/.github/workflows/release.yml:260,432`). | Change to `!cancelled() && github.event_name != 'pull_request' && needs.resolve.outputs.should_build == 'true'`. The stated intent (partial matrix under `fail-fast: false`) is preserved by `!cancelled()`. |
| F5 | `.github/workflows/release.yml:1735-1738` vs `/tmp/templates/rust/.github/workflows/release.yml:947` | a (error waiting to happen) | Medium | `actions/upload-pages-artifact` with `path: src/web` and **no `include-hidden-files: true`**. Any dotfile the site needs (`.nojekyll`, `.well-known/`, `.htaccess`-style assets) is silently dropped from the Pages tarball. The rust template sets `include-hidden-files: true`. | Add `include-hidden-files: true` to the `upload-pages-artifact` step. |
| F6 | `.github/workflows/release.yml:1263-1271` | a (passes while doing nothing) | Medium | `Upload generated /download screenshots` is `if: always()` with no `if-no-files-found`. The default is `warn`, so if the issue-347 spec never regenerates the screenshots the step is green and the artifact is empty — the exact "proving the screenshots can be produced in CI" claim in the comment above it goes unverified. Contrast `:604-608`, which correctly uses `if-no-files-found: error` for LCOV. | Set `if-no-files-found: error` and change `if: always()` to `if: !cancelled()`. |
| F7 | `.github/workflows/desktop-release.yml:492-497`, `:578-583` | a (passes while doing nothing) | Medium | The `checksums-*` fragment uploads have no `if-no-files-found: error`. A leg that packaged nothing but somehow reached this step uploads an empty artifact; `finalize` then reports that label as `missing` only via the label list, and the `built[]`/`missing[]` accounting at `:643-651` depends on fragment presence rather than content. | Add `if-no-files-found: error` to both fragment uploads. |
| F8 | `.github/workflows/desktop-release.yml:161-164` | a (over-scoped permissions) | Medium | The `build` job grants `contents: write`, `id-token: write`, `attestations: write` unconditionally, including for `pull_request` dry runs where `:461` and `:473` disable both attest and upload. Any compromised build/packaging dependency on a fork-less PR run gets a write token it never needs. Best-practices doc: least privilege, elevate only where used. | Job-level `permissions: contents: read` is not expressible per-event; split the write-needing steps into a separate job, or gate on `github.event_name != 'pull_request'` by moving attest+upload into a small dependent job that carries the write scopes. |
| F9 | `.github/workflows/release.yml:913-925`, `:1133-1145`, `:1264` | a (misuse of `always()`) | Medium | Three remaining `if: always()` uses (two `Resolve Pages deploy ref` steps and the screenshot upload). Per the best-practices doc and the js template comments, `always()` defeats cancellation propagation; `!cancelled()` is the intended idiom and is already used at `:219,282,497,569,642,681`. | Replace all three with `!cancelled()`. |
| F10 | `scripts/simulate-fresh-merge.sh:44`, `:54` vs `/tmp/templates/rust/scripts/simulate-fresh-merge.sh` | a (shell safety) + b | Medium | `git rev-list --count HEAD..origin/$BASE_REF` and `git merge origin/$BASE_REF --no-edit` leave `$BASE_REF` unquoted (`:37` quotes it correctly, so the file is internally inconsistent). This repo adopted the **js** variant verbatim; the **rust** variant is materially better: quotes everything, short-circuits with `git merge-base --is-ancestor`, merges into a *detached* checkout of the base tip so the working branch is never mutated, then runs `cargo fmt/clippy/test` under a `FRESH_MERGE_CHECKS` override and restores the branch. | Quote both expansions now; then port the rust template's detached-merge + `FRESH_MERGE_CHECKS` design, which is the correct fit for a Rust repo. |
| F11 | `scripts/install.sh:30` | a (shell safety) | Medium | `set -eu` — no `pipefail`. The script pipes curl output into parsers (e.g. release-JSON extraction); a failing `curl` in the middle of a pipeline is masked by the exit status of the last stage, so a network failure can degrade into "installed nothing, exit 0". Every other shell script in the repo uses `set -euo pipefail`. | Use `set -euo pipefail`. If POSIX `sh` compatibility is the reason, the shebang should be `#!/usr/bin/env bash` (it already is) — so there is no reason not to. |
| F12 | `scripts/check-secrets.sh:80-84` vs `/tmp/templates/rust/.github/workflows/release.yml:147` | a (false negative) + b | Medium | Two divergences. (1) Supply chain: `npx --yes -p secretlint -p @secretlint/secretlint-rule-preset-recommend secretlint` downloads unpinned latest on every run. (2) Scope: this repo scans only the diff against `BASE_REF`; the rust template scans the **whole tree** (`secretlint --secretlintignore .gitignore "**/*"`). A secret committed before the gate existed, or on a branch whose base already contains it, is never reported. | Pin versions (`-p secretlint@x.y.z -p @secretlint/secretlint-rule-preset-recommend@x.y.z`); add a scheduled or `push`-to-main full-tree scan in addition to the fast diff-scoped PR scan. |
| F13 | `.github/workflows/release.yml:365` vs `/tmp/templates/rust/.github/workflows/release.yml:310-324` | a (silently succeeds on empty input) | Medium | `FILE_SIZE_WARNING_BASE: ${{ github.event.pull_request.base.sha \|\| github.event.before }}` is **empty** on `workflow_dispatch` and on the first push of a new branch (all-zeroes SHA). The rust template solves this with a dedicated `Collect changed files` step that falls back to `git ls-files` and passes an explicit `CHANGED_FILES` list. | Port the rust template's `Collect changed files` step (with `git ls-files` fallback) and feed `CHANGED_FILES` to `check-file-size.rs`. |
| F14 | absent: no `cargo-lock` guard job; cf. `/tmp/templates/rust/.github/workflows/release.yml:224-253` and `/tmp/templates/rust/scripts/check-cargo-lock.rs` | b | Medium | The rust template runs `cargo metadata --locked` in a dedicated `cargo-lock` job that gates `lint`, `test` and `coverage` via `needs.cargo-lock.result == 'success'`, so a stale `Cargo.lock` fails in ~30 s instead of after a full build matrix. This repo has no equivalent (`scripts/check-cargo-lock.rs` does not exist). | Port `check-cargo-lock.rs` and the `cargo-lock` job; add it to the `needs` of `lint`/`test`/`coverage`. |
| F15 | `.github/workflows/release.yml:506-510` vs `/tmp/templates/rust/.github/workflows/release.yml:364` | a (false negative) | Medium | `test` matrix is `os: [ubuntu-latest]` only; the rust template runs ubuntu + macos + windows. This is documented as a CI-time trade-off, but this repo **ships macOS and Windows desktop binaries** (`desktop-release.yml:143` 6-way matrix), so platform-specific Rust regressions are only caught by the (much later, release-only) desktop packaging job. | At minimum add macOS + Windows to the matrix for pushes to main, keeping ubuntu-only for PRs; or move the platform legs behind a `paths` condition on `src/**`. |
| F16 | absent: no link checking; cf. `/tmp/templates/js/.github/workflows/links.yml:1-100` | b | Medium | The js template ships a lychee broken-link workflow with `fail: false`, a web-archive fallback (`scripts/check-web-archive.mjs`) and an explicit fail step. This repo has neither a links workflow nor a `.lycheeignore`, while shipping a docs site plus a large `docs/` and `dev/log/` tree full of URLs. | Port `links.yml` + `scripts/check-web-archive.mjs` + `.lycheeignore`, scheduled weekly and on `docs/**` changes. |
| F17 | `.github/workflows/release.yml:1602-1611` vs `/tmp/templates/rust/.github/workflows/release.yml:894-902` | b (arguably c) | Medium | `deploy-pages` requires `auto-release`/`manual-release` to have succeeded, so **a publish failure also blocks the website update**. The rust template deliberately makes `deploy-docs` depend on `build` only, with the comment "Keep this independent from package/GitHub release publication so the website still updates when the release path fails." | Move `deploy-pages` off the release jobs onto `build` (plus whatever supplies the version stamp), or make the release dependency advisory via `needs.*.result != 'failure'`. If the coupling is intentional (the site advertises the just-published version), record that rationale inline so the divergence is auditable. |
| F18 | `.github/workflows/desktop-release.yml:629` | a (`\|\| true`-style masking) | Low | `commit="$(gh release view "$TAG" ... --jq .targetCommitish \|\| echo unknown)"` turns an API failure into the literal string `unknown` in the published `BUILD-PROVENANCE.txt`. Provenance that silently says "unknown" is worse than a failed step. | Fail the step, or emit `::warning` and mark the provenance file `Commitish : UNKNOWN (gh release view failed, see run log)`. |
| F19 | `.github/workflows/desktop-release.yml:661` | a (error waiting to happen) | Low | `printf 'Builders   : %s\n' "${built[0]}"` is an unguarded index under `set -u`. If fragments exist but none match the seven hardcoded labels at `:645` (e.g. a new matrix label added at `:143` without updating `builder_of` at `:634-642`), the step dies with an opaque `built[0]: unbound variable`. | Guard with `if [ ${#built[@]} -eq 0 ]; then echo "::error::no recognised builder labels in fragments"; exit 1; fi`, or derive the label list from `fragments/*.partial` instead of hardcoding it. |
| F20 | `.github/workflows/desktop-release.yml:427-431`, `:482-486` | a (shell safety) | Low | Both glob loops lack `shopt -s nullglob`; on no match, `$f` is the literal pattern `release/formal-ai-desktop-*`. The collect loop survives because of the `:435` emptiness guard, but the **upload** loop at `:482` has no such guard: an empty `files` array makes `:490` call `gh release upload "$TAG" --repo ... --clobber` with zero file arguments, producing a confusing `gh` usage error instead of a clear diagnostic. Also note the two loops duplicate a third, independent extension list in the node snippet at `:445` — three places to keep in sync. | Add `shopt -s nullglob` to both loops, add the emptiness guard to the upload loop, and hoist the extension pattern into one shared variable (or into `scripts/`, which also helps F1). |
| F21 | `scripts/desktop-release-resolve.sh:79,141,158,214`; `scripts/stamp-pages-artifact.sh:63,72`; `scripts/free-runner-disk.sh:50` | c (verified benign) | Info | Audited every `\|\| true` in `scripts/`. All are legitimate: `desktop-release-resolve.sh` uses them for optional `gh` lookups whose emptiness is explicitly handled on the next line (`:215-216` "leaving build enabled rather than risking a silent skip"); `stamp-pages-artifact.sh:63,72` uses `grep ... \|\| true` and then **fails** on non-empty output (`:65-69`, `:74-78`) — inverted-sense grep, correct; `free-runner-disk.sh:50` is a best-effort docker prune. `check-secrets.sh:40` `grep -zv ... \|\| true` is the standard empty-result guard, and `:70-73` exits 0 on an empty file list with an explicit "No files to scan." message. No action. |
| F22 | `/tmp/templates/python/.github/workflows/release.yml` (no top-level `permissions:`; only per-job at `:413` and `:524`) | d | Medium | The python template is the only one of the three without a workflow-level `permissions: contents: read`. Jobs without an explicit block therefore inherit the repository default, which is `write-all` in older orgs and any repo whose default was never tightened. See draft issue in section 2.1. |
| F23 | `/tmp/templates/python/.github/workflows/release.yml:105,186,517` | d | Low | `always() && !cancelled()` is redundant and misleading — `always()` is the identity element for `&&` here, so the expression is exactly `!cancelled()`. The js template already documents the correct idiom. See draft issue in section 2.2. |

## 2. Draft upstream issues (not filed)

### 2.1 python template: missing workflow-level `permissions:` in `release.yml`

**Title:** `release.yml` has no workflow-level `permissions:` — jobs inherit the repository default token scope

**Body:**

> `.github/workflows/release.yml` declares `permissions:` only on two jobs
> (around lines 413 and 524). There is no workflow-level block. Every other job in
> the file — `detect-changes`, `lint`, `test`, `coverage`, `build`, `docs` — therefore
> inherits the repository's *default* `GITHUB_TOKEN` permissions.
>
> The sibling templates both set a restrictive default:
>
> * `js-ai-driven-development-pipeline-template/.github/workflows/release.yml:37-38` → `permissions:\n  contents: read`
> * `rust-ai-driven-development-pipeline-template/.github/workflows/release.yml:33-34` → `permissions:\n  contents: read`
>
> **Reproduction**
>
> 1. Create a repository from this template in an organisation whose default workflow
>    permissions are still "Read and write permissions"
>    (Settings → Actions → General → Workflow permissions — the pre-2023 default, still
>    in force for many existing orgs).
> 2. Open a pull request from a branch in the same repository.
> 3. In the `lint` job add a step: `run: gh api -X PATCH repos/$GITHUB_REPOSITORY -f description=pwned`
>    with `GH_TOKEN: ${{ github.token }}`.
> 4. It succeeds. The linting job — which only needs to read code — has repo write.
>
> Any compromised transitive dependency executed during `pip install`/lint/test in those
> jobs gets the same write scope, on a workflow that is triggered by pull requests.
>
> **Workaround (for template users)**
>
> Add to your own copy, immediately after `on:`:
>
> ```yaml
> permissions:
>   contents: read
> ```
>
> …and set the repository default to "Read repository contents permission" in
> Settings → Actions → General.
>
> **Suggested fix**
>
> ```diff
>  on:
>    push:
>      branches: [main]
>    pull_request:
>  
> +# Least privilege by default; jobs that need more elevate explicitly
> +# (see the `release` and `docs` jobs below).
> +permissions:
> +  contents: read
> +
>  jobs:
> ```
>
> The two jobs that already declare `permissions:` are unaffected — a job-level block
> replaces, not merges with, the workflow-level one. `docs.yml` already gets this right
> (`.github/workflows/docs.yml:31-34`), so this is purely an omission in `release.yml`.

### 2.2 python template: `always() && !cancelled()` is redundant

**Title:** `always() && !cancelled()` in `release.yml` should be just `!cancelled()`

**Body:**

> `.github/workflows/release.yml` uses `always() && !cancelled()` at lines 105, 186 and
> 517 (with explanatory comments at 101, 182, 513). `always()` evaluates to `true`
> unconditionally, so `always() && X` is identical to `X`. The expression reads as though
> the two functions combine to produce something neither does alone, which invites the
> copy-paste of a genuinely wrong `always()` elsewhere — line 260 in the same file is a
> bare `always() && (...)`, which *does* differ (it runs on cancellation).
>
> **Reproduction**
>
> Add a workflow with two jobs:
>
> ```yaml
> jobs:
>   a: { runs-on: ubuntu-latest, steps: [{ run: sleep 300 }] }
>   b:
>     needs: [a]
>     if: ${{ always() && !cancelled() }}
>     runs-on: ubuntu-latest
>     steps: [{ run: echo ran }]
>   c:
>     needs: [a]
>     if: ${{ !cancelled() }}
>     runs-on: ubuntu-latest
>     steps: [{ run: echo ran }]
> ```
>
> Cancel the run: `b` and `c` are both skipped. Let `a` fail: `b` and `c` both run.
> The two conditions are indistinguishable in every case.
>
> **Workaround**
>
> None needed — behaviour is already correct; this is a clarity/consistency defect.
>
> **Suggested fix**
>
> ```diff
> -      always() && !cancelled() && (
> +      !cancelled() && (
> ```
>
> at lines 105, 186 and 517, and update the comments at 101, 182 and 513 to the wording
> already used in the js template: "Use `!cancelled()` instead of `always()` so
> cancellation propagates correctly (hive-mind issue #1278)". Separately, line 260's bare
> `always()` should be reviewed — if the intent there is also "run even when an upstream
> dependency was skipped", it is a latent bug and should become `!cancelled()` too.

## 3. Confirmed parity (no action)

* `.github/actions/setup-buildx-resilient/action.yml` — byte-identical to both the rust
  and js template copies apart from provenance comments. (class c)
* `scripts/install-rust-script.sh` — superset of the rust template's: adds
  `RUST_SCRIPT_INSTALL_ATTEMPTS` / `RUST_SCRIPT_INSTALL_RETRY_DELAY_SECONDS` retry knobs.
  This repo is ahead; consider upstreaming. (class c/b-upstream)
* `timeout-minutes` — present on all 21 jobs across both workflows and on all template
  jobs. (class c)
* Top-level `permissions: contents: read` — present in `release.yml:32-33` and
  `desktop-release.yml:67-68`, matching rust and js. (class c)
* Concurrency groups — `release.yml:35-42` (`cancel-in-progress` only off main) and
  `desktop-release.yml:70-80` (keyed on PR number / tag / head SHA). Both present and
  documented. The best-practices doc phrases the main-branch guard as
  `github.ref == 'refs/heads/main'` inverted; this repo's `!=` form is equivalent.
  (class c)
* `fail-fast: false` on both matrices (`release.yml:506`, `desktop-release.yml:150`),
  matching the templates. (class c)
* LCOV artifact upload uses `if-no-files-found: error` (`release.yml:604-608`) — the
  correct pattern, and the model for F6/F7. (class c)
* No `continue-on-error:` anywhere in either workflow. (class c)
* No dependabot config and no CI-relevant issue templates in any of the three templates;
  this repo matches. (class c)
