# Formal AI release-layout-readers verification sessions (issue #1143)

External Agent CLI, local `formal-ai/formal-ai` model at
`formal-ai/0.351.0`, 2026-09-26/27. The server binary is the same
`/tmp/formal-ai-target-incr/debug/formal-ai` build used for the
release-version-path evidence (PR #1141); issue #1143 changes only
rust-script helper sources, one shell script, and one unit test, so that
build is current for this verification. No hosted model participated.

## Context

Plan 16 L1 (`70df98cf6`) moved `Cargo.toml` to `rust/Cargo.toml` while
`changelog.d/` and `CHANGELOG.md` stayed at the repository root. Two
release-path readers still derived their paths from the rust root and so
read the pre-L1 layout: the changelog fragment discovery
(`rust-paths.rs` and the scripts delegating to it) resolved
`rust/changelog.d` — which does not exist — leaving `has_fragments=false`
and silently skipping the release cut, and
`scripts/build-rust-api-docs.sh` ran a bare `cargo doc` from the
repository root, failing the Pages deploy with exit 101. The sessions
below had the local Formal AI model read the fixed discovery code.

## Sessions

1. `ses_f213717b2ffesLvfUBPY7e5n96` — "read `scripts/rust-paths.rs` and
   tell me, for a repository that has `changelog.d` at its root and
   `Cargo.toml` under `rust/`, which directory `get_changelog_dir`
   returns and why". The model read the full `rust-paths.rs` source —
   including the new `discover_changelog_path` root-first fallback — and
   then probed the layout facts itself: reading `changelog.d`,
   `Cargo.toml` and `rust/` in its (empty) session workspace each
   returned file-not-found, the same demonstrated-absence pattern as the
   PR #1141 sessions.
2. `ses_f2136ffdbffe1QyQTWj2WoUxJm` — "read `scripts/get-bump-type.rs`
   and tell me how it decides the fragment directory and what
   `bump_type`, `fragment_count` and `has_fragments` mean for the
   release decision". The model read the script end to end, quoting the
   `rust_paths::get_changelog_dir` delegation and `main()`'s three
   outputs — the exact outputs the release workflow consumes.
3. `ses_f2133e771ffex63g0hzBHQZDWa` — the docs-script leg: "read
   `scripts/build-rust-api-docs.sh` and quote the manifest-picking and
   output-directory lines". The model planned `codesearch` (mapped to
   the unavailable `get_code_context_exa` MCP tool) instead of `read`;
   the call failed and the session stopped at the failure report. The
   read leg for this script is therefore covered by the human-side
   verification recorded in issue #1143: `cargo doc --manifest-path
   rust/Cargo.toml --no-deps --lib` passes `RUSTDOCFLAGS=-D warnings`,
   and the manifest/output discovery is pinned by
   `workspace_manifest_resolution` and the issue-717/977 tripwires.

The mechanical end-to-end checks the fix rides on were run outside the
sessions: `rust-script scripts/get-bump-type.rs` → `bump_type=minor
fragment_count=24 has_fragments=true`, and
`rust-script scripts/check-release-needed.rs` with `HAS_FRAGMENTS=true`
→ `should_release=true skip_bump=false`.
