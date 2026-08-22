---
bump: patch
---

### Fixed

- Retry past a connection reset instead of reporting it as a broken link. Run
  32586546161 failed `main` on four links with `Network error: Connection reset
  by peer (os error 104)`; all three distinct hosts answer 200 from a
  workstation. A reset happens below HTTP and carries no status code, so neither
  the accept list nor `--cache-exclude-status` could name it — and one of the
  four came back as `Error (cached)`, the same reset replayed from cache.
  Retries go from three to six with a growing wait, so a later attempt meets a
  different moment. 404, 403 and 410 still fail the build.

- Stop link-checking a recorded lychee report. The fixture under
  `experiments/issue-1021-link-checker-false-positive/` is captured evidence of
  a past failure whose URLs are *supposed* to be broken, so checking them could
  only ever produce a false positive.
