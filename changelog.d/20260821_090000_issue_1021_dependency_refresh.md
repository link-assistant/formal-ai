---
bump: minor
---

### Changed

- Every direct Rust dependency is on its newest release that builds on stable.
  Six needed more than a version number: `lino-objects-codec` 0.2.1 → 0.4.1
  (library and dev-dependency both), `links-notation` 0.13.0 → 0.14.0,
  `meta-language` 0.54.0 → 0.58.2, `sha2` 0.10 → 0.11, `which` 7 → 8, and
  `web-capture` 0.3.36 → 0.3.37. `command-stream` stays pinned at `=0.16.0`,
  which issue #1014 pinned deliberately and `tests/unit/ci-cd/issue_1014.rs`
  asserts.
- `sha2` 0.11 returns its digest as a `hybrid_array::Array` rather than a
  `GenericArray`, and that type does not implement `LowerHex` -- so the nine
  places that rendered a digest with `format!("{:x}", ..)` all stopped
  compiling at once. They now go through `source_fetch::sha256_hex`, and the
  encoding itself is written once, in `source_fetch::hex_lower`. Adopting the
  new major rather than pinning back to the old one is what this costs, and it
  leaves one implementation of "digest bytes as text" where there were nine.
- `browser-commander`, the browser runtime both the desktop app and the VS Code
  extension override inside `@link-assistant/web-capture`, goes 0.15.0 → 0.16.0.
  The new release adds a native `better-sqlite3` addon and twenty-five more
  transitive packages, which grows the bundled `web-tools.cjs` the VSIX ships
  from 9.3 MB to 11.8 MB. It is taken rather than held: the addon backs
  browser-commander's cookie database, `web-capture`'s `src/browser.js` never
  reaches it, the esbuild bundle the VSIX is built from still builds, and both
  lockfiles audit clean.
