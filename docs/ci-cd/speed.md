# CI speed: small parallel jobs, longest first

The [default-branch build policy](default-branch-builds.md) supersedes older
full-PR restore instructions. PR admission uses JavaScript; selected JavaScript
unit regressions share one runner. The full main suite retains its measured
shards. Rust tests are not compiled in the PR profile, so release optimization
settings cannot increase PR feedback time. Main retains its measured test
optimization: earlier CI observed 52.86 seconds unoptimized versus 9.34 seconds
optimized for one computation-heavy test. Do not trade a longer main test run
for a guessed compilation saving.


CI is only useful while it is quick to iterate on. PR #1188 sets the rule
and a gate that enforces it.

## The rule

Issue drafting resolves current Formal AI and Hive Mind source once per attempt
and shares exact source identities between jobs. Keep the single Formal AI CI
build and select the matching versioned Hive Mind image without a stale-image
fallback. Log versions and hashes; retain failure evidence and respond to the
issue even when initial source discovery fails. CLI integration uses the latest
published Hive Mind package. Frozen historical replays remain explicitly
historical, not current-runtime qualification.

Native release import materializes its authenticated offline worktree once.
Verify the commit, tree, lock bytes and native-input bytes after restoration;
do not repeat a full checkout-index rewrite. Keep complete unfiltered history,
use at most four checkout workers, bound each history command to four minutes
and the restore transaction to six, and give the workflow import step a
seven-minute backstop. Tests trace actual Git operations against small offline
fixtures; all Rust compilation and package execution remain in CI.

Decorative live badge outages must not block coding and release delivery. Keep
exceptions anchored to the exact image URL with the observed CI evidence; keep
checking its artifact destination and other URLs. A badge exception grants no
publication evidence and must not accept HTTP 408 globally or expand retry limits.
The link workflow runs the small real-HTTP timeout and missing-page regressions.

Keep main stable throughout production publication, not just version preparation.
The release checks freshness again after compilation. Continue drafting fixes
while CI runs, but hold routine merges until publication and the release consumers
are verified. If a critical fix must supersede a failed release, let the fresh
main run deliver it; do not weaken source checks or retry the superseded run.
See [the release handoff guide](release-handoff.md).

Documentation-only automatic changes use the policy in
[`scripts/documentation-only-ci-policy.mjs`](../../scripts/documentation-only-ci-policy.mjs): skip
heavy workflows and automatic releases, retain lightweight checks and explicit
manual controls. The bot dispatch action observes the same policy. A pending
changelog fragment cannot override it. Mixed source changes and unknown runtime
Markdown retain normal CI; GitHub's large-diff limits can reduce filtering savings.
See [the requirements](../requirements/issue-1455-documentation-and-evidence.md).
The separate provenance performance qualification and real Agent CLI
failure-report replay are retained as main-only manual qualifications; their native
compilation does not run in the default PR profile. Default deferral is recorded
explicitly and provides no execution credit. Production compilation and native
tests still run on main.

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
retained for main. FORMAL_AI_FULL_PR_CI no longer restores builds on PRs or
branch dispatches. Main retains production builds, native tests, critical coding
integration and public-download checks. Browser E2E and box-language integration
are retained behind FORMAL_AI_FULL_INTEGRATION=run on main. The profile notice
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

Desktop dependency collection excludes the private parent repository package.
The renderer already lives in dist-web; copying the parent also copies native-bin,
browser downloads and desktop/release back into the application. The captured
Windows NSIS failure is replayed by desktop-package-boundary.test.mjs in fast PR
checks. Hosted packaging also checks those paths with its actual electron-builder
matcher, then audits the real ASAR directory (including unpacked entries) before
creating installers. Keep required runtime dependencies and explicit browser/bin
resources. An oversized archive is a content defect, not a retryable transport error.

The fast profile also checks governed specification fixture and native-source
digests directly against current files. When a referenced configuration changes,
refresh its complete witness through the existing reviewed JavaScript generator
and inspect the full object delta. This cheap source-binding check avoids a main
cycle for stale embedded content; full reader and complete-object parity stays on
main. Do not replace source content with counters or remove the governed checks.

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
retained for main or explicit main-only qualification; default PRs use fast-pr-lint with the maintained
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
retained for explicit qualification on main. Production extension publication is
unchanged; the full source-content workflow remains available on explicit
dispatch. Their original build payloads are included in the retention check.

Release consumers reuse the digest-verified same-run complete source-history
bundle, as the CLI packager already did. Their pinned protocol stays separate;
restoration still verifies run/head/selection, the complete tree and native input
bytes. Published tag and default-branch ancestry are independently checked.
Detached source checkouts explicitly advertise HEAD during Git negotiation;
otherwise a checkout without named refs can download its entire history again.
The v0.355.15 native failures captured over 800 MiB of one redundant transfer
before its finite deadline. Tiny real-Git controls cover the detached state and
an advancing default branch, without native compilation or a large local clone.
Keep the original transfer and retry limits. Packaging console staging runs only
after a package operation is attempted, and missing evidence after an attempted
operation still fails. An unacquired hosted runner has no console to fabricate;
retain its structured capacity annotations and explicit unavailable outcome.
Source-transport changes must run native-release-workflow,
relocated-build-stage-controls and packaging-console-retention in the fast inventory. These preserve pinned
protocol ordering, complete-history custody, compiler/default-feature contracts
and packaging receipt checks without compiling native code.
