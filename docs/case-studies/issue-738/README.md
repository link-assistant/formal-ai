# Issue 738: release map self-healing

Issue: [link-assistant/formal-ai#738](https://github.com/link-assistant/formal-ai/issues/738)
Pull request: [link-assistant/formal-ai#741](https://github.com/link-assistant/formal-ai/pull/741)

## Root cause

The reconstruction map stored `fragment`, `first_release`, and
`first_release_commit`. Automatic releases consumed fragments but only checked
the map; they could not update it before committing because the final column was
the SHA of that same not-yet-created commit. The next unrelated pull request to
run the path-filtered lint job therefore inherited a failing reconstruction
check.

Release `v0.296.2` reproduces the defect: it consumed
`changelog.d/20260715_issue_715_contextual_code_artifacts.md`, while the map's
latest row remained at `v0.296.1`. Before this fix,
`node experiments/issue_711_rebuild_changelog.mjs --check` exited 1.

## Fix

The map now stores only the relation it needs: `fragment -> first_release`.
After the release collector deletes the tracked fragments, the reconstruction
script reads those pending deletions from `HEAD`, adds them to the new release,
and rewrites `CHANGELOG.md` and the map before the release commit. The release
helper stages the regenerated map with the version, changelog, and fragment
deletions. The instant/manual workflow now uses that same atomic helper instead
of consuming fragments in a separate step before the version script runs.

## Regression coverage

`experiments/issue_738_fragment_release_map.test.mjs` provides:

1. a schema test proving map bytes do not change when a commit SHA changes;
2. a pending-release composition test with no commit SHA;
3. a whole-release test in a real local Git clone that commits a fragment,
   deletes it, regenerates before a release commit exists, and checks both
   artifacts.

CI runs that suite immediately before the reconstruction check.

## Agent CLI evidence

The documented live self-coding wrapper was attempted first and stopped before
launching the Agent CLI because its model registry rejected `formal-ai`; it
posted [the diagnostic issue comment](https://github.com/link-assistant/formal-ai/issues/738#issuecomment-5000759552).
The direct fallback then completed a real external Agent CLI ↔ local Formal AI
round-trip. That exposed a bounded-planner limitation: the server emitted a bare
`git` tool call and made no edits. The preserved raw logs are under `raw-data/`.
