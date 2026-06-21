# Issue #554 Requirements

Exhaustive, numbered breakdown of every requirement and sub-requirement extracted
from the issue text. IDs are stable (R1, R1.1, …). "AC" = acceptance criteria.

## R1 — VS Code extension landing page with manual install ("VS Code Extension only mode")

> "We should have separate page on our landing page for VS Code extension, with
> detailed instructions how to download and install manually with VS Code
> Extension only mode."

| ID | Statement | Acceptance criteria | Notes / ambiguity |
|---|---|---|---|
| R1 | A dedicated landing page for the VS Code extension at `/vscode/`. | Page exists, reachable from the site root chooser, themed/localized like the rest of the site. | Mirror `landing.js`/`download.js` patterns; en/ru/zh/hi. |
| R1.1 | "Detailed instructions how to download" the extension. | Page links to the downloadable `.vsix` (a GitHub Release asset) and shows the resolved version. | Depends on R4.x (CI must publish the `.vsix`). Until then, page documents the build-it-yourself fallback. |
| R1.2 | "Install manually" instructions. | Step-by-step: `code --install-extension formal-ai-vscode-<version>.vsix` plus the **Extensions: Install from VSIX** GUI path. | Two paths (CLI + GUI). Note VSIX installs disable auto-update. |
| R1.3 | "VS Code Extension only mode" — install just the extension without the Desktop app. | Page frames this as the desktop-host extension standalone; clarifies the web host (`vscode.dev`) runs in-process and cannot sideload a `.vsix`. | "only mode" = extension alone, no Electron Desktop app. Web host cannot be sideloaded — must be stated. |

## R2 — One-click install of the VS Code extension from the installed Desktop app

> "We should also add ability to install it from already installed Desktop app in
> one click."

| ID | Statement | Acceptance criteria | Notes / ambiguity |
|---|---|---|---|
| R2 | The Electron Desktop app can install the VS Code extension in one click. | A button in the desktop UI installs the extension and reports success/failure. | New IPC + main-process handler. |
| R2.1 | The click downloads the released `.vsix` and runs `code --install-extension <vsix>`. | Handler resolves the `code` CLI (PATH/bundled), downloads the release `.vsix`, runs install with a fixed arg vector. | Requires `code` on PATH; degrade gracefully when absent. Avoid the `vscode:` URL (no Marketplace presence; opens nothing unless VS Code already running). |
| R2.2 | The control is hidden/disabled when prerequisites are missing. | If `code` CLI not found or the `.vsix` cannot be fetched, the button is disabled with a clear note (mirrors the Docker-unavailable Services pattern). | Reuse the disabled-with-note UX from `desktop/lib/service-control.cjs`. |

## R3 — Manual install via `curl/wget … .sh | bash` with a direct raw-GitHub link

> "the only way to install VS Code extension should be manual, by command
> downloadable from our repository `curl/wget ... .sh | bash`. With direct link to
> download the installation script from raw github files."

| ID | Statement | Acceptance criteria | Notes / ambiguity |
|---|---|---|---|
| R3 | Provide a one-line `curl … | bash` (and `wget … | bash`) install command. | Command shown on the relevant landing page(s) with a copy button. | Must use HTTPS and `--proto '=https'`-style hardening. |
| R3.1 | The script is downloadable via a **direct raw GitHub URL**. | A stable `raw.githubusercontent.com/link-assistant/formal-ai/<ref>/scripts/install.sh` link is documented and clickable. | Pin to a tag for reproducibility; also mirror via Pages. |
| R3.2 | Manual install is the supported way **because there is no official store yet**. | Page text states formal-ai is not yet on Marketplace/Open VSX and that manual install is the supported path. | Once a store is added, revisit. |
| R3.3 | The script can install the VS Code extension specifically. | Running the installer (e.g. with a `vscode` target) downloads the released `.vsix` and runs `code --install-extension`. | Ties R3 to R4.2 and the universal installer R5. |

## R4 — Build and publish the `.vsix` (implied prerequisite)

The issue requires downloading/installing the extension, which is impossible
until the `.vsix` is produced and hosted. These are implied, load-bearing
sub-requirements.

