# Essential PR checks and default-branch builds

Maintainer requirements of 2026-10-31 supersede earlier full-PR build opt-ins.

- PRs and checks dispatched on feature branches run essential Linux JavaScript
  unit, coding, question-answering, reporting, policy and security checks. They
  never compile Cargo packages, Docker images, WASM or applications, and never
  deploy Pages. The old FORMAL_AI_FULL_PR_CI variable cannot override this rule.
- Production builds run on the current default branch, main. Release-tag and
  workflow-completion consumers must independently authorize their source against
  main; a release event is not permission to compile arbitrary feature code.
- A failed compiler/build job stays red and blocks its dependent publisher. Fix
  main failures through a follow-up PR. Preserve source, producer, attempt and
  checksum verification when reusing outputs; never manufacture success receipts.
- Build each distinct source/target/feature/profile once where the artifact
  protocol permits reuse. Cargo package creation uses --no-verify after its
  required compiler producer; staged crate publication uses --verified-build true
  after authenticating both binary and package transfers. Standalone publishing
  retains Cargo verification. These flags skip redundant compilation, not source
  or archive validation.
- Keep the independent full Docker cold-build job, but select it only with
  FORMAL_AI_COLD_CONTAINER_REBUILD=run on main. Actual image publication still
  requires the native compiler and exercises the produced container. It no longer
  waits for a second independent compile of the same image.
- Keep browser E2E and box-language integration jobs as main-only opt-ins with
  FORMAL_AI_FULL_INTEGRATION=run. Critical agent coding/reporting checks remain
  selected on main. Existing coverage and real-GitHub effect controls remain
  opt-in. Skipped qualifications grant no execution or acceptance credit.
- Use complete CI failure evidence for offline regression tests. Existing PR
  fixtures cover browser startup, seed command evaluation, TLS connection retries,
  release response loss, job-observer errors, package boundaries and test-issue
  cleanup. Retain the corresponding real suites for explicit qualification;
  fixtures do not prove real transport or deployment success.
- Small PR unit suites share one JavaScript runner. The broad main inventory
  retains its measured eight-shard plan. Do not shard merely to add runners.
  Native test optimization remains on main: prior measurements found expensive
  tests slower without optimization. No Rust compilation or tests run locally.
- Documentation-only changes retain lightweight checks and no automatic release.

## Recorded timing baseline

The successful [main run 41123995486](https://github.com/link-assistant/formal-ai/actions/runs/41123995486)
spent 143 minutes in the pipeline. Initial validation took about 33 minutes,
package/release staging 71 minutes and subsequent multiarchitecture images 40
minutes. The optional independent amd64 image rebuild alone took 20.4 minutes.
PR checks across workflows took 2.6 minutes; repeated checkout/tool installation
was material compared with the checks themselves. These are one run's timings,
not averages or predicted savings. Compare future Actions job and step timings.

## Publication boundary

Publication spans several services and is not atomic. Crates and initial images
are published before the GitHub release; desktop builds and additional image
consumers currently follow that release. A later consumer failure cannot retract
already-public artifacts. Continue monitoring every configured consumer and fix
failures; do not describe initial release creation as complete delivery. Moving
all platforms before stable-release promotion requires a separate orchestration
change, not suppressing an existing failure or weakening source authority.

The first PR #1478 execution ran the complete selected unit-test step on one
runner in 77 seconds (job 124478342329). Workflow lint spent 126 seconds in a
full checkout (job 124478292166); its follow-up checkout is narrowed to .github
and the real ShellCheck canary fixture, with blob filtering and a three-minute
step deadline. Lint still examines every workflow/action and exercises that
canary. These observations do not predict release-build speed.

## Complete governed source fixtures

A fixture digest alone does not refresh a governed program. The program can also
cache the source text in its initial values. After a source fixture changes,
regenerate the complete witness with the reviewed source-reader command and
compare the full objects. Preserve native source hashes, case identities and
assertions; review each derived-data change against the actual changed file.
Keep the complete object/source consistency test in the fast PR inventory. It
parses source in JavaScript and does not compile or execute Rust. This essential
check prevents a later main-only failure that the cheaper digest check cannot
find. Local verification runs this one file with a bounded heap and deadline.

## Derived CI budgets and registry identities

Compute the conservative PR timeout bound from current workflow definitions and
compare it with the existing ceiling. Do not maintain a second copy of the
computed maximum, workflow or dependency chain. Actual Actions step/job timings
remain separate observations; a declared timeout allowance is not runtime.
Unknown timeouts, malformed dependencies and cycles must still fail closed.

Dependency currency checks identify the publisher behind npm aliases while
looking up installed versions under the original lockfile key. Local file,
link and workspace dependencies have no registry latest version; continue
checking the registry dependencies declared inside those packages. Automatic
updates must preserve the alias and its range operator. Keep the small shared
Rust/JavaScript identity fixtures in fast PR checks and native gate tests.
