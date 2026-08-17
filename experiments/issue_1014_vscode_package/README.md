# Issue 1014 VSIX packaging experiment

The JavaScript advisory remediation upgraded Puppeteer to 25.7.0. Its
`chromium-bidi` 17 dependency no longer has the legacy `lib/cjs` paths which
Playwright deliberately leaves as optional imports. The VSIX esbuild step found
those dead-path imports and failed.

Reproduce the package and verify its bundled browser capture:

```sh
cd vscode
npm run package
cd ..
node experiments/issue_1014_vscode_package/runtime-smoke.mjs
```

To exercise an extracted VSIX rather than the generated files in the checkout,
set `FORMAL_AI_VSIX_ROOT` to its `extension/` directory.

`before-fix.log` records the failing bundle regression test and
`after-fix.log` records the same test passing after Playwright was excluded
from the server-side bundle, following the Playwright maintainers' guidance in
<https://github.com/microsoft/playwright/issues/33031>. The VSIX retains
`playwright` and `playwright-core` as ordinary production packages. The
Browser Commander 0.10.0 is the minimum release that forwards
`executablePath`; pinning it avoids both the broken 0.8 behavior and unrelated
native dependencies added in later releases.
