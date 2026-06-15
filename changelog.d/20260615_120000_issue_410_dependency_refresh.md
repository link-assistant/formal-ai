---
bump: minor
---

### Changed
- Raised the Rust toolchain MSRV to 1.96 (latest stable) and updated the Docker builder image to `rust:1.96-slim`, matching the `web-search` and `web-capture` crate MSRVs.
- Updated all Rust workspace dependencies to their latest versions (`clap` 4.6, `doublets` 0.4.0, `link-calculator` 0.19.0, `meta-language` 0.45, plus transitive updates).
- Updated web bundle dependencies to the latest versions (`react`/`react-dom` 19.2.7, `marked` 18.0.5, `dompurify` 3.4.10) and rebuilt `src/web/vendor.bundle.js`; pinned Bun to 1.3.14.
- Updated desktop (`electron` 42, `electron-builder` 26) and VS Code (`@vscode/test-web` 0.0.80, `@vscode/vsce` 3.9.2) dependencies to the latest versions.
- Refreshed the issue #410 case study to reflect that the upstream `web-search`/`web-capture` readiness blockers are resolved (`web-search` published at npm 0.10.3 / crates.io 0.3.1 with full provider parity; `web-capture` at npm 1.10.9 / crates.io 0.3.31).

### Fixed
- Resolved Clippy lints newly reported by the latest stable toolchain (1.96) so `cargo clippy --all-targets --all-features` stays clean under `-Dwarnings`: added `const fn` where derivable, switched to `Option::is_none_or`, `std::iter::repeat_n`, `f64::midpoint`, and `u*::is_multiple_of`.
- Preserved code-block enhancements (highlighting and copy buttons) under React 19 by memoizing rendered markdown by message content; React 19 compares `dangerouslySetInnerHTML` by object identity, which otherwise re-assigned `innerHTML` and wiped the out-of-band DOM enhancements on unrelated re-renders.
