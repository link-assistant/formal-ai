# Requirements and solution map

| Requirement | Implementation and evidence |
|---|---|
| Read the complete issue and comments | Raw issue and paginated comment payloads are in `raw-data/issue-711.json` and `raw-data/issue-711-comments.json`. |
| Find the actual release root cause | The automatic collector retained fragments and staged only Cargo/CHANGELOG files; see the root-cause section in `README.md`. |
| Reproduce before fixing | `raw-data/regression-test-before.log` records the failing assertion that a collected fragment still exists. |
| Consume fragments safely | `scripts/version-and-commit.rs` removes them only after a successful changelog write. |
| Stage consumed files | The release commit now runs `git add -A` on the changelog directory and treats staging failures as fatal. |
| Prove the second release is empty | The unit test invokes the collector twice and requires the changelog to remain byte-identical after the second invocation. |
| Preserve README | The same test requires `changelog.d/README.md` to remain present. |
| Repair historical pollution | The deterministic experiment reconstructed 391 released fragments exactly once and reduced `CHANGELOG.md` from 609,927 to 5,261 lines at the time of the fix; after merging releases through v0.294.0 it tracks 396 fragments in 5,344 lines. |
| Remove stale fragments | All 388 fragment files stale at the time of the issue are deleted; README and genuinely unreleased fragments remain. |
| Make cleanup reproducible | CI executes `node experiments/issue_711_rebuild_changelog.mjs --check`; the release map is committed. |
| Preserve all related data | `raw-data/` contains GitHub payloads, releases, histories, diffs, CI output, agent traces, and test/build logs. |
| Create case study, timeline, root causes, requirements, and solutions | `README.md`, this file, `template-audit.md`, and `online-research.md` provide those records. |
| Research established solutions online | Official Towncrier, Scriv, Changesets, and release-plz references and conclusions are in `online-research.md`. |
| Compare all CI/CD files with JS, Rust, Python, and C# templates | Full file inventories and directory diffs are preserved in `raw-data/`; conclusions are in `template-audit.md`. |
| Report the defect to affected templates | The Rust template already fixed it in issue 65/PR 66. The other templates consume fragments, so no duplicate issue was opened. |
| Use the repository's external-agent contribution workflow | The live wrapper failure and successful direct three-round Agent CLI session are archived in `raw-data/` and `agent-evidence/`. |
| Keep regression checks in CI | The lint job runs both the Rust collector test and deterministic reconstruction check. |
| Update and finalize PR 718 | PR title/body, readiness, commits, checks, and final CI state are handled during PR finalization. |
