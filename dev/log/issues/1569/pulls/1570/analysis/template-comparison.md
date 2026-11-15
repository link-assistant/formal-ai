# Template comparison for issue #1569

All eight templates were shallow-cloned on 2026-11-15 into
`references/templates/<lang>`; their `.git` directories were removed. The commits are in
`references/templates/SOURCES.tsv`. The shared guide is saved as
`references/CI-CD-BEST-PRACTICES.md` (hive-mind `docs/CI-CD-BEST-PRACTICES.md`, 582 lines).

A file:line reference below points into
`references/templates/<lang>/...` at the recorded SHA. "Observed" means a real GitHub
Actions run in the template repository on `main` (read with `gh api`, read-only).

| lang   | SHA                                        |
|--------|--------------------------------------------|
| rust   | 309387f761f9f0f1b2ea15bfe181cb51f5a9a1fb   |
| js     | 74de898dd6ce548af5bcb9359e1dae75fd4ebd75   |
| python | c532c173801dfb12e681b0730c4abbf93aefd0ca   |
| php    | 3a034b27a56067971ca74dd4c74c7707056b215d   |
| csharp | a5b9a1a46686cfe844f9afa2ace030094969e766   |
| java   | dd803a2d9fa5b3a992b435acce73162274fb9e8b   |
| cpp    | 3769fa70a1bc30ce9516e01f78bcd7edea3b3f80   |
| go     | 65c686846d7327d8c835a0d30b1a76cc6345cfc6   |

## Formal-ai defects D-A..D-D: are they in the templates?

| Defect | Result |
|---|---|
| D-A: test binaries found by globbing `target/*/deps` (Rust 1.100 layout) | **Not present.** No template script or workflow refers to `deps/`. The Rust template measures coverage with `cargo llvm-cov` (`rust/.github/workflows/release.yml:773` `tool: cargo-llvm-cov@0.9.1`, `:780`). cargo-llvm-cov v0.9.1 `src/report.rs` `object_files()` handles the v2 build-dir layout: it accepts `build/<unit>/out` when `fingerprint/` exists and `run/` does not. Observed: run 43566214814 on Rust 1.100.0 passed `Code Coverage` and every `Test` job. |
| D-B: a gate runs only on push to main, so the PR passes and main fails | **Present** in rust, php and csharp (R-1, P-1, C-1 below). The release-preflight design gives the same PR-green/main-red result in js, python and csharp (X-1, informational). |
| D-C: a single PR test job near its timeout | **Not present.** The longest job in each latest release run was 2 min or less (rust 2m, go 2m, others ≤1m), and the timeouts are 15–45 min. |
| D-D: changelog/version checks misbehave | **No verified defect found.** |

## Per-template verified defects

### rust

| ID | Evidence | Why it is a defect | Reproduction | Workaround | Suggested fix |
|---|---|---|---|---|---|
| R-1 (D-B) | `.github/workflows/release.yml:1467-1479` `deploy-docs` runs only when `(github.event_name == 'push' && github.ref == 'refs/heads/main') \|\| ... instant`; `:1531` `uses: actions/configure-pages@v6`. It has no `DEPLOY_GITHUB_PAGES` gate and does not check the Pages source on PRs. | If Pages is not set to "GitHub Actions", every PR passes and every push to main fails at `Configure GitHub Pages`. The aggregate `Pipeline Status` then fails too. | Observed: main run 43566214814 (2026-11-14): `Deploy Rust Documentation` failed with `Get Pages site failed ... Not Found`, followed by `Pipeline Status`. | Set Settings → Pages → Source = GitHub Actions. | Copy python/cpp/csharp: run the deploy only when `vars.DEPLOY_GITHUB_PAGES == 'true'` and emit a `::notice::` otherwise. Or copy js `scripts/check-pages.sh` (`js/.github/workflows/example-app.yml:96-101`, `PAGES_MODE` `warn` on PR, `require` on main) so a PR also reports the missing Pages source. |
| R-2 (minor) | Mixed pinning: `release.yml:90` `uses: actions/checkout@v7` (85 tag refs), while `dtolnay/rust-toolchain@683b58…` is SHA-pinned (29 SHA refs). | zizmor `unpinned-uses` (guide §14) is applied inconsistently. | `grep -nE 'uses: [^ .][^ ]*@v[0-9]' .github/workflows/*.yml` | — | Pin every action to a SHA, as csharp, cpp and go already do. |

### php

