# macOS Gatekeeper screenshots

These three PNGs are embedded on the `/download` page (`src/web/download/download.js`,
`MACOS_GATEKEEPER_SHOTS`) to show what macOS 15 (Sequoia) Gatekeeper looks like the
first time you open an ad-hoc-signed `formal-ai Desktop` build.

| File | macOS dialog | Maps to System Settings step |
| --- | --- | --- |
| `macos-gatekeeper-not-opened.png` | *"… Not Opened"* warning (**Done** / **Move to Trash**) | Step 1 — double-click, click **Done** |
| `macos-gatekeeper-open-anyway.png` | **System Settings → Privacy & Security** with the **Open Anyway** button | Step 2 — open Privacy & Security |
| `macos-gatekeeper-confirm.png` | *"Open … ?"* confirmation (**Open Anyway** / **Move to Trash** / **Done**) | Step 3 — click **Open Anyway**, confirm |

## Provenance — these are real captures, not synthetic renders

Issue [#479](https://github.com/link-assistant/formal-ai/issues/479) asked for macOS
screenshots like the ones on <https://konard.github.io/vk-bot-desktop>, and the
maintainer was explicit that drawn/synthetic images are **not** acceptable — the
screenshots must be *"copied from our code"* at
<https://github.com/konard/vk-bot-desktop>.

macOS Gatekeeper cannot be triggered on a hosted macOS CI runner (it only blocks
apps that were quarantined by a real download), so we cannot regenerate these
programmatically in CI. Instead these are the **genuine** macOS Sequoia Gatekeeper
captures from our sibling desktop app **VK Bot Desktop**, which ships with the
**same explicit** `electron-builder` ad-hoc signing hook (`identity: "-"`) when
Apple Developer ID secrets are unavailable. PR #487 copied that method into
formal-ai's `.github/workflows/desktop-release.yml`, including the ad-hoc signer
and pre-upload DMG smoke test. The dialog wording, layout and buttons are
byte-identical for `formal-ai Desktop`; **only the app name shown in the prompt
differs** (`"VK Bot Desktop"` vs `"formal-ai Desktop"`). The localized
`alt`/caption copy in `download.js` says so.

Upstream sources (mirrored into this repo's case study for offline reference):

| This file | Upstream capture (konard/vk-bot-desktop) |
| --- | --- |
| `macos-gatekeeper-not-opened.png` | `issue-31-macos-done.png` |
| `macos-gatekeeper-open-anyway.png` | `issue-31-macos-open-anyway-settings.png` |
| `macos-gatekeeper-confirm.png` | `issue-31-macos-open-anyway-confirm.png` |

A copy of the upstream originals lives at
`docs/case-studies/issue-479/raw-data/vk-bot-desktop-current/macos-screenshots/`.

## How to refresh them

When `formal-ai Desktop` eventually gets its own signed builds (or someone captures
the Gatekeeper flow on a real Mac), drop the new PNGs in here under the same three
filenames and update the `alt`/caption copy + this README. Do **not** re-introduce a
synthetic generator — the maintainer rejected that approach in issue #479.
