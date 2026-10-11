bump: minor
---

### Added

- Every dependency now sits at its latest release, enforced by a gate
  (issue #1169): `scripts/check-dependencies-latest.rs` reads
  `Cargo.toml`/`Cargo.lock`, every `package.json` and its lockfile, the
  workflow `uses:` refs, the Dockerfile base image and the `rust-version`
  floor, and compares each against the publisher's own registry (crates.io,
  npm, GitHub releases, Docker Hub, the rust stable channel) — never a
  memorized list. A dependency may be held back only with a same-line
  `# blocked: <issue-url>` annotation (`CONTRIBUTING.md` documents the
  rule); drift without one fails the gate. A daily
  `dependencies-latest` workflow refreshes the registry snapshot the
  offline mode reads, applies every unblocked bump, re-resolves the
  lockfiles, re-runs the CI gates and opens one PR with what moved.

### Changed

- The tree itself moved to latest: `cargo update` re-resolved the Rust
  lockfile (command-stream 1.2.0, webrtc 0.21.0, toml_edit 0.25.15,
  clap 4.6.7, …; `links-notation` stays 0.16.1 behind an annotated block on
  lino-objects-codec 0.7.0), the npm manifests and lockfiles were
  hand-bumped with real registry integrity (react/react-dom 19.3.0,
  dompurify 3.4.16, marked 18.0.14, command-stream 1.2.0, electron 44.4.5,
  the electron-builder 26.17.0 ring, playwright 1.63.0 everywhere), and the
  image/action pins moved (`konrad/box-dind` 2.10.2, `actions/cache` v6,
  `zizmor-action` v0.6.4).
