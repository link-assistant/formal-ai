# Solution Plans

## Plan A: Electron Builder + electron-updater on GitHub Releases

Use Electron Builder's built-in update feed generation and `electron-updater` in the Electron main process.

Advantages:
- Matches the existing `electron-builder` packaging stack.
- Supports macOS, Linux, and Windows from one maintained client library.
- Uses the existing GitHub Release distribution channel.
- Keeps update logic in the main process and exposes a small IPC surface to the renderer.

Risks and mitigations:
- macOS updates require signed apps in production. The release workflow already has macOS signing/notarization paths and an ad-hoc fallback for direct testing; production updater reliability depends on the signed path.
- Update metadata can be accidentally omitted. The workflow and resolver tests now guard `latest*.yml` and blockmap handling.
- Linux x64 artifact aliases differ between Electron Builder and the download contract. The artifact normalizer now also rewrites `latest-linux.yml`.

Decision: selected.

## Plan B: Custom GitHub Release Polling + Manual Installer Download

Poll the GitHub Releases API in the renderer or main process, compare semantic versions, and open the release download URL.

Advantages:
- Smaller runtime dependency footprint.
- Full control over release selection and UI copy.

Disadvantages:
- Does not satisfy the "without reinstalling" requirement for a real app update flow.
- Requires custom per-platform installer handling.
- Reimplements signature, metadata, and differential update logic already maintained by Electron Builder.

Decision: rejected.

## Plan C: Vendor-Specific Updater Services

Use a separate update hosting service such as a hosted Electron update server.

Advantages:
- Can centralize update channels and staged rollouts.

Disadvantages:
- Adds new infrastructure and credentials.
- The project already publishes installers to GitHub Releases.
- No requirement called for staged deployments or a private update service.

Decision: rejected.

## Implementation Notes

- Main process:
  - `desktop/lib/auto-update.cjs` owns status transitions and is covered by `node:test`.
  - `desktop/main.cjs` starts a packaged-app check, updates `desktopStatus`, and emits `formalAiDesktop:updateStatus`.
  - `desktop/preload.cjs` exposes `checkForUpdates`, `installUpdate`, and `onUpdateStatus`.

- Renderer:
  - `src/web/app.js` normalizes desktop `appVersion` and updater state.
  - The sidebar has a visible Updates row with state, progress, Check, and Update.
  - Existing browser, VS Code, and older desktop mocks continue to work because updater fields and methods are optional.

- Release:
  - `.github/workflows/desktop-release.yml` no longer filters out updater metadata.
  - `scripts/desktop-release-resolve.sh` requires update metadata before skipping automatic desktop builds.
  - `desktop/scripts/normalize-artifacts.mjs` rewrites Linux metadata references after filename normalization.
