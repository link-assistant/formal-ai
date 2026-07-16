---
bump: patch
---

### Fixed
- Stop the issue #656 traceability test from asserting that a changelog fragment
  exists forever. Fragments are consumed by the release that ships them, so the
  test began failing on every run once the v0.296.0 release deleted the fragment
  it pinned. It now follows the entry across its lifecycle: a fragment before
  release, a `CHANGELOG.md` section after one.
