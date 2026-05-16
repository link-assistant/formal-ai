---
bump: patch
---

### Fixed
- Issue #72: GitHub Pages demo no longer advertises a stale hardcoded version. `src/web/index.html` now uses a `__FORMAL_AI_VERSION__` placeholder that `scripts/stamp-pages-artifact.sh` substitutes from `Cargo.toml` during the Pages deploy.

### Added
- CLI `--version` flag prints `formal-ai <CARGO_PKG_VERSION>` via clap's `version` attribute.
- Telegram `/version` (and `/version@formal_ai_bot`) command replies with `formal-ai <CARGO_PKG_VERSION>`.
