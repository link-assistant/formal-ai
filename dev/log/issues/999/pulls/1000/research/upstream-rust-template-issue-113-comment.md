The platform behavior has changed since this was closed, so the removal in #114 is now stale.

Reproduction (2026-08-11):

```yaml
jobs:
  publish:
    runs-on: ubuntu-latest
    concurrency:
      group: repository-writes
      queue: max
    steps:
      - run: echo publish
```

GitHub now accepts and documents `queue: max`: it retains up to 100 pending jobs/runs and processes them FIFO by the time they begin waiting. GitHub also documents that it cannot be combined with `cancel-in-progress: true`:

https://docs.github.com/en/actions/how-tos/write-workflows/choose-when-workflows-run/control-workflow-concurrency#example-queueing-multiple-pending-runs

The current actionlint v1.7.12 still emits the error originally quoted here; that is a known actionlint schema lag tracked by rhysd/actionlint#657 and PR #654:

https://github.com/rhysd/actionlint/issues/657

Workaround until actionlint ships support: keep `queue: max` for non-cancellable write jobs and narrowly suppress only actionlint's `unexpected key "queue" for "concurrency" section` diagnostic. Do not disable actionlint globally and do not combine the key with `cancel-in-progress: true`.

Suggested template fix: restore `queue: max` on every shared repository-write concurrency group, reverse/update the regression test introduced by #114, and add the narrow actionlint suppression with a link to actionlint#657. Without `queue: max`, a third writer cancels/replaces the already-pending second writer, which can silently drop a release/deploy/generated-content update.
