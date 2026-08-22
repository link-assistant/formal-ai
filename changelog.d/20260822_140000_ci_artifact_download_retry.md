---
bump: patch
---

### Fixed

- Retry the macOS test-archive download instead of reddening the pipeline on a
  transient storage failure. Run 32555911181 failed `main` with `Artifact
  download failed after 5 retries` on one of sixteen slices; the other fifteen
  downloaded the same artifact from the same run and passed, no test ran, and
  the blob URL named GitHub's own storage backend. `actions/download-artifact`
  spends its five internal retries back-to-back, so a backend having a bad
  minute exhausts them; the wrapper pauses between attempts so a later one meets
  a different minute. Exhausting the attempts still fails the step, with an
  `::error` annotation naming the cause.

- Let a single macOS slice be reran on its own. The archive is uploaded as
  `macos-core-tests-<run_id>-<run_attempt>`, but `gh run rerun --failed` puts
  the reran slice on attempt 2 while the archive job — which succeeded, so it is
  not rerun — left its artifact named `...-1`. The slice looked for a name that
  does not exist, so every partial rerun of a macOS slice failed with "artifact
  not found" and forced a full rerun of the whole pipeline. The download now
  resolves the artifact by name prefix.

- Raise the macOS slice job cap from 15 to 18 minutes so the retry fits beneath
  it. A retry that cannot finish inside its cap converts a transient failure
  into a *terminated* step, which GitHub reports as `cancelled` rather than
  `failure` — the issue #977 and #1017 failure this must not reintroduce. Worst
  case is now 140s download + 600s slice budget + 133s setup + 15s grace = 888s
  against a 1080s cap, and a test pins that arithmetic.
