---
bump: minor
---

### Changed
- Every manifest moves to its publisher's latest release (issue #1169). Rust: links-notation 0.16.1 → 0.23.0 (the lino-objects-codec#60 hold-back is lifted), lino-objects-codec 0.7.0 → 0.8.0, lino-arguments =0.3.0 → 0.4.0, link-calculator 0.20.3 → 0.22.0, link-cli 0.2.11 → 1.0.0, web-search 0.5.0 → 0.6.0, command-stream 1.2.0 → 1.5.3, plus a full `cargo update`. npm: `@link-assistant/web-search` 0.11.1, `lino-i18n` 0.3.0, `marked` 18.1.0, electron ^44.6.0, desktop command-stream 1.5.0, puppeteer/puppeteer-core overrides ^25.12.0, and the `browser-commander` override 0.16.1 → 0.20.0.
- links-notation 0.23 reads `#` comments and reports syntax errors with their line and column. The seed adapter now uses the crate's `comments::strip_comments` in place of its own whole-line filter and returns the crate's located errors. The line-by-line locator the tests used for issue #1076 is retired. `data/meta/dreaming-lexicon.lino` no longer writes `#540` after an open parenthesis, where it would now start a comment.
- The browser bundle takes `lino-i18n/browser`. In 0.3, the package root wires Node file loaders. `js/i18n.js` loads its three catalogs through the library's `loadCatalogs`, which replaces its own fetch-and-merge code.
- `web-search` is taken with its recommended merge-only `merge` feature.
- The `has_bare_dot` guard in `rust/src/calculation.rs` is removed, along with the test that pinned it. link-calculator has rejected `2. 3` as a recoverable error since 0.18.0 (link-assistant/calculator#168).
- Agent orchestration runs commands through command-stream on Windows too. command-stream now keeps exact argv there (command-stream#190) and, since 1.5.1, stops the whole process tree on cancel, so the std-process fallback is removed. The desktop installs the VS Code CLI through the shared adapter's shell `{ file, args }` form (command-stream#191) instead of spawning `code.cmd` itself.

### Security
- Some releases are held back, each with a `"<name>//"` note in its manifest. Desktop command-stream stays at 1.5.0: 1.6.0+ and 2.0.0 depend on shelljs, which pulls in `braces` (GHSA-vfj7-8cjw-p6xm, no patched release). The `browser-commander` override stays at 0.20.0, because 0.21+ launches the installed Chrome by default (web-capture#160). The `@kreuzberg/html-to-markdown-node` override stays at 3.5.5, because later releases declare musl packages that npm does not have.
- The `proxy-addr`, `compression`, `global-agent` and `qs` overrides are still required. bun.lock and all three package-locks audit clean at `--audit-level=moderate`.
