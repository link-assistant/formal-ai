# Issue #554 Solution Plan

For each requirement ID: one or more approaches, a recommendation, and the
existing components/libraries (repo + online) that help. Concrete file-level plan
fits this repo's static-first, CSP-strict, `site-chrome.js`-driven web
architecture and its `gh release upload` distribution.

## Architecture fit (shared by all web pages)

New landing pages reuse the **chooser pattern** already proven by
`src/web/landing.js` and the **release-asset resolver** proven by
`src/web/download/download.js`:

- Each page = `src/web/<name>/index.html` + `src/web/<name>/<name>.js`, loading
  `../preferences.js` then `../site-chrome.js` then the page config, with the
  same strict CSP. Theme/locale ride the `formal-ai.preferences.v1` localStorage
  key automatically; copy is an inline `{ en, ru, zh, hi }` object.
- Pages that resolve release assets (`.vsix`, installer URLs) add only
  `connect-src https://api.github.com` to the CSP, exactly like `download.js`.
- The site root chooser (`landing.js` `destinations`) gains `vscode`, `telegram`,
  `cli` cards.

These are **content pages with copy-buttons**, not new app runtimes, so they
inherit the existing e2e/lint coverage shape (`window.FormalAi<Name>` exposed for
Playwright, like `FormalAiLanding`/`FormalAiDownload`).

---

## R4 — Build & publish the `.vsix` (do this first; everything else depends on it)

**Approaches.** (a) Add a `.vsix` step to `desktop-release.yml`; (b) add it to the
main `release.yml`; (c) a small dedicated `vscode-release.yml`.

**Recommendation: (a)** — `desktop-release.yml` already resolves the release tag,
uploads with `gh release upload … --clobber`, emits `SHA256SUMS`, and runs
`prepare-resources.mjs`. The `.vsix` is OS-independent, so it slots into the
existing `finalize` job (which already aggregates checksums) rather than the
6-way build matrix.

