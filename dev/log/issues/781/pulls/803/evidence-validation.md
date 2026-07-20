# Evidence validation

The final publication checks found no symlinks and no file near GitHub's
100 MiB per-file limit. The largest retained artifact is the 19,946,540-byte
reference solution-session log.

- All 63 JSONL dialog files parse line by line.
- All 70 non-empty `.json` captures parse. Two zero-byte early capture attempts
  (`github/related-pr-795-ci-runs.json` and `github/run-29709092598.json`) are
  retained as collection-history artifacts; the successful run metadata and
  logs are present under `ci-logs/`.
- Both downloaded screenshots have valid PNG signatures, complete chunk CRCs,
  terminal `IEND` chunks, exact byte lengths, and 2262×1130 dimensions; see
  `github/screenshot-validation.log`. Both were also visually inspected.
- The 22,382-line successful GitHub Actions log was reviewed in ranges no
  larger than 1,500 lines. Its chunk audit reports no Actions error markers,
  non-zero process exits, or failing test summaries.
- `security-scan-summary.md` documents the credential-shape scan and manual
  review of its false-positive/public-browser-key candidates.
- `file-inventory.tsv` records the final byte count and SHA-256 digest of every
  other file in the bundle.
