# Issue #808 — CI/CD false positives, false negatives, warnings and errors

Session: `issue-808-claude-20260720`
Repository: formal-ai (`link-assistant/formal-ai`)
Pull request: [#809](https://github.com/link-assistant/formal-ai/pull/809)
Evidence collected in: `dev/log/issues/808/pulls/809/`

## 1. Evidence collected

| File | Content |
| --- | --- |
| `ci-logs/README.md` | Index of the excerpts, the filter used, and how to re-download the full logs |
| `ci-logs/run-29723467001-auto-release.log` | CI/CD Pipeline, `main` @ `76bdb6b5`, failed |
| `ci-logs/run-29719602956-excerpt.log` | CI/CD Pipeline, `main` @ `8b5acee0`, failed (same signature) |
| `ci-logs/run-29724500254-macos-codesign-excerpt.log` | Desktop Release, `main` @ `76bdb6b5`, failed |
| `ci-logs/run-29720321919-macos-codesign-excerpt.log` | Desktop Release, `main` @ `8b5acee0`, failed (same signature) |

The complete logs are 2.5–3.5 MB each and are not committed; `ci-logs/README.md`
gives the exact `gh run view --log` command to reproduce them. Statements below
that rest on an *absence* of output ("not a single `[adhoc-sign-mac]` line")
were checked against the complete logs before they were reduced to excerpts.

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

Decisive observation from the complete log of run 29724500254 (excerpt in `ci-logs/run-29724500254-macos-codesign-excerpt.log`):

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

### Second root cause, found by the added debug output (pull-request runs)

The instrumentation added in the first commit did its job. With packaging now
running on pull requests, both macOS jobs of run `29728829534` failed a *later*
check:

```
##[error]Mounted app is missing its signed CodeResources envelope.
```

and the hook's unconditional entry banner never appeared. The packaging log
explains why:

```
• Current build is a part of pull request, code signing will be skipped.
```

`app-builder-lib/out/codeSign/macCodeSign.js:28` calls builder-util's
`isPullRequest()` from `isSignAllowed()` and returns `false` before
`MacPackager.sign()` reaches `findSigningIdentity()`, so the custom
`mac.sign` hook is never invoked at all. `isPullRequest()` is true whenever
`GITHUB_BASE_REF` is set, i.e. on every `pull_request` event.

Fix: `CSC_FOR_PULL_REQUEST: "true"` on the ad-hoc packaging step only. The
security concern documented for that flag is about *secret* certificates being
usable by fork builds; this step runs only when no signing secrets are
configured and signs ad-hoc with identity `-`, so there is no secret to expose.

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

1. ~~Docker image build + `scripts/verify-docker-runtime.sh` (`release.yml`
   `auto-release` / `manual-release` only).~~ **Fixed in this pull request**:
   the new `docker-build` job builds the image with `push: false, load: true`
   (works for fork pull requests, no registry credentials needed) behind a GHA
   layer cache, then runs `docker run --privileged … verify-formal-ai-dind`.
   `scripts/verify-docker-runtime.sh` previously had *no caller anywhere in CI*
   — it was only baked into the image. Without this job a broken Dockerfile
   produced a half-finished release: crate published to crates.io, no image and
   no GitHub Release (runs 29312084458, 29485000765).
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
| Fresh-merge simulation (`simulate-fresh-merge.sh`) | ~~absent~~ **adopted** | script copied verbatim from the js template; called in `lint` and `test` on pull requests |
| Secrets scan (secretlint / trufflehog) | ~~absent~~ **adopted** | new `secrets-scan` job + `scripts/check-secrets.sh`; diff-scoped because `secretlint "**/*"` did not finish in 25 min here |
| Dedicated fast `cargo-lock` job | inline in `lint` | rust template has a separate job others depend on |
| Documentation/link validation | partial | `cargo doc -D warnings` only; no link checker |
| `.github/actions/setup-buildx-resilient` composite | ~~absent~~ **adopted** | copied verbatim from the rust template; all three `docker/setup-buildx-action@v4` call sites rewired |
| ~~`.github/dependabot.yml`~~ | n/a | **Correction:** I listed this as a gap without a source. Verified: none of the three templates ships a `dependabot.yml`, so this is not a template gap and no action is taken. |
| `!cancelled()` rather than `always() && !cancelled()` | ~~not applied~~ **fixed** | all 9 occurrences in `release.yml` rewritten |

## 8. Defects reported upstream (R4)

All five were verified against the upstream sources before filing, and all are
filed:

| # | Defect | Filed |
| --- | --- | --- |
| T1 | Docker image only built inside the release jobs, i.e. after the package is published, so a broken `Dockerfile` cannot fail a pull request | [rust#100](https://github.com/link-foundation/rust-ai-driven-development-pipeline-template/issues/100), [python#35](https://github.com/link-foundation/python-ai-driven-development-pipeline-template/issues/35), [js#106](https://github.com/link-foundation/js-ai-driven-development-pipeline-template/issues/106) |
| T2 | `simulate-fresh-merge.sh` missing (best-practices §7 declares it mandatory) | rust#100, python#35 |
| T3 | No secrets scan (best-practices §11) | rust#100, python#35 |
| T4 | Self-contradictory `always() && !cancelled()` job guards | rust#100, python#35 |
| T5 | electron-builder false-positive `disable-library-validation` warning | [electron-builder#10027](https://github.com/electron-userland/electron-builder/issues/10027) |

Corrections to my earlier reading, made while verifying:

* **T4 does not apply to the js template.** Measured, not assumed:

  ```console
  $ for r in rust js python; do gh api repos/link-foundation/$r-ai-driven-development-pipeline-template/contents/.github/workflows/release.yml \
      -q .content | base64 -d | grep -c 'always() && !cancelled()'; done
  7
  0
  6
  ```

  So `CI-CD-BEST-PRACTICES.md` §10 does not contradict *all three* templates —
  the js template already complies and is the reference implementation. The
  document is right; rust and python are the outliers.
* **T5's trigger is not the custom `mac.sign` hook.** In
  `app-builder-lib@26.15.3`, `out/mac/MacTargetHelper.js:36-41` emits the
  warning on `qualifier === "-" && hardenedRuntime` alone; `config.entitlements`
  is never read on that path. The warning therefore fires for *every* ad-hoc
  hardened-runtime build, hook or not, even when the entitlement is granted —
  which is a stronger and more easily fixed claim than the one I first wrote.

T1 applies to the js template as well (its `docker-publish` job is gated on
`needs: [release, instant-release]`), so it got its own, narrower report; T2–T4
do not apply there.

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
4. `.github/workflows/release.yml` — new `docker-build` job; replaced the
   contradictory `always() && !cancelled()` guard with plain `!cancelled()` in
   all nine places (CI-CD-BEST-PRACTICES.md §10: `always()` makes a job run even
   when the run is cancelled, which is the opposite of the intent, and the
   `&& !cancelled()` half is dead weight that reads as if it did something).
5. `.github/workflows/desktop-release.yml` — path-filtered `pull_request`
   trigger running the full desktop + `.vsix` packaging matrix in dry-run mode
   (see §6); publishing steps and `finalize` stay release-only.
6. `desktop/scripts/adhoc-sign-mac.test.cjs` — regression test asserting the
   `mac.signIgnore` patterns match the browser runtime and not the app binary.
7. `scripts/simulate-fresh-merge.sh` (adopted verbatim from the js template) —
   called from `lint` and `test` on pull requests, so a PR whose merge preview
   is stale against `main` fails before it can break `main` on merge.
8. `scripts/check-secrets.sh` + `.secretlintrc.json` — new `secrets-scan` job.
   Diff-scoped rather than the template's `secretlint "**/*"`, which did not
   finish in 25 minutes here; measured 2.6 s on this pull request's diff.
   Verified it actually fires: a planted `ghp_…` token is reported as
   `[GITHUB_TOKEN] found GitHub Token` with exit 123.
9. `.github/actions/setup-buildx-resilient/` (adopted verbatim from the rust
   template) — replaces all three raw `docker/setup-buildx-action@v4` uses;
   pre-pulls the BuildKit image with retries and a `mirror.gcr.io` fallback.
10. `dev/log/issues/808/pulls/809/` — CI log excerpts and this analysis.
