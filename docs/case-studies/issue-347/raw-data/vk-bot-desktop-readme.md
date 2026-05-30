# vk-bot-desktop

Cross-platform desktop app that wraps [`konard/vk-bot`](https://github.com/konard/vk-bot) in a React + Electron UI. Runs the bot locally or on a remote SSH host inside Docker or `screen`.

![VK Bot Desktop landing page in English dark theme](docs/screenshots/issue-26-pages-en-dark.png)

## Features

- **Local mode**: run the bot inside the desktop app, no Docker required, dependencies installed automatically on first launch.
- **Server mode**: SSH into a remote host and run the bot under [`link-foundation/start`](https://github.com/link-foundation/start)'s `$ --isolated docker` or `$ --isolated screen` wrapper.
- **Bot behaviours** (all enabled by default and individually toggleable):
  - keep online status while running,
  - auto-accept friend requests using top-10% mutuals below 10 000 friends and mutuals-only above,
  - delete deactivated/blocked friends,
  - cancel outgoing friend requests,
  - post invitation messages to selected communities with your avatar (default text: «Приму заявки в друзья.»),
  - send randomized birthday greetings (10 short messages, ≤ 2 emojis each).
- **Priority friend list**: always send a request to listed users; never delete them automatically.
- **Single window UI** with mode switch, Start/Stop, light/dark/auto theme, en/ru auto-detected language.
- **Configuration** in human-readable [Links Notation](https://github.com/link-foundation/lino-objects-codec) (no JSON, no type markers), layered: local `./.vk-bot-desktop/config.lino` overrides global `~/.vk-bot-desktop/config.lino`.
- **CLI** options via [`lino-arguments`](https://github.com/link-foundation/lino-arguments): `--token`, `--mode`, `--config`.
- **Verbose logs** in the UI with tokens, passwords, cookies redacted everywhere.

## Install

Download the desktop binary for your OS from the latest
[GitHub release](https://github.com/konard/vk-bot-desktop/releases) or the
[download page](https://konard.github.io/vk-bot-desktop/). Release filenames
include the version so cached installers stay unambiguous.

| Platform            | Artifact examples                                      |
| ------------------- | ------------------------------------------------------ |
| macOS Apple silicon | `vk-bot-desktop-macos-arm64-0.9.9.dmg`, `.zip`         |
| macOS Intel         | `vk-bot-desktop-macos-x64-0.9.9.dmg`, `.zip`           |
| Windows x64         | `vk-bot-desktop-windows-installer-x64-0.9.9.exe`       |
| Windows arm64       | `vk-bot-desktop-windows-installer-arm64-0.9.9.exe`     |
| Linux x64           | `vk-bot-desktop-linux-x64-0.9.9.AppImage`, `.deb`      |
| Linux arm64         | `vk-bot-desktop-linux-arm64-0.9.9.AppImage`, `.deb`    |
| Verification        | `SHA256SUMS.txt`, `BUILD-PROVENANCE.txt`, attestations |

Verify the SHA-256 checksum against `SHA256SUMS.txt` from the same release:

```sh
sha256sum -c SHA256SUMS.txt
```

On Windows, PowerShell can compute the same SHA-256 value:

```powershell
Get-FileHash .\vk-bot-desktop-windows-installer-x64-0.9.9.exe -Algorithm SHA256
```

For stronger supply-chain checks, inspect `BUILD-PROVENANCE.txt` and verify
GitHub artifact attestations when they are attached to the release:

```sh
gh attestation verify ./vk-bot-desktop-linux-x64-0.9.9.AppImage --repo konard/vk-bot-desktop
```

The download page is published from this repository with GitHub Pages after
changes to `site/` are merged to `main`. It reads GitHub's latest Release API
and only renders direct download buttons for assets that are attached to that
release. If the API is unavailable, it opens the release page instead of
guessing binary URLs.

### Open the app on macOS

macOS releases are ad-hoc signed without an Apple Developer ID, so Gatekeeper
blocks the first launch with `"VK Bot Desktop" Not Opened — Apple could not
verify "VK Bot Desktop" is free of malware...`. Verify the SHA-256 checksum
first, then use either of the workflows below to allow the app once. These
steps only need to be done one time per install; subsequent launches do not
show the warning.

**Terminal one-liner.** Drag `VK Bot Desktop.app` to `/Applications`, then
remove the quarantine attribute:

```sh
sudo xattr -dr com.apple.quarantine "/Applications/VK Bot Desktop.app"
```

**System Settings (macOS 15 Sequoia and later).** On Sequoia, Apple removed
the Control-click → Open bypass, so the flow is:

1. Double-click `VK Bot Desktop`, then click **Done** when the
   "Apple could not verify…" dialog appears.
   ![macOS warning dialog with Done button](docs/screenshots/issue-31-macos-done.png)
2. Open **System Settings → Privacy & Security** and scroll to the **Security**
   section.
3. Click **Open Anyway** next to `VK Bot Desktop`, confirm, and authenticate
   with Touch ID or your admin password.
   ![macOS Privacy & Security settings with Open Anyway button](docs/screenshots/issue-31-macos-open-anyway-settings.png)
   ![macOS confirmation dialog with Open Anyway button](docs/screenshots/issue-31-macos-open-anyway-confirm.png)

Only run these steps for VK Bot Desktop release artifacts whose SHA-256 matches
`SHA256SUMS.txt` from the same GitHub release. The same workflow applies to
the `.zip` archive after expanding `VK Bot Desktop.app`.

## Develop

```sh
npm install
npm run build:renderer
npm run electron:dev
```

Build distributable artifacts:

```sh
npm run electron:build           # current OS
npm run electron:build:linux
npm run electron:build:mac
npm run electron:build:win
```

Run the headless bot directly:

```sh
node src/cli.mjs --token "$VK_TOKEN"
```

## Tests

```sh
npm test
```

## Case study

A detailed walk-through of the design decisions, library choices, and reproducibility steps is in
[`docs/case-studies/issue-1`](docs/case-studies/issue-1/README.md).

The full requirements are maintained in [`docs/REQUIREMENTS.md`](docs/REQUIREMENTS.md).
