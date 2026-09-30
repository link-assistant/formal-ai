---
bump: patch
---

### Added

- Every release now publishes standalone `formal-ai` CLI archives:
  `formal-ai-cli-<target>.tar.gz` for `x86_64-unknown-linux-musl`,
  `aarch64-unknown-linux-musl`, `x86_64-apple-darwin` and
  `aarch64-apple-darwin`, and
  `formal-ai-cli-x86_64-pc-windows-msvc.zip` — built by a new `cli` job in
  the Desktop Release workflow, each holding the binary plus `LICENSE` and
  `README.md`, sha256-verified through the same consolidated
  `SHA256SUMS.txt`/`BUILD-PROVENANCE.txt` as the desktop assets, with SLSA
  attestations and a per-leg smoke test that extracts the archive and runs
  `formal-ai --version` on its own runner. `cargo binstall formal-ai`
  resolves the archives through new `[package.metadata.binstall]` metadata,
  and `scripts/install.sh` / `scripts/install.ps1` now install the CLI from
  the prebuilt archive first, falling back to `cargo install formal-ai`
  only when no archive matches the host (issue #1181).

### Fixed

- Boolean `FORMAL_AI_*` environment switches share one spelling table —
  `1/0/true/false/yes/no/on/off`, case-insensitive — through the new
  `cli_env` module, so `FORMAL_AI_SILENT=1 formal-ai chat …` no longer fails
  with `invalid value '1' for '--silent'` (the same fix covers
  `FORMAL_AI_TRACE_*`, `FORMAL_AI_MEMORY_DEBUG`, `FORMAL_AI_DREAMING*`,
  `FORMAL_AI_RECORD_CHAT`, `FORMAL_AI_LIVE_FETCH`/`_LIVE_API`,
  `FORMAL_AI_LINK_CLI_DEBUG` and `FORMAL_AI_SEED_LINKS_MIRROR`); an
  unrecognised value keeps each switch's documented default instead of being
  silently coerced (issue #1181).
