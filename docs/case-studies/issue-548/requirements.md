# Requirements

| ID | Requirement | Source | Status |
| --- | --- | --- | --- |
| R548-1 | The desktop app supports updates without requiring a manual reinstall. | Issue #548 | Implemented with `electron-updater` and Electron Builder update metadata. |
| R548-2 | The user is notified inside the app when a new version is released. | Issue #548 | Implemented with main-process updater events bridged to renderer state. |
| R548-3 | Pressing Update performs the actual update. | Issue #548 | Implemented with `formalAiDesktop:installUpdate`, `downloadUpdate`, and `quitAndInstall`. |
| R548-4 | Auto update supports macOS, Linux, and Windows desktop builds. | Issue #548 | Implemented for Electron Builder's auto-updatable targets: macOS DMG/ZIP metadata, Linux AppImage metadata, and Windows NSIS metadata. |
| R548-5 | The desktop version display must not show `vdev`. | Issue #548 | Implemented by passing `app.getVersion()` through the desktop bridge and preferring it in the renderer. |
| R548-6 | Release assets include the update feed metadata required by updater clients. | Derived from Electron Builder docs | Implemented by collecting/uploading `latest*.yml` and `*.blockmap` files. |
| R548-7 | Existing release self-healing must not skip releases that lack updater metadata. | Derived from existing issue #479 release resolver behavior | Implemented by adding updater metadata to `expected_desktop_assets()`. |
| R548-8 | The issue solution includes a deep case study under `docs/case-studies/issue-548`. | Issue #548 | Implemented in this directory. |

## Non-Goals

- Silent background installation. The issue asks for a user-triggered update action, so the implementation keeps downloads explicit through the in-app Update button.
- Replacing the existing desktop release matrix. The fix preserves the current CI shape and extends the artifacts it uploads.
- Adding a new update server. GitHub Releases are already the project's desktop release distribution point, and Electron Builder supports that provider directly.
