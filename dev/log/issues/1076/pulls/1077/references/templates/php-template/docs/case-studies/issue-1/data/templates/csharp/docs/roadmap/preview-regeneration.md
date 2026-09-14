# Preview-regeneration parity (deferred)

> **Status:** placeholder — waiting on an example-app surface to land in this
> template before the recipe can be ported.

This template is one of four `link-foundation` AI-driven development pipeline
templates auditing parity with the [`browser-commander`][bc] +
[Playwright][pw] preview-regeneration pattern that originated in
[`konard/vk-bot-desktop`][vk]. The primary upstream tracker is
[link-foundation/js-ai-driven-development-pipeline-template#62][js62]; the
mirror for this repository is [issue #17][cs17].

## Why it is deferred here

The pattern auto-regenerates README, GitHub Pages, and `og:image` previews
from a real browser screenshot of an example app. **This template does not
yet ship an example-app surface** — the only runnable artifact is
`examples/BasicUsage`, a console app with no rendered output. Until a web
surface exists (DocFX site sample, Blazor demo, or similar), there is no
viewport to capture, so the recipe stays parked here.

## When to port

Port the recipe the next time **any** of the following lands in this repo:

- A Blazor / WASM / Razor Pages example app under `examples/`.
- A custom DocFX landing page with a hero image worth keeping in sync.
- An `og:image` or social preview asset committed to the repo.

## Recipe to port

Pull these building blocks from the upstream implementation
([`scripts/update-preview-images.mjs`][script], [`preview-regen` job][job]):

- Static HTTP server over the built site/app (no devserver, no Electron).
- `browser.newContext({ locale })` to drive the locale axis without UI toggles.
- `commander.emulateMedia({ colorScheme })` + `localStorage` to drive theme.
- `commander.page.screenshot()` — `browser-commander@0.8`+ exposes the raw
  Playwright page; there is no native screenshot method as of `0.10.1`.
- `git status --porcelain` drift detection + `git commit -m "... [skip ci]"`
  push-back so the loop is self-healing.
- `PREVIEW_VERBOSE=1` to dump DOM probes (`data-theme`, `lang`, `h1`) for
  CI-only diagnostics.

The integration point in this repo will be a new job in
`.github/workflows/docs.yml` (next to the DocFX deploy) or a sibling
`example-app.yml` once one exists. Run on push to `main`, on release tag
pushes, and on `workflow_dispatch`. Surface drift either as an auto-commit
back to `main` (push events) or a workflow artifact (pull request events).

## Related

- [konard/vk-bot-desktop#51][vk51] — original issue
- [konard/vk-bot-desktop#52][vk52] — implementation PR
- [link-foundation/js-ai-driven-development-pipeline-template#62][js62] — primary upstream tracker
- [Per-template parity survey][survey]

[bc]: https://www.npmjs.com/package/browser-commander
[pw]: https://playwright.dev/
[vk]: https://github.com/konard/vk-bot-desktop
[vk51]: https://github.com/konard/vk-bot-desktop/issues/51
[vk52]: https://github.com/konard/vk-bot-desktop/pull/52
[js62]: https://github.com/link-foundation/js-ai-driven-development-pipeline-template/issues/62
[cs17]: https://github.com/link-foundation/csharp-ai-driven-development-pipeline-template/issues/17
[script]: https://github.com/konard/vk-bot-desktop/blob/issue-51-60ec0489f01f/scripts/update-preview-images.mjs
[job]: https://github.com/konard/vk-bot-desktop/blob/issue-51-60ec0489f01f/.github/workflows/js.yml#L657
[survey]: https://github.com/konard/vk-bot-desktop/blob/issue-51-60ec0489f01f/docs/case-studies/issue-51/data/templates/survey.md
