# Release CLI reports success after a rejected branch or tag push

Observed at template commit `3a034b27a56067971ca74dd4c74c7707056b215d` through the original `scripts/version-and-commit.php` entrypoint, in both `changeset` and `instant` modes.

`Git::push()` and `Git::pushTags()` return `Process::run()` results. The release orchestration in `VersionReleaser` ignores both results, so a failed Git publication still exits zero and writes `version_committed=true` to `GITHUB_OUTPUT`.

Reproduced remote states:

| Remote rejection | CLI status | Success output | Remote state |
| --- | --- | --- | --- |
| Branch push | 0 | `new_version=1.2.4`, `version_committed=true` | Branch remains at the input commit; `v1.2.4` is nevertheless pushed and points at the new release commit. |
| Tag push | 0 | `new_version=1.2.4`, `version_committed=true` | Branch advances to the new release commit; no release tag is published. |

These outcomes reproduce in both release modes. A later workflow step can therefore treat unsuccessful publication as a completed version commit.

## Reproduction

The [nine-case executable regression](https://github.com/link-assistant/formal-ai/blob/035cb62b99a0d5f1336114dbcc593d9e361c2906/experiments/issue-1580/template-php-push-integrity.test.mjs) copies the hash-verified original PHP sources into disposable Git checkouts and uses actual local bare remotes. An `update` hook rejects only `refs/heads/*` or only `refs/tags/*`. The test invokes the original PHP CLI, checks its exit status and exact Actions output records, then reads the remote branch and annotated tag targets.

```sh
git clone https://github.com/link-assistant/formal-ai.git
cd formal-ai
git checkout 035cb62b99a0d5f1336114dbcc593d9e361c2906
node --max-old-space-size=256 --test --test-concurrency=1 experiments/issue-1580/template-php-push-integrity.test.mjs
```

Requires Node, PHP and Git; no Composer installation, credentials, networked remote or hosted release is needed. This verifies the original release entrypoint and its dependencies with the fixture's minimal namespace autoloader. The original eight source files retain their Git blob and SHA-256 identities in `fixtures/php-template/source.json`.

All **9/9 focused cases** pass: four observations of the defect, four failure-propagation candidate controls and one positive-control test covering both modes before and after the candidate. All **52/52 tests** in the maintained template-observation entrypoint also pass. Full logs and digest receipts are retained in the [immutable evidence directory](https://github.com/link-assistant/formal-ai/tree/73da561d993cee3506e9f77dba353255888c4acc/experiments/issue-1580/evidence/checks/php-push-integrity).

## Suggested correction

Propagate unsuccessful branch and tag publication before emitting success outputs. In a disposable copy, changing exactly the two `Git` push boundaries from the existing `Process::run()` helper to the existing `Process::mustRun()` helper makes the CLI fail, suppresses `version_committed=true`, and stops tag publication after a branch rejection. Successful branch and tag publication still works in both modes.

The candidate does not make the two pushes atomic: if the branch push succeeds and the tag push fails, the earlier branch update remains visible. That state is explicitly asserted by the regression. No change has been submitted to the template here.

This is distinct from #26 (failed Git comparisons and no-code change classification) and #27 (automatic release guard after failed quality checks). The duplicate/source preflight is recorded with the publication receipt.

Reported while comparing CI/CD template behavior for https://github.com/link-assistant/formal-ai/issues/1580; tracked in https://github.com/link-assistant/formal-ai/pull/1581.
