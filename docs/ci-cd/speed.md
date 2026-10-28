# CI speed: small parallel jobs, longest first

CI is only useful while it is quick to iterate on. PR #1188 sets the rule
and a gate that enforces it.

## The rule

- **No job or step runs longer than 30 minutes.** 15 minutes is the target.
  Every job declares `timeout-minutes` at or under 30, and so does every step
  in it. No job's measured maximum is above 30 minutes either.
- **Long work starts first.** Small jobs fill the capacity left once the long
  jobs are running. This holds at two levels:
  - **Jobs on parallel machines.** A long job (measured median of 10 minutes
    or more) has `needs:` only on jobs whose outputs, results or artifacts it
    uses. It never waits on a fast lint job it does not depend on.
  - **Tests inside a job.** Sharded suites are split longest-first from
    recorded durations, never dealt out by listed index. The files of a
    JavaScript shard run on every core, longest first.
- **Sharding loses nothing.** The plan puts every listed test in exactly one
  shard. Each shard re-checks this on the real listing before it runs its
  part.

## How it is enforced

`node scripts/check-ci-speed.mjs` is the gate. It is registered in
[`data/meta/ci-gates/check-ci-speed.lino`](../../data/meta/ci-gates/check-ci-speed.lino)
and runs in the `lint` job, and `--report` also prints each workflow's
critical path. The rule and its exceptions live in
[`data/meta/ci-speed.lino`](../../data/meta/ci-speed.lino). The gate fails when:

- a job has no `timeout-minutes`, or a job or step cap is above the limit,
  and no `over-limit-job` entry allows it;
- a job's measured maximum in
  [`data/meta/ci-durations.lino`](../../data/meta/ci-durations.lino) is above
  the limit, and no entry covers it;
- an `over-limit-job` entry no longer excuses anything, because the job now
  meets the limit or is gone. The exception list only shrinks;
- a long job `needs:` a job whose outputs, results or artifacts it does not
  use;
- a workflow or `scripts/*.sh` file shards tests by index
  (`(NR - 1) % n`, `position % SHARD_TOTAL`, `--test-shard`,
  `--partition slice:`);
- a `sharded-suite` does not plan its shards with
  `scripts/plan-test-shards.mjs`;
- the planner loses, repeats or misorders a recorded test for any shard
  count from 1 to 12.

`rust/tests/web/ci-speed.test.mjs` pins each of these on small inputs, and
then on the real tree.

## Test shards, longest first

`scripts/plan-test-shards.mjs` reads a test listing on stdin, one test per
line, optionally as `<executable>\t<test>`. It prints one shard's part:

```sh
dist/tests/unit --list --format terse | sed -n 's/: test$//p' \
  | node scripts/plan-test-shards.mjs --shard 2 --of 4
```

The planner works like this:

- It sorts the tests longest first.
- It gives each test to the shard with the least work so far. This is the
  longest-processing-time rule; Graham's bound keeps the slowest shard within
  4/3 of the optimum.
- Ties break by name, so every shard computes the same plan on its own.
- `--reserve "1=330,2=80"` tells it that a shard already carries other work,
  such as the full lane's data-integrity and census gates on shard 1, so that
  shard gets fewer tests.
- `--check` exits non-zero unless the shards cover the listing exactly once.
- `--report` prints each shard's predicted seconds.

The planner is used by:

- the full Rust lane (`scripts/run-prebuilt-tests.sh`, four shards over the
  unit, integration and source executables together);
- the specification lane (the `Run specification tests from the prebuilt
  binary` step in `release.yml`);
- the coverage shards (`scripts/run-coverage-shard.sh`);
- the Layered CI js tier, which plans the files under `rust/tests/web/` and
  runs them with `node --test --test-concurrency="$(nproc)"`.

A test with no recorded duration counts as `default-seconds` (0.1 s unless
the file sets it). A new test still runs exactly once, and its weight only
affects balance.

## Refreshing the durations

Every record is generated, never edited by hand. The generator only reads
from GitHub with `gh`:

