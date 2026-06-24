# Issue #554 User-Friendly Interface Distribution Case Study

## Source Material

- Issue: https://github.com/link-assistant/formal-ai/issues/554
- PR: https://github.com/link-assistant/formal-ai/pull/555
- Repository interfaces overview: [README.md](../../../README.md)
- VS Code extension: [vscode/README.md](../../../vscode/README.md),
  [vscode/package.json](../../../vscode/package.json),
  [docs/vscode/extension.md](../../vscode/extension.md)
- Web landing/download architecture: [src/web/landing.js](../../../src/web/landing.js),
  [src/web/site-chrome.js](../../../src/web/site-chrome.js),
  [src/web/download/download.js](../../../src/web/download/download.js),
  [src/web/download/index.html](../../../src/web/download/index.html)
- Distribution/CI: [.github/workflows/release.yml](../../../.github/workflows/release.yml),
  [.github/workflows/desktop-release.yml](../../../.github/workflows/desktop-release.yml)
- Prior case-study structure reference: [docs/case-studies/issue-552](../issue-552/README.md),
  [docs/case-studies/issue-451](../issue-451/README.md)
- Raw GitHub exports, online research notes, and the interface inventory are saved
  in `raw-data/`.

## Summary

Issue #554 asks for **friendlier distribution of every formal-ai interface**, with
the VS Code extension as the headline gap. Today the extension is the only
shipped surface with **no landing page, no downloadable artifact, and no
one-click path** — its `.vsix` is not built or uploaded anywhere in CI
(confirmed against `.github/workflows/`; `docs/vscode/extension.md` says so
plainly). The Desktop app, by contrast, already has a polished `/download/` page
and a one-click Docker **Services** panel.

The issue decomposes into seven requirement families (see `requirements.md`):
a `/vscode/` landing page with manual "extension only mode" install (R1); a
one-click install of the extension from the Desktop app (R2); a `curl/wget … .sh
| bash` install with a direct raw-GitHub link (R3); an implied prerequisite to
**build & publish the `.vsix`** (R4); a universal `.sh` + PowerShell installer
covering Desktop, VS Code, and the other interfaces (R5); `/telegram/` and
`/cli/` landing pages (R6); and this case study (R7).

The recommended solution reuses what the repo already proves: the
`site-chrome.js` chooser pattern and the `download.js` release-asset resolver for
new pages; the existing `@vscode/vsce` dependency + `gh release upload` +
`SHA256SUMS` flow to publish the `.vsix`; and the desktop's "shell out to a local
tool" model to run `code --install-extension` for the one-click path. The
universal installer follows rustup/Bun/ollama conventions (`curl|sh` + `irm|iex`,
served from stable raw URLs, truncation-safe, HTTPS-only).

## Requirements at a glance

| ID | Requirement | Recommended approach |
|---|---|---|
| R1 | `/vscode/` page + manual "extension only mode" install | Chooser page; resolve `.vsix` via Releases API; show `code --install-extension` CLI+GUI; explain web-host no-sideload |
| R2 | One-click install from the Desktop app | Shell out to `code --install-extension <downloaded .vsix>` (not the `vscode:` deep link) |
| R3 | `curl/wget … .sh \| bash` with direct raw link | Universal installer `vscode` target; raw `raw.githubusercontent.com/.../scripts/install.sh` + Pages mirror |
| R4 | Build & publish the `.vsix` (implied prerequisite) | CI step in `desktop-release.yml finalize`: `vsce package` → `gh release upload --clobber` + checksum |
| R5 | Universal `.sh` + PowerShell installer | `scripts/install.sh` + `scripts/install.ps1`, targets `desktop\|vscode\|cli\|all`, env-var config |
| R6 | `/telegram/` and `/cli/` landing pages | Two more chooser pages reusing README content + BotFather flow |
| R7 | Case study | This directory |

## Key findings from research

- **`curl\|bash` / `irm\|iex` are the accepted distribution norm** (rustup,
  Homebrew, Deno, Bun, nvm, Starship, Volta, ollama) when served over HTTPS from a
  stable URL, with the script body wrapped in a `main` function (truncation-safe)
  and a documented inspect-then-run alternative. PowerShell scripts must be
  parameterized via env vars, not `$Args`, to survive `irm … | iex`.
