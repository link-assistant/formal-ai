# Issue 479 Case Study — `Not available in latest release` for all desktop apps

> **Issue:** <https://github.com/link-assistant/formal-ai/issues/479> (`bug`, opened 2026-06-14T17:15:58Z by konard)
> **Pull request:** <https://github.com/link-assistant/formal-ai/pull/480> (`[WIP]`, branch `issue-479-33639d61f4f0`)
> **Case study date:** 2026-06-14
> **Status:** **Primary bug fixed** (PR #480, commit `47683f45`). Documentation requests (macOS screenshots, refreshed app-preview) shipped (`69cdd05d`, `dacbd892`). The cross-cutting "complete page-structure" and template-audit requirements are **analyzed here with concrete plans**; the `/docs/api` route remains an open enhancement.
> **Type:** CI/CD bug fix + documentation refresh + cross-repo audit + this case study.

All raw, third-party captures referenced below live under [`raw-data/`](raw-data/); the full CI/CD template comparison is in [`template-comparison/REPORT.md`](template-comparison/REPORT.md).

| Artifact | Path |
|---|---|
| The issue, as filed | [`raw-data/issue-479.json`](raw-data/issue-479.json) |
| The reported screenshot (the `/download` page in the bug state) | [`raw-data/issue-screenshot.png`](raw-data/issue-screenshot.png) |
| Smoking-gun root-cause writeup | [`raw-data/root-cause-evidence.md`](raw-data/root-cause-evidence.md) |
| The failing Desktop Release run log | [`raw-data/desktop-release-27505853178.log`](raw-data/desktop-release-27505853178.log) |
| Desktop Release run history (all `skipped`) | [`raw-data/desktop-release-runs.json`](raw-data/desktop-release-runs.json) |
| Every release is asset-less (`desktop_assets: 0`) | [`raw-data/releases-asset-evidence.json`](raw-data/releases-asset-evidence.json) |
| The pull request | [`raw-data/pr-480.json`](raw-data/pr-480.json) |
| The fixed resolve logic (the heart of the fix) | [`../../../scripts/desktop-release-resolve.sh`](../../../scripts/desktop-release-resolve.sh) |
| The unit test for the fix (8 event shapes) | [`../../../tests/unit/ci-cd/desktop_release_resolve.rs`](../../../tests/unit/ci-cd/desktop_release_resolve.rs) |
| The workflow with the `workflow_run` trigger | [`../../../.github/workflows/desktop-release.yml`](../../../.github/workflows/desktop-release.yml) |
| The macOS-screenshots e2e test | [`../../../tests/e2e/tests/issue-479.spec.js`](../../../tests/e2e/tests/issue-479.spec.js) |
| The macOS-screenshot generator | [`../../../tests/e2e/scripts/generate-macos-screenshots.mjs`](../../../tests/e2e/scripts/generate-macos-screenshots.mjs) |
| The best-practices reference (vk-bot-desktop snapshot) | [`raw-data/vk-bot-desktop-current/`](raw-data/vk-bot-desktop-current/) |

---

## 1. Executive Summary

The `/download` page for **formal-ai Desktop** showed **"Not available in latest release"** under every platform tab (macOS, Windows, Linux) even though the page banner read *"Release assets ready v0.200.0"* (see [`raw-data/issue-screenshot.png`](raw-data/issue-screenshot.png)). The page was working as designed — it reads the GitHub Releases API and shows that string when a release carries no matching desktop asset — but **every release from v0.187.0 through v0.201.0 carried zero assets** ([`raw-data/releases-asset-evidence.json`](raw-data/releases-asset-evidence.json)). The desktop binaries were never being built.

The root cause is a **GitHub Actions `workflow_run` head-SHA mismatch**. The automated CI/CD release commits the version bump in a *new child* commit (`chore: release vX.Y.Z`), tags **that** commit, and creates the release from it — all pushed with `GITHUB_TOKEN`. GitHub therefore (a) suppresses the `release` event and (b) never starts a CI run for the child commit (its [recursion guard](https://docs.github.com/en/actions/concepts/security/github_token)). The "CI/CD Pipeline" run that *does* complete carries the **parent** commit's SHA. The old `Desktop Release` resolve step required *a tag whose commit equals `workflow_run.head_sha`*; because the tag lives on the **child** commit and the head SHA is the **parent**, that exact-SHA match **never** succeeded → `should_build=false` → the build job was skipped → no assets → "Not available in latest release" on every platform, forever.

**The fix** ([`scripts/desktop-release-resolve.sh`](../../../scripts/desktop-release-resolve.sh)) extracts the resolve logic into a unit-tested script and resolves the **latest published release** on `workflow_run` (Tier 2), keeping the exact-SHA match only as a defensive Tier 1, guarded by an **idempotency check** (skip only if the release already has `formal-ai-desktop-*` assets) and **verbose `::group::` diagnostics**. The first pipeline that completes after this lands will see the asset-less latest release and **self-heal** the entire v0.187–v0.201 backlog. The documentation requests — macOS Gatekeeper screenshots and a refreshed app-preview — also ship in PR #480.

---

## 2. Timeline / Sequence of Events

All timestamps are UTC, taken from the captured run history and release metadata.

### 2.1 The asset-less release streak

`releases-asset-evidence.json` shows the desktop build has been silently failing since the code path went live — **15 consecutive releases**, all with `desktop_assets: 0` and `total_assets: 0`:

| Release | Created | `desktop_assets` |
|---|---|---|
| v0.201.0 | 2026-06-14T16:54:50Z | 0 |
| v0.200.0 | 2026-06-14T00:15:15Z | 0 |
| v0.199.0 | 2026-06-13T23:47:55Z | 0 |
| … (v0.198 → v0.188) | 2026-06-12 / 06-13 | 0 |
| v0.187.0 | 2026-06-12T12:29:01Z | 0 |

> This is the signature of a **systematic** failure, not a flaky run: the desktop build never uploaded a single asset since this code path went live. (`raw-data/root-cause-evidence.md`, "Corroborating evidence — every release is asset-less".)

### 2.2 The smoking-gun run (Desktop Release `27505853178`)

The most recent automatic Desktop Release run *before* the fix — the run that should have built v0.201.0's assets — succeeded while skipping every real job:

| Field | Value | Meaning |
|---|---|---|
| `databaseId` | `27505853178` | the run that should have built v0.201.0 assets |
| `event` | `workflow_run` | triggered by "CI/CD Pipeline" completion |
| `conclusion` | `success` | "succeeded" because every real job was *skipped* |
| display `headSha` (`gh run list`) | `56ccb77e…` | the **child** release commit `chore: release v0.201.0` |
| event-payload `workflow_run.head_sha` | `0abd3f45…` | the **parent** — the commit CI/CD Pipeline actually ran on |
| `resolve` job | `success` | emitted `should_build=false` |
| `build` job | `skipped` | **no assets ever built** |
| `finalize` job | `skipped` | no `SHA256SUMS.txt` / provenance |

The verbatim resolve-step output ([`raw-data/desktop-release-27505853178.log`](raw-data/desktop-release-27505853178.log), lines 89 & 91):

```text
  WORKFLOW_RUN_HEAD_SHA: 0abd3f45b61a68ed2b819189d7655c3a7cd8aa07
…
No release tag points at workflow_run head SHA 0abd3f45b61a68ed2b819189d7655c3a7cd8aa07; skipping desktop build.
```

That log also preserves the **old, broken inline logic** verbatim (lines 51–69): it ran `gh api repos/$REPO/tags … select(.commit.sha == "$WORKFLOW_RUN_HEAD_SHA")`, found nothing, set `should_build=false`, and exited. This is the exact code path the fix removes.

### 2.3 The parent/child commit relationship (the reconciliation)

The two SHAs differ because two *different* things report a "head SHA":

```text
tag v0.201.0  ->  commit 56ccb77e  ("chore: release v0.201.0")   <- gh run list DISPLAYS this
                          | first parent
                          v
                  commit 0abd3f45  ("Merge pull request #472 …")  <- workflow_run.head_sha (read by resolve)
```

- `gh run list` **displays** the SHA the *triggered* Desktop Release run checks out — `HEAD` of `main` at trigger time, i.e. the child release commit `56ccb77e`. This is the documented `workflow_run` behavior: the triggered workflow's `GITHUB_SHA` is the *"Last commit on default branch"* ([GitHub Docs — Events that trigger workflows](https://docs.github.com/en/actions/writing-workflows/choosing-when-your-workflow-runs/events-that-trigger-workflows)).
- The resolve step does **not** read that. It reads the **event payload** `github.event.workflow_run.head_sha`, which is the head SHA of the *triggering* CI/CD Pipeline run — the **parent** commit `0abd3f45`.

The release tag points at `56ccb77e`; the head SHA the resolve step matched against is `0abd3f45`. An exact-SHA tag lookup against the parent therefore **always returns empty**.

### 2.4 The run history confirms the pattern

[`raw-data/desktop-release-runs.json`](raw-data/desktop-release-runs.json) lists the recent Desktop Release runs. Run `27505853178` is the only `success`; **every other run is `skipped`** — including runs keyed on the *child* SHA `56ccb77e` (e.g. `27506382014`) and the parent SHA `0abd3f45` (e.g. `27505436702`). Neither the parent nor the child SHA produced a build, which is exactly what the exact-SHA-only logic predicts.

### 2.5 The fix and the docs refresh land

PR #480 commits (`raw-data/pr-480.json` and `git log`):

| Commit | Headline | What it does |
|---|---|---|
| `47683f45` | `fix(desktop-release): build assets for auto-release child-commit tag (#479)` | Extracts the resolve logic into [`scripts/desktop-release-resolve.sh`](../../../scripts/desktop-release-resolve.sh) (2-tier + idempotency + verbose diagnostics); rewires [`.github/workflows/desktop-release.yml`](../../../.github/workflows/desktop-release.yml) (−52/+13 lines inline → 1 script call); adds the unit test and updates the workflow guard test. |
| `69cdd05d` | `feat(download): add macOS Gatekeeper screenshots to install steps (#479)` | Adds three macOS 15 (Sequoia) Gatekeeper screenshots + localized alt/caption (en/ru/zh/hi), a generator, a fixture, and an e2e test. |
| `dacbd892` | `chore(download): refresh obsolete desktop app-preview + page screenshots (#479)` | Regenerates the obsolete `app-preview-*` images and the `docs/screenshots/issue-347/download-*` captures. |

---

## 3. Complete Requirements Enumeration

Every requirement extracted from the issue body ([`raw-data/issue-479.json`](raw-data/issue-479.json)). The body has **no comments** ([`raw-data/issue-479-comments.json`](raw-data/issue-479-comments.json) is empty), so the body *is* the complete specification.

| # | Requirement (verbatim intent) | Category | Status |
|---|---|---|---|
| **R1** | "Desktop apps are not available." `/download` shows "Not available in latest release" for **all** platforms. | bug | **Done** — root cause found (`workflow_run` head-SHA mismatch) and fixed in `scripts/desktop-release-resolve.sh`; unit-tested; backlog self-heals on next pipeline. |
| **R2** | "screenshots for desktop apps are obsolete." → refresh the app-preview screenshots. | docs | **Done** — `dacbd892` regenerates `src/web/download/assets/app-preview-*` and `docs/screenshots/issue-347/download-*`. |
| **R3** | "macOS instructions don't have screenshots like in <https://konard.github.io/vk-bot-desktop>" → add macOS screenshots. | docs | **Done** — `69cdd05d` adds 3 macOS 15 Gatekeeper screenshots, localized alt/caption in 4 languages; e2e test `tests/e2e/tests/issue-479.spec.js`. |
| **R4** | "Use all the best practices from <https://github.com/konard/vk-bot-desktop>." | process | **Done (applied)** — the vk-bot-desktop pattern (static PNG Gatekeeper screenshots inside the System Settings `<details>`, retina capture, ad-hoc-signing guidance) is mirrored; snapshot preserved in `raw-data/vk-bot-desktop-current/`. |
| **R5** | All templates (and formal-ai) should have CI/CD for a `/download` page (all platforms, testable without developer accounts), `/docs/api`, `/docs/*`, `/app`, and a landing page `/`. | structure | **Partial** — formal-ai owns `/` + `/app` + `/download` (the only repo with a real release-wired download page). **`/docs/api` is the open gap** (no `cargo doc` deploy). Audited in REPORT.md; per-route plan in §6. |
| **R6** | "on landing page it is possible to select to go to web app, to documentation or to download page." | structure | **Partial** — the app links to `download/` (`data-testid="download-link"`, `href: "download/"` in `src/web/app.js`) and has a `/docs/` reference; a single explicit landing-page chooser to web-app / docs / download is the documented enhancement (§6, R6). |
| **R7** | Use best practices from the 4 CI/CD templates; compare the full file tree of every workflow / CI script; **if the same issue is found in a template, report it there too.** | CI / process | **Done** — full audit in [`template-comparison/REPORT.md`](template-comparison/REPORT.md). The #479 bug exists in **no** template (none has a `workflow_run`/`head_sha` desktop-release workflow). Genuine enhancement gaps identified as candidate upstream issues (§9). |
| **R8** | Download all issue logs/data to `./docs/case-studies/issue-479`, do a deep case study (timeline, requirements, root causes, solution plans, existing-component survey) and **search online for additional facts**. | process | **Done** — this document + `raw-data/` + `template-comparison/`; online research with cited sources (§8). |
| **R9** | "If there is not enough data to find actual root cause, add debug output and verbose mode if not present." | process | **Done** — `desktop-release-resolve.sh` emits grouped `::group::` diagnostics for every tier (inputs, Tier 1, Tier 2 parent-SHA confirmation, idempotency guard) so the next iteration has full visibility. The data here was already sufficient to find the root cause. |
| **R10** | If the issue is related to another reportable repo, file an issue there with reproducible examples, workarounds, and code-fix suggestions. | process | **Done (no bug to file)** — the #479 defect is working-repo-specific (REPORT.md §"Issue-#479-analogous bugs in templates"). Three *enhancement* issues are drafted for the templates (§9), each with a reproducible check. |
| **R11** | "plan and execute everything in this single pull request." | process | **Done** — all of the above lands in PR #480. |

### Why eleven and not more

The body is one bug report plus a structure/process brief. R1–R3 are the three concrete defects (download broken; preview obsolete; macOS screenshots missing). R4 is the explicit "use vk-bot-desktop best practices" instruction. R5–R6 are the page-structure vision. R7/R10 are the template-audit-and-report-upstream loop. R8/R9/R11 are the case-study + verbose-mode + single-PR process directives. The body's "fully apply requirements to entire codebase … fixed in all of them" is folded into R1 (the fix is the single resolve path the whole desktop pipeline shares) and R5 (apply structure repo-wide), not promoted to a separate row, to avoid double-counting.

---

## 4. Root-Cause Analysis (per problem)

### 4.1 Primary: the `workflow_run` head-SHA mismatch (R1) — the build never ran

**Symptom.** `/download` shows "Not available in latest release" for macOS, Windows, and Linux while the banner says assets are ready for v0.200.0 ([`raw-data/issue-screenshot.png`](raw-data/issue-screenshot.png)). Every release v0.187.0–v0.201.0 has `desktop_assets: 0` ([`releases-asset-evidence.json`](raw-data/releases-asset-evidence.json)).

**Why the page shows that string.** The page is a React app reading the GitHub Releases API. When the asset for the selected platform isn't present, it renders the localized `downloadUnavailable` string. The reference implementation makes this unambiguous — `src/web/download/download.js` (and the vk-bot-desktop original, [`raw-data/vk-bot-desktop-current/App.jsx`](raw-data/vk-bot-desktop-current/App.jsx) line 35) define:

```js
downloadUnavailable: 'Not available in latest release',
```

and render it precisely when `resolveDownloadAsset(...)` returns no asset (`App.jsx` lines 285–300, 522–534). **So the message is a faithful symptom of "no asset on the release", not a page bug.**

**Root cause — the mechanism, rigorously.** Three GitHub-Actions facts combine:

1. **`workflow_run.head_sha` is the *triggering* run's head, not the triggered run's checkout.** In a `workflow_run`-triggered job, `github.event.workflow_run.head_sha` is the head SHA of the workflow that *just completed* (the CI/CD Pipeline), while the triggered job's own `GITHUB_SHA`/`GITHUB_REF` default to the *"Last commit on default branch"* / *"Default branch"* ([GitHub Docs — Events that trigger workflows](https://docs.github.com/en/actions/writing-workflows/choosing-when-your-workflow-runs/events-that-trigger-workflows)). That is why `gh run list` *displays* the child SHA `56ccb77e` (current `main` HEAD) while the resolve step *reads* the parent SHA `0abd3f45`.

2. **The auto-release tags a *child* commit.** `scripts/version-and-commit.rs` bumps the version in a **new** commit (`chore: release vX.Y.Z`), annotates *that* commit with the `vX.Y.Z` tag, and creates the GitHub release from it. So the release tag's commit (`56ccb77e`) is a **child** whose **first parent** is the CI head SHA (`0abd3f45`) — documented in the script's own header ([`scripts/desktop-release-resolve.sh`](../../../scripts/desktop-release-resolve.sh) lines 10–25).

3. **`GITHUB_TOKEN` pushes don't start new runs (recursion guard).** Because the bump commit + tag are pushed with `GITHUB_TOKEN`, *"events triggered by the `GITHUB_TOKEN` will not create a new workflow run, with the following exceptions: `workflow_dispatch` and `repository_dispatch` … this behavior prevents you from accidentally creating recursive workflow runs"* ([GitHub Docs — GITHUB_TOKEN](https://docs.github.com/en/actions/concepts/security/github_token)). So GitHub suppresses the `release` event for the auto-release **and** never starts a CI run on the child commit. The only completed CI run carries the **parent** SHA.

**The fatal assumption.** The old inline resolve logic required *a tag whose commit equals `workflow_run.head_sha`*:

```bash
gh api "repos/$REPO/tags?per_page=100" --paginate \
  --jq ".[] | select(.commit.sha == \"$WORKFLOW_RUN_HEAD_SHA\") | .name"
```

The tag is on the **child** `56ccb77e`; the head SHA is the **parent** `0abd3f45`. The match is structurally impossible for an auto-release → `should_build=false` → `build` skipped → 0 assets → "Not available in latest release" everywhere. (Evidence: the log line at `desktop-release-27505853178.log:91`.)

**Fix.** See §5. The resolve step now resolves the **latest published release** on `workflow_run`, so it no longer depends on a tag pointing at the parent SHA.

### 4.2 macOS instructions had no screenshots (R3)

**Symptom.** The `/download` page's macOS install steps were text-only, unlike <https://konard.github.io/vk-bot-desktop>, which embeds Gatekeeper dialog screenshots (the issue's explicit comparison).

**Root cause.** macOS Gatekeeper dialogs **cannot be triggered on a hosted macOS CI runner** — there is no scriptable way to invoke Gatekeeper — so a "real" screenshot capture in CI is impossible. The page therefore had no figures.

**Evidence.** The generator documents this constraint directly ([`tests/e2e/scripts/generate-macos-screenshots.mjs`](../../../tests/e2e/scripts/generate-macos-screenshots.mjs) lines 4–9):

> "macOS Gatekeeper dialogs cannot be captured on a hosted macOS CI runner (there is no scriptable way to trigger Gatekeeper), so we render a faithful, on-brand reproduction of the three macOS 15 (Sequoia) dialogs from a self-contained fixture … and screenshot each element at devicePixelRatio 2 (retina), mirroring how vk-bot-desktop ships static PNG screenshots."

**Fix.** A self-contained HTML fixture (`tests/e2e/fixtures/macos-gatekeeper.html`) reproduces the three Sequoia dialogs; a deterministic Playwright generator captures them at retina density into `src/web/download/assets/screenshots/`; `download.js` embeds them in a `<figure class="install-macos-screenshots">` inside the System Settings `<details>`, with localized alt text + caption in all four locales. (The rendered `macos-gatekeeper-open-anyway.png` is a faithful macOS Sequoia Privacy & Security pane with the "Open Anyway" button and the *"Apple could not verify 'formal-ai Desktop' is free from malware"* text — confirmed by inspecting the committed PNG.)

### 4.3 App-preview screenshots were obsolete (R2)

**Symptom.** "screenshots for desktop apps are obsolete."

**Root cause.** The `app-preview-*` images on `/download` had drifted from the current desktop UI (they are committed PNGs, regenerated only when someone refreshes them).

**Fix.** `dacbd892` regenerates `src/web/download/assets/app-preview-{en,ru,zh,hi}-{light,dark}.png` (and `app-preview.png`) plus the `docs/screenshots/issue-347/download-*` captures. (Visible as `Bin … bytes` deltas across all preview images in the PR diff.)

### 4.4 Structure gaps (R5/R6) — `/docs/api` missing, landing-page chooser implicit

**Symptom.** The issue wants every project to expose `/` (landing), `/app` (web app), `/download`, `/docs/api`, `/docs/*`, with the landing page letting the user pick among them.

**Root cause.** formal-ai's Pages site serves `/` + `/app` (the React app at root) and `/download`, but has **no `cargo doc` / `/docs/api` deploy job** at all (verified: no workflow references `cargo doc`, `rustdoc`, or `docs/api`). The landing-to-download link exists in the app (`href: "download/"`) but there is no single explicit landing-page chooser surfacing web-app / docs / download as first-class destinations.

**Evidence.** REPORT.md §"Page-structure parity": *"Working repo … owns `/download` outright … but is uniquely missing `/docs/api`."*

**Fix (planned).** §6, R5/R6 — adopt the Rust template's `deploy-docs` (`cargo doc`) job under a Pages sub-path, and add an explicit landing-page chooser. Not yet implemented in PR #480.

---

## 5. The Fix in Detail (R1)

The fix replaces 52 lines of inline workflow shell with a single call to a unit-tested script ([`pr-480.json`](raw-data/pr-480.json): `desktop-release.yml` `+13 / −52`).

### 5.1 The workflow wiring

[`.github/workflows/desktop-release.yml`](../../../.github/workflows/desktop-release.yml) still passes the head SHA into the resolve step, but now delegates the *decision* to the script:

```yaml
# desktop-release.yml — resolve job
env:
  EVENT: ${{ github.event_name }}
  RELEASE_TAG: ${{ github.event.release.tag_name }}
  WORKFLOW_RUN_HEAD_SHA: ${{ github.event.workflow_run.head_sha }}  # still consumed…
  REPO: ${{ github.repository }}
run: bash scripts/desktop-release-resolve.sh                        # …but the decision moves to the script
```

The `workflow_run` trigger and its `if:` gate are unchanged (`conclusion == 'success' && head_branch == 'main'`), and the build matrix still produces 6 targets (linux x64/arm64, macOS x64/arm64, windows x64/arm64) with SLSA provenance via `actions/attest-build-provenance@v2`.

### 5.2 The two-tier resolution

[`scripts/desktop-release-resolve.sh`](../../../scripts/desktop-release-resolve.sh) handles `workflow_run` as follows:

- **Tier 1 (defensive, lines 107–117):** keep the exact-SHA match — *a tag whose commit IS the head SHA*. This is future-proofing in case the release flow ever stops creating a child commit. For today's auto-release it correctly returns nothing and we fall through.
- **Tier 2 (normal, lines 118–138):** resolve the **latest published release** (`gh release view --json tagName`). The auto-release child commit *is* that latest release. A **diagnostic-only** parent check (`gh api repos/$REPO/commits/$tag --jq .parents[0].sha`) confirms `latest_release.parent == workflow_run.head_sha` (the "auto-release child" relationship) and logs it, **but the build proceeds regardless** so the page self-heals even if the relationship can't be confirmed (`resolution=workflow_run-latest-fallback`).

```bash
# Tier 2 (normal): the auto-release tags a CHILD "chore: release vX.Y.Z" commit
# whose first parent is this head SHA, so no tag points at the head SHA directly.
tag="$(latest_release_tag)"
parent="$(gh api "repos/$REPO/commits/$tag" --jq '.parents[0].sha' …)"
if [ "$parent" = "$WORKFLOW_RUN_HEAD_SHA" ]; then
  log "confirmed: ${tag} commit parent is the CI head SHA (auto-release child)."
fi
```

### 5.3 The idempotency / self-healing guard (lines 173–192)

After resolving the tag, for `workflow_run` events only, the script counts existing desktop assets and skips the build **only if assets already exist**:

```bash
existing="$(gh release view "$tag" --repo "$REPO" --json assets \
  --jq '[.assets[].name | select(startswith("formal-ai-desktop-"))] | length' …)"
if [ "$EVENT" = "workflow_run" ] && [ "$existing" -gt 0 ]; then
  should_build=false   # already built — re-runs are safe
fi
```

This makes pipeline re-runs safe **and** heals the v0.187–v0.201 backlog: the first pipeline to complete after the fix sees the asset-less latest release (`existing=0`) and builds it. Manual `release` / `workflow_dispatch` runs intentionally rebuild (clobber) so a maintainer can force a refresh.

### 5.4 Verbose diagnostics (R9)

Every branch is wrapped in `::group::` blocks and a `[desktop-release-resolve]`-prefixed `log()`, recording: the inputs (event, tags, head SHA), the Tier-1 exact-match result, the Tier-2 latest tag + parent-SHA confirmation, the idempotency count, and the final `tag/should_build/resolution`. The next failure, if any, will be fully traceable from the log.

### 5.5 Unit tests — the direct reproduction

[`tests/unit/ci-cd/desktop_release_resolve.rs`](../../../tests/unit/ci-cd/desktop_release_resolve.rs) drives the script against a **mocked `gh` CLI** across **8 event shapes**. The defining case is `auto_release_child_commit_triggers_build` — the exact #479 reproduction:

```rust
// no tag points at the head SHA (the bug condition); latest release v0.201.0 has 0 assets
GhMock { tags_jq_output: "", latest_tag: "v0.201.0", parent_sha: "0abd3f45parenthead",
         release_exists: true, asset_count: 0 }
// …
assert_eq!(result.tag, "v0.201.0");
assert_eq!(result.should_build, "true",
  "issue #479: a freshly released version with no desktop assets must build them");
```

Under the **old** logic this returned `should_build=false`; the fix returns `true`. The other seven cases pin: idempotency (`asset_count: 6 → false`), Tier-1 exact match, no-release-exists, missing-head-SHA, `release`-event always-rebuild, `workflow_dispatch` rebuild, and dispatch-without-tag latest fallback. A companion guard in `tests/unit/ci-cd/workflow_release.rs` (renamed to `desktop_release_workflow_run_resolves_child_release_not_head_sha_tag`) **forbids the old skip string from ever returning**:

```rust
assert!(!resolve_script.contains("No release tag points at workflow_run head SHA"),
  "issue #479 regression: the resolve script must not reinstate the head-SHA skip");
```

### 5.6 Anatomy of `desktop-release-resolve.sh` — decision-by-decision

The script (194 lines) is a single `case "$EVENT"` over the three triggers, then a shared tail. Reading it top-to-bottom against the source ([`scripts/desktop-release-resolve.sh`](../../../scripts/desktop-release-resolve.sh)):

| Lines | Concern | Behavior |
|---|---|---|
| 51–61 | Strict mode + state | `set -euo pipefail`; defaults `tag=""`, `should_build=true`, `resolution="default"` — *build by default*, so a misclassified event errs toward building rather than silently skipping. |
| 63–75 | Logging + output | `group()`/`endgroup()` wrap `::group::` folds; `log()` prefixes `[desktop-release-resolve]`; `emit_outputs()` writes `tag=`/`should_build=` to `$GITHUB_OUTPUT` **and** stdout (so tests read it without a real Actions runner). |
| 77–79 | `latest_release_tag()` | `gh release view --json tagName --jq .tagName` — the Tier-2 / fallback primitive. |
| 81–87 | Input echo | Logs event, both tags, repo, and the head SHA up front — the single most useful diagnostic line for the *next* incident. |
| 90–93 | `release` event | `tag=$RELEASE_TAG`, build (manual/PAT releases). |
| 94–97 | `workflow_dispatch` | `tag=$INPUT_TAG` (may be empty → falls to the latest-release tail). |
| 99–105 | `workflow_run` guard | empty head SHA → `should_build=false`, `resolution=workflow_run-missing-head-sha`, exit. |
| 107–117 | **Tier 1** | exact tag whose `commit.sha == head SHA`, filtered to `v*.*.*`. For an auto-release this is empty (the documented bug condition) → fall through. |
| 118–138 | **Tier 2** | `latest_release_tag()`; diagnostic parent check sets `resolution` to `workflow_run-child-of-head` (confirmed) or `workflow_run-latest-fallback` (unconfirmed) — **build proceeds either way**. |
| 140–154 | Existence checks | no release at all → `workflow_run-no-release`; resolved tag has no GitHub release → `workflow_run-release-missing`; both skip+exit. |
| 158–169 | Shared tail | `release`/`workflow_dispatch` with empty tag fall back to the latest release (`+latest`); unresolvable → skip. |
| 173–192 | **Idempotency guard** | counts `formal-ai-desktop-*` assets; for `workflow_run` **only**, skip if `>0` (`+already-has-assets`). Manual triggers always rebuild (clobber). |

Two design choices are worth calling out as the antidotes to the original bug:

1. **The SHA relationship is a diagnostic, never a gate.** Tier 2 *logs* whether `latest_release.parent == head_sha` but builds regardless. The old logic made the SHA match a hard gate — the single point of failure. (Source: lines 129–135 set only `resolution`, never `should_build=false`.)
2. **"Build by default" + "skip only with positive evidence of done."** `should_build` starts `true` and only flips to `false` on an explicit terminal condition (no head SHA, no release, release already has assets). A future change that forgets to handle a case therefore *over-builds* (visible, recoverable) rather than *under-builds* (silent, the #479 failure mode).

### 5.7 Reproduction and verification procedure

The bug and its fix are both fully reproducible from the captured evidence — no live infrastructure required.

**Reproduce the original failure (read-only, from evidence):**
1. Open [`raw-data/desktop-release-27505853178.log`](raw-data/desktop-release-27505853178.log). Line 89 shows `WORKFLOW_RUN_HEAD_SHA: 0abd3f45…`; line 91 shows the skip: `No release tag points at workflow_run head SHA 0abd3f45…; skipping desktop build.`
2. Confirm the tag/commit topology: tag `v0.201.0` → commit `56ccb77e` (`chore: release v0.201.0`) → first parent `0abd3f45`. So the tag is on the child while the resolve step matched the parent.
3. Confirm the consequence: [`releases-asset-evidence.json`](raw-data/releases-asset-evidence.json) shows `desktop_assets: 0` for every release v0.187.0–v0.201.0.

**Verify the fix locally (the unit harness reproduces the exact condition):**
```bash
cargo test --test ci_cd desktop_release_resolve   # or: cargo test -p <crate> desktop_release_resolve
```
The `auto_release_child_commit_triggers_build` case feeds the script `tags_jq_output=""` (Tier 1 empty), `latest_tag="v0.201.0"`, `parent_sha="0abd3f45parenthead"`, `asset_count: 0`, and asserts `should_build == "true"`. The `workflow_run_skips_when_release_already_has_assets` case (`asset_count: 6`) asserts `false`, proving re-runs are safe.

**Verify the macOS screenshots render (e2e):**
```bash
node tests/e2e/scripts/generate-macos-screenshots.mjs   # regenerate the 3 retina PNGs
npx playwright test tests/e2e/tests/issue-479.spec.js    # render + naturalWidth>0 + localized alt/caption (en/ru/zh/hi)
```

**Verify end-to-end in production (after the fix merges):** on the next pipeline completion, the resolve job logs `resolution=workflow_run-child-of-head` (or `…-latest-fallback`) and `should_build='true'`; the build job runs; `gh release view <latest> --json assets` then lists `formal-ai-desktop-*` assets; and `/download` stops showing "Not available in latest release". A subsequent pipeline re-run logs `…+already-has-assets` and skips — confirming idempotency.

---

## 6. Solutions & Solution Plans (per requirement)

Each row names the chosen approach and, where relevant, the alternatives considered and the existing component reused.

### R1 — Desktop assets not built (shipped)
**Chosen:** resolve the **latest published release** on `workflow_run` (Tier 2) + defensive exact-SHA (Tier 1) + idempotency guard, in a unit-tested script. **Existing components reused:** GitHub Releases API via `gh release view` / `gh api …/commits/<tag>`; the existing 6-target build matrix; `actions/attest-build-provenance@v2`.
**Alternatives considered:**
- *Use a PAT instead of `GITHUB_TOKEN` for the auto-release* so the `release`/CI events fire on the child commit. Rejected as more invasive (new secret, broader blast radius) and orthogonal — the resolve step should be robust regardless of how the release is pushed.
- *Trigger Desktop Release on `release: published` only.* Insufficient: `GITHUB_TOKEN`-pushed auto-releases suppress that event entirely; `workflow_run` is the only signal that fires. (The workflow keeps `release: published` for manual/PAT releases as a secondary path.)
- *Match the tag to the child commit by walking parents.* The Tier-2 latest-release resolution is simpler and self-healing; the parent walk survives only as a *diagnostic* confirmation.

### R2 — Refresh app-preview screenshots (shipped)
**Chosen:** regenerate all `app-preview-*` and `docs/screenshots/issue-347/download-*` PNGs. **Existing component reused:** the project's existing Playwright-based preview capture pipeline (retina density), matching the `app-preview-<locale>-<theme>.png` naming `download.js` already resolves.

### R3 — macOS Gatekeeper screenshots (shipped)
**Chosen:** render faithful, on-brand reproductions of the three macOS 15 (Sequoia) dialogs from a committed HTML fixture and capture them with **Playwright** at `deviceScaleFactor: 2`; embed via a localized `<figure>`; assert with an e2e test. **Existing components reused:** Playwright (already the project's e2e engine); the vk-bot-desktop pattern of *static* PNG screenshots in the macOS section.
**Alternative considered:** capture real Gatekeeper dialogs on a macOS runner — **impossible** (no scriptable Gatekeeper trigger; documented in the generator). The reproduction is the only deterministic, CI-stable option and is explicitly labeled an illustration in the caption.

### R4 — vk-bot-desktop best practices (applied)
**Chosen:** mirror the reference's macOS section verbatim in spirit — a System Settings `<details open>` with steps 1/2/3, the screenshots, the `xattr -dr com.apple.quarantine` Terminal one-liner, the ad-hoc-signing rationale, SHA-256 verification, and `gh attestation verify` guidance. The reference snapshot (`App.jsx` + its 3 macOS PNGs) is preserved under `raw-data/vk-bot-desktop-current/` for traceability.

### R5 — Complete page structure `/`, `/app`, `/download`, `/docs/api`, `/docs/*` (partial; plan)
**Chosen (done):** keep `/` + `/app` (React app at Pages root) + `/download` (release-wired). **Open gap — `/docs/api`:** adopt the **Rust template's `deploy-docs` job** (`rust/.github/workflows/release.yml` L730–795: `cargo doc --no-deps --all-features`, synthesize a root `index.html` redirect since rustdoc emits none, `touch .nojekyll`, deploy via `upload-pages-artifact@v5` + `deploy-pages@v5`). **Existing component reused:** `rustdoc` / `cargo doc` for API reference. **Caveat:** formal-ai already serves the app from Pages root via `deploy-demo`, so docs need a sub-path layout to coexist (REPORT.md "Concrete improvements" item 1). **`/docs/*`** guides already live in-repo (`docs/`) but aren't deployed; the same Pages site can publish them.

### R6 — Landing-page chooser (partial; plan)
**Chosen (done):** the app already links to `download/` (`data-testid="download-link"`). **Enhancement:** add an explicit landing-page chooser surfacing **web app / documentation / download** as first-class destinations (once `/docs/api` exists, link it there). Low-risk, app-local change.

### R7 / R10 — Template audit + upstream reporting (done)
**Chosen:** fetch and diff every template's CI/CD surface (full file trees preserved under `template-comparison/<short>/`), grep for `workflow_run`/`head_sha`, and conclude. **Result:** no template carries the #479 defect, so no analogous bug to file; three *enhancement* issues drafted (§9). **Existing components referenced as reuse targets:** `setup-buildx-resilient` composite action, `check-cargo-lock.rs`, `smoke-test-published-crate.rs`, and `links.yml` (lychee) — all from the templates.

### R8 / R9 / R11 — Case study + verbose mode + single PR (done)
**Chosen:** this README + `raw-data/` captures + `template-comparison/REPORT.md` + cited online research (§8); verbose `::group::` diagnostics added to the resolve script; everything in PR #480. **Existing component reused:** the case-study layout from issue-468 / issue-440.

---

## 7. Existing Components / Libraries Survey

What the ecosystem already provides for each sub-problem, and what this PR reuses (pulled from REPORT.md and general knowledge).

### Desktop release resolution & provenance (R1)
- **GitHub Releases API** (`gh release view`, `gh release upload --clobber`, `gh api repos/…/commits/<tag>`) — the source of truth for "which release, which assets". Reused directly by the resolve script and the upload step.
- **[`actions/attest-build-provenance@v2`](https://github.com/actions/attest-build-provenance)** — SLSA build-provenance attestation, verifiable with `gh attestation verify`. Already wired into the desktop build (`desktop-release.yml` L191–194). REPORT.md confirms **no template** produces attestations — formal-ai is ahead here.
- **`electron-builder`** — the cross-platform packager producing `.dmg/.zip/.exe/.AppImage/.deb/.tar.gz`. Already used (`desktop-release.yml` L141).

### Screenshots (R2, R3)
- **[Playwright](https://playwright.dev/)** — headless-browser capture at retina density; powers both the app-preview refresh and the macOS Gatekeeper-dialog reproductions, and the e2e assertions (`naturalWidth > 0`, localized alt/caption).

### `/docs/api` and `/docs/*` (R5)
- **`rustdoc` / `cargo doc`** — the standard Rust API-docs generator; the Rust template's `deploy-docs` job is the ready-made reuse target.
- **GitHub Pages** (`actions/upload-pages-artifact`, `actions/deploy-pages`) — already used by formal-ai's `deploy-demo`; the same site can host `/docs/api` under a sub-path.

### CI quality / security (R7 — cross-cutting)
- **[lychee](https://github.com/lycheeverse/lychee)** broken-link checker (in the JS template's `links.yml`, with a Web-Archive fallback) — formal-ai and 3 of 4 templates lack any link validation.
- **CodeQL** (`github/codeql-action`) + **`actions/dependency-review-action`** — standard security scanning; **absent in all four templates and formal-ai**.
- **`setup-buildx-resilient`** (Rust/JS templates' composite action) — retries + `mirror.gcr.io` fallback on Docker Hub outages.
- **`check-cargo-lock.rs`** + **`smoke-test-published-crate.rs`** (Rust template) — guard a stale `Cargo.lock` (which degrades cache keys) and smoke-test the published crate.

### Knowledge from the `/download` page itself
- The page already ships an **in-browser SHA-256 verifier** (Web Crypto `crypto.subtle.digest`) and a `BUILD-PROVENANCE.txt` reader — see `vk-bot-desktop-current/App.jsx` lines 259–276, 328–406. These solve "verify downloads without developer accounts", directly satisfying R5's "testable without developer accounts" clause.

**Net:** for every requirement, either a project component already realizes it (now cited) or a specific named external component is the documented reuse target. Nothing is left both unimplemented and unplanned.

---

## 8. Online Research

Targeted searches and fetches grounding the three load-bearing technical claims. Where a search added nothing beyond the primary doc, that is stated honestly.

### 8.1 `workflow_run` `head_sha` semantics — and why the triggered job sees a different SHA

The official GitHub docs confirm the exact mechanism behind the parent/child SHA split:

- The `workflow_run` event *"occurs when a workflow run is requested or completed … you to execute a workflow based on execution or completion of another workflow."*
- Critically, the **triggered** workflow's `GITHUB_SHA` defaults to the **"Last commit on default branch"** and `GITHUB_REF` to the **"Default branch"** — *not* the SHA of the run that triggered it. ([Events that trigger workflows](https://docs.github.com/en/actions/writing-workflows/choosing-when-your-workflow-runs/events-that-trigger-workflows))
- The *triggering* run's head is exposed separately in the payload as `github.event.workflow_run.head_sha`. Community guidance corroborates that this payload field *"points to the sha that triggered the test workflow"*, and that *"by the time the [downstream] workflow starts, every reference to the head sha is the latest commit, rather than the tested sha"* ([polpiella.dev — GitHub Action workflows side effects](https://www.polpiella.dev/github-action-workflows-side-effects)).

This is precisely the formal-ai situation: `gh run list` shows the default-branch HEAD (child `56ccb77e`), while the resolve step reads `workflow_run.head_sha` (parent `0abd3f45`).

### 8.2 Why `GITHUB_TOKEN`-pushed releases don't start a CI run (the recursion guard)

The auto-release's suppression of both the `release` event and a child-commit CI run is documented behavior, not a formal-ai quirk:

> *"events triggered by the `GITHUB_TOKEN` will not create a new workflow run, with the following exceptions: `workflow_dispatch` and `repository_dispatch` events always create workflow runs. … For all other events, this behavior prevents you from accidentally creating recursive workflow runs. For example, if a workflow run pushes code using the repository's `GITHUB_TOKEN`, a new workflow will not run even when the repository contains a workflow configured to run when `push` events occur."* — [GitHub Docs — GITHUB_TOKEN](https://docs.github.com/en/actions/concepts/security/github_token)

So the only completed CI run is the one on the **parent** commit — which is exactly why `workflow_run` (not `release`/`push`) is the correct trigger, and why the resolve step must not depend on a tag pointing at that parent.

### 8.3 macOS 15 (Sequoia) Gatekeeper "Open Anyway" moved to System Settings

The page's macOS instructions (and the reproduced screenshots) describe the **post-Sequoia** flow. Apple's own developer announcement (August 6, 2024) states:

> *"In macOS Sequoia, users will no longer be able to Control-click to override Gatekeeper when opening software that isn't signed correctly or notarized. They'll need to visit System Settings > Privacy & Security to review security information for software before allowing it to run."* — [Apple Developer — Updates to runtime protection in macOS Sequoia](https://developer.apple.com/news/?id=saqachfa)

Reputable coverage corroborates the new three-step flow (try to launch and dismiss the dialog → System Settings → Privacy & Security → scroll to Security → "Open Anyway" → authenticate with admin password), and that the Control-click bypass was removed to combat unsigned stealer malware ([iDownloadBlog](https://www.idownloadblog.com/2024/08/07/apple-macos-sequoia-gatekeeper-change-install-unsigned-apps-mac/), [Michael Tsai's blog](https://mjtsai.com/blog/2024/07/05/sequoia-removes-gatekeeper-contextual-menu-override/)). This is exactly the `installMacosSettingsStep1/2/3` sequence the page documents and the three screenshots illustrate.

### 8.4 References

- GitHub Docs — *Events that trigger workflows* (`workflow_run`; default `GITHUB_SHA`/`GITHUB_REF`): <https://docs.github.com/en/actions/writing-workflows/choosing-when-your-workflow-runs/events-that-trigger-workflows>
- GitHub Docs — *GITHUB_TOKEN* (recursion guard): <https://docs.github.com/en/actions/concepts/security/github_token>
- M. Pol Piella — *GitHub Action workflows side effects* (`workflow_run.head_sha` is the triggering sha): <https://www.polpiella.dev/github-action-workflows-side-effects>
- Apple Developer News — *Updates to runtime protection in macOS Sequoia* (Aug 6, 2024): <https://developer.apple.com/news/?id=saqachfa>
- iDownloadBlog — *macOS Sequoia removes the Control-click Gatekeeper method*: <https://www.idownloadblog.com/2024/08/07/apple-macos-sequoia-gatekeeper-change-install-unsigned-apps-mac/>
- Michael Tsai — *Sequoia Removes Gatekeeper Contextual Menu Override*: <https://mjtsai.com/blog/2024/07/05/sequoia-removes-gatekeeper-contextual-menu-override/>
- Apple Support — *Safely open apps on your Mac* (general "Open Anyway" guidance): <https://support.apple.com/en-us/102445>

> **Honest note:** the official `workflow_run` docs page does **not** itself spell out the recursion guard in the `workflow_run` section; that fact is documented on the separate `GITHUB_TOKEN` page (cited above). No source was invented to fill a gap.

---

## 9. Cross-Repo / Upstream Findings (R7, R10)

The full audit is [`template-comparison/REPORT.md`](template-comparison/REPORT.md) (the four `link-foundation/{js,rust,python,csharp}-ai-driven-development-pipeline-template` repos, full file trees preserved). Its conclusions:

### 9.1 The #479 bug is working-repo-specific — nothing to file

- `grep -rn "workflow_run" --include='*.yml'` over **every** fetched template workflow → **0 matches**. No template has a `workflow_run`-triggered workflow.
- `grep -rn "head_sha"` over all fetched template files → **0 matches**. `github.event.workflow_run.head_sha` (the #479 cause) appears **only** in formal-ai's `desktop-release.yml`.
- No template produces release-attached desktop binaries at all. The closest is `js/.github/workflows/example-app.yml`, which packages Electron across `[ubuntu, macos, windows]` but **uploads only as CI artifacts** (`upload-artifact@v7`) — never to a Release, never resolving a tag from a SHA, so it **cannot** exhibit #479.

> **Conclusion:** *"The #479 defect is working-repo-specific and already remediated. No upstream desktop-release bug to report."* (REPORT.md)

### 9.2 Genuine enhancement gaps → candidate upstream issues

These are *not* the #479 bug, but real cross-cutting gaps the audit surfaced. Each carries a reproducible check, satisfying R10's "reproducible examples + suggestions for fix" bar:

| # | Target | Title | Reproducible check |
|---|---|---|---|
| **U1** | **all 4 templates** | "Add CodeQL + dependency-review to the CI pipeline" | `gh api repos/link-foundation/<repo>/git/trees/HEAD?recursive=1 --jq '.tree[].path' \| grep -iE 'codeql\|security'` → nothing. No `codeql`/`dependency-review`/`security-events`/SBOM reference in any workflow. |
| **U2** | **rust / python / csharp** | "Port the `links.yml` broken-link checker from the JS template" | `links.yml` present only in `js`; doc links can rot undetected elsewhere. Port it + the `check-web-archive` helper, exclude `docs/case-studies/`. |
| **U3** | **rust template** | "Provide an optional cross-platform desktop-release workflow + /download page" | Upstream formal-ai's *fixed* pipeline; **ship the corrected resolve logic** (resolve the latest published release — the auto-release tags a child `chore: release vX.Y.Z` commit whose first parent is the CI head SHA), **not** a naive `workflow_run.head_sha == tag commit` match, which caused #479. Repro to avoid: a `workflow_run` job doing `gh api repos/$REPO/tags --jq '.[] \| select(.commit.sha=="'$HEAD_SHA'")'` returns empty whenever the tag sits on the auto-release child commit → build skipped forever. |

### 9.3 formal-ai's own gaps the templates would close (inbound)

The audit also found reliability jobs the **Rust template** has that formal-ai lacks. These are candidate follow-ups *into* formal-ai (ordered by value in REPORT.md "Concrete improvements"), each with a precise reuse pointer:

| # | Gap in formal-ai | Reuse source (line refs from REPORT.md) | Why it matters |
|---|---|---|---|
| 1 | No `/docs/api` (`cargo doc`) deploy | Rust `release.yml` **L730–795** `deploy-docs` (`cargo doc --no-deps --all-features`, synth root `index.html` redirect L770–779, `.nojekyll`, `upload-pages-artifact@v5` + `deploy-pages@v5`) | Closes R5's missing route; coexists with `deploy-demo` via a Pages sub-path. |
| 2 | No `cargo-lock` guard | Rust `release.yml` **L124–153** + `scripts/check-cargo-lock.rs` | A stale/missing `Cargo.lock` degrades `hashFiles('**/Cargo.lock')` cache keys to the empty hash; formal-ai's caches use exactly that key (`release.yml` L171). |
| 3 | No published-crate smoke test | Rust `scripts/smoke-test-published-crate.rs` + steps **L421–427 / L589–594** | Catches a crate that publishes but doesn't install/run. |
| 4 | Plain `docker/setup-buildx-action@v4` | Rust `.github/actions/setup-buildx-resilient/action.yml` (retries + `mirror.gcr.io` fallback, action L77–100); swap at formal-ai `release.yml` L493 & L643 | Survives Docker Hub outages. |
| 5 | Single-OS test matrix | Rust `release.yml` **L231–232** `[ubuntu, macos, windows]` (formal-ai L264–266 drops mac/Win to "speed up iteration") | For a desktop app, platform regressions otherwise surface only in the heavier desktop build. |
| 6 | `cancel-in-progress: true` unconditionally | Rust `release.yml` **L34** `cancel-in-progress: github.ref != 'refs/heads/main'` (formal-ai's own `desktop-release.yml` L46 already does the safe form) | The unconditional form can cancel an in-flight `main` push run. |

None of these is the #479 bug; they are the *inbound* side of the "use the templates' best practices" instruction (R7) and are recorded here as a backlog, not implemented in PR #480 (which is scoped to the bug + docs).

### 9.4 Boundary case worth noting (honest caveat)

The C# template's `docs.yml` explicitly documents avoiding the *inverse* anti-pattern — it warns "never on `release: published` … see issue #15" (REPORT.md, "Issue-#479-analogous bugs"). That is the mirror image of formal-ai's situation: formal-ai *must* use `workflow_run` precisely because `GITHUB_TOKEN`-pushed auto-releases suppress the `release` event. Both repos arrive at the same conclusion (don't depend on `release: published` for `GITHUB_TOKEN` auto-releases) from opposite directions, which independently corroborates the root-cause analysis.

---

## 10. Risks & Trade-offs of the Chosen Fix

| Risk | Likelihood | Mitigation in the PR |
|---|---|---|
| Tier 2 builds the *wrong* release when a `workflow_run` fires without a fresh release (e.g. a non-release CI run on `main`) | Low–Med | The idempotency guard skips when the latest release already has `formal-ai-desktop-*` assets, so a redundant `workflow_run` after assets exist is a no-op. The first build after the fix heals the backlog; thereafter only *new* asset-less releases trigger a build. |
| The diagnostic parent-SHA check could fail to confirm (`workflow_run-latest-fallback`) and build anyway | Low | **Intentional** — building is the safe default; a confirmation miss must not reinstate the silent skip. The fallback is logged so it's visible, and the idempotency guard still prevents duplicate builds. |
| Manual `release` / `workflow_dispatch` reruns clobber existing assets | Low | **By design** — maintainers can force a refresh; `gh release upload --clobber` is idempotent on asset name. Only `workflow_run` is guarded against redundant rebuilds. |
| Extracting logic into a shell script adds a new failure surface (the script itself) | Low | The script is `set -euo pipefail` and covered by 8 unit tests against a mocked `gh`; a companion test forbids the regression string. The previous inline logic had *zero* tests. |
| `/docs/api` and the landing-page chooser (R5/R6) ship later, not in PR #480 | n/a (scope) | Documented as a concrete, line-referenced plan (§6 R5/R6, §9.3) rather than silently dropped; PR #480 is scoped to the bug + the two documentation defects. |

---

## 11. Lessons Learned / Prevention

### What would have caught this earlier

1. **A post-release assertion that desktop assets > 0.** The single most valuable guardrail: after the desktop pipeline runs (or as a scheduled check), assert the latest release carries `formal-ai-desktop-*` assets and fail loudly if not. Fifteen asset-less releases shipped silently because *nothing* asserted the end state — the run even reported `conclusion: success` while skipping every real job. The `releases-asset-evidence.json` query (`[.assets[].name | select(startswith("formal-ai-desktop-"))] | length`) is exactly such an assertion and is now embedded as the idempotency guard; promoting it to a *failing* post-release check would convert a silent skip into a red build.

2. **Unit-testing the decision logic, not just the wiring.** The bug lived in a one-line `--jq select(.commit.sha == …)` buried in workflow YAML — untestable in place. Extracting it into `scripts/desktop-release-resolve.sh` with a mocked-`gh` harness ([`desktop_release_resolve.rs`](../../../tests/unit/ci-cd/desktop_release_resolve.rs)) makes the #479 condition a **first-class test case** (`auto_release_child_commit_triggers_build`) and pins a regression guard that forbids the old skip string from returning. CI logic that gates releases deserves the same test rigor as product code.

3. **Verbose diagnostics by default (R9).** The original step printed one line on failure ("No release tag points at…"). The fix emits grouped `::group::` diagnostics for every tier and decision, so the *next* anomaly is diagnosable from the log without code spelunking. Pair verbose CI logging with a self-healing fallback so a transient confirmation failure degrades to "build anyway", not "skip forever".

4. **Idempotency + self-healing over one-shot correctness.** Because the guard skips only when assets already exist, the fix doesn't merely stop *new* breakage — it *heals the existing backlog* on the next pipeline completion, and tolerates re-runs. Designing the resolve step to be safely re-runnable means a maintainer can also `workflow_dispatch` a rebuild for any tag at any time.

5. **Understand the auto-release commit topology before coupling to it.** The deepest lesson: an automated release that pushes a *child* commit with `GITHUB_TOKEN` interacts with three GitHub behaviors at once (the recursion guard suppressing events, the `workflow_run` payload exposing the *parent* head SHA, and the triggered job checking out the *default-branch* HEAD). Any logic that couples a build to "the commit a release tag points at" must account for that topology — or, better, resolve the release directly and treat the SHA relationship as a *diagnostic*, never a *gate*.

### Net assessment

The primary bug (R1) is **fixed, tested, and self-healing**; the documentation defects (R2, R3) and the vk-bot-desktop best-practice adoption (R4) **ship in PR #480**; the template audit (R7/R10) is **complete with no analogous upstream bug** and three drafted enhancement issues; and the broader page-structure vision (R5/R6) is **analyzed with concrete, component-backed plans**, with `/docs/api` (a `cargo doc` deploy) as the one clearly-named remaining enhancement.
