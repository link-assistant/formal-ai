# Online Research

Research date: 2026-06-20

## Electron Builder Auto Update

Source: [Electron Builder Auto Update](https://www.electron.build/docs/features/auto-update/)

Key facts used:

- Electron Builder enables auto updates through `electron-updater`.
- Release builds must include both installer artifacts and update metadata files such as `latest.yml`.
- macOS auto update requires a signed application.
- Electron Builder lists auto-updatable targets for macOS, Linux, and Windows, including macOS DMG, Linux AppImage/DEB/Pacman/RPM, and Windows NSIS.
- The docs recommend using `autoUpdater` from `electron-updater` and warn not to call `setFeedURL`; the publish config generates the feed configuration.
- The docs describe `autoDownload`, progress events, staged rollouts, and signature validation.

Impact on implementation:

- The project already uses Electron Builder, so `electron-updater` is the narrowest change.
- The release workflow must upload `latest.yml`, `latest-mac.yml`, `latest-linux.yml`, and blockmaps.
- The runtime client should live in Electron main, not in the browser renderer.

## Electron Builder Publish

Source: [Electron Builder Publish](https://www.electron.build/docs/publish/)

Key facts used:

- The `publish` key configures where artifacts and update metadata are published.
- GitHub is a supported provider with owner/repo options.
- Publishing can be controlled by the build command, and metadata generation is tied to the publish provider configuration.
- `publishAutoUpdate` defaults to true.

Impact on implementation:

- `desktop/package.json` now declares the GitHub provider under the Electron Builder `build.publish` key.
- The existing workflow can keep manual `gh release upload` control while still producing and uploading update metadata.
