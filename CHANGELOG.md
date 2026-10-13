# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

Older releases are archived under `docs/changelog/` so that no file
exceeds the repository's 1500-line cap (newest first):

- [0.353.0](docs/changelog/releases-from-0.353.0.md)
- [0.352.0 to 0.352.1](docs/changelog/releases-from-0.352.0.md)
- [0.346.0 to 0.351.0](docs/changelog/releases-from-0.346.0.md)
- [0.307.0 to 0.345.0](docs/changelog/releases-from-0.307.0.md)
- [0.242.0 to 0.306.1](docs/changelog/releases-from-0.242.0.md)
- [0.183.0 to 0.241.0](docs/changelog/releases-from-0.183.0.md)
- [0.177.0 to 0.182.0](docs/changelog/releases-from-0.177.0.md)
- [0.105.0 to 0.175.0](docs/changelog/releases-from-0.105.0.md)
- [0.1.0 to 0.104.0](docs/changelog/releases-from-0.1.0.md)

<!-- changelog-insert-here -->

## [0.353.1] - 2026-10-13

The layered JavaScript-to-Rust check now overlaps full retranslation with compilation of the committed projections. Both checks must succeed, freshly translated projections must remain byte-equal, and failures cancel the remaining process groups. The original module set, equivalence assertions, and 540-second execution budget are preserved.

### Changed
- Temporarily defer only PR1196's positive authorship requirements for release recovery, with explicit zero credit and named restoration debt. Strict evidence validation, measurement, debt checks and the standing main/daily floor remain active.

### Security

- Upgrade meta-language to 1.0.0 and its maintained Lindera dependency, removing the unmaintained unsafe-libyaml dependency while preserving the default language engine and CJK segmentation.

Keep exact Cargo manifest and lockfile bytes when validating the release version child, so trailing LF or CRLF line endings match the governed source witness. Retain strict source and dependency checks and cover the complete physical Git transition.

Temporarily defer only the two positive-authorship gates for exact open release-repair PR1197, with zero credit and explicit restoration debt; retain strict evidence measurement and all standing main gates.
