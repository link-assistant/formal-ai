# Issue 548 Case Study: Desktop Auto Update

## Summary

Issue: [link-assistant/formal-ai#548](https://github.com/link-assistant/formal-ai/issues/548)

Pull request: [link-assistant/formal-ai#549](https://github.com/link-assistant/formal-ai/pull/549)

The desktop app needed a real auto-update path for macOS, Linux, and Windows, an in-app update notification/action, and a fix for the desktop version badge rendering `vdev`. The implemented solution uses Electron Builder's release metadata plus `electron-updater` in the packaged Electron main process. The renderer now prefers the Electron app version from the desktop bridge, subscribes to updater status events, and exposes Check/Update controls in the desktop sidebar.

## Inputs Reviewed

- Issue body and requirements: `raw-data/issue-548.json`
- Issue comments: `raw-data/issue-548-comments.json`
- Existing PR details and placeholder state: `raw-data/pr-549.json`
- PR conversation, review, and inline comment snapshots: `raw-data/pr-549-comments.json`, `raw-data/pr-549-reviews.json`, `raw-data/pr-549-review-comments.json`
- Official upstream docs: [Electron Builder Auto Update](https://www.electron.build/docs/features/auto-update/) and [Electron Builder Publish](https://www.electron.build/docs/publish/)

## Selected Design

1. Package-time update feed:
   - Add `electron-updater` and Electron Builder `publish` metadata for the GitHub release provider.
   - Keep macOS `zip`, Windows `nsis`, and Linux `AppImage` targets in the release build because those are the Electron Builder auto-update targets.
   - Upload `latest.yml`, `latest-mac.yml`, `latest-linux.yml`, and `*.blockmap` files to releases instead of filtering them out.

2. Runtime update flow:
   - Main process owns a testable `createAutoUpdateController`.
   - Packaged apps check for updates on startup and on user request.
   - `autoDownload` is disabled so the renderer can show an explicit in-app update action.
   - Pressing Update downloads the update if needed and then calls `quitAndInstall(false, true)`.

3. Renderer flow:
   - `getStatus()` now includes `appVersion` and `updater`.
   - Preload exposes `checkForUpdates`, `installUpdate`, and `onUpdateStatus`.
   - The sidebar shows current version, updater status, progress, Check, and Update.
   - The topbar/drawer version label uses Electron's app version when the desktop bridge is present, fixing `vdev`.

## Verification Added

- Desktop unit tests for the auto-update controller state machine.
- Playwright regression tests for desktop version display and update-available in-app notification.
- CI/release tests for uploading updater metadata and requiring metadata before the desktop release resolver skips a build.
- Normalizer regression coverage so Linux `latest-linux.yml` points at the normalized `linux-x64` artifact names.

See [requirements.md](requirements.md), [solution-plans.md](solution-plans.md), and [raw-data/online-research.md](raw-data/online-research.md) for the detailed trace.
