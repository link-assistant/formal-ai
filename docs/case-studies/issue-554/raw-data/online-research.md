# Issue #554 Online Research Notes

Sourced notes for each requirement family. Each bullet is `claim — takeaway (URL)`.
Captured 2026-06-21.

## 1. `curl … | bash` and PowerShell `irm … | iex` installer patterns

- The pattern serves an install script from a **stable HTTPS URL** and pipes it
  to the shell. Transport security (TLS) is the only guarantee; the script is
  not signed or checksummed by the pipe itself, so a stable, auditable URL plus
  a "download-then-inspect" alternative is the accepted mitigation. —
  Provide both `curl … | sh` convenience and a "save the file, read it, then
  run it" path. (https://www.arp242.net/curl-to-sh.html)
- A truncated download can execute a partial script. The mitigation is to
  **wrap the entire script body in a function** that is only invoked on the last
  line, so a truncated transfer never runs a half-script. — Wrap install logic in
  a `main` function called at the very end.
  (https://www.chef.io/blog/5-ways-to-deal-with-the-install-sh-curl-pipe-bash-problem)
- TLS cert validation by `curl`/`Invoke-RestMethod` defends against MITM as long
  as the host is HTTPS with a CA-trusted cert; GitHub raw + Pages both qualify. —
  Serve the script over HTTPS only. (https://www.kicksecure.com/wiki/Dev/curl_bash_pipe)
- **rustup**: canonical `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`;
  downloads over HTTPS, does not yet verify signatures; offers Homebrew
  (`brew install rustup-init`) as an auditable alternative. — Pin `--proto`/`--tlsv1.2`
  and document a non-pipe path. (https://rust-lang.github.io/rustup/installation/other.html,
  https://rust-lang.github.io/rustup/security.html)
- **Homebrew / Deno / Bun / nvm / Starship / Volta / ollama** all ship the same
  shape: one `curl|sh` (or `irm|iex`) line on the homepage plus a documented
  manual download. Bun and Deno publish both `curl -fsSL https://bun.sh/install | bash`
  and `irm bun.sh/install.ps1 | iex`. — A universal installer should ship a
  `.sh` and a `.ps1` at parallel stable URLs. (https://bun.com/docs/pm/cli/install)
- **PowerShell**: `irm <url> | iex` = `Invoke-RestMethod` downloads, `Invoke-Expression`
  evaluates. ollama uses `irm https://ollama.com/install.ps1 | iex`. Note the
  PowerShell-7 `$Args`/`$args` pitfall: a script that reads `$Args` breaks under
  `irm|iex` because there is no arg vector — pass options via env vars instead. —
  Keep the `.ps1` parameterized through env vars, not positional args.
  (https://knowledge.buka.sh/powershell-one-liners-for-installation-what-does-irm-bun-sh-install-ps1-iex-really-do/,
  https://github.com/JuliusBrussee/caveman/issues/381)
- Security critics (Sysdig, Kicksecure) argue `curl|bash` runs unreviewed code as
  the invoking user; the consensus mitigation is **not** to drop the convenience
  line but to (a) keep the script short and readable, (b) host it at a permanent
  versioned URL, (c) offer checksum/signature verification, (d) never require
  `sudo` silently. — Document what the script does and offer SHA256 verification.
  (https://www.sysdig.com/blog/friends-dont-let-friends-curl-bash)

## 2. Installing a VS Code extension from a `.vsix` without the Marketplace

- Official command: `code --install-extension myextension.vsix`. The
  `--install-extension` flag can be passed multiple times to install several
  extensions. There is also **Extensions: Install from VSIX** in the Command
  Palette / Extensions view `…` menu. — Two manual paths: CLI and GUI.
  (https://code.visualstudio.com/docs/configure/extensions/extension-marketplace)
- VSIX-installed extensions have **auto-update disabled by default** (no
  Marketplace backing version to compare against). — Document that updates are
  manual (re-run the installer / re-download the `.vsix`).
  (https://code.visualstudio.com/docs/configure/extensions/extension-marketplace)
- **Web host limitation**: web extensions on `vscode.dev` / `github.dev` run in a
  browser Web Worker and **cannot use Node.js, `child_process`, `fs`, raw sockets,
  or `importScripts`**; sideloading a local `.vsix` into the browser host is not a
  general distribution mechanism — it is documented as "a good final sanity check
  before publishing," done via `npm run serve-and-watch` + `vscode.dev`, not by
  pointing the browser at an arbitrary file. So manual `.vsix` install only
  targets the **desktop** host. (https://code.visualstudio.com/api/extension-guides/web-extensions)
- This matches formal-ai's own dual-host design: `extension.web.cjs` is
  in-process-only and cannot spawn a server. So "install the extension manually"
  is inherently a **desktop-host** instruction.

## 3. Publishing a `.vsix` as a GitHub Release asset / `@vscode/vsce package`

- `@vscode/vsce` (the official "VS Code Extension Manager", npm `@vscode/vsce`) is
  the tool for packaging and publishing. `vsce package` produces a `.vsix`;
  `vsce publish` pushes to the Marketplace. — formal-ai already depends on
  `@vscode/vsce@^3.9.2` and has `"package": "vsce package"`.
  (https://github.com/microsoft/vscode-vsce,
  https://code.visualstudio.com/api/working-with-extensions/publishing-extension)
- "Some extensions publish `.vsix` files as a part of their GitHub releases." You
  can disable Marketplace publish and only attach the `.vsix` as a release asset.
  — Exactly the path issue #554 wants (not yet on the official store).
  (https://code.visualstudio.com/api/working-with-extensions/publishing-extension)
- `semantic-release-vsce` automates package + publish + GitHub-asset attach, and
  publishes to Open VSX when `OVSX_PAT` is set / Marketplace when `VSCE_PAT` is
  set. — A future option once a store is targeted; for now a plain CI step that
  runs `vsce package` then `gh release upload` matches the existing
  desktop-release upload mechanism. (https://github.com/felipecrs/semantic-release-vsce)

## 4. "Open in VS Code" / one-click install deep links + Electron shelling out

- VS Code registers the `vscode://` (and `vscode-insiders://`) URI protocol at
  install time ("Deep Links"). Extensions register `window.registerUriHandler()`
  with a `handleUri()` method. — The protocol is OS-registered only when desktop
  VS Code is installed. (https://github.com/microsoft/vscode-extension-samples/blob/main/uri-handler-sample/README.md)
- Marketplace install deep link form: `vscode:extension/<publisher>.<name>` (e.g.
  `vscode:extension/ms-python.python`). **Caveat**: this opens the extension's
  Marketplace page inside VS Code; since formal-ai is **not published to the
  Marketplace**, this link cannot install it yet — and the known bug is that the
  URL does nothing unless VS Code is already running.
  (https://github.com/Microsoft/vscode/issues/20289) — So for now the reliable
  one-click path is **shelling out to the `code` CLI**, not the `vscode:` URL.
- A desktop (Electron) app can run the bundled-or-PATH `code` CLI:
  `code --install-extension <downloaded.vsix>`. This is the most robust
  "one-click from the Desktop app" because it does not depend on Marketplace
  presence or a registered URI handler. — Recommended approach for R2 (one-click
  from Desktop). (https://code.visualstudio.com/docs/configure/extensions/extension-marketplace)
- Security note: argument-injection research on VS Code URI/`code` argument
  handling means any shelled-out `code` call must use a fixed argument vector and
  a validated, app-controlled `.vsix` path, never user-interpolated strings. —
  Pass a hardcoded flag + a path the app itself downloaded.
  (https://www.sonarsource.com/blog/securing-developer-tools-argument-injection-in-vscode/)

## 5. Landing-page patterns for CLI tools and chat bots

- CLI tools document a one-line install plus a package-manager path: `cargo install`,
  the `curl|sh`/`irm|iex` installers, and Docker `docker run`. formal-ai already
  has all three in `README.md` (cargo, Docker compose, Telegram). The landing
  pages should surface, not reinvent, those commands with a copy-button.
- Telegram bot onboarding is standardized around **@BotFather**: `/newbot`, pick a
  name + a `…bot` username, receive the `7123…:AAH…` token, store it as a secret,
  `/revoke` to rotate. — A Telegram landing page should walk through BotFather,
  then show formal-ai's two run modes (`docker compose up` with
  `TELEGRAM_BOT_TOKEN`, or `cargo run -- telegram`).
  (https://core.telegram.org/bots/tutorial)
- Token must be treated as a password; never commit it; pass via env/secret. —
  The page should reuse the README's `TELEGRAM_BOT_TOKEN` env pattern and warn
  against committing it. (https://core.telegram.org/bots/tutorial)

## Cross-cutting takeaways for this repo

- A universal installer must ship **two parallel artifacts** (`install.sh` +
  `install.ps1`) at **stable raw URLs**. The most stable raw URLs in this repo
  are `raw.githubusercontent.com/link-assistant/formal-ai/main/scripts/install.sh`
  (pinned to a tag for reproducibility) and the GitHub Pages copy under
  `https://link-assistant.github.io/formal-ai/install.sh`.
- The installer should reuse the **existing release asset naming**
  (`formal-ai-desktop-<os>-<arch>-<version>.<ext>`) and the GitHub Releases API
  the download page already calls, plus `cargo install formal-ai` for the CLI and
  `code --install-extension` for the `.vsix` once CI publishes it.
- "VS Code Extension only mode" = install just the `.vsix` against desktop VS Code
  without the Desktop app — i.e. download the release `.vsix` and run
  `code --install-extension`.
</content>
</invoke>
