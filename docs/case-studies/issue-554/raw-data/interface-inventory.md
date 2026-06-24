# formal-ai Interface Inventory (issue #554)

Every interface formal-ai ships today, how it is distributed, how a user installs
it, and its current landing/docs coverage. Derived strictly from the repository;
file paths are cited inline.

## Summary table

| Interface | Distribution today | Install today | Landing page? | Docs coverage |
|---|---|---|---|---|
| Library crate | crates.io (`cargo publish` in `.github/workflows/release.yml`) | `cargo add formal-ai` / dep in `Cargo.toml` | No | `README.md` "Rust Library"; `docs/api/` (cargo doc) |
| CLI chat | Source build / crate | `cargo run -- chat …` / `cargo install` | **No** | `README.md` Quick Start |
| HTTP API server | Source build / crate / Docker | `cargo run -- serve …`; `docker compose --profile server up` | No | `README.md` "Agentic AI Tools" |
| Telegram bot | Docker image `ghcr.io/link-assistant/formal-ai:latest`; crate CLI | `docker compose up` with `TELEGRAM_BOT_TOKEN`; `cargo run -- telegram` | **No** | `README.md` "Telegram Bot" |
| Docker image | GHCR (built in `release.yml`) | `docker run … ghcr.io/link-assistant/formal-ai:latest` | No | `README.md` Docker section |
| Web app (Pages) | GitHub Pages `/formal-ai/` (deploy in `release.yml`) | Open in browser, nothing to install | Yes (`/app/`) | Linked from landing |
| Desktop app | GitHub Release assets (`desktop-release.yml`) | `/download/` page → OS asset | **Yes** (`/download/`) | `README.md` "Desktop app"; `docs/desktop/service-control.md` |
| VS Code extension | **Nowhere — `.vsix` not built/published in CI** | Manual `npm run vscode:package` only | **No** | `vscode/README.md`; `docs/vscode/extension.md` |

## Per-interface detail

### Library crate (`formal-ai`)
- Published to crates.io by the `auto-release` job in
  `.github/workflows/release.yml` (`scripts/publish-crate.rs`), version from
  `Cargo.toml` driven by `changelog.d/` fragments.
- Install: add as a dependency; usage example in `README.md` "Rust Library".
- Docs: cargo-doc API reference copied to `src/web/docs/api/` during the
  `deploy-demo` job, surfaced at `/docs/`.

### CLI chat (`formal-ai chat`, `dataset`, `memory …`, `telegram`, `serve`)
- Same crate binary; no standalone published binary beyond what the Desktop
  release bundles. Distributed via source build (`cargo run --`) today.
- Install: `cargo run -- chat --prompt "Hi"` (`README.md` Quick Start). A
  `cargo install formal-ai` path exists implicitly via the published crate but is
  not documented as an install flow.
- Landing/docs: only `README.md`. **No dedicated landing page.**

### HTTP API server (`formal-ai serve`)
- OpenAI Chat Completions / Responses / Anthropic Messages adapters; bound to
  loopback in examples. Runnable from the crate or the Docker `server` profile.
- Install/run: `cargo run -- serve --host 127.0.0.1 --port 8080`, or
  `docker compose --profile server up -d` (`README.md`).
- Landing/docs: `README.md` "Agentic AI Tools". No dedicated page.

### Telegram bot
- Distributed as the DinD image `ghcr.io/link-assistant/formal-ai:latest`
  (default `CMD` = `formal-ai telegram --mode polling`) and via the crate CLI.
- Install/run: `TELEGRAM_BOT_TOKEN=… docker compose up`, or
  `cargo run -- telegram`. Token comes from BotFather (env/`.lenv`/`.env` via
  `lino-arguments`).
- Landing/docs: `README.md` "Telegram Bot". **No dedicated landing page.**
- The Desktop app already has a one-click **Services** panel that starts the
  Telegram container (`desktop/lib/service-control.cjs`,
  `docs/desktop/service-control.md`).

