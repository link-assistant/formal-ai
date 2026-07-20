# Issue #808 — CI/CD false positives, false negatives, warnings and errors

Session: `issue-808-claude-20260720`
Repository: formal-ai (`link-assistant/formal-ai`)
Pull request: [#809](https://github.com/link-assistant/formal-ai/pull/809)
Evidence collected in: `dev/log/issues/808/pulls/809/`

## 1. Evidence collected

| File | Content |
| --- | --- |
| `ci-logs/run-29723467001-failed.log` | CI/CD Pipeline, `main` @ `76bdb6b5`, failed |
| `ci-logs/run-29724500254-failed.log` | Desktop Release, `main` @ `76bdb6b5`, failed |
| `ci-logs/run-29724500254-full.log` | Same run, complete log (used to prove an *absence* of output) |
| `ci-logs/run-29720321919-failed.log` | Desktop Release, `main` @ `8b5acee0`, failed (same signature) |
| `ci-logs/run-29719602956-failed.log` | CI/CD Pipeline, `main` @ `8b5acee0`, failed |
| `ci-logs/run-*.json` | Run/job metadata |

## 2. Timeline

| Time (UTC, 2026-07-20) | Event |
| --- | --- |
| 05:38 | CI/CD Pipeline run 29719602956 fails on `8b5acee0` |
| 05:56 | Desktop Release run 29720321919 fails on `8b5acee0` (macOS x64 + arm64) |
| 06:20–07:02 | Five Desktop Release runs conclude `skipped` (gated on `workflow_run` from a non-successful CI/CD Pipeline) |
| 07:04 | CI/CD Pipeline run 29723467001 fails on `76bdb6b5` — Auto Release, "Collect changelog and bump version" |
| 07:24 | Desktop Release run 29724500254 fails on `76bdb6b5` — identical macOS codesign failure |
| 07:51 | Desktop Release run 29725916384 concludes `skipped` |
| 08:10 | Issue comment: *"All checks must be in Pull Request stage, not only on main."* |

The `skipped` runs are **not** a defect: `desktop-release.yml` triggers on
`workflow_run` from "CI/CD Pipeline" and its `resolve` job correctly refuses to
build when the upstream run did not succeed. They are downstream symptoms of the
two real failures below.

## 3. Requirements extracted from the issue

| # | Requirement | Source |
| --- | --- | --- |
| R1 | Fix every false positive, false negative, warning and error in CI/CD | issue body |
| R2 | Every check must run at the **pull request** stage, not only on `main`, so each PR can produce a working release | issue comment 5020083365 |
| R3 | Compare every workflow/CI file against the three `link-foundation/*-ai-driven-development-pipeline-template` repos and adopt their best practices | issue body |
| R4 | Report upstream issues against the templates when the same defect exists there | issue body |
| R5 | Follow `link-assistant/hive-mind` `docs/CI-CD-BEST-PRACTICES.md` | issue body |
| R6 | Collect logs and evidence into `dev/log/issues/808/pulls/809` | task brief |

## 4. Failure A — macOS ad-hoc codesign

### Symptom

Both `Build macos-x64` and `Build macos-arm64` fail in *Package desktop app
(macOS ad-hoc)*:

```
.../formal-ai Desktop.app/Contents/Resources/browser-runtime/Frameworks/Google Chrome for Testing Framework.framework:
  replacing existing signature
  unsealed contents present in the root directory of an embedded framework
⨯ Command failed: codesign --sign - --force --timestamp=none --options runtime \
  --entitlements build/entitlements.mac.plist .../Google Chrome for Testing Framework.framework
```

`Publish SHA256SUMS.txt + provenance` then fails closed with
`No artifacts from: macos-x64 macos-arm64`. That second failure is **correct
behaviour** (added deliberately in `1b0ed3f8`, "stop desktop-release from
reporting partial builds as complete") — it is a true negative, not a defect.

### Root cause

`desktop/package.json` ships Playwright's Chrome-for-Testing tree as
`extraResources` → `Contents/Resources/browser-runtime`. That tree contains
`Google Chrome for Testing Framework.framework`, whose layout is **not** a
sealed macOS framework bundle: files sit directly in the framework root rather
than under `Versions/A`. `codesign` refuses such a bundle with *"unsealed
contents present in the root directory of an embedded framework"*. It is signed
upstream by Google and must be treated as an opaque resource.

`desktop/scripts/adhoc-sign-mac.cjs` already contains an exclusion
(`isBundledBrowserRuntime`, added in `46aa1dd7`) and its unit tests pass
locally. It nevertheless did not take effect in CI.

Decisive observation from `ci-logs/run-29724500254-full.log`:

* the step's env block shows `FORMAL_AI_MACOS_SIGN_DEBUG: 1` (workflow line 232);
* electron-builder logs `• executing custom sign  file=release/mac/formal-ai Desktop.app`;
* yet `grep -c "\[adhoc-sign-mac\]"` over the **complete** run log returns `0`,
  and `grep -c "Skipped\.\.\."` (electron-osx-sign's log line for an ignored
  path) also returns `0`.

So the exclusion never ran, even though the hook was resolved. The failing
`codesign` invocation has electron-osx-sign's own argument order
(`--sign - --force …`), i.e. it comes from `signApplication()`'s child loop in
`@electron/osx-sign`, which consults `opts.ignore` — the very list the hook was
supposed to extend.

Two things were therefore wrong, and both are fixed:

1. **The exclusion depended on a single, unverified layer.** It lived only in
   the custom `mac.sign` hook. If the hook does not run (or runs with an
   `app` path that does not resolve as expected), nothing excludes the browser
   runtime. Fix: declare the exclusion in electron-builder configuration itself,
   as `build.mac.signIgnore`. `app-builder-lib`'s
   `MacTargetHelper.buildSignOptions()` compiles `signIgnore` entries into
   regexes and folds them into the `ignore` predicate handed to
   `@electron/osx-sign` — **independently of whether a custom sign hook is
   used**. Verified against the installed `app-builder-lib@26.15.3`
   (`out/mac/MacTargetHelper.js:53-107`, option declared at
   `out/options/macOptions.d.ts:163`).
2. **The diagnostics were unobservable.** The debug logger wrote to
   `process.stdout`; electron-builder's logger and electron-osx-sign's `debug`
   both write to `stderr`. Fix: route `[adhoc-sign-mac]` output to `stderr`, and
   emit exactly one unconditional line when the hook is entered, so the next run
   answers "did the hook execute at all?" without guesswork. The detailed
   per-file trace stays behind `FORMAL_AI_MACOS_SIGN_DEBUG`, **default off**.

### Related warning (electron-builder false positive)

```
• ad-hoc signing with hardenedRuntime enabled requires the
  com.apple.security.cs.disable-library-validation entitlement …
```

`desktop/build/entitlements.mac.plist` already grants
`com.apple.security.cs.disable-library-validation`. electron-builder emits this
warning whenever a custom `mac.sign` hook is configured, because it cannot see
the entitlements the hook will apply. It is a **false positive** and is a
candidate for an upstream report against `electron-userland/electron-builder`.

## 5. Failure B — release blocked by a self-hosting evidence violation

### Symptom

CI/CD Pipeline run 29723467001, job *Auto Release*, step *Collect changelog and
bump version*:

```
Final release version: 0.301.0
Error recording self-hosting release metric: no committed Formal-AI-Evidence in
10e65ae2a22f206589ecba9974c95151e0124bc3 records session issue-804-claude-20260720
##[error]Process completed with exit code 1
```

### Root cause

`CONTRIBUTING.md` (lines 94–106) requires that a commit carrying
`Formal-AI-Session: <id>` also carry `Formal-AI-Evidence: <path>` pointing at a
committed file containing both `formal-ai` and *the exact session id*.

Commit `10e65ae2` declares:

```
Formal-AI-Session: issue-804-claude-20260720
Formal-AI-Evidence: dev/log/issues/804/pulls/805/analysis.md
```

but that file, as committed, contains neither the session id nor the string
`formal-ai`:

```console
$ git show 10e65ae2:dev/log/issues/804/pulls/805/analysis.md | grep -niE 'session|issue-804'
$ # (no output)
```

Reproduced exactly, from the release code path:

```console
$ rust-script scripts/self-hosting-metric.rs --since 10e65ae2~1 --until 10e65ae2
self-hosting metric error: no committed Formal-AI-Evidence in 10e65ae2… records session issue-804-claude-20260720
exit=1
```

The check itself is correct and fail-closed. The **process** defect is where it
runs: `scripts/version-and-commit.rs:635-641` invokes it during Auto Release, on
`main`, *after* merge. A malformed trailer therefore cannot be caught by the
author, and it takes the entire release down. This is precisely the class of
problem R2 describes.

### Fix

A new `evidence-check` job in `.github/workflows/release.yml` runs the identical
code path over the pull request's own commits
(`git merge-base origin/$base HEAD` … `HEAD`). No logic is duplicated — the same
`scripts/self-hosting-metric.rs` measurement that gates the release now gates
the PR, so the release-time gate becomes a backstop rather than the first
detection point.

## 6. Checks that still cannot gate a pull request (R2 backlog)

Derived from a full reading of both workflows. Each item runs only on
push-to-`main`, `release`, `workflow_run` or `workflow_dispatch`:

1. Docker image build + `scripts/verify-docker-runtime.sh` (`release.yml`
   `auto-release` / `manual-release` only).
2. crates.io publish path (`publish-crate.rs`, `wait-for-crate.rs`,
   `smoke-test-published-crate.sh`). Partially covered on PRs by `build`'s
   `cargo package --list` + crate-size check.
3. `deploy-pages` — Pages artifact assembly, `stamp-pages-artifact.sh`,
   `cargo doc` for `/docs/api`.
4. `test-e2e-pages` — the whole GitHub-Pages Playwright suite is post-deploy.
5. ~~`desktop-release.yml` `build` — **the very job that is red today**. Desktop
   packaging and macOS signing are only exercised after a release exists.~~
   **Fixed in this pull request** (see §10).
6. ~~`desktop-release.yml` `vscode` (`.vsix` packaging)~~ **fixed**; `finalize`
   (SHA256SUMS + provenance) intentionally stays release-only — it uploads to a
   GitHub release that does not exist for a pull request. Its inputs (the
   per-target `SHA256SUMS-*.partial` fragments) *are* produced and uploaded as
   artifacts on pull requests, so a fragment-format regression still surfaces.
7. `scripts/desktop-release-resolve.sh` release-resolution logic.
8. Version-bump machinery (`version-and-commit.rs`, `get-bump-type.rs`,
   `collect-changelog.rs`, `create-github-release.rs`).

Item 5 was the highest-value gap: had `desktop-release.yml`'s macOS build run on
pull requests, Failure A would have been caught in the PR that introduced the
bundled browser runtime instead of in a release. It is now closed — the workflow
gained a path-filtered `pull_request` trigger that runs the full six-target
matrix in dry-run mode:

* `resolve` is skipped (no release to heal, no tag to check out); `build` and
  `vscode` admit the skipped dependency with `!cancelled() && (github.event_name
  == 'pull_request' || …)` and check out the PR head instead of the tag.
* Everything through packaging, ad-hoc/notarised signing, the macOS DMG smoke
  test and the Linux/Windows artifact smoke test runs unchanged.
* Only the publishing steps are skipped: `Attest build provenance`, `Upload
  assets to release`, `Upload .vsix to release`, and the whole `finalize` job.
* `cancel-in-progress` becomes `true` for pull requests only, so a superseding
  push does not queue a second six-runner matrix; release runs still never
  cancel mid-upload.

This is what makes Failure A a *pull-request* failure from now on, which is the
explicit ask in issue comment 5020083365.

## 7. Gaps versus the templates and CI-CD-BEST-PRACTICES.md

Compared against `link-foundation/{rust,js,python}-ai-driven-development-pipeline-template`
and `link-assistant/hive-mind/docs/CI-CD-BEST-PRACTICES.md`:

| Practice | formal-ai | Notes |
| --- | --- | --- |
| `detect-changes` gating job | present | `release.yml:62` |
| rustfmt / clippy with `-Dwarnings` | present | `release.yml:51` |
| changelog-fragment gate | present | `release.yml:93` |
| Fresh-merge simulation (`simulate-fresh-merge.sh`) | **absent** | js template calls it in three jobs; best-practices §7 requires it |
| Secrets scan (secretlint / trufflehog) | **absent** | js template only; best-practices §11 |
| Dedicated fast `cargo-lock` job | inline in `lint` | rust template has a separate job others depend on |
| Documentation/link validation | partial | `cargo doc -D warnings` only; no link checker |
| `.github/actions/setup-buildx-resilient` composite | **absent** | raw `docker/setup-buildx-action@v4` |
| `.github/dependabot.yml` | **absent** | — |
| `!cancelled()` rather than `always() && !cancelled()` | not applied | best-practices §10 |

## 8. Defects to report upstream against the templates (R4)

* **T1 — no Docker/artifact PR gate** (rust template): Docker images are built
  and pushed only inside `auto-release`/`manual-release`, in the *same job* that
  runs `cargo publish`. A broken `Dockerfile` is discovered after the crate is
  already published. Workaround/fix: a `docker-pr-check` job with
  `push: false, load: true` on pull requests.
* **T2 — `simulate-fresh-merge` missing from the rust and python templates**
  although `CI-CD-BEST-PRACTICES.md` §7 declares it mandatory and the js
  template ships it. The script is language-agnostic bash and can be copied
  verbatim.
* **T3 — no secrets scan in the rust and python templates**
  (best-practices §11); only the js template runs `secretlint`.
* **T4 — `CI-CD-BEST-PRACTICES.md` §10 contradicts all three templates.** The
  document says to use `!cancelled()` so cancellation propagates; every template
  uses `always() && !cancelled()`, and downstream repositories (including this
  one) have copied that verbatim together with a comment asserting `always()` is
  required. Either the templates or the document must change.
* **T5 — electron-builder false-positive warning** (upstream
  `electron-userland/electron-builder`): with a custom `mac.sign` hook,
  electron-builder warns that `com.apple.security.cs.disable-library-validation`
  is required even when the configured entitlements file already grants it.

## 9. Reusable components considered

* `@electron/osx-sign` `ignore` predicate and electron-builder `mac.signIgnore`
  — used, rather than hand-rolling a codesign walker. `signIgnore` is the
  upstream-supported way to exclude a pre-signed third-party bundle and is
  already how electron-builder excludes `node_modules/playwright/.local-browsers`
  (`MacTargetHelper.js:100-102`).
* `scripts/self-hosting-metric.rs` — reused as-is for the PR gate instead of
  writing a second trailer validator, so the PR check and the release check can
  never drift apart.
* `secretlint`, `lychee` (link checking) and `simulate-fresh-merge.sh` from the
  templates are the recommended off-the-shelf answers to the §7 gaps.

## 10. Changes in this pull request

1. `desktop/package.json` — `build.mac.signIgnore` excludes
   `/Contents/Resources/browser-runtime/` from code signing, independently of
   the custom sign hook.
2. `desktop/scripts/adhoc-sign-mac.cjs` — diagnostics moved to `stderr`; one
   unconditional hook-entry banner; verbose per-file trace unchanged and still
   default-off behind `FORMAL_AI_MACOS_SIGN_DEBUG`.
3. `.github/workflows/release.yml` — new `evidence-check` job validating
   `Formal-AI-Session` / `Formal-AI-Evidence` trailers at pull-request time.
4. `.github/workflows/desktop-release.yml` — path-filtered `pull_request`
   trigger running the full desktop + `.vsix` packaging matrix in dry-run mode
   (see §6); publishing steps and `finalize` stay release-only.
5. `desktop/scripts/adhoc-sign-mac.test.cjs` — regression test asserting the
   `mac.signIgnore` patterns match the browser runtime and not the app binary.
6. `dev/log/issues/808/pulls/809/` — CI log excerpts and this analysis.