| ID | Statement | Acceptance criteria | Notes |
|---|---|---|---|
| R4.1 | CI builds the `.vsix` (`vsce package`). | A CI step runs `npm run vscode:package` and produces `formal-ai-vscode-<version>.vsix`. | Reuse existing `@vscode/vsce` dep + `prepare-resources.mjs`. |
| R4.2 | CI uploads the `.vsix` as a GitHub Release asset. | The `.vsix` is attached to the release with `gh release upload … --clobber`, alongside a SHA-256 entry. | Mirror `desktop-release.yml` upload + checksum flow. Not Marketplace publishing. |
| R4.3 | The download/landing pages can resolve the `.vsix` from the Releases API. | Page reads the latest release and finds the `.vsix` by name, like `download.js` resolves desktop assets. | Reuse `resolveDownloadAsset` style logic. |

## R5 — Universal `.sh` + PowerShell installer for all interfaces

> "We should add universal .sh + power shell installer, for all Desktop app, VS
> Code and all other interfaces we have."

| ID | Statement | Acceptance criteria | Notes / ambiguity |
|---|---|---|---|
| R5 | A universal installer exists as both `scripts/install.sh` and `scripts/install.ps1`. | Both files exist, served from stable raw URLs, parallel feature set. | `.ps1` must be parameterized via env vars (the `irm|iex` `$Args` pitfall). |
| R5.1 | The installer covers the **Desktop app**. | A target downloads the OS/arch desktop asset (`formal-ai-desktop-<os>-<arch>-<version>.<ext>`) and installs/places it. | Reuse release asset naming. |
| R5.2 | The installer covers the **VS Code extension**. | A target downloads the `.vsix` and runs `code --install-extension`. | Depends on R4. |
| R5.3 | The installer covers **all other interfaces** (CLI, server, Telegram bot, Docker). | Targets/flags install the CLI (`cargo install formal-ai` or prebuilt binary) and document the Docker/Telegram run commands. | "all other interfaces" is broad; minimum = CLI binary + documented Docker/Telegram. |
| R5.4 | Installer is safe and robust. | Wrap body in a `main` function (truncation-safe), HTTPS-only, no silent `sudo`, optional SHA-256 verification, `--help`/target selection. | Follows rustup/Homebrew/Bun conventions. |

## R6 — Landing pages for Telegram bot and CLI

> "We also need to add pages in landing for telegram bot and CLI."

| ID | Statement | Acceptance criteria | Notes |
|---|---|---|---|
| R6 | A dedicated Telegram bot landing page at `/telegram/`. | Page covers BotFather token setup, the `docker compose up` + `TELEGRAM_BOT_TOKEN` run path, the `cargo run -- telegram` path, and the Desktop **Services** one-click. | Reuse `README.md` Telegram content + `core.telegram.org/bots/tutorial`. |
| R6.1 | A dedicated CLI landing page at `/cli/`. | Page covers install (universal installer / `cargo install`), `formal-ai chat`, `serve`, `memory`, and links to agentic-tool config (Codex/Claude Code/OpenCode). | Reuse `README.md` Quick Start + Agentic sections. |
| R6.2 | Both pages are reachable from the site root chooser and localized. | Site root lists VS Code, Telegram, CLI (and existing app/docs/download) cards in en/ru/zh/hi. | Extend `landing.js` `destinations`. |

## R7 — Case study (this deliverable)

> "We need to collect data … compile that data to `./docs/case-studies/issue-{id}`
> … deep case study analysis … list of each and all requirements … propose
> possible solutions and solution plans … check known existing
> components/libraries."

| ID | Statement | Acceptance criteria | Notes |
|---|---|---|---|
| R7 | Compile repo + online data under `docs/case-studies/issue-554/`. | `raw-data/` holds issue/PR JSON, online research, and the interface inventory. | Done in this PR. |
| R7.1 | Enumerate every requirement. | This file. | — |
| R7.2 | Propose solutions/plans per requirement, citing existing components/libraries. | `solution-plan.md`. | — |
| R7.3 | Synthesis README listing source material and evidence. | `README.md`. | — |

## Non-goals / out of scope for this analysis slice

- Publishing to the official VS Code Marketplace or Open VSX (the issue
  explicitly wants manual install "as we are not yet published to official store").
- Code signing/notarizing the `.vsix` (desktop signing already exists; `.vsix`
  signing is not required for `code --install-extension`).
- Rewriting the existing `/download/` desktop page; new pages reuse its patterns.
- Shipping the actual page/installer/CI code — this case study is analysis +
  plan only (PR #555 may implement separately).
</content>
