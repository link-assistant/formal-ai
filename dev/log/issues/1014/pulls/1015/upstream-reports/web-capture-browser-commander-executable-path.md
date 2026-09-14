## Problem

`@link-assistant/web-capture@1.11.2` depends on
`browser-commander@^0.8.0`. Its `createBrowser(engine, options)` correctly
forwards unknown launch options to `launchBrowser`, but browser-commander
0.8.x destructures only `engine`, `userDataDir`, `headless`, `slowMo`,
`verbose`, `args`, and `colorScheme`. It silently drops `executablePath` and
`channel`.

This breaks packaged consumers that ship their own Chromium and cannot rely on
Playwright's default cache. formal-ai's VSIX passes its packaged executable,
but 0.8.1 ignores it and then reports that Playwright's default browser is not
installed.

## Minimal reproduction

```sh
npm init -y
npm install @link-assistant/web-capture@1.11.2
npm ls browser-commander
rg 'executablePath' node_modules/browser-commander/src/browser/launcher.js
```

The resolved version is 0.8.1 and the final command has no matches. The
launcher's options destructuring and Playwright/Puppeteer launch objects omit
the supplied path. By contrast, browser-commander 0.10.0 adds
`buildPlaywrightLaunchOptions` / `buildPuppeteerLaunchOptions` and forwards
both `executablePath` and `channel`.

The downstream real-artifact check is:

1. Package a VSIX containing Chromium plus `executable-path.txt`.
2. Call `createBrowser("playwright", { executablePath: packagedPath })`.
3. Fetch a local JavaScript-rendered page.

With 0.8.1 it searches Playwright's default cache instead of `packagedPath`.
With 0.10.0 the page is rendered successfully.

## Workaround

Consumers can add a scoped override:

```json
{
  "overrides": {
    "@link-assistant/web-capture": {
      "browser-commander": "0.10.0"
    }
  }
}
```

Pinning 0.10.0 is intentional: it is the first compatible release with these
options and avoids unrelated native dependencies introduced in 0.16.x.

## Suggested source fix

- Raise web-capture's dependency floor from `^0.8.0` to at least `^0.10.0`.
- Add a unit test that stubs both browser engines and asserts `channel` and
  `executablePath` reach `launchPersistentContext` / `launch`.
- Add an integration test that launches a browser from an explicit temporary
  path rather than an engine-managed cache.

This was found while fixing
<https://github.com/link-assistant/formal-ai/issues/1014>; the retained
downstream runtime experiment passes against an extracted VSIX.
