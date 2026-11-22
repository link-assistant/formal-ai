Manual release dispatch can bypass lint in the current release workflow.

Reviewed source: `dd803a2d9fa5b3a992b435acce73162274fb9e8b`, `.github/workflows/release.yml`, Git blob `15fbed88c68753227e4d12abbbc3eb3ad23b7bff`.

`detect-changes` excludes `workflow_dispatch` (line 44). `lint` needs that job (line 80), but its guard at lines 87–94 has no status function. GitHub supplies the default `success()` check, so a skipped detection dependency skips lint despite the dispatch event branch. The test job has an explicit status function, and the build guard (lines 191–196) accepts successful tests with skipped lint. Both manual release modes can then accept a successful build.

This is a bounded expression-model reproduction against the exact pinned YAML, not an observed hosted release. The model implements the documented implicit status rule: https://docs.github.com/en/actions/reference/workflows-and-actions/expressions#status-check-functions . The current manual-run query returned no runs; it supplies no execution evidence.

The executable reproduction in Formal AI PR #1581 is `experiments/issue-1580/template-java-boundary-controls.test.mjs`. Run `node --test --test-concurrency=1 experiments/issue-1580/template-java-boundary-controls.test.mjs`: five cases pass. The original dispatch guards reproduce skipped lint followed by eligible build/release. A disposable guard candidate uses explicit `!cancelled()` for lint and requires successful lint plus tests on dispatch. Push/PR behavior retains all five change flags, failed/skipped/cancelled detection controls and existing non-dispatch skipped-lint admission. Failed, cancelled or skipped quality jobs cannot admit a manual candidate build.

Setting `ISSUE_1580_JAVA_KEEP_ORIGINAL_DISPATCH_GUARD=1` reproduces the candidate regression failure. Other cases remain positive. No workflow dispatch, Maven build, hosted publication or upstream source modification is performed by these tests. The candidate is a local proposal and does not establish an upstream repair.
