# Case study — Issue #347: Add a `/download` page (cross-platform desktop, themed, CI screenshots)

> Source issue: [link-assistant/formal-ai#347](https://github.com/link-assistant/formal-ai/issues/347)
> Tracking PR: [#348](https://github.com/link-assistant/formal-ai/pull/348)
> Reference implementation: [konard/vk-bot-desktop](https://github.com/konard/vk-bot-desktop) ([live download page](https://konard.github.io/vk-bot-desktop/))
> Compiled: 2026-05-29

This folder is the durable record for issue #347. Raw inputs (the issue JSON,
the reference site's source, repository metadata for every library named in the
issue, and the four CI/CD templates' file trees) live under [`raw-data/`](./raw-data).
This README is the analysis built on top of that data: a complete requirements
catalogue, a per-requirement solution plan, a prior-art survey, the CI/CD
template comparison, and the execution plan for PR #348.

---

## 1. Timeline & framing

| When (UTC) | Event |
|---|---|
| 2026-05-29 22:11 | Issue #347 opened by @konard with labels `documentation`, `enhancement`. |
| 2026-05-29 22:18 | Issue last edited. No comments. |
| 2026-05-29 | PR #348 opened (draft) on branch `issue-347-4cd6329090e8`; raw data compiled into this folder. |

The issue body is short but dense. It bundles a concrete, shippable feature (a
`/download` page that mirrors vk-bot-desktop) with a set of much larger
architectural aspirations (an in-process agent, an opt-in local server API, deep
web↔desktop code reuse with DB sync and request routing, `lino-rest-api`, and a
"LinksQL"). The explicit instruction is to *"plan and execute everything in this
single pull request … until each and every requirement is fully addressed."*

The honest reading — and the strategy this PR follows — is:

1. **Ship the concrete, testable feature to a high standard** (the download page,
   cross-platform build wiring, CI screenshots, verification UX). This is the
   issue title and the bulk of its acceptance criteria, and it is fully
   achievable and verifiable now.
2. **Land the small, real slices of the architectural asks** that can be done
   safely and tested (in-process agent is *already* the default; web-app
   auto-detection of a local server; documentation of the opt-in server API and
   CLI configuration).
3. **Design, document, and roadmap the genuinely large items** (`lino-rest-api`,
   LinksQL, local-database sync, HTTP/tool/code-exec routing to local app +
   Docker) with concrete plans, interface sketches, and prior-art links, rather
   than ship a half-built version that cannot be verified. Every deferral is
   recorded in §9 with a rationale and a roadmap entry.

This split is itself a requirement-driven decision: the issue asks us to *"check
known existing components/libraries"* and to *"propose possible solutions and
solution plans for each requirement"* — i.e. analysis and planning are
first-class deliverables, not just code.

---

## 2. Reference implementation — what vk-bot-desktop's download page actually does

Captured from the reference repo's `site/` directory (saved verbatim under
[`raw-data/vk-bot-desktop-site/`](./raw-data/vk-bot-desktop-site)). The reference
page is a small React app (`App.jsx` + `downloads.js`, bundled to
`assets/site.js`). The feature inventory below is the acceptance checklist for
"have exactly all the features we have at vk-bot-desktop":

| # | Feature | Where in reference | Mirror in formal-ai |
|---|---|---|---|
| F1 | **OS detection** (macOS / Windows / Linux / unknown) from `navigator.userAgentData.platform`, `navigator.platform`, and the UA string | `App.jsx:detectOperatingSystem` | `download.js:detectOperatingSystem` |
| F2 | **Primary download** auto-selected for the detected OS, with a graceful "choose your OS" empty state | `App.jsx` hero `download-panel` | `download.js` hero panel |
| F3 | **OS tabs** to switch the primary download between macOS/Windows/Linux | `App.jsx` `.os-tabs` | same |
| F4 | **Full download grid** — every OS × arch × format family | `downloads.js:downloadFamilies` | `download.js:downloadFamilies` |
| F5 | **Arch coverage**: macOS arm64 + x64 (dmg + zip); Windows installer + portable (x64 + arm64); Linux AppImage + deb + tar.gz (x64 + arm64) | `downloads.js:downloadOptions` | same set, retargeted to formal-ai assets |
| F6 | **GitHub Releases API** fetch of `releases/latest` with `loading` / `ready` / `fallback` status; falls back to the releases page when the API is unreachable | `App.jsx` release effect | `download.js` release effect |
| F7 | **Versioned + legacy asset name resolution** (`<prefix>-<version>.<ext>` then `<prefix>.<ext>`) | `downloads.js:candidateAssetNames` | same |
| F8 | **Theme support** following `prefers-color-scheme`, reacting live to OS theme changes | `App.jsx:detectTheme` + media listener | extended: explicit **auto/light/dark switcher** persisted in shared prefs |
| F9 | **Locale switch** (reference ships en + ru) | `App.jsx` `.locale-switch` | extended to **en / ru / zh / hi** (formal-ai's four UI languages) |
| F10 | **Theme- and locale-specific preview screenshots** in a faux OS window frame (`app-preview-<locale>-<theme>.png`) with traffic-light/titlebar chrome per OS | `App.jsx` `.hero-media` | same, generated in CI |
| F11 | **In-browser checksum verification** — pick the downloaded file + `SHA256SUMS.txt`, hash with `crypto.subtle`, compare the matching line, all client-side | `App.jsx:VerificationTool` | `download.js` verification tool |
| F12 | **Command-line verification** snippets (Windows PowerShell `Get-FileHash`, macOS `shasum`, Linux `sha256sum -c`, plus `gh attestation verify`) | `App.jsx:verificationCommands` | same, retargeted |
| F13 | **Build provenance** link (`BUILD-PROVENANCE.txt`) + reproducible-build caveat | `App.jsx` verification section | same + `actions/attest-build-provenance` |
| F14 | **macOS Gatekeeper instructions** (System Settings flow + `xattr -dr com.apple.quarantine` one-liner) with screenshots | `App.jsx` `install-macos` | same |
| F15 | **Strict CSP** meta tag (`default-src 'self'; connect-src https://api.github.com; …`) | `index.html` | same, `connect-src` for the GitHub API |
| F16 | **Responsive layout** (≤860px and ≤480px breakpoints) | `styles.css` media queries | same |
| F17 | **CI screenshot generation** of the page | reference `js.yml` workflow + Playwright | `download` e2e spec + screenshots committed |

Design language: CSS custom-property tokens (`--bg`, `--surface`, `--text`,
`--muted`, `--line`, `--accent`, `--accent-strong`, `--warm`, `--shadow`) defined
on `:root` (light) and overridden on `:root[data-theme='dark']`, with
`color-scheme` set per theme. formal-ai's existing `styles.css` uses the same
`:root[data-theme="dark"]` convention plus a `@media (prefers-color-scheme: dark)`
block, so the download page reuses the **same `data-theme` contract** and the
**same `formal-ai.preferences.v1` storage** as the chat app — the theme/locale a
visitor picks on either surface carries to the other.

### Intentional improvements over the reference

The issue says "exactly all the features … and more". Two deliberate upgrades:

- **Explicit theme switcher.** The reference only *follows* the OS theme. formal-ai
  already models `theme: auto | light | dark` in its preferences, and the issue
  emphasises "respects themes, switching". The download page therefore ships an
  `auto / light / dark` control that round-trips through `FormalAiPreferences`
  (the same Links-Notation-backed `localStorage` the chat app uses), and resolves
  `auto` against `prefers-color-scheme` live.
- **Four languages, not two.** formal-ai supports en/ru/zh/hi; the page ships all
  four so the download experience matches the product.

---

## 3. Requirements catalogue

Every requirement extracted from the issue body, each with an ID, the verbatim
intent, type, and acceptance criteria. "Status in PR #348" is filled per §9.

| ID | Requirement (from the issue) | Type | Acceptance criteria |
|---|---|---|---|
| **R1** | "support all the Linux, Windows, macOS builds for the application in similar way." | Build/CI | electron-builder produces predictably named per-OS/arch artifacts; a release workflow uploads them with checksums + provenance. |
| **R2** | "Make sure download page have exactly all the features we have at vk-bot-desktop … well designed respects themes, switching, we generate screenshots in CI/CD and more." | Feature | `/download` page covers F1–F17 (§2); themed; theme + locale switching; CI screenshots committed. |
| **R3** | "by default use in process agent, that works similar to github.com/link-assistant/agent." | Architecture | Desktop/app default is the in-process agent (no external server required); documented + verified. |
| **R4** | "option to use server API (by default turned off), with docs to how configure claude/codex/agent CLIs with the local server of formal AI." | Feature + docs | A documented, default-off local OpenAI-compatible server mode; copy-paste config for claude/codex/agent CLIs. |
| **R5** | "reuse as much code from the web app inside electron … sync with local database … web app should auto detect local API server if turned on … extend web app features by executing and routing http requests, tool calls, code execution … to local app + dockers." | Architecture | (a) Electron reuses `src/web` verbatim ✅ already; (b) web app **auto-detects** a local server when present; (c) DB sync; (d) request/tool/code-exec routing to local app + Docker. |
| **R6** | "Ideally … implement both link-foundation/lino-rest-api and universal LinksQL like extension of idea from link-assistant/link-cli and features of GraphQL." | Architecture (stretch) | Design for a Links-Notation REST surface + a LinksQL query language; prototype if feasible. |
| **R7** | "only OpenAI compatible APIs should be supported in traditional REST APIs … All other communication … should prefer using Links Notation." | Constraint | REST surface stays OpenAI-shaped only; internal/new channels prefer Links Notation. |
| **R8** | "Use all the best practices from CI/CD templates (check full file tree to compare …) … if the same issue is found in template report issue also in templates" (js/rust/python/csharp pipeline templates). | Process | Compare every workflow/CI file against the four templates; adopt improvements; file upstream issues for shared defects. |
| **R9** | "compile that data to ./docs/case-studies/issue-{id} … deep case study analysis (search online …), list of each and all requirements, propose solutions and solution plans for each, check known existing components/libraries." | Process | This folder: raw data + this analysis. |
| **R10** | "plan and execute everything in this single pull request … until each and every requirement fully addressed." | Process | One PR (#348); every requirement either shipped or explicitly designed + roadmapped with rationale. |

Implicit requirements (derived, not stated): the page must be served by the
existing GitHub Pages pipeline (no new front-end build system bolted onto a Rust
repo); PR CI must stay green (so heavy desktop builds run on release, not on PR
push); the page must degrade gracefully before any binaries exist (so it is
useful and testable on day one); new files must respect the repo's file-size
guard and lint/test gates.

---

## 4. Current state of formal-ai (pre-#348)

- **`src/web/`** — vanilla-JS chat demo (no bundler for app code; `app.js` is
  hand-authored, React via `vendor.bundle.js`). Theme via `data-theme`;
  preferences persisted as Links Notation in `localStorage`
  (`formal-ai.preferences.v1`, key `theme` = `auto|light|dark`, `uiLanguage` =
  `auto|en|ru|zh|hi`). i18n catalog `i18n-catalog.lino` loaded through
  `lino-i18n`. Published to GitHub Pages by the `deploy-demo` job, which uploads
  `src/web` wholesale — so a new `src/web/download/` subtree is published with
  **no pipeline changes**.
- **`desktop/`** — Electron wrapper (`main.cjs`) that serves `src/web` and spawns
  the Rust API as a subprocess. `package.json` defines linux/mac/win targets but
  **no `artifactName`** (so artifacts get electron-builder's default messy names
  that the download page cannot predict) and its `version` (0.129.0) is **out of
  sync** with `Cargo.toml` (0.152.0).
- **`.github/workflows/release.yml`** — builds the Rust binary, runs lint/test/
  coverage/e2e, deploys the demo. **No Electron packaging job and no release
  asset uploads** — so today there is nothing for a download page to point at.
- **In-process agent** — the Rust core already runs the solver in-process; the
  HTTP server is opt-in. So R3's "default = in-process" is effectively already
  true; the work is to make it explicit and documented.

The gaps map cleanly onto R1 (no packaging/release-assets), R2 (no page), R4 (no
documented opt-in server + CLI config), R5b (no auto-detect), and the version/
artifact-naming defects.

---

## 5. Per-requirement solution plans

### R1 — cross-platform builds
**Chosen:** keep `electron-builder` (already a dependency); add explicit
`artifactName` templates so every artifact is `formal-ai-desktop-<os>-<arch>-<version>.<ext>`
(matching the asset prefixes the page resolves); add a `desktop-release.yml`
workflow with a per-OS matrix (`macos-latest`, `windows-latest`, `ubuntu-latest`),
each building its targets, then a `checksums` job that downloads all artifacts,
writes `SHA256SUMS.txt` + `BUILD-PROVENANCE.txt`, runs
`actions/attest-build-provenance`, and uploads everything to the GitHub Release.
Triggers: `release: published` and `workflow_dispatch` only — **never** on PR/push,
so this PR's CI is unaffected. Sync `desktop/package.json` version from
`Cargo.toml` in `prebuild`.
**Alternatives considered:** Tauri (smaller binaries, but a rewrite of the
Electron shell — out of scope); hand-rolled `zip`/`tar` packaging (loses
installers, code-sign hooks, auto-update metadata).

### R2 — the `/download` page
**Chosen:** vanilla `src/web/download/{index.html,download.css,download.js}` — no
new build step, served by the existing Pages job, mirroring vk-bot-desktop's
feature set (F1–F17) but reusing formal-ai's `data-theme` + `FormalAiPreferences`
and shipping en/ru/zh/hi. Self-contained i18n `copy` object (like the reference)
to avoid coupling the lightweight page to the chat app's heavy vendor bundle and
catalog. CI screenshots via a Playwright spec, committed to
`docs/screenshots/issue-347/`.
**Alternatives considered:** porting the reference's React/JSX verbatim (would
add esbuild/vite to a Rust repo for one page — rejected as disproportionate);
server-side rendering (no server on Pages).

### R3 — in-process agent default
**Chosen:** document that the in-process agent is the default and that the server
API is strictly opt-in; verify the desktop shell launches without requiring the
server. Cross-link [link-assistant/agent](https://github.com/link-assistant/agent)
as the behavioural reference.

### R4 — opt-in server API + CLI config
**Chosen:** a focused doc (`docs/desktop/server-api.md`) describing how to enable
the local OpenAI-compatible server, the exact base URL / endpoint
(`/v1/chat/completions`), and copy-paste configuration for each CLI: `codex` via
its `~/.codex/config.toml` `model_providers` block (`wire_api = "chat"`), `agent`
via the `OPENAI_BASE_URL` / `OPENAI_API_KEY` conventions, and `claude` via an
Anthropic→OpenAI adapter (it speaks the Anthropic Messages protocol, not OpenAI,
so it cannot target the endpoint directly). Default remains off.

### R5 — web↔desktop reuse, auto-detect, sync, routing
- **5a reuse** — already satisfied (`main.cjs` serves `src/web`); documented.
- **5b auto-detect** — shipped: the web app detects a running local server through
  the Electron preload bridge (`window.FormalAiDesktop.getStatus()`), **not** a
  network scan — the browser cannot probe loopback ports. When the bridge reports
  `apiReady` + `apiBase` the chat routes to the local server; otherwise it stays
  in-process. Loopback-only and privacy-preserving. See
  [`../../desktop/server-api.md` §5b](../../desktop/server-api.md#5b-auto-detecting-the-local-server).
- **5c DB sync** — shipped: `src/memory_sync.rs` (`SyncStore` + union-by-id merge)
  with `GET /v1/memory/since` + `POST /v1/memory/import`, reconciled from the
  desktop by `desktop/lib/memory-sync.cjs`. See [§9](#9-execution-plan-for-pr-348)
  and [ROADMAP D1](./ROADMAP.md#d1--r5c-local-database-sync-).
- **5d request/tool/code-exec routing to local app + Docker** — shipped:
  `desktop/lib/tool-router.cjs` is a default-deny dispatcher that serves
  `http_fetch` / `read_local_file` from the local process and routes
  `code_exec` / `shell` into `konard/box-dind:2.1.1`. See
  [ROADMAP D2](./ROADMAP.md#d2--r5d-route-http-requests-tool-calls-and-code-execution-to-the-local-app--docker-).

### R6 — lino-rest-api + LinksQL
**Shipped.** Survey ([§6](#6-prior-art--library-survey)) shows `lino-rest-api` is
an early-stage Deno/TypeScript project and `link-cli` lives at **link-foundation**
(not link-assistant). This PR implements a Links-Notation request/response
envelope (`GET /v1/bundle`, `GET /v1/links`, `POST /v1/links/query`) and a LinksQL
grammar (link pattern + GraphQL-style field selection) in `src/links_query.rs`,
evaluated against the knowledge-graph projection. See
[ROADMAP D3](./ROADMAP.md#d3--r6-lino-rest-api--universal-linksql-).

### R7 — OpenAI-only REST, Links Notation elsewhere
**Chosen:** a documented constraint/ADR. The page itself honours it: the only
REST it speaks is the GitHub Releases API (external, unavoidable) over a strict
CSP `connect-src`; all *new internal* persistence it does (theme/locale) is Links
Notation via `FormalAiPreferences`.

### R8 — CI/CD template comparison
**Chosen:** compare formal-ai's workflows against the four templates' trees
([raw-data](./raw-data)); record findings in [§7](#7-cicd-template-comparison);
adopt low-risk improvements; open upstream issues for defects shared with a
template.

### R9 / R10 — case study + single PR
This document + the raw data satisfy R9. R10 is the PR itself.

---

## 6. Prior-art / library survey

From the repository metadata captured in `raw-data/*-meta.json` and READMEs:

| Library | Owner | What it is | Relevance |
|---|---|---|---|
| [agent](https://github.com/link-assistant/agent) | link-assistant | In-process agent reference | R3 behavioural model. |
| [lino-rest-api](https://github.com/link-foundation/lino-rest-api) | link-foundation | Early Links-Notation REST experiment | R6 starting point; informs R7 envelope. |
| [link-cli](https://github.com/link-foundation/link-cli) | **link-foundation** | CLI over a links store | R6 LinksQL inspiration. **Note:** the issue says `link-assistant/link-cli`, but it actually lives at `link-foundation/link-cli` (recorded in `raw-data/link-cli-NOT-FOUND.txt`). |
| [lino-objects-codec](https://github.com/link-foundation/lino-objects-codec) | link-foundation | Object ↔ Links Notation codec | R7 transport (object encoding). |
| [lino-arguments](https://github.com/link-foundation/lino-arguments) | link-foundation | CLI args as Links Notation | R7 config channel. |
| lino-i18n | (npm, already a dep) | Links-Notation i18n catalogs | Already powers the chat app's i18n. |

Reference page mechanics worth reusing verbatim (and adapted in `download.js`):
`crypto.subtle.digest('SHA-256', …)` for client-side hashing; the
`^([a-f0-9]{64})\s+\*?(.+)$` line parser for `SHA256SUMS.txt`; versioned+legacy
candidate-name resolution; `AbortController`-guarded release fetch.

External best practices grounded by online research:
- **electron-builder `artifactName`** defaults to `${productName}-${version}.${ext}`
  and supports `${arch}`/`${version}`/`${ext}`/`${productName}`; a missing
  `${arch}` is stripped with its separator
  ([electron.build/configuration](https://www.electron.build/configuration.html),
  [#1493](https://github.com/electron-userland/electron-builder/issues/1493)).
  We build per-arch so `${arch}` is always present.
- **Build provenance** via
  [`actions/attest-build-provenance`](https://github.com/actions/attest-build-provenance)
  yields SLSA v1 Build Level 2 attestations verifiable with `gh attestation verify`
  ([GitHub Docs: artifact attestations](https://docs.github.com/en/actions/concepts/security/artifact-attestations)).

---

## 7. CI/CD template comparison

**Method.** Compare formal-ai's `.github/workflows/*` and CI scripts against the
file trees of the four templates (captured in
`raw-data/{js,rust,python,csharp}-template-tree.txt` +
`-meta.json`). formal-ai is a Rust project, so the **rust** template is the
primary baseline; the others are cross-referenced for shared conventions
(changelog automation, file-size guards, release flow).

**Findings (initial, expanded in commits):**

| # | Observation | Action |
|---|---|---|
| C1 | `desktop/package.json` `version` (0.129.0) drifts from `Cargo.toml` (0.152.0). | Fix: sync in `prebuild`. Check whether the rust template's release flow guards a similar secondary manifest; if the drift is structural, report upstream. |
| C2 | No artifact-naming convention for desktop builds → unpredictable asset names. | Fix: explicit `artifactName`. |
| C3 | Desktop builds absent from CI entirely. | Fix: dedicated `desktop-release.yml` gated on release/dispatch. |
| C4 | `CONTRIBUTING.md` retains generic template boilerplate. | Note; align with the rust template's current wording where it diverges. |
| C5 | File-size guard covers `.rs`/`.lino` only, not web assets. | Acceptable for this PR (page files are modest); note as a possible upstream enhancement. |

Upstream issues are filed only for defects that are genuinely shared with a
template (per R8's "if the same issue is found in template"); formal-ai-specific
gaps are fixed here without upstream noise. Any filed issue is linked here.

---

## 8. Online research notes

- vk-bot-desktop's page is JS-rendered, so the feature inventory (§2) was built
  from its **source** (`site/App.jsx`, `site/downloads.js`, `site/styles.css`)
  rather than the rendered HTML — saved under `raw-data/vk-bot-desktop-site/`.
- electron-builder template-variable behaviour and the `${arch}` stripping caveat
  are confirmed against the official docs and issue tracker (links in §6).
- GitHub artifact attestations (Sigstore-backed, SLSA L2, `gh attestation verify`)
  are confirmed against GitHub Docs and the `actions/attest-build-provenance`
  action (links in §6). This matches the reference page's
  `gh attestation verify … --repo …` snippet and `BUILD-PROVENANCE.txt`.

---

## 9. Execution plan for PR #348

Every requirement (R1–R10) is implemented and tested in this PR.

**Delivered in this PR (shippable + tested):**

- ✅ **R2** `/download` page (F1–F17), themed + theme/locale switching, en/ru/zh/hi,
  in-browser checksum verification, provenance + macOS Gatekeeper guidance, CSP.
- ✅ **R2/R8** Playwright e2e spec + CI-committed screenshots across themes/locales.
- ✅ **R1** electron-builder `artifactName` templates + version sync +
  `desktop-release.yml` (release/dispatch-gated) with `SHA256SUMS.txt`,
  `BUILD-PROVENANCE.txt`, attestation, and release upload.
- ✅ **R3** in-process-agent-default documented + verified.
- ✅ **R4** opt-in server API doc + claude/codex/agent CLI configuration, including
  the first-party Anthropic→OpenAI adapter (`POST /v1/messages`,
  `src/anthropic.rs`) so `claude` needs no third-party proxy.
- ✅ **R5a** documented web reuse; **R5b** web-app local-server auto-detect (opt-in,
  loopback-only).
- ✅ **R5c** local-database sync — `src/memory_sync.rs` (`SyncStore` + union-by-id
  merge) over `GET /v1/memory/since` + `POST /v1/memory/import`, reconciled from
  the desktop by `desktop/lib/memory-sync.cjs`.
- ✅ **R5d** HTTP/tool/code-exec routing to the local app + Docker —
  `desktop/lib/tool-router.cjs`, a default-deny dispatcher that serves
  `http_fetch` / `read_local_file` locally and routes `code_exec` / `shell` into
  `konard/box-dind:2.1.1`; denied calls return a structured refusal.
- ✅ **R6** `lino-rest-api`-style Links-Notation REST envelopes
  (`GET /v1/bundle`, `GET /v1/links`, `POST /v1/links/query`) + the LinksQL query
  language (`src/links_query.rs`).
- ✅ **R7** OpenAI-only-REST constraint documented and honoured; all new internal
  interfaces speak Links Notation.
- ✅ **R8** template comparison (§7) + any upstream issues linked.
- ✅ **R9/R10** this case study; everything in the one PR.

Each item is detailed in [`ROADMAP.md`](./ROADMAP.md) with its delivered code and
the acceptance test that verifies it.

---

## 10. Verification strategy

- **Unit/logic:** the page's pure helpers (OS detection, asset-name resolution,
  `SHA256SUMS` line parsing, family grouping) are exercised by the e2e spec via a
  test hook and by Node `--check` syntax validation.
- **e2e (Playwright):** OS-tab switching changes the primary download; theme
  switch toggles `data-theme` and persists to `formal-ai.preferences.v1`; locale
  switch swaps copy across en/ru/zh/hi; the releases fetch is mocked so the grid
  renders deterministically; the checksum tool reports match/mismatch/missing.
- **Screenshots:** committed under `docs/screenshots/issue-347/` and referenced
  from the PR description for human review.
- **Gates:** `cargo fmt`/`clippy`/`test`, the file-size guard, `node --check` on
  new JS, and the e2e suite all pass locally before push.
