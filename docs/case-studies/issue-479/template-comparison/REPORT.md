# CI/CD Best-Practices Comparison: `link-assistant/formal-ai` vs. the four AI-driven-development-pipeline templates

**Issue:** #479 ("Not available in latest release" for all desktop apps) — broader audit.
**Date:** 2026-06-14
**Mode:** Read-only research. No working-repo files were modified except this report and the fetched template evidence under `docs/case-studies/issue-479/template-comparison/`.

Compared repositories (all `link-foundation`, default branch `main`):

| Short | Repo | Lang |
|-------|------|------|
| `js` | `js-ai-driven-development-pipeline-template` | JS/TS |
| `rust` | `rust-ai-driven-development-pipeline-template` | **Rust (most relevant)** |
| `python` | `python-ai-driven-development-pipeline-template` | Python |
| `csharp` | `csharp-ai-driven-development-pipeline-template` | C# |

All fetched template CI/CD files are preserved verbatim under
`docs/case-studies/issue-479/template-comparison/<short>/...` (full file trees in each `FULL-FILE-TREE.txt`).

---

## Summary (top findings)

1. **The issue-#479 bug does NOT exist in any template.** No template has a desktop-release / asset-build / publish workflow at all, and **none of the four templates reference `workflow_run` or `head_sha` anywhere** (verified by grep across every fetched `.yml`). The flawed `workflow_run.head_sha == tag commit` assumption is unique to the working repo's `desktop-release.yml`. **There is therefore no upstream bug to file for this specific defect.**
2. **The working repo's resolve logic is sound, but it was only ONE of three coupled layers.** `scripts/desktop-release-resolve.sh` implements a 2-tier resolution (exact-SHA tier 1 + "latest published release / auto-release child commit" tier 2) plus an idempotency guard, and is unit-tested in `tests/unit/ci-cd/desktop_release_resolve.rs`. It landed (merged) in PR #480 but stayed **dormant**: the desktop-release job was gated on full-pipeline `conclusion == 'success'`, and the E2E Pages probe kept timing out, so the build never ran and `/download` stayed broken through v0.203.0. PR #486 relaxes the gate and makes the Pages probe marker-authoritative so the (sound) resolve logic finally executes. See the case-study README for the full three-layer analysis.
3. **The working repo is far more advanced than every template on desktop/release surface.** It is the **only** repo with: a cross-platform desktop matrix (6 targets incl. arm64), SLSA build provenance / attestation (`actions/attest-build-provenance@v2`), consolidated `SHA256SUMS.txt` + `BUILD-PROVENANCE.txt`, and a `/download` page wired to the GitHub Releases API. No template produces release-attached binaries or attestations.
4. **Best practices the working repo is MISSING that the Rust template has** (highest-value gaps): (a) a **`cargo-lock` guard job** (`scripts/check-cargo-lock.rs`); (b) a **published-artifact smoke test** (`scripts/smoke-test-published-crate.rs`) run after every crates.io publish; (c) a **`setup-buildx-resilient` composite action** (`.github/actions/`) that retries + falls back to `mirror.gcr.io` on Docker Hub outages; (d) a **multi-OS test matrix** (`ubuntu/macos/windows` — the working repo dropped macOS/Windows to "speed up iteration"); (e) a **dedicated, separate docs-deploy job** (the working repo's API-docs/`/docs/api` *route* is now CLOSED — `release.yml`'s `deploy-demo` job runs `cargo doc --no-deps --lib` and copies it into `src/web/docs/api/` — but the Rust template still keeps it as an independent job rather than folding it into the demo deploy).
5. **Best practice in the JS template the working repo (and Rust/Python/C# templates) lack: a broken-link checker** (`.github/workflows/links.yml`, lychee + Wayback-Machine fallback). The working repo has *zero* markdown/link validation in CI.
6. **No template has security scanning.** None of the four has CodeQL, `dependency-review`, SBOM, Trivy/Grype/OSV, or `permissions: security-events`. The working repo also lacks these. This is a genuine cross-cutting gap worth filing upstream against **all** templates (and adding to the working repo).
7. **Page-structure (`/`, `/download`, `/docs/api`, `/docs/*`, `/app`) parity is poor and inconsistent — but the working repo is now the most complete.** Working repo: has `/` (landing chooser) + `/app` + `/download` + `/docs/` (hub) + **`/docs/api`** (its `deploy-demo` job now runs `cargo doc` and copies the output into `src/web/docs/api/`). Rust: deploys rustdoc to Pages root (its `/docs/api`), nothing else. Python: Sphinx docs. C#: DocFX docs. JS: an example web+desktop+mobile app to Pages, no docs. **The working repo is now the only repo implementing the full `/`, `/app`, `/download`, `/docs/api`, `/docs/*` structure** issue #479 envisions; the templates each cover only a slice.
8. **The working repo now publishes API docs (gap CLOSED).** All three *other-language* templates (rust/python/csharp) publish API docs to Pages; the working repo previously deployed only the React demo, but `release.yml`'s `deploy-demo` job now runs `cargo doc --no-deps --lib` and copies it into `src/web/docs/api/` (release.yml L874-920), serving `/docs/api` alongside the demo under one Pages site. The Rust template's standalone `deploy-docs` job remains a structural reference (a *separate* job vs. folding it into the demo deploy).
9. **Action-version hygiene is consistent and good across the board** (`actions/checkout@v6`, `actions/cache@v5`, `dtolnay/rust-toolchain@stable`, `actions/deploy-pages@v5`). No template pins to commit SHAs, and neither does the working repo — a minor, shared, low-severity hardening gap.
10. **The working repo's `release.yml` has materially more PR-gating checks** (i18n catalog coverage, language parity, intent coverage, web-TDZ, VS Code extension tests, web-bundle freshness) than any template — these are application-specific and not expected upstream.

---

## Per-template file-tree overview (CI/CD surface)

### `rust` (most relevant)
```
.github/workflows/release.yml                         # "CI/CD Pipeline" (795 lines)
.github/actions/setup-buildx-resilient/action.yml     # resilient buildx boot (mirror fallback)
.pre-commit-config.yaml                               # IDENTICAL to working repo's
scripts/*.rs                                          # bump-version, check-cargo-lock, check-changelog-fragment,
                                                       #   check-crate-size, check-file-size, check-release-needed,
                                                       #   check-version-modification, collect-changelog,
                                                       #   create-changelog-fragment, create-github-release,
                                                       #   detect-code-changes, get-bump-type, get-version, git-config,
                                                       #   publish-crate, release-naming, rust-paths,
                                                       #   smoke-test-published-crate, version-and-commit, wait-for-crate
tests/unit/ci-cd/*.rs                                 # workflow_release.rs, changelog_parsing.rs,
                                                       #   release_naming_tests.rs, workspace_manifest_resolution.rs
```
**No** `desktop-release.yml`, **no** docs workflow, **no** security workflow.

### `python`
```
.github/workflows/release.yml   # "CI/CD Pipeline" (612 lines)
.github/workflows/docs.yml      # Sphinx -> GitHub Pages (89 lines)
.pre-commit-config.yaml         # ruff + ruff-format + mypy --strict (richer than rust/working)
.ruff.toml
scripts/*.py                    # bump_version, check_file_size, create_github_release, create_manual_changeset,
                                #   detect_code_changes, format_release_notes, publish_to_pypi, release_naming,
                                #   smoke_test_published_package, validate_changeset, version_and_commit
```

### `csharp`
```
.github/workflows/release.yml   # "release" (801 lines)
.github/workflows/docs.yml      # DocFX -> GitHub Pages (96 lines)
.pre-commit-config.yaml
docs/toc.yml, docs/roadmap/toc.yml
scripts/*.mjs (+ *.test.mjs)    # node-based, WITH co-located unit tests:
                                #   bump-version, check-file-size, check-release-needed(+test),
                                #   create-github-release(+test), detect-code-changes, merge-changesets,
                                #   release-naming(+test), release-workflow-policy.test, smoke-test-nuget-package(+test),
                                #   validate-changeset, version-and-commit(+test), wait-for-nuget(+test)
```

### `js`
```
.github/workflows/release.yml      # "Checks and release" (630 lines)
.github/workflows/example-app.yml  # web + desktop(Electron) + Android + iOS + preview-regen (296 lines)
.github/workflows/links.yml        # lychee broken-link checker + Web Archive fallback (93 lines)
.github/actions/publish-dockerhub/action.yml
.github/actions/setup-buildx-resilient/action.yml
.husky/pre-commit                  # husky instead of pre-commit framework
scripts/*.mjs                      # 30+ scripts incl. changeset tooling, publish-to-npm, smoke-test-package, etc.
```

### Working repo (`formal-ai`) — for reference
```
.github/workflows/release.yml          # "CI/CD Pipeline" (920 lines) — the big one
.github/workflows/desktop-release.yml  # "Desktop Release" (266 lines) — UNIQUE; not in any template
.pre-commit-config.yaml                # IDENTICAL to rust template
clippy.toml
scripts/*.rs / *.sh / *.mjs / *.py     # ~40 scripts incl. desktop-release-resolve.sh, stamp-pages-artifact.sh,
                                        #   wait-for-pages-deployment.sh, sync-seed.sh, verify-docker-runtime.sh
```
**No** *standalone* docs-deploy job (the working repo does run `cargo doc` for `/docs/api`, but folded into `deploy-demo` rather than a separate job), **no** link checker, **no** security workflow, **no** `setup-buildx-resilient` action, **no** `check-cargo-lock` / `smoke-test-published-crate` scripts.

---

## Rust template deep-dive (side-by-side with the working repo)

Both files are named `CI/CD Pipeline` and share the same DNA (changelog-fragment driven auto-release, rust-script jobs, Docker Hub publish, Pages deploy). The working repo's is a heavily extended fork. Feature matrix:

| Feature | Rust template `release.yml` | Working `release.yml` / `desktop-release.yml` | Verdict |
|---|---|---|---|
| `concurrency` guard | yes, `cancel-in-progress: github.ref != main` (L32-34) | yes, but `cancel-in-progress: true` **unconditionally** (release.yml L32-35) | **Working repo slightly worse**: it can cancel an in-flight `main` push run; template lets `main` runs finish. |
| Per-job `timeout-minutes` | yes, all jobs | yes, all jobs | parity |
| Per-job `permissions` (least-privilege) | yes | yes | parity |
| Job-level change detection | yes | yes (more outputs) | parity |
| `cargo-lock` guard job | **YES** (L128-153 -> `scripts/check-cargo-lock.rs`); lint/test/coverage gate on it | **NO** | **GAP in working repo** |
| Test OS matrix | **`[ubuntu, macos, windows]`** (L231-232) | **`[ubuntu-latest]` only** (release.yml L266) | **GAP in working repo** (acknowledged trade-off) |
| `fail-fast: false` on matrix | yes (L230) | yes (release.yml L262; desktop L89) | parity |
| Cargo caching (`actions/cache@v5`) | yes | yes | parity |
| Coverage -> Codecov | yes (L256-301) | yes (release.yml L293-336) | parity |
| Crate package-size check | yes | yes | parity |
| Changelog-fragment automation | yes | yes | parity |
| Auto + manual + changelog-PR modes | yes | yes | parity |
| Crates.io publish + wait-for-crate | yes | yes | parity |
| **Published-artifact smoke test** | **YES** (L421-427 & L589-594) | **NO** | **GAP in working repo** |
| Docker Hub publish (opt-in) | yes | yes | parity |
| **Resilient buildx action** | **YES** (`uses: ./.github/actions/setup-buildx-resilient`, L474 & L640) | **NO** — plain `docker/setup-buildx-action@v4` (release.yml L493, L643) | **GAP in working repo** |
| **API-docs deploy to Pages** | **YES** — standalone `deploy-docs` job (`cargo doc --no-deps`, L730-795) | **YES** — `cargo doc --no-deps --lib` folded into `deploy-demo` -> `/docs/api` (release.yml L874-920) | **Route parity** (working repo folds it into the demo deploy vs. a separate job) |
| GitHub Release creation | yes | yes | parity |
| Desktop binaries as release assets | **no** | **YES** (6-target matrix) | **Working repo ahead** |
| SLSA provenance / attestation | **no** | **YES** (`attest-build-provenance@v2`, desktop L191-194) | **Working repo ahead** |
| E2E (Playwright) in CI | **no** | **YES** (`test-e2e-local`, `test-e2e-pages`) | **Working repo ahead** |
| i18n / language / intent gates | **no** | **YES** (release.yml L206-223) | **Working repo ahead (app-specific)** |
| VS Code extension tests | **no** | **YES** (release.yml L230-231) | **Working repo ahead (app-specific)** |
| `.pre-commit-config.yaml` | identical | identical | parity |
| Broken-link checker | **no** (only `js`) | **no** | shared gap |
| Security scanning | **no** | **no** | shared gap |
| Action SHA-pinning | no (tag-pinned) | no (tag-pinned) | shared minor gap |

**Net:** working repo is a superset on application/release surface but a **subset on four reliability/quality jobs** the Rust template ships: cargo-lock guard, published-crate smoke test, resilient buildx, and a multi-OS test matrix. (API-docs deploy is no longer a gap — the working repo now serves `/docs/api` via `cargo doc` in `deploy-demo`; the Rust template merely keeps it as a *separate* job.)

---

## Concrete, actionable improvements for `formal-ai` (ordered by value)

1. ~~**Add an API-docs (`/docs/api`) deploy job.**~~ **DONE / CLOSED.** The working repo's `deploy-demo` job now runs `cargo doc --no-deps --lib` and copies the rustdoc output into `src/web/docs/api/` (release.yml L874-920), synthesizing a root redirect (`src/web/docs/api/index.html`) and serving `/docs/api` under the same Pages site as the demo via a sub-path layout — exactly the structure this item recommended. The Rust template's standalone `deploy-docs` job (`rust/.github/workflows/release.yml` L730-795) remains a reference for splitting it into an *independent* job if desired, but the `/docs/api` route is no longer missing.
2. **Add a `cargo-lock` guard job.** Copy `rust/.github/workflows/release.yml` **L124-153** + `scripts/check-cargo-lock.rs`; gate lint/test/coverage on it. Rationale (template L124-127): a missing/stale `Cargo.lock` degrades `hashFiles('**/Cargo.lock')` cache keys to the empty hash; working repo caches use exactly that key (release.yml L171).
3. **Add a published-crate smoke test.** Copy `scripts/smoke-test-published-crate.rs` + steps `rust/.github/workflows/release.yml` **L421-427 / L589-594**.
4. **Adopt `setup-buildx-resilient`.** Copy `rust/.github/actions/setup-buildx-resilient/action.yml` into the (nonexistent) working `.github/actions/`; replace plain `docker/setup-buildx-action@v4` at working `release.yml` **L493** and **L643**. Retries + `mirror.gcr.io` fallback (action L77-100).
5. **Add a broken-link checker.** Copy `js/.github/workflows/links.yml` (lychee + Web-Archive fallback, helper `scripts/check-web-archive.mjs` L67-72; exclude `docs/case-studies/` per L55). Working repo has no link validation.
6. **Restore (or document dropping) the multi-OS test matrix.** Working `release.yml` L264-266 drops macOS/Windows; Rust template runs all three (L231-232). For a desktop app, platform regressions otherwise surface only in the heavier desktop build.
7. **Make `release.yml` concurrency main-safe.** Change working `release.yml` L33-35 to `cancel-in-progress: ${{ github.ref != 'refs/heads/main' }}` (matches Rust template L34 and the working repo's own `desktop-release.yml` L46).
8. **(Cross-cutting) Add security scanning** (CodeQL + dependency-review) — absent everywhere.

---

## Issue-#479-analogous bugs in templates

**None found.** Evidence:

- `grep -rn "workflow_run" --include='*.yml'` over every fetched template workflow -> **0 matches** (case-studies excluded). No template has a `workflow_run`-triggered workflow.
- `grep -rn "head_sha"` over all fetched template files -> **0 matches**. `github.event.workflow_run.head_sha` (the #479 cause) appears **only** in the working repo (`desktop-release.yml` L75), now consumed safely by `scripts/desktop-release-resolve.sh` (Tier-2 logic L119-138 resolves the latest release instead of requiring a tag on the head SHA).
- Desktop hit in any template workflow: only `js/.github/workflows/example-app.yml` **L95-141** (`desktop-package`). It packages Electron across `[ubuntu, macos, windows]` but **uploads only as CI artifacts** (`upload-artifact@v7`, L136-141) — never to a Release, never resolving a tag from a SHA. No release-tag/head-SHA coupling -> **cannot** exhibit #479.
- `release: published`-related jobs exist only in `js` (L370) and `csharp` (L344) and are *publishers*, not `workflow_run` consumers. C# `docs.yml` **L6-9** even documents avoiding the inverse anti-pattern ("never on `release: published` ... see issue #15"), matching the working repo's own decoupling.

### PR #510 follow-up: the electron-builder 26 macOS signing-skip is also absent upstream

The PR #510 root cause (electron-builder **26**'s `findSigningIdentity()` skips
signing entirely — never invoking a custom `mac.sign` hook — unless
`mac.identity === "-"`; see case-study §0.6) is **electron-builder-specific**.
Checked against every template:

- `grep -niE "electron-builder|mac\.identity|csc_|notarize|adhoc"` over all
  fetched template workflows -> **0 matches**.
- The only Electron packaging in any template is `js/example-app.yml`'s
  `desktop-package` job (L95-141). It uses **Electron Forge**
  (`npm run example:desktop:package`), **not electron-builder**, and performs
  **no macOS code-signing at all** (no `CSC_*`, no `mac.identity`, no `--mac`,
  no ad-hoc hook). It cannot reach `MacPackager.findSigningIdentity()`, so the
  EB26 regression is structurally impossible there.

**Conclusion:** Neither the #479 head-SHA defect nor the PR #510 EB26
signing-skip defect exists in any template (no template uses electron-builder or
signs macOS bundles). There is **no upstream bug to file** for either; the
remediation lives only in the working repo's `desktop-release.yml`. The single
actionable upstream item remains the optional hardened desktop-release workflow
enhancement (below), which should ship the EB26-correct `-c.mac.identity=-`
ad-hoc command if Electron + electron-builder is ever upstreamed.

---

## Page-structure (`/`, `/download`, `/docs/api`, `/docs/*`, `/app`) parity

| Route | Working repo (`formal-ai`) | `rust` | `python` | `csharp` | `js` |
|---|---|---|---|---|---|
| `/` (landing) | **Yes** — `src/web/index.html` via `deploy-demo` (release.yml L811-874) | rustdoc root redirect | Sphinx index | DocFX index | example-app index |
| `/app` (interactive app) | served at `/` (`src/web/app.js`, 306 KB) | no | no | no | example web app at Pages root |
| `/download` (desktop installers) | **Yes** — `src/web/download/{index.html,download.js,download.css,assets}` from Releases API via `desktop-release.yml` | **no** | **no** | **no** | **no** (desktop builds are CI artifacts only) |
| `/docs/api` (API reference) | **Yes** — `cargo doc --no-deps --lib` folded into `deploy-demo` -> `src/web/docs/api/` (release.yml L874-920) | **Yes** — rustdoc (release.yml L730-795) | **Yes** — Sphinx (`docs.yml`) | **Yes** — DocFX (`docs.yml`) | **no** |
| `/docs/*` (guides) | **Yes** — `/docs/` hub (`src/web/docs/index.html`) deployed via `deploy-demo` | via rustdoc | Sphinx site | DocFX site (+ `docs/toc.yml`) | no |

**Reading:** The working repo is now the **most complete** of the five. It **owns `/download` outright** (the only one with a real desktop-download page wired to release assets) **and** now serves `/` (landing chooser) + `/app` + `/docs/` (hub) + `/docs/api` (rustdoc), all unified under one Pages site via sub-paths — the full structure issue #479 envisions. Each template still covers only a slice (rustdoc-only, Sphinx-only, DocFX-only, or example-app-only).

---

## Recommended upstream issues to file

### Per-template desktop-release / #479 bug
**None.** No template carries the #479 defect (or any desktop-release workflow). Filing a #479-analogous bug against any template would be invalid.

### Genuine gaps worth filing upstream (enhancements)

**1. All four templates — add CI security scanning.**
- *Title:* "Add CodeQL + dependency-review to the CI pipeline"
- *Body:* "The pipeline has thorough lint/test/coverage/release automation but no security analysis. Add a `github/codeql-action` job (language-appropriate) and `actions/dependency-review-action` on `pull_request`. Verified absent: no `codeql`/`dependency-review`/`security-events`/SBOM/scanner reference in any workflow."
- *Check:* `gh api repos/link-foundation/<repo>/git/trees/HEAD?recursive=1 --jq '.tree[].path' | grep -iE 'codeql|security'` returns nothing.

**2. rust / python / csharp — port the `links.yml` broken-link checker from `js`.**
- *Title:* "Port the `links.yml` broken-link checker from the JS template"
- *Body:* "`js` ships `.github/workflows/links.yml` (lychee + Web-Archive fallback) but rust/python/csharp do not, so doc links can rot undetected. Port it + the `check-web-archive` helper; exclude `docs/case-studies/`."
- *Check:* `links.yml` present only in `js`.

**3. rust template — optional hardened desktop-release workflow + /download page (parity with formal-ai).**
- *Title:* "Provide an optional cross-platform desktop-release workflow + /download page"
- *Body:* "Downstream `link-assistant/formal-ai` built a complete desktop pipeline (6-target matrix, SLSA attestation via `actions/attest-build-provenance`, consolidated `SHA256SUMS.txt`, `/download` page from the Releases API). Consider upstreaming. **Ship the FIXED resolve logic** from `scripts/desktop-release-resolve.sh` (resolve the latest published release — the auto-release tags a child `chore: release vX.Y.Z` commit whose first parent is the CI head SHA), **not** a naive `workflow_run.head_sha == tag commit` match, which caused formal-ai #479."
- *Reproducible bug to avoid:* a `workflow_run` job doing `gh api repos/$REPO/tags --jq '.[] | select(.commit.sha=="'$HEAD_SHA'")'` returns empty whenever the tag sits on the auto-release child commit -> build skipped forever.
- *Second reproducible bug to avoid (electron-builder 26):* an unsigned/ad-hoc macOS build must pass `-c.mac.identity=-`. On electron-builder **26**, `MacPackager.findSigningIdentity()` returns `null` for any qualifier other than `"-"` when no Apple certificate is present (even with `CSC_IDENTITY_AUTO_DISCOVERY=false`), so `isSignAllowed()` skips signing entirely and a custom `-c.mac.sign=<hook>` is **never invoked** — the produced `.app` ships with no `Contents/_CodeSignature/CodeResources`. This is a behavior change from electron-builder 25 (where the hook ran without the flag) and caused formal-ai #479's macOS-only failure at v0.205.0/v0.206.0. Minimal repro: `electron-builder --mac --publish never -c.mac.sign=./sign.cjs` on EB26 with no cert -> hook never runs; adding `-c.mac.identity=-` makes it run. Workaround/fix: always pass `-c.mac.identity=-` on the ad-hoc path.

---

## Evidence index (fetched & preserved)

All under `docs/case-studies/issue-479/template-comparison/`:

- `rust/.github/workflows/release.yml`, `rust/.github/actions/setup-buildx-resilient/action.yml`, `rust/.pre-commit-config.yaml`, `rust/FULL-FILE-TREE.txt`
- `python/.github/workflows/release.yml`, `python/.github/workflows/docs.yml`, `python/.pre-commit-config.yaml`, `python/FULL-FILE-TREE.txt`
- `csharp/.github/workflows/release.yml`, `csharp/.github/workflows/docs.yml`, `csharp/.pre-commit-config.yaml`, `csharp/FULL-FILE-TREE.txt`
- `js/.github/workflows/release.yml`, `js/.github/workflows/example-app.yml`, `js/.github/workflows/links.yml`, `js/.github/actions/publish-dockerhub/action.yml`, `js/.github/actions/setup-buildx-resilient/action.yml`, `js/.husky/pre-commit`, `js/FULL-FILE-TREE.txt`

Working-repo files read for comparison: `.github/workflows/release.yml`, `.github/workflows/desktop-release.yml`, `scripts/desktop-release-resolve.sh`, `.pre-commit-config.yaml`, `clippy.toml`, plus `src/web/` layout and `tests/unit/ci-cd/desktop_release_resolve.rs`.
