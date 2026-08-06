---
bump: patch
---

### Fixed
- The `deploy-pages` job no longer fails the pipeline when GitHub's Pages
  deployment queue is backlogged. `actions/deploy-pages` waited only its
  600 000 ms default, so a run whose artifact had uploaded successfully still
  aborted with `Timeout reached, aborting!` while the deployment was still
  `deployment_queued` — a red `main` pipeline that said nothing about the
  commit. The wait is now pinned to 1 200 000 ms and the job budget raised to
  35 minutes so the longer wait is not undone by a `timeout-minutes` kill.
  ([#909](https://github.com/link-assistant/formal-ai/issues/909))
