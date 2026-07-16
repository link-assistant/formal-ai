---
bump: patch
---

### Fixed
- Stop the desktop release from reporting success when only some targets built.
  `BUILD-PROVENANCE.txt` listed all six builders unconditionally, so a run where
  most of the matrix failed still published a green, authoritative-looking
  `SHA256SUMS.txt` claiming builds that never happened. The manifest now lists
  only the builders that produced a fragment, names the missing targets, and the
  run fails after publishing the partial manifest.
- Verify the Linux and Windows artifacts before uploading them. Only the macOS
  artifacts were smoke tested, so the other four targets shipped with nothing
  checking that they were produced under the expected names and non-empty.
- Attach the SLSA build provenance before publishing assets to the release,
  rather than after, so assets are never downloadable without an attestation.
- Deduplicate concurrent desktop releases on the automatic (`workflow_run`) path.
  The concurrency group read `release.tag_name`/`inputs.tag`, neither of which
  that event carries, so it fell through to the always-unique `run_id` and
  concurrent runs for the same tag raced on `gh release upload --clobber` and on
  the consolidated `SHA256SUMS.txt`.