**Components/libraries.** `@vscode/vsce@^3.9.2` (already a devDependency,
`vscode/package.json`); `npm run vscode:package` (already wired);
`scripts/prepare-resources.mjs` (already mirrors web+seed); `gh release upload`
(already used); SHA-256 aggregation in `finalize` (already exists).
(https://github.com/microsoft/vscode-vsce)

**File-level plan.**
- `.github/workflows/desktop-release.yml`: in `finalize` (runs once, has Node),
  add steps — checkout at the resolved tag, `npm ci` at root + `npm --prefix vscode ci`,
  `npm run vscode:package`, rename output to `formal-ai-vscode-<version>.vsix`,
  `gh release upload "$TAG" formal-ai-vscode-*.vsix --clobber`, and append its
  `sha256sum` line into `SHA256SUMS.txt` before upload.
- No new dependency; version already synced from `Cargo.toml` by
  `prepare-resources.mjs`.

**Risks / CI.** `vsce package` can fail on manifest lint (icon, repository, README
present — all already satisfied). The `.vsix` build runs Node-only and is
verifiable in CI (the package step is deterministic). Installing it via `code`
cannot be exercised in CI (no VS Code in the runner) — that path stays
manually verified, consistent with `docs/vscode/extension.md`'s honesty about
un-CI-able paths.

---

## R1 — VS Code extension landing page (`/vscode/`)

**Approach.** New chooser-style page that (1) detects and links the latest
release `.vsix` via the GitHub Releases API, (2) shows the manual
`code --install-extension` CLI + GUI steps, (3) shows the `curl|bash` / `irm|iex`
one-liners (R3/R5), (4) explains "extension only mode" and the web-host
no-sideload caveat, and (5) links the Desktop one-click (R2).

**Recommendation.** Reuse `download.js`'s asset-resolution helpers
(`assetNameFor`/`candidateAssetNames`/`resolveDownloadAsset`) adapted to match
`formal-ai-vscode-<version>.vsix`. Keep CSP = download page CSP.

**Components.** `site-chrome.js` (`createChooser`), `download.js` resolver
pattern, GitHub Releases API; install command from VS Code docs
(`code --install-extension myextension.vsix`, **Extensions: Install from VSIX**).
(https://code.visualstudio.com/docs/configure/extensions/extension-marketplace,
https://code.visualstudio.com/api/extension-guides/web-extensions)

**File-level plan.**
- `src/web/vscode/index.html` — CSP with `connect-src https://api.github.com`;
  loads `../preferences.js`, `../site-chrome.js`, `vscode.js`; `<noscript>`
  fallback linking the releases page (like `download/index.html`).
- `src/web/vscode/vscode.js` — page config + `{en,ru,zh,hi}` copy: download
  button (resolved `.vsix` URL), CLI/GUI manual steps, the installer one-liners,
  "extension only mode" + web-host caveat, link to Desktop one-click. Expose
  `window.FormalAiVscode`.
- `src/web/vscode/vscode.css` (or reuse `download.css`).
- `src/web/landing.js` — add `{ id: "vscode", href: "vscode/", icon: "🧩", … }`.

**Risks.** Until R4 ships a `.vsix`, the resolver finds nothing; the page must
fall back to a "build it yourself" note (`npm run vscode:package`). Verifiable in
CI via Playwright against the rendered page + a mocked release fixture.

---

## R2 — One-click install from the Desktop app

**Approaches.** (a) Shell out to `code --install-extension <downloaded .vsix>`;
(b) open the `vscode:extension/link-assistant.formal-ai-vscode` deep link.

**Recommendation: (a).** The `vscode:` deep link opens the *Marketplace page*
inside VS Code; since formal-ai is **not** on the Marketplace it cannot install,
and the link is a no-op unless VS Code is already running
(https://github.com/Microsoft/vscode/issues/20289). Shelling out to the `code`
CLI is Marketplace-independent and matches the desktop's existing "shell out to a
local tool" model (it already shells to Docker via `service-control.cjs`).

**Components.** Existing Electron IPC + preload bridge and the
disabled-with-note UX from `desktop/lib/service-control.cjs` /
`docs/desktop/service-control.md`; `code --install-extension`
(https://code.visualstudio.com/docs/configure/extensions/extension-marketplace).
Argument-injection guidance: fixed arg vector, app-controlled path
(https://www.sonarsource.com/blog/securing-developer-tools-argument-injection-in-vscode/).

**File-level plan.**
- `desktop/lib/vscode-install.cjs` (new, testable like `service-control.cjs`):
  `findCodeCli()` (probe `code`/`code-insiders` on PATH), `resolveVsixUrl()`
  (Releases API, same resolver shape as the web page), `downloadVsix(dest)`,
  `installVsix({ codeCli, vsixPath, run })` calling
  `run(codeCli, ["--install-extension", vsixPath])`. All effects injected for unit
  tests.
- Wire IPC in `desktop/main.cjs` (e.g. `formalAiDesktop:installVscodeExtension`)
  and expose through the preload bridge; add a sidebar control next to Services.
- Disable the control with a clear note when `findCodeCli()` is null.

**Risks / CI.** `findCodeCli`, URL resolution, and arg construction are pure and
unit-testable; the real `child_process` spawn of `code` is **not** CI-verifiable
(no VS Code in runner) — manual verification, documented as such.

---

## R3 — Manual `curl/wget … .sh | bash` with a raw-GitHub link

**Approach.** The universal `scripts/install.sh` (R5) accepts a `vscode` target;
the page shows `curl -fsSL <raw>/scripts/install.sh | bash -s -- vscode` and the
`wget -qO- … | bash -s -- vscode` equivalent, plus the **direct raw link** to the
script itself for inspection.

**Recommendation.** Serve the canonical script from
`https://raw.githubusercontent.com/link-assistant/formal-ai/<tag>/scripts/install.sh`
(pinned tag for reproducibility) and **also** mirror it onto GitHub Pages
(`https://link-assistant.github.io/formal-ai/install.sh`) by copying `scripts/`
into the Pages artifact during `deploy-demo`, so a short, branded URL exists too.

**Components.** rustup/Bun/ollama one-liner conventions; `--proto '=https'`
hardening; truncation-safe `main`-function wrapping
(https://rust-lang.github.io/rustup/installation/other.html,
https://www.chef.io/blog/5-ways-to-deal-with-the-install-sh-curl-pipe-bash-problem,
https://bun.com/docs/pm/cli/install).

**File-level plan.**
- The one-liner + raw link live in the `/vscode/` (and `/cli/`) page copy.
- `release.yml` `deploy-demo`: add `cp scripts/install.sh scripts/install.ps1 src/web/`
  before the upload-pages step so the Pages mirror exists; stamp the version line.

**Risks.** Pinned-tag raw URL must be updated per release (or point at `main` and
accept moving target). Pages mirror is verifiable (file present in artifact);
end-to-end `curl|bash` execution is not CI-gated by default.

---

## R5 — Universal `.sh` + PowerShell installer

**Approach.** One `scripts/install.sh` (POSIX) + one `scripts/install.ps1`, each
with target selection: `desktop` (default), `vscode`, `cli`, `all`. Resolve the
latest release via the GitHub API; download the matching asset by the existing
naming scheme.

**Recommendation.** Single script per platform with a `main` wrapper, env-var
configuration (`FORMAL_AI_INSTALL_TARGET`, `FORMAL_AI_INSTALL_VERSION`) to dodge
the PowerShell `irm|iex` `$Args` pitfall, HTTPS-only, optional SHA-256 check
against `SHA256SUMS.txt`, and a `--help`.

**Components.** Release asset naming `formal-ai-desktop-<os>-<arch>-<version>.<ext>`
and `SHA256SUMS.txt` (from `desktop-release.yml`); the `.vsix` from R4;
`cargo install formal-ai` for the CLI; `code --install-extension` for the
extension. PowerShell `irm … | iex` + env-var parameterization
(https://knowledge.buka.sh/powershell-one-liners-for-installation-what-does-irm-bun-sh-install-ps1-iex-really-do/,
https://github.com/JuliusBrussee/caveman/issues/381). Truncation-safe wrapping
(https://www.chef.io/blog/5-ways-to-deal-with-the-install-sh-curl-pipe-bash-problem).

**File-level plan.**
- `scripts/install.sh`: detect OS/arch → map to asset name; `desktop` =
  download+place the artifact; `vscode` = download `.vsix` + `code --install-extension`;
  `cli` = `cargo install formal-ai` (or download a prebuilt CLI binary if/when
  published) and print Docker/Telegram run commands for `all`. Body inside
  `main() { … }; main "$@"`.
- `scripts/install.ps1`: same targets via `Invoke-RestMethod`/`Invoke-WebRequest`;
  Windows asset names; env-var config; `code.cmd --install-extension`.
- `scripts/check-file-size.rs` thresholds apply — keep each under the repo limit.

**Risks.** "All other interfaces" is broad; minimum viable = desktop + vscode +
cli, with Docker/Telegram documented as run commands (they are services, not
"installs"). CI can shellcheck `install.sh` and PSScriptAnalyzer `install.ps1`,
and dry-run target parsing; it cannot fully exercise downloads/`code`/`cargo
install` without network + toolchains. Lint/static-check is the CI-verifiable
slice.

---

## R6 — Telegram bot (`/telegram/`) and CLI (`/cli/`) landing pages

**Approach.** Two more chooser-style content pages, same scaffold as R1.

**Telegram page.** BotFather walkthrough (`/newbot` → name → `…bot` username →
token → `/revoke`), then formal-ai run modes: `docker compose up` with
`TELEGRAM_BOT_TOKEN`, `cargo run -- telegram`, and the Desktop **Services**
one-click. Token-as-password warning.
(https://core.telegram.org/bots/tutorial; `README.md` "Telegram Bot";
`docs/desktop/service-control.md`.)

**CLI page.** Install via the universal installer / `cargo install formal-ai`;
core commands (`chat`, `serve`, `memory`, `dataset`, `telegram`); link to the
agentic-tool configs (Codex/Claude Code/OpenCode) already in `README.md`.

**File-level plan.**
- `src/web/telegram/index.html` + `telegram.js` (+ optional `.css`), expose
  `window.FormalAiTelegram`.
- `src/web/cli/index.html` + `cli.js`, expose `window.FormalAiCli`.
- `src/web/landing.js` — add `telegram` and `cli` destination cards (with
  `vscode` from R1) and the matching `{en,ru,zh,hi}` copy keys.

**Risks.** Pure content pages; fully verifiable in CI via Playwright
(render + copy-button + locale switch), reusing the `FormalAiLanding`/
`FormalAiDownload` test hook pattern. No moving parts beyond static copy.

---

## R7 — Case study

Delivered by this PR: `requirements.md`, this `solution-plan.md`, `README.md`,
and `raw-data/` (issue/PR JSON, `online-research.md`, `interface-inventory.md`).

## Sequencing

1. **R4** (build/publish `.vsix`) — unblocks everything.
2. **R5** (`install.sh` + `install.ps1`) — referenced by every page.
3. **R1 / R6** (pages) — consume R4/R5.
4. **R2** (desktop one-click) — consumes R4.
5. **R3** is satisfied by R5's `vscode` target + the raw link surfaced on R1.

## What is and isn't CI-verifiable (consolidated)

| Verifiable in CI | Not verifiable in CI (manual) |
|---|---|
| `vsce package` produces a `.vsix`; it uploads to the release | `code --install-extension` actually installing (no VS Code in runner) |
| Pages render, locale switch, copy buttons (Playwright) | Real `curl\|bash` / `irm\|iex` end-to-end install on each OS |
| `install.sh` shellcheck + `install.ps1` PSScriptAnalyzer + target parsing | `cargo install` / desktop-asset placement on a clean machine |
| Desktop handler pure helpers (`findCodeCli`, URL resolve, arg build) unit-tested | Real `child_process.spawn` of `code` from Electron |
| Releases-API asset resolution against a fixture | Live Marketplace/`vscode:` deep-link behavior |
</content>
