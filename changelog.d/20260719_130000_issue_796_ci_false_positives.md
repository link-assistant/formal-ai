---
bump: patch
---

### Fixed
- `scripts/install-node-dependencies.sh` matched reviewed npm deprecation
  warnings by exact `name@version`. Transitive versions float without any change
  on our side, so when `archiver-utils` resolved `glob` from `7.2.3` to
  `10.5.0` the warning stopped matching, was treated as an unexpected
  diagnostic, and failed the `.vsix` packaging job plus every desktop build even
  though `npm install` itself succeeded (issue #796). Reviewed deprecations are
  now matched by package name, so a version float can no longer break CI, and
  each one carries an accurate upstream tracking URL — `glob` reaches both
  workspaces through `@link-assistant/web-capture -> archiver -> archiver-utils`,
  not through vsce or electron-builder as previously implied. Unreviewed
  diagnostics still fail the build.
- `scripts/self-hosting-metric.rs` read commit trailers through git's
  `%(trailers:key=...)` placeholder, which only parses the last paragraph of a
  message. A release commit that separated `Formal-AI-Session` and
  `Formal-AI-Evidence` with a blank line therefore reported only the evidence
  trailer, and the resulting "must record both" error failed the whole Auto
  Release job after the version had already been computed. Trailers are now read
  from the full commit body, so their placement no longer decides whether a
  compliant commit is recognised.

### Added
- `INSTALL_NODE_DEPENDENCIES_VERBOSE=1` traces how each npm stderr line is
  classified, so an unexpected diagnostic can be diagnosed from CI logs without
  a local reproduction. Off by default.
