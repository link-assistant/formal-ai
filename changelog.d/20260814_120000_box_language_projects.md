---
bump: minor
---

### Added
- Build and run every generated language project inside the matching `link-foundation/box` image, using the language's own init commands (`cargo new`, `npm init`, `go mod init`, …), as a `box-language-projects` CI matrix and as a Docker-gated `cargo test`.
- `data/meta/box-image-survey.lino` records which box image variants are actually published and which tag the matrix pins, so the language contract can no longer name an image nobody publishes.

### Fixed
- Convert an installation guide into a script even when its steps name project creation and a build, instead of answering with a software-project plan.
