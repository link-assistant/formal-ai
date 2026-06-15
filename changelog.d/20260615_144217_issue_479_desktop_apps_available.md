---
bump: minor
---

### Fixed

- **Desktop apps are actually built and available on `/download` (issue #479).**
  The automated release tags a *child* `chore: release vX.Y.Z` commit whose
  CI run carries the *parent* SHA, so the desktop-release resolve step (which
  required a tag pointing at `workflow_run.head_sha`) never matched and zero
  desktop assets were uploaded — every release since the path went live showed
  "Not available in latest release". The resolve script now targets the latest
  published release with a defensive exact-SHA tier and an idempotency guard,
  and emits grouped verbose diagnostics (`[desktop-release-resolve]` logs) so
  the resolution decision is auditable for future triage; the
  `desktop-release` workflow no longer gates on full-pipeline
  `conclusion == 'success'` (the release is published early, so a later job
  failure used to suppress the whole desktop build); and
  `scripts/wait-for-pages-deployment.sh` is now marker-authoritative
  (`deployment.json`'s SHA proves the matching stamped build is live, since
  GitHub Pages deploys atomically) so the E2E Pages probe stops timing out and
  failing the pipeline. Landing/docs assets are cache-busted with
  `?v=__FORMAL_AI_ASSET_VERSION__` like `/app/`.

- **Linux desktop assets recover after partial desktop releases (issue #479).**
  The follow-up `v0.204.0` Desktop Release uploaded macOS and Windows assets
  but no Linux assets because `electron-builder` rejected `.deb` metadata with
  a missing homepage, author email, and Linux maintainer. The desktop package
  manifest now provides that metadata, the workflow normalizes Electron
  Builder's Linux x64 aliases (`x86_64` AppImage and `amd64` `.deb`) to the
  `x64` names used by `/download`, and the automatic desktop-release
  idempotency guard skips only when all 14 expected desktop assets are present.
  Releases missing Linux `AppImage`, `.deb`, or `.tar.gz` artifacts rebuild
  instead of staying permanently partial.

- **macOS install screenshots are real captures, not synthetic renders
  (issue #479).** The `/download` macOS Gatekeeper figures are now genuine
  macOS 15 (Sequoia) captures from the sibling app `konard/vk-bot-desktop`,
  which ships the identical `electron-builder` ad-hoc signing flow, replacing
  the previously generated images the maintainer rejected as fake. The
  synthetic generator and HTML fixture are removed; provenance is documented
  in `src/web/download/assets/screenshots/README.md`.

### Added

- **Source code is a big hero button on the landing page (issue #479).** The
  landing surfaces the source repository as a prominent `.source-cta` call to
  action (translated for every supported locale) instead of a small footer
  link.
