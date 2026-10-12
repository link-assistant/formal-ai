---
bump: minor
---

### Changed
- `self-authored-pull-request.yml` takes its token from the automation-token resolver instead of `FORMAL_AI_BOT_TOKEN`, and at the `default` layer dispatches the bot pull request's checks so they start without approval (issue #1187).
- `layered-ci.yml`, `evidence-check.yml`, `workflows.yml` and `web-ui-boundary.yml` accept a checks-mode `workflow_dispatch`; the dispatcher dispatches on the head branch and only workflows that declare the checks mode.
- Every `pull_request` workflow ignores `e2e/**` base branches, so isolated end-to-end pull requests never trigger this repository's pipeline.
- The credentials test now rejects any GitHub-token secret outside the `AUTOMATION_*` names, a `pull_request` workflow without a dispatch trigger or the `e2e/**` ignore, and an unguarded release dispatch not listed as pending.
- The GitHub credential resolver now takes the `AUTOMATION_*` secrets as inputs. A composite action cannot read `secrets`, so the self-authored pull request workflow had failed before doing anything.
- Every pull-request workflow can be dispatched in checks mode, except four with named reasons. When the App layer allows it, an isolated end-to-end run gets its own private repository. At the default layer, cross-repository findings go to one tracking issue in this repository.
- The full test lane now runs as four parallel shards over the prebuilt test executables. The runner reports every failing target instead of stopping at the first one, and a failed data or census gate no longer hides the rest of the suite.
- The agentic CLI matrix starts every CLI that declares a config relocation variable from a relocated config with an empty HOME. The session must still reach the server through the recording proxy (#1161 R7).
- The browser worker logs the native `procedure_cache` miss (with `research_missing` from the cache-policy seed) for an unmodified catalog program request, and the JS derivation module gains `persist`, `load` and `explainAnswer` over an injected file io (#1165 R10, #1184 R9).
