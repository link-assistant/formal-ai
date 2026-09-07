# Upstream report 3 - no step declares a deadline, so the only timeout these pipelines have is the one that reports `cancelled`

**Targets:** `link-foundation/php-ai-driven-development-pipeline-template`,
`link-foundation/csharp-ai-driven-development-pipeline-template`

**Severity:** these two templates have `timeout-minutes` and nothing else. The
other three treat the job cap as a backstop and give each long step its own
budget precisely because the cap cannot fail a job. This is the other half of
the gap in report 2: report 2 is the missing observer, this is the missing
deadline.

---

## Title

`scripts/run-with-budget-warning.sh` does not exist and no step is wrapped; the
job cap is the only timeout, and a job killed by it is reported `cancelled`

## Evidence

```console
$ for t in rust js python php csharp; do
    printf '%-7s script=%-3s wrapped_steps=%s\n' "$t" \
      "$([ -f $t-template/scripts/run-with-budget-warning.sh ] && echo yes || echo NO)" \
      "$(grep -rh 'run-with-budget-warning\.sh' $t-template/.github/workflows \
         | grep -v '^\s*#' | wc -l)"
  done
rust    script=yes wrapped_steps=4
js      script=yes wrapped_steps=6
python  script=yes wrapped_steps=4
php     script=NO  wrapped_steps=0
csharp  script=NO  wrapped_steps=0
```

The three that have it explain what the cap alone cannot do, in the script's
own header:

```
# Why this exists: GitHub reports a job killed by `timeout-minutes` as
# `cancelled`, not `failed`. On a non-default ref that is indistinguishable
# from a superseded run, so a genuine timeout produces no red anywhere. The
# fix is to make the step own the deadline: when the step's budget expires
# first, the step fails, the job fails, and the annotation names the budget and
# the overrun.
```
(`rust-ai-driven-development-pipeline-template/scripts/run-with-budget-warning.sh:5`)

php's and csharp's longest steps are exactly the ones the other templates
budget -- the test suite and the release publish:

| | php | csharp |
|---|---|---|
| `test` cap | 30 min | 30 min |
| `test` budgeted steps | **0** | **0** |
| `build` cap | 20 min | 20 min |
| `build` budgeted steps | **0** | **0** |
| release job cap | 30 min | 30 min |
| release budgeted steps | **0** | **0** |

Compare `js`, same shape of job: `test` is capped at 15 min and each runtime
leg carries a budget (300s / 200s / 100s), so a hung suite fails the step at
its budget and never reaches the cap.

## Reproducible example

A step that hangs, with only a cap:

```yaml
      - name: Run tests
        run: composer test        # hangs
    timeout-minutes: 30
```

After 30 minutes GitHub kills the job and reports it **cancelled**. On a pull
request that is the same conclusion a superseded run gets, and neither template
has a `pipeline-status` job to tell them apart (report 2).

The same step with a budget:

```console
$ cd rust-ai-driven-development-pipeline-template
$ bash scripts/run-with-budget-warning.sh 3 "Hanging suite" sleep 30
::warning title=Hanging suite is approaching its execution budget::Hanging suite has been running for 2s of its 3s budget (70%). If it keeps growing, raise the budget and the job's timeout-minutes together, or split the work.
::error title=Hanging suite exceeded its execution budget::Hanging suite was terminated after 3s. This budget expires before the job's timeout-minutes backstop on purpose: a job killed by timeout-minutes is reported as 'cancelled' and hides the failure, while this reports 'failure'. Command: sleep 30
$ echo $?
124
```

Exit 124, an `::error` annotation naming the budget, and a failed step -- which
fails the job, which is red.

## Workaround

`timeout(1)` covers the single-process case with no new file:

```yaml
      - name: Run tests
        run: timeout 1500 composer test
```

It is not equivalent, and the difference is why the templates ship a script
rather than this line. `timeout(1)` signals the direct child only, so a test
runner that spawns workers leaves orphans holding the runner; there is no
warning before the deadline, so a suite drifting toward its budget is silent
until the day it fails; and the label in the annotation is whatever the shell
prints, not the name of the budget that expired.

## Suggested fix in code

**1. Copy `scripts/run-with-budget-warning.sh` from the rust template
verbatim.** It is `bash` with no language-specific dependency; the rust, js and
python templates run byte-identical copies. It already handles the three things
that make the naive version wrong:

* `set -m` so the command gets its own process group and the whole tree is
  signalled, not just the direct child;
* a `::warning` at `BUDGET_WARN_PERCENT` (default 70) so a step drifting toward
  its budget is visible before it fails;
* SIGTERM, then `BUDGET_GRACE_SECONDS`, then SIGKILL, with a fallback to
  signalling the direct child on platforms where the process group is not
  addressable (Git Bash on Windows).

**2. Wrap the long steps.** Four each, mirroring the other templates:

```yaml
# php release.yml
      - name: Run tests
        run: bash scripts/run-with-budget-warning.sh 1200 "Test suite" composer test
      - name: Build
        run: bash scripts/run-with-budget-warning.sh 800 "Build" composer build
      - name: Publish
        run: bash scripts/run-with-budget-warning.sh 900 "Packagist release" php scripts/version-and-commit.php --mode=changeset
```

```yaml
# csharp release.yml
      - name: Run tests
        run: bash scripts/run-with-budget-warning.sh 1200 "Test suite" dotnet test
      - name: Build
        run: bash scripts/run-with-budget-warning.sh 800 "Build" dotnet build -c Release
      - name: Publish
        run: bash scripts/run-with-budget-warning.sh 900 "NuGet release" bun run scripts/version-and-commit.mjs ...
```

The numbers are placeholders for whatever the measured p95 is; the invariant
below is what keeps them honest.

**3. `uses:` steps take `timeout-minutes` instead.** A `uses:` step cannot be
wrapped, so it declares its deadline at the step level, where an expiry *fails*
the step rather than cancelling the job. The js template says so at the site:

```yaml
      - name: Build Docker image (no push)
        # A `uses:` step cannot be wrapped by run-with-budget-warning.sh, so
        # it declares its deadline with a step-level timeout-minutes: an
        # exhausted step budget fails the step, while an exhausted job cap
        # only cancels the job.
        timeout-minutes: 20
```

**4. Add the invariant test.** Every budget must expire with room to spare
before the cap, or the cap fires first and the budget was decorative. The other
templates assert every budget is at most 70% of its job's cap
(`js-ai-driven-development-pipeline-template/tests/ci-timeouts.test.js`,
`MAX_BUDGET_SHARE_PERCENT`). Note that the js implementation of that test has
two false-negative bugs of its own, reported separately -- copy the invariant,
not the implementation.
