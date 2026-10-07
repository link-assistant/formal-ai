---
bump: minor
---

### Changed
- `self-authored-pull-request.yml` takes its token from the automation-token resolver instead of `FORMAL_AI_BOT_TOKEN`, and at the `default` layer dispatches the bot pull request's checks so they start without approval (issue #1187).
- `layered-ci.yml`, `evidence-check.yml`, `workflows.yml` and `web-ui-boundary.yml` accept a checks-mode `workflow_dispatch`; the dispatcher dispatches on the head branch and only workflows that declare the checks mode.
- Every `pull_request` workflow ignores `e2e/**` base branches, so isolated end-to-end pull requests never trigger this repository's pipeline.
- The credentials test now rejects any GitHub-token secret outside the `AUTOMATION_*` names, a `pull_request` workflow without a dispatch trigger or the `e2e/**` ignore, and an unguarded release dispatch not listed as pending.