- **A `.vsix` installs manually with `code --install-extension myextension.vsix`**
  (or **Extensions: Install from VSIX**); VSIX installs disable auto-update. The
  **web host (`vscode.dev`/`github.dev`) cannot sideload** — no Node, no
  `child_process`, no `fs`, no `importScripts` — so manual install targets the
  **desktop** host only. This matches formal-ai's own dual-host design.
- **`@vscode/vsce package` produces the `.vsix`** and it can be attached as a
  **GitHub Release asset** (Marketplace publish disabled), which is exactly the
  "not yet on the official store" path the issue wants. formal-ai already depends
  on `@vscode/vsce@^3.9.2`.
- **The `vscode:extension/<publisher>.<name>` deep link opens the Marketplace
  page inside VS Code** and is a no-op unless VS Code is running and the extension
  is on the Marketplace — so the reliable Desktop one-click is **shelling out to
  the `code` CLI**, with a fixed argument vector to avoid argument injection.
- **Telegram onboarding standardizes on @BotFather** (`/newbot` → token →
  `/revoke`); the landing page should reuse the README's `TELEGRAM_BOT_TOKEN`
  env pattern and the Desktop **Services** one-click.

Full sourcing with URLs is in `raw-data/online-research.md`.

## Proposed solution at a glance

1. **R4 first** — add a `.vsix` build+upload to `desktop-release.yml`'s
   `finalize` job; it unblocks every download/install path.
2. **R5** — author `scripts/install.sh` + `scripts/install.ps1` with
   `desktop|vscode|cli|all` targets, served from raw GitHub + a Pages mirror.
3. **R1 / R6** — three new chooser pages (`src/web/vscode/`, `src/web/telegram/`,
   `src/web/cli/`) reusing `site-chrome.js` + the `download.js` resolver, wired
   into the `landing.js` chooser, localized en/ru/zh/hi.
4. **R2** — a testable `desktop/lib/vscode-install.cjs` + IPC handler that
   downloads the `.vsix` and runs `code --install-extension`, disabled-with-note
   when `code` is absent (mirroring the Docker-unavailable Services UX).

CI can verify the `.vsix` build/upload, page rendering, installer static-analysis,
and the desktop helper's pure functions; it **cannot** verify real
`code`/`cargo`/`curl|bash` execution (no VS Code/clean-machine in the runner) —
those stay manually verified, consistent with the repo's existing honesty about
un-CI-able paths.

## Evidence

- `raw-data/issue-554.json` and `raw-data/issue-554-comments.json`: original issue
  (no comments) and its labels.
- `raw-data/pr-555.json`, `raw-data/pr-555-conversation-comments.json`,
  `raw-data/pr-555-review-comments.json`: the WIP PR state at investigation start
  (no comments/reviews yet).
- `raw-data/merged-prs-landing.json`: prior merged PRs that built the
  landing/download/site structure (#480, #486, #510) for pattern reference.
- `raw-data/repo-enhancement-issues.json`: the open/closed issue corpus used to
  locate related work (e.g. #347 `/download` page, #353 VS Code extension, #438
  one-line Telegram Docker, #423 README↔sh/ps1 conversion, #548 desktop
  auto-update).
- `raw-data/related-issues-vscode.json`, `raw-data/related-issues-installer.json`:
  targeted related-issue queries (empty result sets — no prior dedicated VS Code
  packaging or installer issue beyond #554).
- `raw-data/online-research.md`: sourced notes (with URLs) on `curl|bash`/`irm|iex`
  patterns, VSIX/`code --install-extension`, `vsce` GitHub-release publishing,
  `vscode:` deep links + Electron shelling out, and CLI/Telegram landing
  conventions.
- `raw-data/interface-inventory.md`: every shipped interface, how it is
  distributed/installed today, and its current landing/docs coverage, cited to
  repo paths.
- `requirements.md` and `solution-plan.md`: the requirement breakdown and the
  file-level implementation plan.

## Dependency / follow-up findings

- The `.vsix` build is the single blocking prerequisite; everything user-facing
  (download links, one-click, installer `vscode` target) depends on it.
- A short stable installer URL benefits from a GitHub Pages mirror of `scripts/`,
  added to the `deploy-demo` job — a small, low-risk change.
- Desktop auto-update (#548, closed) and the README↔sh/ps1 conversion work (#423,
  closed) are adjacent; the universal installer should not duplicate them but can
  reuse their conventions.
</content>
