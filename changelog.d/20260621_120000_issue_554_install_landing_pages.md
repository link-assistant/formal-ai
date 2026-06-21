---
bump: minor
---

### Added

- **Dedicated install landing pages for every interface (issue #554).** The site
  chooser now links to three new pages rendered by the shared
  `src/web/site-chrome.js`: `/vscode/` for the VS Code extension, `/cli/` for the
  command-line tool, and `/telegram/` for the Telegram bot. Each page carries
  copy-paste install commands (with one-click copy buttons), ordered manual
  steps, and direct links to the raw installer and the latest release.
- **A universal one-line installer** — [`scripts/install.sh`](scripts/install.sh)
  (POSIX `sh`) and [`scripts/install.ps1`](scripts/install.ps1) (PowerShell) —
  that installs the desktop app, the VS Code extension, the CLI, or the Telegram
  bot from a single command (`curl -fsSL …/install.sh | sh -s -- <target>`). The
  VS Code page documents the manual-only ".vsix" flow ("VS Code Extension only"
  mode) while the extension is still off the Marketplace.
- **One-click VS Code extension install from the desktop app.** Settings now
  offers *Install VS Code extension*: the Electron shell
  (`desktop/lib/vscode-install.cjs`) detects an installed `code`/`code-insiders`/
  `codium`/`cursor`/`windsurf` CLI, downloads the published `.vsix` from the
  latest GitHub release, and runs `code --install-extension … --force` — all
  exposed through the `formalAiDesktop:installVsCodeExtension` IPC bridge.
- The desktop release CI now builds and uploads the `formal-ai-vscode-*.vsix`
  asset so the installers and the one-click flow have a release artifact to fetch.

### Changed

- The shared chooser (`src/web/site-chrome.js`) gained a sectioned-content
  renderer (`section-<id>`, `command-<testid>`, `copy-<testid>`) used by the new
  install pages; the existing landing/docs/download pages are unchanged.
- The root landing chooser now surfaces six destinations (web app, docs,
  download, VS Code, CLI, Telegram) instead of three.
