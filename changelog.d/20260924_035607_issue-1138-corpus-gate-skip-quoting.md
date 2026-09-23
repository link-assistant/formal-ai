---
bump: patch
---

### Fixed
- `scripts/run-prebuilt-tests.sh` handed the corpus-gate skip to the test
  binaries as one quoted argument, so libtest saw the single option
  `skip issue_1138_no_silent_unknown` and refused to start the suite:
  run 35912188200 (job 107358064235) died with exit 101 twenty-six seconds
  into the `Run tests` step, before any test ran. The skip is now a bash
  array expanded as `"${CORPUS_GATE_SKIP[@]}"`, giving libtest the two words
  `--skip` and `issue_1138_no_silent_unknown` the way the three skip flags
  beside it already do. Verified by executing the script against arg-echo
  stubs for all three suites -- the earlier check was `bash -n` only, which
  cannot see word-splitting.
