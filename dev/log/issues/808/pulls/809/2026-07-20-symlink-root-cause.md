# Issue #808 / PR #809 — macOS packaging failure, iteration 3

## Evidence
- `cicd-29733397142.log` — Lint job, "Run desktop packaging regression tests":
  `not ok 3 ... Cannot find module '@electron/osx-sign/dist/cjs/sign.js'`.
  The lint job does not install `desktop/node_modules`, so the upstream
  cross-check added in the previous iteration cannot resolve the module there.
- `desktop-29733397144.log` — Build macos-x64 / macos-arm64:
  the browser-runtime exclusion now works (no more "unsealed contents"), and
  signing succeeds. The build dies later, in
  `codesign --verify --deep --strict --verbose=2 ... formal-ai Desktop.app`:
  `invalid destination for symbolic link in bundle`
  preceded by `file modified:` lines for
  `Contents/Resources/browser-runtime/Frameworks/Google Chrome for Testing Framework.framework/{Resources,Versions/Current,Libraries,Helpers,...}` — all of them symlinks.

## Root cause
`desktop/scripts/prepare-resources.mjs` copies the Playwright Chromium
directory with `fs.cpSync(from, to, { recursive: true })`. Node's `cpSync`
defaults `verbatimSymlinks: false`, which **resolves** every symlink target to
an absolute path. The framework aliases inside `browser-runtime` therefore point
at `~/.cache/ms-playwright/chromium-*/...`, i.e. outside the `.app`. `codesign
--verify --deep` refuses such links with "invalid destination for symbolic link
in bundle", so every macOS build fails regardless of what the signer excludes.

## Fix
1. `prepare-resources.mjs`: `fs.cpSync(..., { verbatimSymlinks: true })` so the
   framework aliases stay relative and inside the bundle.
2. `adhoc-sign-mac.test.cjs`: skip the upstream-source cross-check when
   `@electron/osx-sign` is not installed (lint job), instead of failing.
3. New `prepare-resources.test.mjs` regression test, wired into
   `desktop/package.json` `test` and into the release workflow lint step.
