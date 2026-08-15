Downstream packaging found two compatibility constraints beyond the clean unit
suites reported above:

1. A server-side esbuild cannot inline Playwright. Puppeteer 25 resolves
   `chromium-bidi@17`, while Playwright retains optional private
   `chromium-bidi/lib/cjs/...` imports; esbuild follows those dead paths and
   fails. Playwright documents bundling as unsupported in
   <https://github.com/microsoft/playwright/issues/33031>. The working consumer
   configuration externalizes `playwright` and `playwright-core` and ships them
   as ordinary production packages.
2. web-capture's current `browser-commander@^0.8.0` silently drops a packaged
   browser's `executablePath`. The minimum compatible 0.10.0 forwards it. That
   dependency-floor defect and a real extracted-VSIX reproduction are now in
   <https://github.com/link-assistant/web-capture/issues/154>.

The complete validated downstream workaround is therefore the Puppeteer 25
override above, a scoped `browser-commander: "0.10.0"` override, and external
Playwright packages for bundled applications. The extracted 180.94 MB VSIX
successfully launched its packaged Chromium and rendered a local page with
this combination.
