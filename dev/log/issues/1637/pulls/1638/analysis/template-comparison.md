# Issue #1637 — CI/CD comparison: formal-ai vs. the eight pipeline templates

Collected 2026-11-25. Baseline for "what changed": 2026-11-14 (the previous
comparison lives in `dev/log/issues/1569/pulls/1570/analysis/template-comparison.md`).
Reference guide: hive-mind `docs/CI-CD-BEST-PRACTICES.md` (cited as §N).

## Template heads inspected (shallow clones, `/tmp/f1637-templates/<lang>`)

| Template | HEAD | Relevant commits since 2026-11-14 |
|---|---|---|
| js | `8e72538` | `526c34a` bind Pages deploy retries to the artifact; `7bdabe9` whole-file lint + bounded dependency checks |
| rust | `1d29f55` | `d1df227` require complete release validation / no `gh` deadlocks; `cd96c10` paginate runner annotations; `0ff7dfa` preserve changelog fragments; `5061bb5` Pages opt-in (fixes R-1) |
| python | `7a05d05` | `03daa6b` scope Pages recovery to the artifact; `70ad2b4` run checks for JS-only changes; `eab449e` budget network installs |
| php | `e12a25d` | `2841339` Pages opt-in (fixes P-1); `01b1cce` fail on unavailable comparison evidence; `fb40c38` refresh action pins |
| csharp | `10edf2e` | `1220708`, `6a755e5` Codecov on same-repo PRs (fixes C-1) |
| java | `dd803a2` | none (J-1 still open as java#29) |
| cpp | `c1c60fb` | `524fd69` require complete evidence; `8b65947` paginate annotations |
| go | `785292e` | `6df6df3` enforce release prerequisites; `09f6d30` verify pins; G-1 fixed (all jobs have timeouts) |

Status of defects reported in #1569: R-1, P-1, C-1, G-1 **fixed upstream**; J-1
**open** (java#29); X-1 (preflight report-mode on PRs) unchanged.

## Practice matrix

Legend: Y = present, N = missing, P = partial, — = not applicable.

| Practice (guide §) | js | rust | python | php | csharp | java | cpp | go | **formal-ai** |
|---|---|---|---|---|---|---|---|---|---|
| Runtime/toolchain pinned to exact version (§1) | P (`24.x`) | Y (`rust-toolchain.toml` 1.100.0 + 17 `toolchain:` inputs) | Y (3.15.2 + `.python-version`) | Y (8.5.12) | Y (`global.json` 11.0.100) | Y (27.0.1) | Y (gcc-15/clang-22) | Y (1.27.4) | **N** — `dtolnay/rust-toolchain@stable` ×68, no `rust-toolchain.toml` (release-staged only pins via captured `rust_version`) |
| Toolchain action SHA-pinned (§14) | — | Y (dtolnay `683b588…`) | — | — | — | — | — | — | **N** (tag `@stable`) |
| Full lint on PRs (§1 commit tier) | Y | Y (`clippy --all-targets`) | Y | Y | Y | Y | Y | Y | **P** — PR clippy is `--lib --bins` only; `--tests`, `cargo check --examples`, fmt, ordered-lists and dependency-currency gates are main-only |
| Full tests on PRs | Y | Y | Y | Y | Y | Y | Y | Y | **P** — curated subset (`native-type-check`); full shards, specification, server-parity, e2e are main-only |
| Docs build (`-D warnings`) on PRs | Y | Y | — | — | — | — | — | — | **N** — `cargo doc` with `RUSTDOCFLAGS=-D warnings` only in `build-pages` (main) |
| MSRV leg actually tests MSRV | — | **N** (leg overridden by `rust-toolchain.toml`, see upstream report) | — | — | — | — | — | — | **N** (`rust-version = "1.99"`, no leg) |
| Action majors current | **N** (`actions/cache/*@v5`, latest v6) | Y | Y | Y | Y | Y | Y | Y | **N** — `setup-python@v6` ×2, `setup-node@v6` ×2 (latest v7) |
| Third-party actions SHA-pinned (§14), pinned/total | P 12/87 | P 29/117 | N 3/61 | N 0/61 | Y 67/68 | N 0/37 | Y 73/74 | Y 24/25 | **N** 2/586 |
| Automated action bumps (dependabot) | **N** | Y (+`rust-toolchain` ecosystem) | Y | Y | Y | Y | Y | Y | P (own `dependencies-latest` workflow; catches drift only on main) |
| Concurrency (§10) | Y | Y | Y | Y | Y | Y | Y | Y | Y (39/61 workflows, 65/181 jobs; no cancellable job has write perms) |
| `timeout-minutes` on every job (§5) | Y | Y | Y | Y | Y | Y | Y | Y | Y (0 of 181 missing) |
| Aggregate pipeline-status | Y | Y | Y | Y | Y | **N** (java#29) | Y | Y | Y |
| Pages deploy gated by variable/mode (§17) | Y (`PAGES_MODE`) | Y (`DEPLOY_GITHUB_PAGES`) | Y | Y | Y | Y | Y | Y | **N** — push-to-main only, no gate; deploy currently fails with `pages/deployments` HTTP 404 |
| Pages artifact built on PRs | Y | Y | Y | Y | Y | Y | Y | Y | **N** |
| Changeset/changelog + version check on PRs | Y | Y | Y | Y | Y | Y | Y | Y | Y (`changelog`, `version-check`) |
| Fresh-merge simulation (§7) | Y | Y | Y | N | N | N | Y | N | Y (PR triggers on merge ref) |
| Paginated rerun of unacquired runners | Y | Y (`cd96c10`) | Y | Y | Y | Y | Y (`8b65947`) | Y | Y (`scripts/rerun-unacquired-jobs.mjs`, `per_page=100 --paginate`) |
| No dead/contradictory jobs | Y | Y | Y | Y | Y | Y | Y | Y | **N** — `docker-build`, `docker-slim` can never run |
| Build scripts warning-free / warnings denied | Y | **N** (rust-script `unused_dependencies`) | Y | Y | Y | Y | Y | Y | **N** (same scripts, same warnings) |
| Static workflow audit (zizmor/actionlint) | Y | Y | Y | Y | Y | Y | Y | Y | Y |

## Concrete formal-ai gaps

### G1. Floating Rust toolchain + main-only lint ⇒ false negatives on PRs (actionable here, P0)
Evidence: run 45549993600 (eee485204), lint lane 1 job 138989128440 on
`rustc 1.100.0 (0ef1bc386 2026-11-09)` failed `run_clippy` with new 1.100 lints
(octal-looking escape ×4, regex compiled in a loop ×4, "file is loaded as a module
multiple times" for six shared fixture/helper files), which also broke compilation of
the `unit`, `workflow_contracts`, `reasoning_contracts` test targets. PRs merged
before the bump were green because `scripts/ci-clippy-profile.sh --library-and-binaries` omits
`--tests`. Release.yml has been red on main since at least run 44968089085
(2026-11-22).
Fix:
1. Add `rust/rust-toolchain.toml` (`channel = "1.100.0"`, `components = ["clippy","rustfmt"]`, `profile = "minimal"`) **and** pass `toolchain: 1.100.0` to every `dtolnay/rust-toolchain` call (or SHA-pin `dtolnay/rust-toolchain@683b588c487a69d01f6c6ddf40f036ff682cc2c8` and drop `@stable`). Keep `scripts/generate-staged-release.mjs:147` consistent — the captured `rust_version` must equal the toml channel; add a contract test asserting toml == workflow inputs (template lacks this, see upstream report).
2. Add `--tests` (or `--all-targets`) to the PR clippy profile; the 1.100 lints only fire on test targets.
3. Bump the toolchain deliberately via PR (dependabot `rust-toolchain` ecosystem or `dependencies-latest`) so new lints land on a PR, not on main.

Status on branch `issue-1637-2272b664dc2d`: the *symptoms* are already fixed —
836fe921e (1.100 clippy lints + unused script deps), 2ba2ae143 (ordered-lists false
positive for Cargo `[[test]]` targets), fe376028e (rustfmt). Items 1–3 (the *cause*)
are still open: no `rust-toolchain.toml`, PR clippy still `--lib --bins`.

### G2. Dependency-currency gate fails on main (actionable here, P0)
`check-dependencies-latest` exits 3 because of `actions/setup-python@v6`
(`.github/actions/download-artifact/action.yml:31`, `.github/workflows/workflows.yml:306`)
and `actions/setup-node@v6` (`.github/workflows/security.yml:125,264`); latest is v7.1.0.
Fix: bump the four sites to `@v7`; run the same gate on PRs that touch `.github/**`
so drift is caught pre-merge. The gate also reports 12 "unverifiable" entries
(absent from the snapshot) — refresh the snapshot or make them a hard failure (§17:
missing evidence must not pass silently).

### G3. rust-script `unused_dependencies` warnings (actionable here, P1)
Cargo 1.100 warns `unused dependency` for `scripts/version-and-commit.rs` (serde),
`scripts/publish-crate.rs` (regex), `scripts/wait-for-crate.rs` (regex) — copied from
the rust template, which has the same defect. Fix: delete the unused entries from the
`//! [dependencies]` manifests and set `CARGO_BUILD_WARNINGS: deny` in the job that
runs release scripts so the next one fails loudly (verified: exit 1 under rust-script).

### G4. Dead jobs `docker-build` and `docker-slim` (actionable here, P0 false negative)
`.github/workflows/release.yml` (docker-build `if:` at ~line 150, docker-slim at ~213):

```yaml
if: ${{ (github.event_name != 'pull_request' && github.event_name != 'pull_request_target' && (...main...)) && (needs.build-artifacts.result == 'success' && github.event_name == 'pull_request' && ...) }}
```

`event_name != 'pull_request' && event_name == 'pull_request'` is always false. The
main-only prefix was mechanically prepended in 66ec6d1f1 (2026-10-28) while the
`vars.FORMAL_AI_FULL_PR_CI == 'run'` escape hatch still allowed PR runs; 4c6d5653b
(2026-11-21) removed that escape hatch, making both jobs permanently skipped. Skipped
jobs report as success to `pipeline-status`, so the Docker image is never built or
smoke-tested. Fix: drop the stale `github.event_name == 'pull_request'` clause (run on
main/release), or drop the main-only prefix (run on PRs with code changes); add a
workflow-contract test that rejects `if:` expressions containing both
`event_name != 'X'` and `event_name == 'X'`.

### G5. rustdoc and Pages only on main (actionable here, P1)
`build-pages` → `pages-artifact.yml` runs `cargo doc` with `RUSTDOCFLAGS=-D warnings`
only on push to main; run 45053448033 failed there on private intra-doc links
(`explicit_count`, `output_again`) and an unclosed `<id>` HTML tag that no PR check
saw. Fix: run `cargo doc --no-deps` with `-D warnings` (or the full artifact build,
without deploy) on PRs that change `rust/**`; gate `deploy-pages` with
`vars.DEPLOY_GITHUB_PAGES == 'true'` and emit a `::notice::` when skipped (§17),
matching rust/php/python/cpp templates. Separate issue: deploy currently fails with
`POST /repos/link-assistant/formal-ai/pages/deployments → HTTP 404` (runs 45549993600,
45490077033) even though Pages `build_type=workflow`; needs a repo-settings check by a
maintainer (not fixable from a PR).

### G6. MSRV declared but untested (actionable here, P2)
`rust/Cargo.toml` declares `rust-version = "1.99"`; nothing builds on 1.99. Either add
a `cargo +1.99 check --all-targets` leg (using `RUSTUP_TOOLCHAIN`, **not** the
matrix `toolchain:` input — see the rust-template report) or drop the declaration.

### G7. Action SHA pinning (§14) (longer-term, not in this PR)
2 of 586 `uses:` are SHA-pinned. csharp, cpp and go templates pin almost everything; doing this here
is a large mechanical change better done with a dedicated tool (e.g. `pinact`) plus a
contract test, in its own PR.

### G8. Main-only full test suite (policy decision, deferred)
Full shards, specification, server-parity and e2e run only on main by design
(issue-1571 case study §5.1, cost decision). Every template runs the full suite on
PRs. Recommendation: at minimum compile all test targets on PRs
(`cargo test --no-run --all-targets`), which would have caught the 1.100 breakage at
negligible cost; keep long-running shards main-only.

## Actionable-here summary

| # | Gap | Action in PR #1638 | Priority |
|---|---|---|---|
| G1 | floating toolchain, PR clippy without `--tests` | lints fixed (836fe921e); still: pin 1.100.0 + add `--tests` to PR clippy + toml/workflow agreement test | P0 |
| G2 | setup-node/setup-python v6 | bump 4 sites to v7 | P0 |
| G4 | dead docker jobs | fix contradictory `if:` + contract test | P0 |
| G3 | unused rust-script deps | deps removed (836fe921e); still: `CARGO_BUILD_WARNINGS=deny` | P1 |
| G5 | rustdoc main-only, ungated Pages deploy | PR `cargo doc -D warnings`; `DEPLOY_GITHUB_PAGES` gate + notice | P1 |
| G8 | no test-target compilation on PRs | `cargo test --no-run --all-targets` on PRs | P1 |
| G6 | MSRV untested | add 1.99 check leg or drop field | P2 |
| G7 | SHA pinning | separate PR | P3 |
| — | Pages deploy HTTP 404 | maintainer settings check | external |

## Upstream reports drafted (`../upstream-reports/`)

| File | Template | Shared with formal-ai | Duplicate check |
|---|---|---|---|
| `rust-msrv-leg-overridden-by-rust-toolchain-toml.md` | rust | yes (toolchain pinning class, G1/G6) | none; related closed #147, #202 |
| `rust-rust-script-unused-dependencies-warnings.md` | rust | yes (identical scripts, G3) | none; related closed #239, #179 |
| `js-actions-cache-v5-major-behind.md` | js | yes (action-major drift class, G2) | none open; related closed #234 |

Not redrafted (already filed): java#29 — no aggregate `pipeline-status` job (J-1).
csharp#116 (preflight deadline) is open and unrelated to formal-ai findings.
