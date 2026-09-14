# Upstream report 2 - every job is capped by `timeout-minutes`, and nothing observes the `cancelled` that a cap produces

**Targets:** `link-foundation/php-ai-driven-development-pipeline-template`,
`link-foundation/csharp-ai-driven-development-pipeline-template`

**Severity:** the failure mode `timeout-minutes` exists to catch -- a job that
hangs -- is the one failure mode these two pipelines cannot report. GitHub
reports a job killed by its cap as **cancelled**, not failed, and neither
template has a job that looks at the result.

---

## Title

`release.yml` caps 7 of 7 (php) / 8 of 8 (csharp) jobs with `timeout-minutes`
but ships no terminal `pipeline-status` job, so a cap that fires produces a
grey run instead of a red one

## Evidence

Both templates cap every job:

```console
$ # php release.yml
  detect-changes    timeout=5   needs=[]
  lint              timeout=20  needs=[detect-changes]
  test              timeout=30  needs=[detect-changes]
  changeset         timeout=10  needs=[detect-changes]
  build             timeout=20  needs=[detect-changes, lint, test]
  auto-release      timeout=30  needs=[lint, test, build]
  manual-release    timeout=30  needs=[lint, test, build]
  total jobs: 7 with timeout: 7

$ # csharp release.yml
  detect-changes    timeout=5   needs=[]
  changeset-check   timeout=10  needs=[detect-changes]
  lint              timeout=20  needs=[detect-changes]
  test              timeout=30  needs=[detect-changes]
  build             timeout=20  needs=[lint, test]
  release           timeout=30  needs=[lint, test, build]
  instant-release   timeout=30  needs=[lint, test, build]
  changeset-pr      timeout=10  needs=[]
  total jobs: 8 with timeout: 8
```

Neither has a job that reads those results. The last job in each file is a
release job, which runs only on the release path:

```console
$ grep -E '^  [A-Za-z0-9_-]+:$' php-template/.github/workflows/release.yml | tail -1
  manual-release:
$ grep -E '^  [A-Za-z0-9_-]+:$' csharp-template/.github/workflows/release.yml | tail -1
  changeset-pr:
```

The other three templates all ship the observer, and each says in a comment
exactly why:

```yaml
  # === PIPELINE STATUS ===
  # GitHub reports jobs killed by timeout-minutes as cancelled. Observe every
  # job so a timeout on main becomes a visible failure instead of a grey run.
  pipeline-status:
    name: Pipeline Status
    needs: [ ... every job ... ]
    if: always()
```
(`rust-ai-driven-development-pipeline-template/.github/workflows/release.yml:1208`)

| template | terminal status job | `scripts/check-pipeline-status.sh` |
|---|---|---|
| rust | `pipeline-status` (`release.yml:1211`) | yes |
| js | `pipeline-status` (`release.yml:854`) | yes |
| python | `pipeline-status` (`release.yml:901`) | yes |
| php | **none** | **no** |
| csharp | **none** | **no** |

## Reproducible example

The detector the other three have, given a job that hit its cap:

```console
$ cd rust-ai-driven-development-pipeline-template
$ NEEDS_JSON='{"lint":{"result":"success"},"test":{"result":"cancelled"},"build":{"result":"skipped"}}' \
  IS_MAIN=true bash scripts/check-pipeline-status.sh
Failed jobs:    <none>
Cancelled jobs: test
::error::Pipeline has cancelled jobs on main: test. A job killed by 'timeout-minutes' is reported as cancelled, which would otherwise hide the failure.
$ echo $?
1
```

On a non-default ref the same input degrades to a warning, because there a
cancelled job is usually a superseded run:

```console
$ NEEDS_JSON='...same...' IS_MAIN=false bash scripts/check-pipeline-status.sh
::warning::Cancelled jobs: test. On a non-default ref this is usually a superseded run — ...
$ echo $?
0
```

In php and csharp there is nothing to run: no such script, and no job that
would call it. The same `test` job hitting `timeout-minutes: 30` on `main`
leaves `build`, `auto-release` and `manual-release` skipped and no red check.

## Workaround

Until the job exists, make the caps visible from the outside: list every job of
`release.yml` individually as a required status check in branch protection, so
a cancelled one blocks the merge. That is a per-repository setting a template
cannot ship, it has to be re-done every time a job is added or renamed, and it
does nothing for pushes to `main` that are not merges -- which is why it is a
workaround.

## Suggested fix in code

Port both halves from the rust, js or python template unchanged; they are
already language-agnostic.

**1. `scripts/check-pipeline-status.sh`** -- 30 lines of `bash` plus a
three-line `python3` filter, with no language-specific dependency. It splits
`needs` by result, errors on any `failure`, and treats `cancelled` as an error
on `main` and a warning elsewhere.

**2. The job**, at the end of `release.yml`:

```yaml
  pipeline-status:
    name: Pipeline Status
    needs:
      - detect-changes
      - lint
      - test
      - changeset          # csharp: changeset-check
      - build
      - auto-release       # csharp: release
      - manual-release     # csharp: instant-release, changeset-pr
    if: always()
    runs-on: ubuntu-latest
    timeout-minutes: 5
    steps:
      - uses: actions/checkout@v4     # csharp: @v6
        with:
          persist-credentials: false

      - name: Check aggregate pipeline status
        env:
          NEEDS_JSON: ${{ toJSON(needs) }}
          IS_MAIN: ${{ github.ref == 'refs/heads/main' && github.event_name == 'push' }}
        run: bash scripts/check-pipeline-status.sh
```

`if: always()` is load-bearing: without it the job inherits the skip of
whichever dependency was cancelled, and the observer disappears exactly when it
is needed.

**3. Pin the `needs:` list to the job list.** The failure mode of this pattern
is a job added later and not added to `needs:`, which is invisible in review.
A test that lists the jobs of `release.yml` and asserts every one of them
appears in `pipeline-status.needs` (minus `pipeline-status` itself) closes it.

`csharp`'s `changeset-pr` is worth a moment: it declares no `needs:` and runs
independently, so it is not reachable from the release chain at all. It still
belongs in the observer's `needs:` -- it has `timeout-minutes: 10` like the
rest, and being unreachable from the chain is precisely why nothing else would
notice it being cancelled.