### Docker image
- Built and pushed to GHCR (and optional Docker Hub mirror) in `release.yml`.
  Root `compose.yaml` profiles: default (telegram), `server`, `agent`, `all`.
- Install/run: `docker run --privileged … ghcr.io/link-assistant/formal-ai:latest`.
- Landing/docs: `README.md`. No dedicated page.

### Web app (GitHub Pages)
- Deployed by the `deploy-demo` job in `release.yml` from `src/web/` to
  `https://link-assistant.github.io/formal-ai/` (base path `/formal-ai/`).
  `bun run build:web` bundles vendor/app; `scripts/stamp-pages-artifact.sh`
  fills `__FORMAL_AI_VERSION__` / `__FORMAL_AI_ASSET_VERSION__`.
- Install: none — open in browser.
- Landing/docs: the site root (`src/web/index.html` + `landing.js`) is the
  chooser; `/app/` is the demo, `/docs/` the docs hub, `/download/` the desktop
  download page.

### Desktop app (Electron)
- Built per-OS/arch by `.github/workflows/desktop-release.yml` and uploaded to the
  GitHub Release with `gh release upload … --clobber`. Asset naming:
  `formal-ai-desktop-<os>-<arch>-<version>.<ext>` (linux AppImage/deb/tar.gz,
  macOS dmg/zip, windows installer/portable exe). `SHA256SUMS.txt` +
  `BUILD-PROVENANCE.txt` are attached by the `finalize` job; SLSA attestation via
  `actions/attest-build-provenance`.
- Install: the `/download/` page (`src/web/download/download.js` +
  `index.html`) calls `https://api.github.com/repos/link-assistant/formal-ai/releases/latest`,
  resolves the asset for the detected OS/arch, and offers client-side SHA-256
  verification. CSP allows `connect-src https://api.github.com` only.
- Landing/docs: dedicated `/download/` page; `README.md` "Desktop app";
  `docs/desktop/service-control.md`.

### VS Code extension (`formal-ai-vscode`)
- Dual host from one manifest (`vscode/package.json`): `extension.node.cjs`
  (desktop/remote) and `extension.web.cjs` (vscode.dev/github.dev, in-process
  only). Reuses the committed `src/web/` chat UI in a Webview
  (`docs/vscode/extension.md`).
- Packaging: `npm run vscode:package` → `vsce package` (`@vscode/vsce@^3.9.2`,
  `prepare-resources.mjs` mirrors web + seed + desktop libs into `dist-web/`).
- **Gap**: the `.vsix` is **NOT built or uploaded anywhere in CI** — no `vsce`,
  `.vsix`, or `vscode:package` reference exists in `.github/workflows/`. The
  extension is not on the Marketplace or Open VSX. `docs/vscode/extension.md`
  states plainly: "Marketplace publishing is not automated … there is no release
  workflow that publishes it yet."
- Install today: only by cloning and running `npm run vscode:package`, then
  `code --install-extension <file>.vsix`. **No landing page, no downloadable
  `.vsix`, no one-click path.**

## Distribution mechanics referenced by the solution

- GitHub Pages base path: `/formal-ai/` (`desktop/package.json` homepage,
  `src/web/app/index.html` `<base href="../">`).
- Release upload mechanism: `gh release upload $TAG … --clobber`
  (`desktop-release.yml` `build`/`finalize` jobs).
- Page version stamping: `scripts/stamp-pages-artifact.sh` replaces
  `__FORMAL_AI_VERSION__` and `__FORMAL_AI_ASSET_VERSION__`.
- Shared page chrome / theme / locale: `src/web/site-chrome.js`
  (`createChooser`), `src/web/preferences.js` (`formal-ai.preferences.v1`
  localStorage key), locales `en/ru/zh/hi`.
- Existing chooser config example: `src/web/landing.js` (three destination cards
  → `app/`, `docs/`, `download/`).
- Existing release-asset-resolving page: `src/web/download/download.js`
  (`assetNameFor`, `candidateAssetNames`, `resolveDownloadAsset`,
  GitHub Releases API).
</content>
