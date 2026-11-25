---
bump: patch
---

### Fixed
- CI/CD false negatives, false positives, warnings and errors found in the issue #1637 audit:
  - the `docker-build` and `docker-slim` jobs could never run, because their condition both refused and required `pull_request`; they now run on `main` after `build-artifacts`, and a contract test rejects any job guard that contradicts itself;
  - the scheduled `dependencies-latest` gate now passes: setup-node and setup-python move to v7, secretlint, the issue-1609 client CLIs and `@link-foundation/ui-links` are bumped, and the registry snapshot gains the records it was missing;
  - the semantic grounding graph is closed (missing Wikidata cache records were added and two dangling relations repointed);
  - Rust 1.100 clippy lint errors are cleared, along with unused rust-script dependencies;
  - ordered-list checks now treat Cargo `[[test]]` and `[[bin]]` targets as module roots.
