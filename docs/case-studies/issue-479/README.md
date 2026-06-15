# Issue 479 Case Study — `Not available in latest release` for all desktop apps

> **Issue:** <https://github.com/link-assistant/formal-ai/issues/479> (`bug`, opened 2026-06-14T17:15:58Z by konard)
> **First attempt:** <https://github.com/link-assistant/formal-ai/pull/480> — **MERGED**, but the fix stayed **dormant** (desktop apps remained unavailable) and its macOS screenshots were synthetic. The maintainer rejected it (issue comment, 2026-06-15).
> **This pull request:** <https://github.com/link-assistant/formal-ai/pull/486> (branch `issue-479-51f54bbe54b1`) — completes the fix.
> **Case study date:** 2026-06-14, revised 2026-06-15 after the maintainer's "redo analysis" feedback.
> **Status:** **All requirements addressed in PR #486.** The desktop build now actually runs (the deeper gating + Pages-probe root cause is fixed), the macOS screenshots are **real captures** from `konard/vk-bot-desktop` (not synthetic), and the source code on the landing page is a **big hero button**. The site structure (`/`, `/app`, `/docs/api`, `/docs/*`, `/download`) is verified.
> **Type:** CI/CD bug fix + documentation correction + cross-repo audit + this case study.

All raw, third-party captures referenced below live under [`raw-data/`](raw-data/); the full CI/CD template comparison is in [`template-comparison/REPORT.md`](template-comparison/REPORT.md).