| Record | Command | Source |
|---|---|---|
| `data/meta/ci-durations.lino` | `node experiments/formal_ai_subagent/ci-durations.mjs --write [--branch B] [--runs N]` | The last N completed, non-cancelled runs of every workflow on the branch. It records each job's median and maximum minutes, and the run wall-clock. |
| `data/meta/test-durations.lino` | `node experiments/formal_ai_subagent/ci-durations.mjs --tests --write [--run ID]` | The `Test (…)` job logs of a CI/CD Pipeline run with per-test times. |
| `data/meta/javascript-test-durations.lino` | `node experiments/formal_ai_subagent/ci-durations.mjs --javascript-files --write` | Each suite file run alone on the local machine. Only the order carries over to CI. |

Per-test Rust times come from libtest's `--report-time`. On a stable
toolchain that flag needs `RUSTC_BOOTSTRAP`, which reaches every process the
tests spawn, so it is off by default. To record the times for one run:

1. Set the repository variable `FORMAL_AI_RECORD_TEST_TIMES` to `true`.
2. Push, or re-run the CI/CD Pipeline.
3. Run `--tests --write` against that run.
4. Unset the variable.

## What is still over the limit

The `over-limit-job` entries in `data/meta/ci-speed.lino` are the current
debt. Each entry gives its measured numbers, the reason it is still over the
limit, and what would bring it under. Most of them never run on a pull
request: release publishing, Pages deployment, scheduled refreshes, and
macOS while non-Linux CI is off. The ones that do run on pull requests are:

- the release `test` cap (measured at most 19.3 minutes, but sized by the
  issue #1081 budget-share rule);
- the coverage build;
- the local web-app E2E suite (30.0 minutes at most);
- the agent CLI E2E legs;
- the desktop packaging builds;
- the agent ladder's static 180-minute cap, which the wall-clock record
  [`data/meta/ci-wall-clock.lino`](../../data/meta/ci-wall-clock.lino) reads.

## Temporary critical coding profile (2026-10-27)

The current priority is a usable Linux/Docker release that can solve small
coding tasks, answer questions, report failures and retain evidence for learning.
The 2026-10-28 fast PR policy supersedes the earlier all-integrations-on-PR
profile: Linux JavaScript/TypeScript behavior and consistency checks remain on
PRs; native production compilation/tests, package and image builds, native parity,
source-network serialization, native measurement and broad benchmark/client integration jobs are
retained for main. Set FORMAL_AI_FULL_PR_CI=run to restore those PR jobs. A manual
branch dispatch also uses the fast profile unless this variable is set. Main's
production builds, native tests, browser/agent integration and public-download
checks retain their original assertions and failure policy. The profile notice
explicitly awards no coverage to deferred jobs.

Keep related implementation, captured-log regressions, requirements, vision,
guidelines and handoff together in the working PR. Qualify its exact current head
before merging. Then monitor the resulting main source and version-preparation
commit, every configured release consumer, and public artifact verification.
Record the actual version and immutable artifact identities; an older published
release does not satisfy delivery of the new source.

Real GitHub client tests have an independent, default-off effect gate:
FORMAL_AI_REAL_GITHUB_E2E=true or workflow_dispatch real-github-e2e=true.
Use the offline JavaScript regressions for frequent iteration. Retained live
first/repeat solves, released-memory lifecycle and authenticated cleanup run only
when deliberately selected; offline fixtures do not establish current live
credentials, network behavior or three-client acceptance. A selected failure
remains a failure. Never rerun an old effect owner.

Public CI captures under rust/tests/fixtures/generated-e2e reproduce the late
scheduled-observation admission race and exact legacy test-issue shapes. Their
unit tests cover failures and negative variants without GitHub effects. The
retirement CLI first writes a reviewed plan, then revalidates every issue before
closing only exact inactive bot-owned fixtures as not_planned with an unverified
outcome comment; it never claims historical test success. Normal freshly run
tests retain the trusted finalizer. See the [requirements](../requirements/doctrine-fast-pull-request-production-ci.md)
for scope, restore controls and coverage limitations.
Noncritical recorded agent scenarios and the long method-learning and
computer-record replay legs are temporarily opt-in with
FORMAL_AI_FULL_AGENT_REPLAY=run; routing-changing pull requests still select
them automatically. Hello World compilation, resumed execution, literal/file
creation, duplicate-issue avoidance, issue submission, readable complete-context
reports, report TUI, client-execution learning, held-out generalization and the
self-coding inner loop remain in the ordinary full main profile. Native learning
unit tests remain required on main; the fast PR profile uses the selected
JavaScript regressions and does not claim those native executions.

The duplicate instrumented coverage execution is temporarily opt-in: its
32 native and 16 browser shards and both report jobs are retained unchanged.
The workflow's scope job records that no coverage was measured when deferred.
Run coverage.yml with full-coverage=true, or set FORMAL_AI_FULL_COVERAGE=run
to restore automatic execution. Both unchanged ratchets and every shard must
pass whenever coverage is selected. Reenable automatic instrumentation after
reducing the queue and release latency; this is temporary delivery debt.

Windows/macOS test reruns already use FORMAL_AI_NON_LINUX_CI=run to opt in.
Platform release producers, signing, package contents and publication smoke
checks remain required for every advertised platform package.

Observed v0.355.13 costs: original-client image download31.4 minutes and
image load5.2 minutes versus6.2 minutes of actual solve; macOS Intel native
compile23.5 minutes; method-learning replay15.4 minutes. Image-transfer and
duplicate compilation are the next optimization targets. Keep the original
coding assertions, exact source/image/binary provenance and single-attempt
effect ownership while improving transport. Do not replace a real solve with
a fixture or rerun an old effect owner to obtain a faster green result.

Artifact consumers now require a successful producer before allocating a
runner. Prebuilt test/project jobs request12GiB of available disk; heavy builds
retain their existing exhaustive cleanup. With an explicit reserve, cleanup
stops at the requested reserve instead of always deleting
all optional SDKs and pruning Docker. This avoids observed2-7 minute cleanup
steps on jobs whose normal hosted disk was already sufficient.

For a live dependency-updater check without opening another PR, dispatch
dependencies-latest.yml with create-pull-request=false (the manual default).
Every refresh, resolution and gate still runs. Scheduled updates and explicit
create-pull-request=true retain the normal single-PR publication step.

Automatic original-client admission ignores feature-branch workflow completions.
Successful main push checks and public-download completion still wake the full
readiness audit, so a slow required main check can qualify later. Explicit manual
candidate checks remain available; all immutable ownership and CI gates remain.

Changes to the release caller also refresh its retained canonical source and
checked projections through the maintained JavaScript generator. Preserve every
production release operation and keep the exact deployed-caller proof; never
relax source identity checks to accommodate workflow edits.

The former lint job also builds WASM and runs native gate programs. It is
retained for main/full PR opt-in; default PRs use fast-pr-lint with the maintained
JavaScript readability, file-size, CI-budget, requirements and status checks.
The changelog check uses its existing JavaScript twin to avoid rust-script
installation solely for that metadata check. Pipeline Status requires the fast
lint job as well as every other selected PR check.

Fast lint also checks every demanded package-data mirror with
node scripts/mirror-package-data.mjs --check. After editing canonical data, run
its --write mode in JavaScript and commit the generated bytes. This catches a
JavaScript/native bootstrap mismatch without a local or PR native build.

Visual replay/font/GIF export tests are temporarily opt-in with
FORMAL_AI_FULL_TUI_REPLAY=run; full agent replay also restores them. The original
assertions are retained with explicit capture/export deadlines. Observed main
export latency was 7.45 seconds against Bun’s implicit five-second test clock.
Ordinary readiness, paste/submission and PTY checks remain on; fast unit tests
reuse the actual captured startup frames. Skipped visual tests have no execution
credit. Restore automatic rendering after its cold startup cost is predictable.

VS Code package/WASM probes and the native full-source-content example are also
retained behind the full PR profile. Production extension publication is
unchanged; the full source-content workflow remains available on explicit
dispatch. Their original build payloads are included in the retention check.
