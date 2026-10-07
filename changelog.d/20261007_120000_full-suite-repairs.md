---
bump: patch
---

### Fixed
- Page formalization and source trust read their seed records under the file's wrapper, so mime hints, sniff rules, code-fence language tags and trust weights apply again (issue #1163).
- `git log` history import passes its record format as `--format=`, not as a revision (issue #1180).
- A comma inside a date ("July 4, 1776") no longer splits the question into two sub-requests.
- A clarifying question about an unfilled slot (`*_unspecified`) and an arithmetic error naming what failed are kept rather than replaced by an open meta reply.
- The word-internal punctuation table no longer lists ASCII characters for spaced scripts, so names like `Node.js` stay whole inside an object phrase.

### Changed
- The coding-ladder gate accepts a lower floor only with a `coding_ladder_correction` note that says "corrected overcount" and names the previous floor.
- Derivation records under `data/cache/derivations/` are registered in the sources registry and git-ignored as run-time output.