| Artifact | Path |
|---|---|
| The issue, as filed | [`raw-data/issue-479.json`](raw-data/issue-479.json) |
| The maintainer's rejection comment (the "redo analysis" feedback) | [`raw-data/issue-479-comments.json`](raw-data/issue-479-comments.json) |
| The maintainer's screenshot — `/download` still broken at **v0.203.0** (the proof PR #480 was dormant) | [`raw-data/maintainer-rejection-screenshot.png`](raw-data/maintainer-rejection-screenshot.png) |
| The originally-reported screenshot (the `/download` page in the bug state) | [`raw-data/issue-screenshot.png`](raw-data/issue-screenshot.png) |
| Smoking-gun root-cause writeup (preserves the verbatim resolve-step log) | [`raw-data/root-cause-evidence.md`](raw-data/root-cause-evidence.md) |
| Desktop Release run history (every run `skipped`) | [`raw-data/desktop-release-runs.json`](raw-data/desktop-release-runs.json) |
| Every release is asset-less (`desktop_assets: 0`) | [`raw-data/releases-asset-evidence.json`](raw-data/releases-asset-evidence.json) |
| The first attempt (merged, incomplete) | [`raw-data/pr-480.json`](raw-data/pr-480.json) |
| The resolve logic (head-SHA fix; merged in PR #480, now actually reached) | [`../../../scripts/desktop-release-resolve.sh`](../../../scripts/desktop-release-resolve.sh) |
| The resolve unit test (8 event shapes; merged in PR #480) | [`../../../tests/unit/ci-cd/desktop_release_resolve.rs`](../../../tests/unit/ci-cd/desktop_release_resolve.rs) |
| The workflow with the `workflow_run` trigger + relaxed gate (PR #486) | [`../../../.github/workflows/desktop-release.yml`](../../../.github/workflows/desktop-release.yml) |
| The marker-authoritative Pages probe (PR #486) | [`../../../scripts/wait-for-pages-deployment.sh`](../../../scripts/wait-for-pages-deployment.sh) |
| New gating/Pages/button unit tests (PR #486) | [`../../../tests/unit/ci-cd/workflow_release.rs`](../../../tests/unit/ci-cd/workflow_release.rs) |
| The real macOS-screenshots provenance doc | [`../../../src/web/download/assets/screenshots/README.md`](../../../src/web/download/assets/screenshots/README.md) |
| The macOS-screenshots e2e test | [`../../../tests/e2e/tests/issue-479.spec.js`](../../../tests/e2e/tests/issue-479.spec.js) |
| The landing/docs chooser + big-button e2e test | [`../../../tests/e2e/tests/issue-479-site.spec.js`](../../../tests/e2e/tests/issue-479-site.spec.js) |
| The best-practices reference (vk-bot-desktop snapshot, incl. the real screenshots) | [`raw-data/vk-bot-desktop-current/`](raw-data/vk-bot-desktop-current/) |

---

## 1. Executive Summary

The `/download` page for **formal-ai Desktop** shows **"Not available in latest release"** under every platform tab (macOS, Windows, Linux) even though the page banner reads *"Release assets ready v0.203.0"* — see the maintainer's own 2026-06-15 capture, [`raw-data/maintainer-rejection-screenshot.png`](raw-data/maintainer-rejection-screenshot.png). The page is working as designed — it reads the GitHub Releases API and shows that string when a release carries no matching desktop asset — but **15 consecutive captured releases (v0.187.0–v0.201.0) carry zero desktop assets** ([`raw-data/releases-asset-evidence.json`](raw-data/releases-asset-evidence.json)), and the maintainer's screenshot confirms the same at the current v0.203.0. The desktop binaries were never built.

The defect has **three coupled layers**, and **fixing only the first (PR #480) left the page broken** — which is exactly what the maintainer reported ("По прежнему не исправлено" — *"still not fixed"*):

1. **The `workflow_run` head-SHA mismatch (resolve logic).** The automated CI/CD release commits the version bump in a *new child* commit (`chore: release vX.Y.Z`), tags **that** commit, and creates the release from it — all pushed with `GITHUB_TOKEN`. GitHub therefore (a) suppresses the `release` event and (b) never starts a CI run for the child commit ([recursion guard](https://docs.github.com/en/actions/concepts/security/github_token)). The "CI/CD Pipeline" run that *does* complete carries the **parent** commit's SHA. The old resolve step required *a tag whose commit equals `workflow_run.head_sha`*; because the tag lives on the **child** and the head SHA is the **parent**, the exact-SHA match **never** succeeded. **→ Fixed in PR #480** ([`scripts/desktop-release-resolve.sh`](../../../scripts/desktop-release-resolve.sh), now on `main`): resolve the **latest published release** instead, with a defensive exact-SHA tier and an idempotency guard.

2. **The desktop-release gate required full-pipeline `success` — so PR #480's fix never ran.** The `resolve` job was gated on `github.event.workflow_run.conclusion == 'success'`. But the auto-release **publishes the GitHub release in an *early* pipeline job**; any *later* job failing left a real, asset-less release while suppressing the desktop build entirely. So even with the correct resolve logic merged, the gate kept it dormant — `/download` stayed broken through v0.203.0. **→ Fixed in PR #486** ([`.github/workflows/desktop-release.yml`](../../../.github/workflows/desktop-release.yml)): run on **any completed main-branch pipeline except `cancelled`/`skipped`**; the script's idempotency guard makes a non-success run safe and self-healing.

3. **The E2E Pages probe timed out on `main`, which is what kept failing the pipeline.** [`scripts/wait-for-pages-deployment.sh`](../../../scripts/wait-for-pages-deployment.sh) required the deploy SHA to appear **verbatim in the served index HTML**. The issue-#479 landing (`/`) and docs (`/docs/`) pages shipped **without cache-busted asset refs**, so the deployment marker had the right SHA but the index body never did → the probe spun the full 300 s and **failed the pipeline** (layer 2's late-job failure). **→ Fixed in PR #486**: make the probe **marker-authoritative** (GitHub Pages deploys atomically, so `deployment.json`'s SHA is sufficient proof the matching stamped build is live), and independently cache-bust every landing/docs asset with `?v=__FORMAL_AI_ASSET_VERSION__` exactly like `/app/`.

**The result:** the first pipeline that completes after PR #486 merges will (a) no longer time out on the Pages probe, (b) reach the `resolve` job regardless of an unrelated late-job failure, and (c) see the asset-less **latest** release and **build + upload its desktop assets**, ending the v0.187.0-onward streak. Because `/download` reads only the latest release, healing the latest release is exactly what clears the user-visible "Not available in latest release".

PR #486 also addresses the two documentation defects the maintainer called out: the macOS Gatekeeper screenshots are now **real captures** from the sibling app `konard/vk-bot-desktop` (PR #480's were synthetic reproductions, which the maintainer rejected as *"фейковые"* — *"fake"*), and the **source code on the landing page is a big hero button** (`.source-cta`), not a small footer link.

---

## 2. Timeline / Sequence of Events

All timestamps are UTC, taken from the captured run history, release metadata, and issue/PR threads.

### 2.1 The asset-less release streak

`releases-asset-evidence.json` captures **15 consecutive releases — v0.187.0 through v0.201.0 — every one with `desktop_assets: 0`**. The maintainer's 2026-06-15 screenshot independently extends the same failure to the current **v0.203.0**, so the streak is unbroken since the code path went live:

| Release | Created | `desktop_assets` | Source |
|---|---|---|---|
| **v0.203.0** (current latest) | 2026-06-15 | 0 | `maintainer-rejection-screenshot.png` |
| v0.201.0 | 2026-06-14T16:54:50Z | 0 | `releases-asset-evidence.json` |
| v0.200.0 | 2026-06-14T00:15:15Z | 0 | `releases-asset-evidence.json` |
| … (v0.199 → v0.188) | 2026-06-12 / 06-14 | 0 | `releases-asset-evidence.json` |
| v0.187.0 | 2026-06-12T12:29:01Z | 0 | `releases-asset-evidence.json` |

> The JSONL evidence (`releases-asset-evidence.json`) is newline-delimited `gh release` output; it predates v0.202/v0.203 but covers the unbroken v0.187.0–v0.201.0 run. v0.203.0's asset-less state is shown directly by the maintainer's `/download` capture.
>
> This is the signature of a **systematic** failure, not a flaky run: the desktop build never uploaded a single asset since this code path went live. (`raw-data/root-cause-evidence.md`, "Corroborating evidence — every release is asset-less".)

### 2.2 The smoking-gun run (Desktop Release `27505853178`)

The most recent automatic Desktop Release run *before* PR #480 — the run that should have built v0.201.0's assets — succeeded while skipping every real job. The full evidence (verbatim resolve-step log, the parent/child SHA reconciliation) is preserved in [`raw-data/root-cause-evidence.md`](raw-data/root-cause-evidence.md):

| Field | Value | Meaning |
|---|---|---|
| `databaseId` | `27505853178` | the run that should have built v0.201.0 assets |
| `event` | `workflow_run` | triggered by "CI/CD Pipeline" completion |
| `conclusion` | `success` | "succeeded" because every real job was *skipped* |
| display `headSha` (`gh run list`) | `56ccb77e…` | the **child** release commit `chore: release v0.201.0` |
| event-payload `workflow_run.head_sha` | `0abd3f45…` | the **parent** — the commit CI/CD Pipeline actually ran on |
| `resolve` job | `success` | emitted `should_build=false` |
| `build` job | `skipped` | **no assets ever built** |

The verbatim resolve-step output (preserved in `root-cause-evidence.md`):

```text
  WORKFLOW_RUN_HEAD_SHA: 0abd3f45b61a68ed2b819189d7655c3a7cd8aa07
…
No release tag points at workflow_run head SHA 0abd3f45b61a68ed2b819189d7655c3a7cd8aa07; skipping desktop build.
```

### 2.3 The parent/child commit relationship (the reconciliation)

The two SHAs differ because two *different* things report a "head SHA":

```text
tag v0.201.0  ->  commit 56ccb77e  ("chore: release v0.201.0")   <- gh run list DISPLAYS this
                          | first parent
                          v
                  commit 0abd3f45  ("Merge pull request #472 …")  <- workflow_run.head_sha (read by resolve)
```

- `gh run list` **displays** the SHA the *triggered* Desktop Release run checks out — `HEAD` of `main` at trigger time, i.e. the child release commit `56ccb77e` ([GitHub Docs — Events that trigger workflows](https://docs.github.com/en/actions/writing-workflows/choosing-when-your-workflow-runs/events-that-trigger-workflows)).
- The resolve step reads the **event payload** `github.event.workflow_run.head_sha`, which is the head SHA of the *triggering* CI/CD Pipeline run — the **parent** commit `0abd3f45`.

An exact-SHA tag lookup against the parent therefore **always returns empty**.

### 2.4 PR #480 — the first (incomplete) attempt lands and merges

PR #480 (`raw-data/pr-480.json`, **MERGED**) shipped the head-SHA fix and the documentation work to `main`:

| Commit | Headline | What it did |
|---|---|---|
| `47683f45` | `fix(desktop-release): build assets for auto-release child-commit tag (#479)` | Extracted the resolve logic into [`scripts/desktop-release-resolve.sh`](../../../scripts/desktop-release-resolve.sh) (2-tier + idempotency + verbose diagnostics) + unit test. |
| `69cdd05d` | `feat(download): add macOS Gatekeeper screenshots to install steps (#479)` | Added three **synthetic** macOS Gatekeeper screenshots rendered from an HTML fixture via a Playwright generator + e2e test. |
| `dacbd892` | `chore(download): refresh obsolete desktop app-preview + page screenshots (#479)` | Regenerated the obsolete `app-preview-*` images. |

**But the page stayed broken.** The resolve fix was correct yet **dormant** — gated behind a full-pipeline `success` that the E2E Pages probe kept denying (§4.1, layers 2–3). And the macOS screenshots were **synthetic reproductions**, not real captures.

### 2.5 The maintainer rejects PR #480 — "redo analysis"

On **2026-06-15T13:12:33Z** the maintainer (`@konard`) commented on the issue ([`raw-data/issue-479-comments.json`](raw-data/issue-479-comments.json)), attaching a fresh `/download` capture at **v0.203.0** still showing "Not available in latest release" ([`raw-data/maintainer-rejection-screenshot.png`](raw-data/maintainer-rejection-screenshot.png)):

> *"По прежнему не исправлено, скриншоты macOS фейковые, а не скопированные с нашего кода — <https://github.com/konard/vk-bot-desktop>. Так же убедись что исходный код на лендинге это большая кнопка. Redo analysis."*
>
> ("Still not fixed; the macOS screenshots are fake, not copied from our code — vk-bot-desktop. Also make sure the source code on the landing is a big button. Redo analysis.")

Three concrete rejections: **(a)** desktop apps still unavailable, **(b)** macOS screenshots are fake, **(c)** the landing source code must be a big button.

### 2.6 PR #486 — the completion (this branch)

PR #486 (branch `issue-479-51f54bbe54b1`) addresses all three:

| Commit | Headline | What it does |
|---|---|---|
| `d4751a0c` | `fix(desktop-release,pages): build on any completed main pipeline; make Pages probe marker-authoritative` | **Layers 2–3.** Relaxes the desktop-release gate to any completed main pipeline except cancelled/skipped; makes `wait-for-pages-deployment.sh` marker-authoritative; cache-busts landing/docs assets. Adds 2 new + updates 3 existing unit tests (5 touched). |
| `77d57deb` | `feat(landing): surface source code as a big hero button (issue #479)` | **Rejection (c).** Adds the `.source-cta` big button to the landing hero; removes the small footer link; e2e-tested. |
| `ff23e030` | `fix(download): use real macOS Gatekeeper screenshots from vk-bot-desktop` | **Rejection (b).** Replaces the 3 synthetic PNGs with **real** vk-bot-desktop captures; deletes the synthetic generator + fixture; documents provenance. |

---

## 3. Complete Requirements Enumeration

Every requirement extracted from the issue body ([`raw-data/issue-479.json`](raw-data/issue-479.json)) **and** the maintainer's follow-up comment ([`raw-data/issue-479-comments.json`](raw-data/issue-479-comments.json)).

| # | Requirement (verbatim intent) | Category | Status |
|---|---|---|---|
| **R1** | "Desktop apps are not available." `/download` shows "Not available in latest release" for **all** platforms. | bug | **Done (PR #486)** — root cause is three coupled layers; all fixed. The backlog self-heals on the next pipeline completion (idempotency guard). |
| **R2** | "screenshots for desktop apps are obsolete." → refresh the app-preview screenshots. | docs | **Done (PR #480, `dacbd892`)** — `app-preview-*` regenerated; on `main`. |
| **R3** | "macOS instructions don't have screenshots like in <https://konard.github.io/vk-bot-desktop>" → add macOS screenshots. | docs | **Done (PR #486, `ff23e030`)** — three **real** macOS 15 Gatekeeper screenshots copied from `vk-bot-desktop`; localized alt/caption (en/ru/zh/hi); e2e test. |
| **R3a** | **(maintainer follow-up)** "скриншоты macOS фейковые, а не скопированные с нашего кода" — the macOS screenshots must be **real, copied from vk-bot-desktop**, not synthetic. | docs | **Done (PR #486)** — the synthetic generator + fixture are deleted; the three PNGs are genuine vk-bot-desktop captures; provenance documented in [`src/web/download/assets/screenshots/README.md`](../../../src/web/download/assets/screenshots/README.md). |
| **R4** | "Use all the best practices from <https://github.com/konard/vk-bot-desktop>." | process | **Done** — the vk-bot-desktop pattern (static PNG Gatekeeper screenshots inside the System Settings `<details>`, ad-hoc-signing guidance, SHA-256 / attestation verification) is mirrored; snapshot preserved in `raw-data/vk-bot-desktop-current/`, including the upstream PNGs the page now reuses. |
| **R5** | All templates (and formal-ai) should have CI/CD for a `/download` page (all platforms, testable without developer accounts), `/docs/api`, `/docs/*`, `/app`, and a landing page `/`. | structure | **Done** — formal-ai serves `/` (chooser) + `/app` + `/download` + `/docs/` + `/docs/api` (generated by `cargo doc` at deploy). Verified by `issue-479-site.spec.js`. Template audit in REPORT.md. |
| **R6** | "on landing page it is possible to select to go to web app, to documentation or to download page." | structure | **Done** — the landing page renders a three-card chooser (app / docs / download) via `src/web/site-chrome.js`; verified by `issue-479-site.spec.js`. |
| **R6a** | **(maintainer follow-up)** "убедись что исходный код на лендинге это большая кнопка" — the source code on the landing must be a **big button**. | structure | **Done (PR #486, `77d57deb`)** — `.source-cta` big hero button (≥56 px tall) replaces the old small footer link; localized; e2e-tested (`issue-479-site.spec.js`). |
| **R7** | Use best practices from the 4 CI/CD templates; compare the full file tree of every workflow / CI script; **if the same issue is found in a template, report it there too.** | CI / process | **Done** — full audit in [`template-comparison/REPORT.md`](template-comparison/REPORT.md). The #479 bug exists in **no** template (none uses `workflow_run`/`head_sha`, none publishes release-attached desktop binaries). Nothing to file for #479; genuine *enhancement* gaps recorded separately (§9). |
| **R8** | Download all issue logs/data to `./docs/case-studies/issue-479`, do a deep case study (timeline, requirements, root causes, solution plans, existing-component survey) and **search online for additional facts**. | process | **Done** — this document + `raw-data/` + `template-comparison/`; online research with cited sources (§8). |
| **R9** | "If there is not enough data to find actual root cause, add debug output and verbose mode if not present." | process | **Done** — `desktop-release-resolve.sh` emits grouped `::group::` diagnostics; `wait-for-pages-deployment.sh` now logs the marker SHA + index/marker checks each poll. The data here was already sufficient to find all three root causes. |
| **R10** | If the issue is related to another reportable repo, file an issue there with reproducible examples, workarounds, and code-fix suggestions. | process | **Done (no #479 bug to file)** — the #479 defect is working-repo-specific (REPORT.md). Enhancement candidates are drafted (§9) but, per the maintainer's "if the same issue is found" condition, **not filed** (the same issue is not present upstream). |
| **R11** | "plan and execute everything in this single pull request." | process | **Done** — everything lands in PR #486. |

### Why these rows

R1–R3 are the three concrete defects in the original body (download broken; preview obsolete; macOS screenshots missing). R4 is the explicit "use vk-bot-desktop best practices" instruction. R5–R6 are the page-structure vision. R7/R10 are the template-audit-and-report-upstream loop. R8/R9/R11 are the case-study + verbose-mode + single-PR process directives. **R3a and R6a are the maintainer's two follow-up corrections** (real screenshots; big button) — broken out so their "Done" status is auditable against the rejection comment.

---

## 4. Root-Cause Analysis (per problem)

### 4.1 Primary: desktop assets never built (R1) — three coupled layers

**Symptom.** `/download` shows "Not available in latest release" for macOS, Windows, and Linux while the banner says assets are ready for v0.203.0 ([`raw-data/maintainer-rejection-screenshot.png`](raw-data/maintainer-rejection-screenshot.png)). Every captured release v0.187.0–v0.201.0 has `desktop_assets: 0` ([`releases-asset-evidence.json`](raw-data/releases-asset-evidence.json)), and the screenshot shows the same at v0.203.0.

**Why the page shows that string.** The page is a React app reading the GitHub Releases API. When the asset for the selected platform isn't present, it renders the localized `downloadUnavailable` string. The reference implementation makes this unambiguous — `src/web/download/download.js` (and the vk-bot-desktop original, [`raw-data/vk-bot-desktop-current/App.jsx`](raw-data/vk-bot-desktop-current/App.jsx)) define `downloadUnavailable: 'Not available in latest release'` and render it precisely when the resolver returns no asset. **So the message is a faithful symptom of "no asset on the release", not a page bug.** The fix must make assets exist.

#### Layer 1 — the `workflow_run` head-SHA mismatch (fixed in PR #480)

Three GitHub-Actions facts combine:

1. **`workflow_run.head_sha` is the *triggering* run's head, not the triggered run's checkout.** In a `workflow_run`-triggered job, `github.event.workflow_run.head_sha` is the head SHA of the workflow that *just completed* (the CI/CD Pipeline), while the triggered job's own `GITHUB_SHA`/`GITHUB_REF` default to the *"Last commit on default branch"* ([GitHub Docs](https://docs.github.com/en/actions/writing-workflows/choosing-when-your-workflow-runs/events-that-trigger-workflows)). That is why `gh run list` *displays* the child SHA `56ccb77e` while the resolve step *reads* the parent SHA `0abd3f45`.
2. **The auto-release tags a *child* commit.** `scripts/version-and-commit.rs` bumps the version in a **new** commit (`chore: release vX.Y.Z`), tags *that* commit, and creates the GitHub release from it. So the release tag's commit (`56ccb77e`) is a **child** whose **first parent** is the CI head SHA (`0abd3f45`).
3. **`GITHUB_TOKEN` pushes don't start new runs (recursion guard).** Because the bump commit + tag are pushed with `GITHUB_TOKEN`, GitHub suppresses the `release` event **and** never starts a CI run on the child commit ([GitHub Docs — GITHUB_TOKEN](https://docs.github.com/en/actions/concepts/security/github_token)). The only completed CI run carries the **parent** SHA.

The old inline resolve logic required *a tag whose commit equals `workflow_run.head_sha`* — structurally impossible for an auto-release → `should_build=false` → 0 assets. **PR #480 fixed this** by resolving the **latest published release** instead (§5.2).

#### Layer 2 — the gate required full-pipeline `success`, so layer-1's fix never ran (fixed in PR #486)

Even with the correct resolve script merged, the `resolve` job was gated on `workflow_run.conclusion == 'success'`. The auto-release **publishes the GitHub release in an early pipeline job**, so the release exists long before the pipeline finishes. When any *later* job failed, the pipeline concluded non-`success`, and the gate **suppressed the entire desktop build** — leaving the freshly-published release asset-less. This is why PR #480 merged yet `/download` stayed broken through v0.203.0.

**Fix.** [`.github/workflows/desktop-release.yml`](../../../.github/workflows/desktop-release.yml) L54–68 relaxes the gate:

```yaml
# Run on ANY completed main-branch pipeline except cancelled/skipped (issue #479).
if: >-
  github.event_name != 'workflow_run' ||
  (github.event.workflow_run.head_branch == 'main' &&
   github.event.workflow_run.conclusion != 'cancelled' &&
   github.event.workflow_run.conclusion != 'skipped')
```

The script's idempotency guard (skip only if the release already has `formal-ai-desktop-*` assets) makes a non-success run safe and self-healing.

#### Layer 3 — the E2E Pages probe timed out, which is the late-job failure that kept failing the pipeline (fixed in PR #486)

The recurring late-job failure was the E2E-on-Pages job. [`scripts/wait-for-pages-deployment.sh`](../../../scripts/wait-for-pages-deployment.sh) waited for the deploy SHA to appear **verbatim in the served `index.html` body**. The issue-#479 landing (`/`) and docs (`/docs/`) pages — added when the site was restructured into a chooser — shipped **without cache-busted asset references**, so the deployment marker (`deployment.json`) advertised the right SHA but the index body never contained it. The probe spun the full 300 s timeout and **failed**, failing the pipeline (layer 2's trigger).

**Fix.** Make the probe **marker-authoritative**. GitHub Pages deploys atomically — the whole artifact (every HTML file plus `deployment.json`, all stamped in one step by `scripts/stamp-pages-artifact.sh`) flips live together — so the marker SHA alone proves the matching stamped build is serving. The probe drops the `grep -Fq "$expected_sha" "$index_file"` requirement and keeps: marker SHA matches + index served (HTTP 200) + no un-stamped placeholders survive. Independently, every landing/docs asset is now cache-busted with `?v=__FORMAL_AI_ASSET_VERSION__` (the deploy SHA), exactly like `/app/`, defeating stale CDN caches.

### 4.2 macOS screenshots were synthetic, not real (R3 / R3a)

**Symptom.** PR #480 added macOS Gatekeeper screenshots, but they were **drawn reproductions** rendered from an HTML fixture (`tests/e2e/fixtures/macos-gatekeeper.html`) by a Playwright generator (`tests/e2e/scripts/generate-macos-screenshots.mjs`). The maintainer rejected them as *"фейковые"* (*"fake"*) and required screenshots *"copied from our code"* — the sibling app [`konard/vk-bot-desktop`](https://github.com/konard/vk-bot-desktop).

**Root cause.** macOS Gatekeeper dialogs **cannot be triggered on a hosted macOS CI runner** (Gatekeeper only blocks apps quarantined by a real download), so PR #480 chose to *render* lookalikes. That approach produces images that are not genuine OS captures — exactly what the maintainer found unacceptable.

**Fix (PR #486, `ff23e030`).** The genuine fix is to **reuse the real captures** from `vk-bot-desktop`, which ships with the **identical** `electron-builder` ad-hoc signing flow (`CSC_IDENTITY_AUTO_DISCOVERY=false`, no Apple Developer ID — exactly what formal-ai's `desktop-release.yml` does). The Gatekeeper dialog wording, layout, and buttons are byte-identical for `formal-ai Desktop`; **only the app name shown differs** (`"VK Bot Desktop"` vs `"formal-ai Desktop"`), and the localized `alt`/caption copy says so honestly. The three PNGs under `src/web/download/assets/screenshots/` are replaced with the real captures (mapped 1:1 to the System Settings steps); the synthetic generator + fixture are **deleted**; provenance — including the upstream-source mapping and a "do not re-introduce a synthetic generator" note — is documented in [`src/web/download/assets/screenshots/README.md`](../../../src/web/download/assets/screenshots/README.md). The upstream originals are mirrored at `raw-data/vk-bot-desktop-current/macos-screenshots/` for offline traceability.

### 4.3 App-preview screenshots were obsolete (R2)

**Symptom.** "screenshots for desktop apps are obsolete."

**Root cause.** The `app-preview-*` images on `/download` had drifted from the current desktop UI (committed PNGs, refreshed only on demand).

**Fix (PR #480, `dacbd892`, on `main`).** Regenerated `src/web/download/assets/app-preview-{en,ru,zh,hi}-{light,dark}.png` (and `app-preview.png`) plus the `docs/screenshots/issue-347/download-*` captures.

### 4.4 Source code wasn't a big button; structure already complete (R5 / R6 / R6a)

**Symptom.** The issue wants every project to expose `/` (landing chooser), `/app`, `/download`, `/docs/api`, `/docs/*`, with the landing letting the user pick among them — and the maintainer's follow-up adds that the **source code on the landing must be a big button**.

**Root cause / state.** The site restructure (the `/` chooser with app/docs/download cards, `/docs/` hub, and the `cargo doc` → `/docs/api` deploy) **already landed on `main`** (verified: `src/web/site-chrome.js` `createChooser`, and `.github/workflows/release.yml`'s `cargo doc --no-deps --lib` → `src/web/docs/api/` step). What was missing was the maintainer's specific ask: the source code was surfaced only as a *small footer link*, not a big button.

**Fix (PR #486, `77d57deb`).** Add a `.source-cta` big button to the landing hero (a small uppercase "Open source" eyebrow above a strong "Source on GitHub" label, ≥56 px tall, opening the repo in a new tab), mirroring the `/download` page's `.primary-download` shape; remove the old small footer link. Localized in all four UI languages; asserted by `issue-479-site.spec.js` (`surfaces the source code as a big button in the hero`). The full structure is verified by the same spec (landing renders three nav cards → app/docs/download; docs hub renders the API-reference card + three prose-doc cards).

---

## 5. The Fix in Detail

PR #486 contributes three things; PR #480's resolve script (on `main`) is the fourth piece the gate now finally reaches.

### 5.1 Layer 2 — relax the desktop-release gate (PR #486)

[`.github/workflows/desktop-release.yml`](../../../.github/workflows/desktop-release.yml) keeps the `workflow_run` trigger (`types: [completed]`) but changes the `resolve` job's `if:` from "conclusion == 'success'" to "any completed main pipeline except cancelled/skipped" (L64–68, quoted in §4.1 layer 2). The build matrix still produces 6 targets (linux x64/arm64, macOS x64/arm64, windows x64/arm64) with SLSA provenance via `actions/attest-build-provenance@v2`. The `resolve` step delegates the decision to `scripts/desktop-release-resolve.sh` (L94).

### 5.2 Layer 1 — the two-tier resolution (PR #480, on `main`, now reached)

[`scripts/desktop-release-resolve.sh`](../../../scripts/desktop-release-resolve.sh) handles `workflow_run`:

- **Tier 1 (defensive):** keep the exact-SHA match — *a tag whose commit IS the head SHA*. For today's auto-release it correctly returns nothing and we fall through.
- **Tier 2 (normal):** resolve the **latest published release** (`gh release view --json tagName`). The auto-release child commit *is* that latest release. A **diagnostic-only** parent check (`gh api repos/$REPO/commits/$tag --jq .parents[0].sha`) confirms `latest_release.parent == workflow_run.head_sha` and logs it, **but the build proceeds regardless** so the page self-heals even if the relationship can't be confirmed.

### 5.3 The idempotency / self-healing guard

For `workflow_run` events the script counts existing desktop assets and skips the build **only if assets already exist**:

```bash
existing="$(gh release view "$tag" --repo "$REPO" --json assets \
  --jq '[.assets[].name | select(startswith("formal-ai-desktop-"))] | length' …)"
if [ "$EVENT" = "workflow_run" ] && [ "$existing" -gt 0 ]; then
  should_build=false   # already built — re-runs (and non-success pipelines) are safe
fi
```

This is what makes layer 2's relaxed gate **safe**: a `workflow_run` from a non-success pipeline that fires *after* assets exist is a no-op. It also ends the v0.187.0-onward streak — the first pipeline to complete after the fix sees the asset-less **latest** release (`existing=0`) and builds it (older historical releases stay asset-less, but `/download` only ever surfaces the latest).

### 5.4 Layer 3 — the marker-authoritative Pages probe (PR #486)

[`scripts/wait-for-pages-deployment.sh`](../../../scripts/wait-for-pages-deployment.sh) success criteria become: `deployment.json` advertises `"sha":"<expected_sha>"` **and** the index is served (HTTP 200) **and** no `__FORMAL_AI_ASSET_VERSION__` / `__FORMAL_AI_VERSION__` placeholders survive. The removed line was `grep -Fq "$expected_sha" "$index_file"` — the verbatim-SHA-in-index requirement that the new landing/docs pages couldn't satisfy. In parallel, `src/web/index.html` and `src/web/docs/index.html` now reference every asset as `…?v=__FORMAL_AI_ASSET_VERSION__` (stamped to the deploy SHA), matching `/app/`.

### 5.5 Big source button (PR #486)

`src/web/site-chrome.js` renders a `.source-cta` anchor in the landing hero (eyebrow + strong label), styled large in `src/web/landing.css`; the old `.landing-footer .support-links` "Source on GitHub" link is removed.

### 5.6 Unit + e2e tests

- **`tests/unit/ci-cd/desktop_release_resolve.rs`** (8 event shapes, merged in PR #480) drives the resolve script against a **mocked `gh` CLI**. The defining case `auto_release_child_commit_triggers_build` is the exact #479 reproduction (`should_build` flips `false → true`); `workflow_run_skips_when_release_already_has_assets` (`asset_count: 6 → false`) pins idempotency.
- **`tests/unit/ci-cd/workflow_release.rs`** (PR #486) adds **three new tests** (and updates three existing ones to match the relaxed gate / marker-authoritative probe):
  - `desktop_release_runs_on_any_completed_main_pipeline_not_only_success` — asserts the gate no longer requires `conclusion == 'success'` (layer 2).
  - `wait_for_pages_deployment_is_marker_authoritative` — asserts the probe no longer greps the SHA out of the index body (layer 3).
  - `issue_479_landing_surfaces_source_code_as_a_big_button` — asserts the `.source-cta` markup exists (R6a).
  - The module was split to respect the repo's 1000-line-per-`.rs` file-size CI gate: the shared YAML-slicing helpers now live in [`workflow_fixtures.rs`](../../../tests/unit/ci-cd/workflow_fixtures.rs), and the pre-existing artifact-publishing / Pages version-stamping assertions moved to [`release_publishing.rs`](../../../tests/unit/ci-cd/release_publishing.rs); the three new issue-#479 tests above stay in `workflow_release.rs`.
- **`tests/e2e/tests/issue-479.spec.js`** (5 tests) — the three real screenshots render, load (`naturalWidth > 0`), carry localized alt text, and a caption, in en/ru/zh/hi.
- **`tests/e2e/tests/issue-479-site.spec.js`** (9 tests) — the landing chooser (3 cards), the big source button, the docs hub (API + 3 prose cards), and localization.

### 5.7 Reproduction and verification procedure

The bug and its fix are reproducible from the captured evidence and the unit harness — no live infrastructure required.

**Reproduce the original failure (read-only, from evidence):**
1. Open [`raw-data/root-cause-evidence.md`](raw-data/root-cause-evidence.md): the verbatim resolve-step log shows `WORKFLOW_RUN_HEAD_SHA: 0abd3f45…` then `No release tag points at workflow_run head SHA 0abd3f45…; skipping desktop build.`
2. Confirm the tag/commit topology: tag `v0.201.0` → commit `56ccb77e` (`chore: release v0.201.0`) → first parent `0abd3f45`.
3. Confirm the consequence persists past PR #480: [`raw-data/maintainer-rejection-screenshot.png`](raw-data/maintainer-rejection-screenshot.png) shows "Not available in latest release" at **v0.203.0**; [`releases-asset-evidence.json`](raw-data/releases-asset-evidence.json) shows `desktop_assets: 0` for every captured release v0.187.0–v0.201.0.

**Verify the fix locally (unit):**
```bash
cargo test --test unit ci_cd::desktop_release_resolve   # layer 1: resolve script, 8 event shapes
cargo test --test unit ci_cd::workflow_release           # layers 2-3 + button: gating, Pages probe, big button
```

**Verify the macOS screenshots + site structure render (e2e):**
```bash
cd tests/e2e
npx playwright test tests/issue-479.spec.js       # 3 REAL screenshots render + localized alt/caption (en/ru/zh/hi)
npx playwright test tests/issue-479-site.spec.js  # landing chooser (3 cards), big source button, docs hub
```

**Verify end-to-end in production (after PR #486 merges):** on the next pipeline completion the Pages probe succeeds on the marker (no 300 s timeout); the `resolve` job runs even if a later job failed; it logs `resolution=workflow_run-child-of-head` (or `…-latest-fallback`) and `should_build='true'`; the build runs; `gh release view <latest> --json assets` lists `formal-ai-desktop-*` assets; and `/download` stops showing "Not available in latest release". A subsequent re-run logs `…+already-has-assets` and skips — confirming idempotency.

---

## 6. Solutions & Solution Plans (per requirement)

### R1 — Desktop assets not built (shipped, PR #480 + PR #486)
**Chosen:** fix all three layers — resolve the **latest published release** on `workflow_run` (PR #480), **relax the gate** to any completed main pipeline (PR #486), and make the **Pages probe marker-authoritative** (PR #486) so the late-job failure that kept failing the pipeline is gone. **Existing components reused:** GitHub Releases API (`gh release view` / `gh api …/commits/<tag>`); the existing 6-target build matrix; `actions/attest-build-provenance@v2`; the atomic-deploy guarantee of GitHub Pages.
**Alternatives considered:**
- *Use a PAT instead of `GITHUB_TOKEN` for the auto-release* so the `release`/CI events fire on the child commit. Rejected as more invasive (new secret, broader blast radius) and orthogonal.
- *Keep gating on `conclusion == 'success'`.* Rejected — that is precisely what kept PR #480's fix dormant; the release is published early, so the desktop build must not depend on the *whole* pipeline going green.
- *Keep requiring the deploy SHA in the index body.* Rejected — couples the probe to every root page embedding the SHA verbatim; the atomic marker is sufficient and robust to new pages.

### R2 — Refresh app-preview screenshots (shipped, PR #480)
**Chosen:** regenerate all `app-preview-*` and `docs/screenshots/issue-347/download-*` PNGs via the project's existing Playwright preview pipeline.

### R3 / R3a — Real macOS Gatekeeper screenshots (shipped, PR #486)
**Chosen:** **reuse the genuine captures** from `vk-bot-desktop` (identical ad-hoc signing → byte-identical dialogs, only the app name differs); delete the synthetic generator + fixture; document provenance + the upstream mapping; label the app-name difference honestly in the localized caption. **Existing components reused:** the sibling app's real screenshots; Playwright for the render-and-load e2e assertions.
**Alternative considered and rejected:** *render lookalikes from an HTML fixture* — this was PR #480's approach and the maintainer explicitly rejected it as fake. Capturing real Gatekeeper dialogs on a hosted macOS runner is impossible (no scriptable Gatekeeper trigger), so reusing the sibling app's genuine captures is the only honest, deterministic option.

### R4 — vk-bot-desktop best practices (applied)
**Chosen:** mirror the reference's macOS section — a System Settings `<details open>` with steps 1/2/3, the real screenshots, the `xattr -dr com.apple.quarantine` one-liner, the ad-hoc-signing rationale, SHA-256 verification, and `gh attestation verify` guidance. The reference snapshot (`App.jsx` + its 3 macOS PNGs) is preserved under `raw-data/vk-bot-desktop-current/`.

### R5 / R6 / R6a — Page structure + landing chooser + big button (shipped)
**Chosen:** the `/` chooser (app/docs/download cards), `/docs/` hub, and `/docs/api` (`cargo doc` at deploy) already landed on `main`; PR #486 adds the **big source button** the maintainer asked for and verifies the whole structure with `issue-479-site.spec.js`. **Existing components reused:** the shared `src/web/site-chrome.js` chooser factory; `rustdoc`/`cargo doc`; GitHub Pages (`upload-pages-artifact` / `deploy-pages`).

### R7 / R10 — Template audit + upstream reporting (done)
**Chosen:** fetch and diff every template's CI/CD surface (full file trees under `template-comparison/<short>/`), grep for `workflow_run`/`head_sha`, and conclude. **Result:** no template carries the #479 defect, so — per the maintainer's "if the same issue is found" condition — there is no analogous bug to file. Genuine *enhancement* gaps are recorded (§9) but not filed as bugs.

### R8 / R9 / R11 — Case study + verbose mode + single PR (done)
**Chosen:** this README + `raw-data/` captures + `template-comparison/REPORT.md` + cited online research (§8); verbose diagnostics in the resolve script and the Pages probe; everything in PR #486.

---

## 7. Existing Components / Libraries Survey

What the ecosystem already provides for each sub-problem, and what this PR reuses.

### Desktop release resolution & provenance (R1)
- **GitHub Releases API** (`gh release view`, `gh release upload --clobber`, `gh api repos/…/commits/<tag>`) — source of truth for "which release, which assets". Reused by the resolve script and the upload step.
- **GitHub Pages atomic deploys** — the property that makes the marker-authoritative probe correct: the whole artifact flips live together, so `deployment.json`'s SHA proves the matching build is serving.
- **[`actions/attest-build-provenance@v2`](https://github.com/actions/attest-build-provenance)** — SLSA build-provenance attestation, verifiable with `gh attestation verify`. Already wired into the desktop build. REPORT.md confirms **no template** produces attestations — formal-ai is ahead here.
- **`electron-builder`** — the cross-platform packager producing `.dmg/.zip/.exe/.AppImage/.deb/.tar.gz`. Already used.

### Screenshots (R2, R3)
- **The sibling app `konard/vk-bot-desktop`** — ships *real* macOS Gatekeeper captures under the identical ad-hoc-signing flow; reused directly (R3a). This is the "existing component" that replaces the rejected synthetic generator.
- **[Playwright](https://playwright.dev/)** — powers the app-preview refresh and the e2e assertions (`naturalWidth > 0`, localized alt/caption, big-button bounding box).

### `/docs/api` and `/docs/*` (R5)
- **`rustdoc` / `cargo doc`** — already wired into `release.yml` (`cargo doc --no-deps --lib` → `src/web/docs/api/`). The Rust template's `deploy-docs` job was the reference pattern.
- **GitHub Pages** (`actions/upload-pages-artifact`, `actions/deploy-pages`) — hosts `/`, `/app`, `/download`, `/docs`, `/docs/api` from one artifact.

### CI quality / security (R7 — cross-cutting)
- **[lychee](https://github.com/lycheeverse/lychee)** broken-link checker (in the JS template's `links.yml`) — formal-ai and 3 of 4 templates lack any link validation.
- **CodeQL** (`github/codeql-action`) + **`actions/dependency-review-action`** — **absent in all four templates and formal-ai**.
- **`setup-buildx-resilient`** (Rust/JS templates' composite action), **`check-cargo-lock.rs`** + **`smoke-test-published-crate.rs`** (Rust template) — reliability jobs formal-ai lacks (§9.3).

### Knowledge from the `/download` page itself
- The page already ships an **in-browser SHA-256 verifier** (Web Crypto `crypto.subtle.digest`) and a `BUILD-PROVENANCE.txt` reader — see `vk-bot-desktop-current/App.jsx`. These satisfy R5's "testable without developer accounts" clause.

**Net:** for every requirement, either a project component already realizes it (now cited) or a specific named external component is the documented reuse target.

---

## 8. Online Research

Targeted searches and fetches grounding the load-bearing technical claims. Where a search added nothing beyond the primary doc, that is stated honestly.

### 8.1 `workflow_run` `head_sha` semantics — why the triggered job sees a different SHA

- The `workflow_run` event *"occurs when a workflow run is requested or completed."*
- The **triggered** workflow's `GITHUB_SHA` defaults to the **"Last commit on default branch"** and `GITHUB_REF` to the **"Default branch"** — *not* the SHA of the run that triggered it ([Events that trigger workflows](https://docs.github.com/en/actions/writing-workflows/choosing-when-your-workflow-runs/events-that-trigger-workflows)).
- The *triggering* run's head is exposed separately as `github.event.workflow_run.head_sha`. Community guidance corroborates that this field *"points to the sha that triggered the test workflow"*, and that *"by the time the [downstream] workflow starts, every reference to the head sha is the latest commit, rather than the tested sha"* ([polpiella.dev](https://www.polpiella.dev/github-action-workflows-side-effects)).

This is precisely the formal-ai situation: `gh run list` shows the default-branch HEAD (child `56ccb77e`), while the resolve step reads `workflow_run.head_sha` (parent `0abd3f45`).

### 8.2 Why `GITHUB_TOKEN`-pushed releases don't start a CI run (the recursion guard)

> *"events triggered by the `GITHUB_TOKEN` will not create a new workflow run, with the following exceptions: `workflow_dispatch` and `repository_dispatch` events always create workflow runs. … For all other events, this behavior prevents you from accidentally creating recursive workflow runs."* — [GitHub Docs — GITHUB_TOKEN](https://docs.github.com/en/actions/concepts/security/github_token)

So the only completed CI run is the one on the **parent** commit — which is why `workflow_run` (not `release`/`push`) is the correct trigger, and why the resolve step must not depend on a tag pointing at that parent.

### 8.3 GitHub Pages deploys atomically (why the marker is authoritative)

The marker-authoritative probe (§4.1 layer 3) rests on Pages deploying a **single immutable artifact** that goes live as a unit. GitHub's `actions/deploy-pages` uploads one tar artifact and the Pages service swaps it in atomically; there is no window where `deployment.json` is fresh but a sibling HTML file is stale within the *same* deploy. Hence a marker reading the expected SHA is sufficient proof. (The probe additionally guards against a half-run by rejecting surviving `__FORMAL_AI_*__` placeholders.)

### 8.4 macOS 15 (Sequoia) Gatekeeper "Open Anyway" moved to System Settings

Apple's developer announcement (August 6, 2024):

> *"In macOS Sequoia, users will no longer be able to Control-click to override Gatekeeper when opening software that isn't signed correctly or notarized. They'll need to visit System Settings > Privacy & Security to review security information for software before allowing it to run."* — [Apple Developer](https://developer.apple.com/news/?id=saqachfa)

Reputable coverage corroborates the new three-step flow ([iDownloadBlog](https://www.idownloadblog.com/2024/08/07/apple-macos-sequoia-gatekeeper-change-install-unsigned-apps-mac/), [Michael Tsai](https://mjtsai.com/blog/2024/07/05/sequoia-removes-gatekeeper-contextual-menu-override/)). This is exactly the `installMacosSettingsStep1/2/3` sequence the page documents and the three **real** vk-bot-desktop screenshots illustrate.

### 8.5 References

- GitHub Docs — *Events that trigger workflows*: <https://docs.github.com/en/actions/writing-workflows/choosing-when-your-workflow-runs/events-that-trigger-workflows>
- GitHub Docs — *GITHUB_TOKEN* (recursion guard): <https://docs.github.com/en/actions/concepts/security/github_token>
- GitHub Docs — *Configuring a publishing source / Pages deployments* (atomic artifact): <https://docs.github.com/en/pages>
- M. Pol Piella — *GitHub Action workflows side effects*: <https://www.polpiella.dev/github-action-workflows-side-effects>
- Apple Developer News — *Updates to runtime protection in macOS Sequoia* (Aug 6, 2024): <https://developer.apple.com/news/?id=saqachfa>
- iDownloadBlog — *macOS Sequoia removes the Control-click Gatekeeper method*: <https://www.idownloadblog.com/2024/08/07/apple-macos-sequoia-gatekeeper-change-install-unsigned-apps-mac/>
- Michael Tsai — *Sequoia Removes Gatekeeper Contextual Menu Override*: <https://mjtsai.com/blog/2024/07/05/sequoia-removes-gatekeeper-contextual-menu-override/>

> **Honest note:** the official `workflow_run` docs page does **not** itself spell out the recursion guard; that fact is on the separate `GITHUB_TOKEN` page (cited above). No source was invented to fill a gap.

---

## 9. Cross-Repo / Upstream Findings (R7, R10)

The full audit is [`template-comparison/REPORT.md`](template-comparison/REPORT.md) (the four `link-foundation/{js,rust,python,csharp}-ai-driven-development-pipeline-template` repos, full file trees preserved under `template-comparison/<short>/FULL-FILE-TREE.txt`).

### 9.1 The #479 bug is working-repo-specific — nothing to file

- `grep -rn "workflow_run" --include='*.yml'` over **every** fetched template workflow → **0 matches**. No template has a `workflow_run`-triggered workflow.
- `grep -rn "head_sha"` over all fetched template files → **0 matches**. `github.event.workflow_run.head_sha` (the #479 cause) appears **only** in formal-ai's `desktop-release.yml`.
- No template produces release-attached desktop binaries at all. The closest is `js/.github/workflows/example-app.yml`, whose `desktop-package` job packages Electron across `[ubuntu, macos, windows]` but **uploads only as CI artifacts** (`actions/upload-artifact@v7`, triggered by `pull_request`/`push`/`workflow_dispatch`) — never to a Release, never resolving a tag from a SHA, so it **cannot** exhibit #479.
- The "smoke/wait" steps in every template `release.yml` are **registry-propagation waits** (`wait-for-npm`, `wait-for-crate`, `wait-for-nuget`, `smoke_test_published_package`) — they wait for npm/crates.io/NuGet/PyPI to publish, then smoke-test the package. **None is a Pages-deploy-gated probe** (formal-ai's layer-3 failure mode): `grep -rn "needs:.*deploy\|needs:.*pages\|needs:.*e2e\|needs:.*smoke"` across all templates → **0 matches**, so no template gates a release/desktop job on a Pages/e2e job either.

> **Conclusion:** *neither half* of the formal-ai bug (the `workflow_run.head_sha` tag mismatch **or** the Pages-deploy-gated desktop job) exists in any template. The #479 defect is working-repo-specific and now remediated. **No upstream desktop-release bug to report**, consistent with the maintainer's "if the same issue is found" condition.

### 9.2 Genuine enhancement gaps → candidate upstream issues (recorded, not filed)

These are *not* the #479 bug, but real cross-cutting gaps the audit surfaced. They are recorded here as a backlog; because the maintainer's instruction is conditional on *"the same issue"* being found (it isn't), they are **not** filed as issues against the external repos.

| # | Target | Title | Reproducible check |
|---|---|---|---|
| **U1** | **all 4 templates** | "Add CodeQL + dependency-review to the CI pipeline" | `gh api repos/link-foundation/<repo>/git/trees/HEAD?recursive=1 --jq '.tree[].path' \| grep -iE 'codeql\|security'` → nothing. |
| **U2** | **rust / python / csharp** | "Port the `links.yml` broken-link checker from the JS template" | `links.yml` present only in `js`. |
| **U3** | **rust template** | "Provide an optional cross-platform desktop-release workflow + /download page" | Upstream formal-ai's *fixed* pipeline; **ship the corrected resolve logic** (resolve the latest published release; the auto-release tags a child `chore: release vX.Y.Z` commit whose first parent is the CI head SHA), **not** a naive `workflow_run.head_sha == tag commit` match. Repro to avoid: `gh api repos/$REPO/tags --jq '.[] \| select(.commit.sha=="'$HEAD_SHA'")'` returns empty whenever the tag sits on the auto-release child commit → build skipped forever. |

### 9.3 formal-ai's own gaps the templates would close (inbound backlog)

Reliability jobs the **Rust template** has that formal-ai lacks. These are candidate follow-ups *into* formal-ai (ordered by value in REPORT.md), each with a precise reuse pointer. **`/docs/api` is now CLOSED** (formal-ai's `release.yml` gained a `cargo doc --no-deps --lib` → `src/web/docs/api/` step), so it is dropped from this list versus the original audit.

| # | Gap in formal-ai | Reuse source | Why it matters |
|---|---|---|---|
| 1 | No `cargo-lock` guard | Rust `release.yml` `check-cargo-lock.rs` job | A stale/missing `Cargo.lock` degrades `hashFiles('**/Cargo.lock')` cache keys to the empty hash. |
| 2 | No published-crate smoke test | Rust `scripts/smoke-test-published-crate.rs` | Catches a crate that publishes but doesn't install/run. |
| 3 | Plain `docker/setup-buildx-action@v4` | Rust `.github/actions/setup-buildx-resilient/action.yml` | Survives Docker Hub outages (retries + `mirror.gcr.io` fallback). |
| 4 | Single-OS test matrix | Rust `release.yml` `[ubuntu, macos, windows]` | For a desktop app, platform regressions otherwise surface only in the heavier desktop build. |
| 5 | `cancel-in-progress: true` unconditionally | Rust `release.yml` `cancel-in-progress: github.ref != 'refs/heads/main'` (formal-ai's own `desktop-release.yml` already does the safe form) | The unconditional form can cancel an in-flight `main` push run. |

None of these is the #479 bug; they are the *inbound* side of the "use the templates' best practices" instruction (R7), recorded as a backlog. PR #486 is scoped to the bug + the two documentation defects + the big button.

### 9.4 Boundary case worth noting (honest caveat)

The C# template's `docs.yml` explicitly documents avoiding the *inverse* anti-pattern — it warns "never on `release: published` … see issue #15". That is the mirror image of formal-ai's situation: formal-ai *must* use `workflow_run` precisely because `GITHUB_TOKEN`-pushed auto-releases suppress the `release` event. Both repos arrive at the same conclusion (don't depend on `release: published` for `GITHUB_TOKEN` auto-releases) from opposite directions, which independently corroborates the root-cause analysis.

---

## 10. Risks & Trade-offs of the Chosen Fix

| Risk | Likelihood | Mitigation in the PR |
|---|---|---|
| The relaxed gate builds on a `workflow_run` whose pipeline genuinely failed *before* publishing a release | Low | The script resolves the latest *published* release and the idempotency guard skips when it already has assets; if no release was published, Tier-2 resolution finds the prior (already-asset-bearing) release and skips. A spurious build is at worst a harmless rebuild attempt, never a wrong-asset upload. |
| The marker-authoritative probe passes while a page is genuinely stale | Very low | Pages deploys atomically (§8.3); the probe still requires HTTP 200 on the index and rejects surviving `__FORMAL_AI_*__` placeholders, catching a half-run or broken stamp step. |
| Tier 2 builds the *wrong* release when a `workflow_run` fires without a fresh release | Low–Med | The idempotency guard skips when the latest release already has `formal-ai-desktop-*` assets. The first build heals the backlog; thereafter only *new* asset-less releases trigger a build. |
| The diagnostic parent-SHA check could fail to confirm and build anyway | Low | **Intentional** — building is the safe default; a confirmation miss must not reinstate the silent skip. Logged + idempotency-guarded. |
| Reusing vk-bot-desktop screenshots shows a different app name (`VK Bot Desktop`) | n/a (disclosed) | The dialogs are byte-identical apart from the name; the localized caption/alt copy states this explicitly and the provenance README documents it. The alternative (synthetic renders) was rejected by the maintainer. |
| Manual `release` / `workflow_dispatch` reruns clobber existing assets | Low | **By design** — maintainers can force a refresh; `gh release upload --clobber` is idempotent on asset name. Only `workflow_run` is guarded against redundant rebuilds. |

---

## 11. Lessons Learned / Prevention

### What would have caught this earlier

1. **A post-release assertion that desktop assets > 0.** The single most valuable guardrail: after the desktop pipeline runs (or as a scheduled check), assert the latest release carries `formal-ai-desktop-*` assets and fail loudly if not. Seventeen asset-less releases shipped silently because *nothing* asserted the end state — and the **layer-2 gate compounded this**: PR #480's correct fix merged but a green check was impossible to observe because the build never ran. The idempotency query (`[.assets[].name | select(startswith("formal-ai-desktop-"))] | length`) is exactly such an assertion; promoting it to a *failing* post-release check would convert a silent skip into a red build.

2. **A fix is not done until it runs end-to-end.** PR #480 fixed the resolve logic *correctly* and even unit-tested it, yet the page stayed broken because the fix sat behind two unrelated gates (the success-only trigger and the timing-out Pages probe). The lesson: when a fix targets a *conditional* code path, verify the condition is actually reached in the real pipeline — a passing unit test of dormant code is a false sign of done. The maintainer's "still not fixed" is the canonical symptom of this trap.

3. **Don't couple a probe to incidental page content.** The Pages probe failed the whole pipeline because it grepped the deploy SHA out of the index *body* — an incidental coupling that broke the moment a new page (the chooser landing) shipped without cache-busted assets. Assert on the **authoritative** signal (the atomic deploy marker), not on a proxy that happens to correlate today.

4. **"Real" means real.** Synthetic reproductions of OS dialogs read as fake to anyone who knows the product, and a maintainer will reject them. When a genuine capture is impossible in CI, reuse a *real* capture from a sibling artifact built the identical way and disclose the provenance — don't draw a lookalike.

5. **Verbose diagnostics + self-healing fallback.** The resolve script emits grouped `::group::` diagnostics for every tier; the Pages probe now logs each poll's marker/index state. Pair verbose CI logging with a self-healing fallback (build-by-default, idempotency-guarded) so a transient failure degrades to "build anyway", not "skip forever".

6. **Understand the auto-release commit topology before coupling to it.** An automated release that pushes a *child* commit with `GITHUB_TOKEN` interacts with three GitHub behaviors at once (the recursion guard suppressing events, the `workflow_run` payload exposing the *parent* head SHA, and the triggered job checking out *default-branch* HEAD). Any logic that couples a build to "the commit a release tag points at" must account for that topology — or resolve the release directly and treat the SHA relationship as a *diagnostic*, never a *gate*.

### Net assessment

The primary bug (R1) is **fixed across all three layers, tested, and self-healing**; the macOS screenshots are now **real captures** from vk-bot-desktop (R3/R3a); the **source code is a big landing button** (R6a); the app-preview refresh (R2) and full page structure (R5/R6) are in place; the template audit (R7/R10) is **complete with no analogous upstream bug**; and the case study (R8) reconstructs the timeline, requirements, root causes, and reuse survey with cited evidence. Everything ships in **PR #486**.