| ID | Evidence | Why | Reproduction | Workaround | Fix |
|---|---|---|---|---|---|
| P-1 (D-B) | `.github/workflows/docs.yml:63-64` `if: github.event_name == 'push' && github.ref == 'refs/heads/main'` / `uses: actions/configure-pages@v6.0.0`; `:75` deploy job has the same `if`. There is no `DEPLOY_GITHUB_PAGES` gate. The header comment (`:7-9`) acknowledges that the first deploy fails. | Same as R-1: green on PR, red on main. | Observed: the last 4 docs runs on main (2026-09-15 … 2026-11-13) all failed. The latest failed in `Build docs` → `Configure GitHub Pages` with `Get Pages site failed`, and `Pipeline Status` reported `Failing jobs: build`. | Enable Pages. | Same as R-1. Note that here even the **build** job fails, because `configure-pages` runs inside it. |
| P-2 (minor) | 42 tag-pinned `uses:` and 0 SHA-pinned (e.g. `docs.yml:41` `actions/checkout@v7.0.1`). | Guide §14 / zizmor `unpinned-uses`. | grep as above | — | Pin to SHAs. |

### csharp

| ID | Evidence | Why | Reproduction | Workaround | Fix |
|---|---|---|---|---|---|
| C-1 (D-B) | `.github/workflows/release.yml:276-289` job `codecov-token`: `if: github.event_name == 'push' && github.ref == 'refs/heads/main'` and `exit 1` when `CODECOV_TOKEN` is empty ("Coverage upload is required on main"). | This requirement exists only on main. Same-repo PRs, which do have the secret available, never check it, so a merge turns main red. No other template has it: rust `:792`, java `:182`, go `:199` and python `:364` all emit a `::notice::` and skip. | Observed: main run 43607027094 (2026-11-14): `Check Codecov token` failed with `Set CODECOV_TOKEN ...`. | Add the `CODECOV_TOKEN` secret. | Make it a notice like the other templates, or also run it on same-repo PRs (`github.event.pull_request.head.repo.full_name == github.repository`). |

### java

| ID | Evidence | Why | Reproduction | Workaround | Fix |
|---|---|---|---|---|---|
| J-1 | `.github/workflows/release.yml` has jobs `detect-changes`(41), `lint`(80), `test`(140), `build`(188), `changeset-check`(231), `auto-release`(259), `manual-release-*`(372, 471) and `changeset-pr`(555), but **no aggregate `pipeline-status` job**. Every other template has one: go `release.yml:415`, csharp, and rust/js/python/php/cpp in each workflow. | With no single always-run status job, branch protection has to list every matrix leg by name. Skipped or failed legs are easy to leave out. | Observed: Dependabot PR #26 (plexus-build-api 1.2.0→1.3.0) had `Test (Java 27.0.1, …)` failing on all 3 OSes, and was merged anyway. Main run 43592069029 then failed `Run tests with coverage` (`maven-resources-plugin:3.5.0 ... NoSuchElementException`). | Require each `Test (...)` check by name in branch protection. | Add `pipeline-status` with `if: always()` and `needs:` on every job, and fail on any `failure`/`cancelled` result. Copy go `release.yml:415`. |
| J-2 (minor) | All 36 `uses:` are tag-pinned. | Guide §14. | — | — | Pin to SHAs. |

### go

| ID | Evidence | Why | Reproduction | Workaround | Fix |
|---|---|---|---|---|---|
| G-1 | `.github/workflows/release.yml`: jobs `detect-changes`(36), `changeset-check`(72), `lint`(101), `test`(155), `build`(203), `auto-release`(227), `instant-release`(299) and `changeset-release`(349) have **no `timeout-minutes`**. Only `pipeline-status`(415-418) has one. | A hung `go test` or release step holds a runner for the 360-minute default. Every other template sets `timeout-minutes` on every non-reusable job. | `grep -n timeout-minutes .github/workflows/release.yml` → only line 418. | Cancel by hand. | Add per-job `timeout-minutes` (for example 5 for detect, 15 for lint/test, 30 for release), and assert it in `scripts/workflow-policy.test.mjs`. |

### js, python, cpp

No template-code defect verified beyond X-1 below. In js, `example-app.yml:104` (`actions/configure-pages@v6`) is push-only, but `check-pages.sh` runs on PRs in `warn` mode (`:96-101`). That is the correct pattern.

## X-1 (informational, js/python/csharp): release preflight is `report` on every PR, `release` on main

Evidence: `js/.github/workflows/release.yml:514-519`, `python/.github/workflows/release.yml:276-282`, `csharp/.github/workflows/release.yml:380-386`. The same expression appears in rust (`:162-168`), php (`:165-171`) and cpp (`:142-148`). This follows guide §16 deliberately, because forks have no secrets. Its consequence: on a **same-repository** PR, where the secrets are present, a refused credential is only annotated, and main goes red after merge. Observed on the template repositories themselves:

- js run 43336120258: "npm has no package @link-foundation/example-package-name yet".
- python: the last 5 pushes to main all failed on PyPI `invalid-publisher`.
- csharp run 43607027094: "Publishing identity is unset or still a template placeholder".

Suggested refinement (not a bug report): use `release` mode when `github.event.pull_request.head.repo.full_name == github.repository`, so the author sees the failure before merging.

## Shared defects worth an upstream issue

### S-1. Push-only GitHub Pages / deploy prerequisites: PR green, main red (rust R-1, php P-1; csharp C-1 is the same class)

Draft title: `CI: GitHub Pages deploy fails on every push to main when Pages is not enabled, while PRs pass (false negative on PRs)`

Draft body:
> **Problem.** The docs/Pages deploy runs only on `push` to `main`, and nothing checks the Pages source on pull requests. In a repository whose Settings → Pages → Source is not "GitHub Actions" (every fresh repository created from this template), every PR passes and every push to `main` fails in `actions/configure-pages` with `Get Pages site failed ... Not Found`. The aggregate pipeline-status job then fails as well.
> rust: `.github/workflows/release.yml:1467-1479,1531`. php: `.github/workflows/docs.yml:63-64,75`.
> **Reproduction.** Create a repository from the template, leave Pages disabled, merge any PR that touches docs/code, and watch the main run. Observed in this repository's own main runs (rust 43566214814; php docs runs on main since 2026-09-15).
> **Workaround.** Set Settings → Pages → Source = GitHub Actions.
> **Suggested fix.** Gate the deploy on `vars.DEPLOY_GITHUB_PAGES == 'true'` and print a `::notice::` otherwise (python/cpp/csharp `docs.yml` already do this). Alternatively, run a Pages-source probe on PRs in `warn` mode and on main in `require` mode, as js `scripts/check-pages.sh` does. In php, move `configure-pages` out of the build job so a docs build can still pass.

csharp C-1 can go in its own issue: `CI: "Check Codecov token" runs only on push to main and fails main when CODECOV_TOKEN is unset`. Same reproduction (run 43607027094). Fix: downgrade to `::notice::` as rust/java/go/python do, or also run it on same-repo PRs.

### S-2. Missing aggregate status gate (java J-1)

Draft title: `CI: release.yml has no aggregate pipeline-status job, so a PR with failing tests can be merged and break main`

Body: evidence from J-1. Reproduction: PR #26 merged with all three `Test` legs red, and main run 43592069029 then failed. Workaround: require each Test leg in branch protection. Fix: add an `if: always()` `pipeline-status` job that `needs:` every job, as the go template has at `release.yml:415`.

### S-3. Jobs without `timeout-minutes` (go G-1)

Draft title: `CI: release.yml jobs have no timeout-minutes (default 360 min)`

Body: evidence and fix as in G-1.

## Formal-ai vs templates (high level)

| Practice | Templates | formal-ai |
|---|---|---|
| Rust 1.100 build-dir layout (D-A) | Rust template is unaffected because it uses cargo-llvm-cov 0.9.1 | Fixed in this PR (`scripts/collect-build-artifacts.sh` reads `executable` from `--message-format=json`) |
| `timeout-minutes` on every job | all except go | all 59 workflows |
| Aggregate pipeline-status job | all except java | yes |
| Fresh-merge validation (guide §7) | rust, js, python, cpp | yes (`scripts/simulate-fresh-merge.sh`) |
| Budget warning before timeout (D-C) | rust, js, python, php, csharp, cpp | yes (`TEST_BUDGET_SECONDS`, 75% warning). This PR splits the PR JS group in two. |
| Rerun-infrastructure-failures / rerun-unacquired workflow | rust, js, csharp, java, cpp, go | **missing** (a template practice formal-ai lacks) |
| Pages deploy gated by variable or PR probe | python, cpp, csharp, js | `release.yml:973-1002` `deploy-pages` is push-only with no variable gate. Pages is enabled for formal-ai, so it does not fail today, but it has the same shape as R-1 |
| SHA-pinned actions (guide §14, zizmor `unpinned-uses`) | csharp, cpp and go fully; rust, js and python partly; php and java not at all | **584 tag-pinned and 2 SHA-pinned** `uses:` |
| Preflight `report` on same-repo PRs (X-1) | same in all | same (`release.yml:286-291`) |
