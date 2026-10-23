# Release blockers: setup deadlines, security audit, and owned cleanup

After PR #1401 merged, default-branch Coverage run
[39971329880](https://github.com/link-assistant/formal-ai/actions/runs/39971329880)
cancelled browser coverage shard 1 at its 15-minute job limit. The search-adapter
setup step ran from 15:02:39 to 15:16:34 UTC on 2026-10-23 without output.
Its full log has SHA-256
`1988ec9520dc888237103a9bde5905a10bca48aaf4d6a601042bd8f618788f12`.
Coverage never started; the missing coverage artifact was a consequence.

The installer used an unbounded, quiet APT index update when ripgrep was absent.
The log does not identify the stalled mirror or distinguish the two quiet APT
commands. Both commands now use the existing shared APT wrapper: two attempts,
60 seconds per attempt, a five-second delay, and the existing official Ubuntu
HTTPS alternate after a deadline. Signed package verification remains enabled.
The installed command is probed under a separate ten-second deadline. Both
coverage callers have a three-minute step backstop within the original job cap.
No coverage test, inventory, threshold, or required result is skipped or waived.

The regression executes the actual installer with fake process boundaries and no
network or package installation. It covers an existing adapter, successful
installation, transient index failure, a stalled update and alternate recovery,
permanent failure exhausting exactly two attempts, and a failed version probe.
The transient-failure test fails against the original installer. The stalled
fixture exercises the shared process-group deadline rather than simulating its
return status.

## Security audit discovered during follow-up validation

The follow-up's Cargo audit job
[120497996615](https://github.com/link-assistant/formal-ai/actions/runs/39978123387/job/120497996615)
reported Tokio 1.53.2 under
[RUSTSEC-2026-0386](https://github.com/RustSec/advisory-db/blob/main/crates/tokio/RUSTSEC-2026-0386.md).
Update both direct requirements and both committed Cargo lockfiles to the patched
1.53.3 release, including the older experimental reproduction's Tokio 1.53.1.
Its checksum comes from the public crates.io index; the old and new releases
have identical dependency, feature, and Rust-version metadata. No other locked
package changes, and the advisory is not ignored. Native builds and the real
Cargo audit validate the update in CI; no crate is downloaded or compiled locally.

The dependency update also requires refreshing the governed source witness.
[Browser shard 4](https://github.com/link-assistant/formal-ai/actions/runs/39990184256/job/120545719484)
correctly rejected its stale Cargo fixture hashes. Regenerate through the
maintained explicit review command with the exact reader and previous-witness
hashes. Full object comparison permits only the reviewed Cargo content-hash
references to change; all cases, assertions, profiles, scope, counts, other
source identities, and the reader remain identical. The guarded test fails before
this refresh and all its controls pass afterward. No guard or test is skipped.

## Split release history before it exceeds the maintained file limit

[Lint lane 1](https://github.com/link-assistant/formal-ai/actions/runs/39990184167/job/120555670495)
correctly rejected the generated ledger at 1,506 lines after the version commit
appended another release. Its full log has SHA-256
`ffcb2e8f00265f91f22ad632beb110e9d99dd5204047f6824c7acb16827e8776`.
Generated committed data remains subject to the existing 1,500-line limit.

Split the historical release records into explicitly referenced parts without
changing a record or its order. The metric reader loads the parts before current
records. The appender rotates complete records whenever another append would
cross the limit; the newest record stays in the root for status projection.
Existing parts are immutable, archive references are canonical and bounded,
nested references and symlinks are rejected, and an interrupted publication
cannot silently overwrite history. The version producer stages the parts; its
freshness and published-source checks admit only the exact numbered `.lino`
metadata paths. Source files and unrelated paths retain their original guards.

The JavaScript storage twin performs the initial lossless split and exercises
hundreds of future appends, invalid references, size bounds, and interrupted
writes without compiling Rust. Native controls run against the Rust storage
implementation in CI. The initial split preserves every historical record line
byte for byte, and generated status surfaces are refreshed through their
maintained renderer.

[Browser coverage shard 7](https://github.com/link-assistant/formal-ai/actions/runs/40006276794/job/120596696326)
also caught the new archive root's underscore spelling under the existing
notation ratchet. Its full log has SHA-256
`336794f529ecb2d5aaf24457e63039390967069094b1411bf8aae6701d5f8f43`.
Use `self-hosting-ledger-archive` in the generated part and both storage readers
and writers. The historical records remain identical, and the existing notation
ceiling stays unchanged. The notation check and storage controls pass locally.

The first native compile after extracting storage correctly rejected the old,
now unused `std::fs` import under `-Dwarnings` in both imports of the metric
module. Remove that import; retain warnings as errors. The full question-gate
log has SHA-256
`6cee9d4bf0c4aedf813a3c97942f5590bb904d820dfd3b42b1173503896510a9`.

## Bound checkouts and include the required release fragment

The follow-up's changelog gate correctly found no new fragment after the base
release consumed earlier fragments. Add a patch fragment describing this PR's
actual fixes. JavaScript tier 8 and native coverage shard 1 independently reported
the same stale witness and oversized ledger already addressed above; their full
logs introduce no additional engine failure.

The Intel Mac producer was cancelled after remaining in the protocol checkout
step, with no compilation step reached. GitHub returns HTTP 404 for its log, so
the underlying infrastructure cause is unconfirmed. Its downstream
[CLI job](https://github.com/link-assistant/formal-ai/actions/runs/39990184348/job/120570130735)
then correctly refused the missing same-run native artifact. Its full log has
SHA-256 `d04041d9e9815c3d6f410410c09dd5654cebac9e44bb4b98b46e7d2c276ba8f5`.
Apply explicit five-minute deadlines to desktop workflow checkouts, preserving
all source refs, credential settings, fetch depths, job caps, and artifact guards.
An automated parsed-object comparison permits only the deadline changes. The
deployed-workflow regression covers every producer and consumer checkout.

The first two-minute backstop proved too short for this repository's 73,608
files. Windows ARM reached 100% of worktree population and Intel Mac reached
83% while still making progress. Their complete logs have SHA-256
`cd95ee6be67632812a91de44771cafb1ab2f11deda79b60d53bada4104f2294e`
and `0baad21d55a9b48cc299f83bfe9503452b0d4386a5f287dc9f703560964561c8`,
respectively. Five minutes allows this observed checkout work while retaining an
explicit step deadline and the original job caps; no platform or artifact
requirement is skipped.

## Repair the unavailable benchmark citation

[Check Links](https://github.com/link-assistant/formal-ai/actions/runs/40000756136/job/120575972934)
exhausted its three bounded direct probes of the Stanford sentiment-treebank
website. Its full log has SHA-256
`26715fd1bee29f06134036eb530796d2ea435cb7cac6049f3b2aa0c023a7ea62`.
The other failing website recovered with HTTP 200 during the same job.
Replace the unavailable sentiment benchmark citation with Stanford NLP's
maintained public `sentiment-treebank` repository, which documents its binary
labels. Keep the link checker and its finite recovery limits enabled.

The benchmark registry is the authority for this generated document. Update the
same URL in `data/meta/llm-task-classes.lino`; the registry and rendered document
then differ from their checked predecessors only by that matching URL. The
unchanged Rust generator checks the full projection in CI.

## Keep script tests in their real compilation context

[Lint lane 1](https://github.com/link-assistant/formal-ai/actions/runs/40006276763/job/120607372614)
reported a redundant standalone invocation of the new Rust library helper.
Its complete log has SHA-256
`6d6a5f9799a3ab069010815ed8026b8b45ff1501d97083cfa3e63c9d1b531342`.
`rust-script` wraps a library without `main` as an expression; the helper's
nested tests then cannot see its functions. Those tests already compile in both
native metric consumers. Extend the existing direct-helper discovery to native
test consumers as well as gate consumers, retaining standalone suites. A
synthetic filesystem regression executes the actual selection shell without
compiling Rust. Clippy's three string-allocation findings use direct appends or
formatted writes, preserving emitted bytes and warnings as errors.

[Browser coverage shard 8](https://github.com/link-assistant/formal-ai/actions/runs/40006276794/job/120596696311)
also flagged a long metadata-policy predicate; split its expression into readable
lines without changing its conditions or adding an exemption. Its full log has
SHA-256 `cb12c4f6505a8b7a19e0eb465f94b89e60e6e2630fc3789b7cc7dde248c6da18`.

## Recheck queued cleanup plans before publishing a claim

Main's finalizer run
[39982797815](https://github.com/link-assistant/formal-ai/actions/runs/39982797815/job/120519109289)
failed before issue writes because two immutable claims existed for the same
owner. Its full 29,383-byte log has SHA-256
`5d282cf8942e7a6afa5dfc469f578debd1d8f7d79735eeacb4aaf433c57a4d93`.
The claims were published at 16:08:41 and 16:19:43 UTC on 2026-10-23.
The second run finished preparation at 16:08:10, before the first claim existed,
and its sweep did not start until 16:19:11.

Preparation ran before the sweep acquired per-owner job concurrency. A queued
sweep then materialized its old plan without checking whether another run had
already claimed the owner. The apply-time uniqueness assertion correctly refused
the duplicate; weakening that assertion would permit uncertain repeated writes.

Revalidate the exact queued claim, producer, policy, immutable owner digest, and
current claim census inside the serialized sweep before publishing anything.
An already claimed owner produces an explicit pre-claim decision and
no new claim or cleanup request. Claim upload, apply, and terminal-outcome upload
all require fresh eligibility. The original apply-time uniqueness and uncertain
write quarantine remain enforced. An incomplete cleanup receipt is retained
instead of being labelled completed. Existing ambiguous claims remain quarantined;
this fix does not delete history or grant another mutation attempt.

Regressions interleave two producers that prepared before either published,
verify the second cannot publish a duplicate, reject changed queued identities
even after quarantine, and evaluate the actual workflow's effect conditions for
both eligibility outcomes. Existing ownership, interrupted-write, and complete
closure tests remain required. All validation uses fake transports and local
temporary files, without writes to real GitHub issues.

## Bound checkpoint discovery independently of accumulated history

Prepared source `43671c8` failed cleanup preparation in
[run 40001125369](https://github.com/link-assistant/formal-ai/actions/runs/40001125369/job/120577171246)
because its daily progress journal exceeded 256 artifacts. Its complete log has
SHA-256 `9208c6d0fc3b34d1f90ca202e9fa5a3a5ac7ac6c95015e74e84f641fde4094f7`.
Every completed admission could start another scan, including cancelled no-op
admissions; enumerating the entire daily checkpoint history eventually exhausted
a fixed cap even though each individual cleanup scan was bounded.

Read at most one 100-artifact candidate page for each of the two existing day
names. Authenticate the complete artifact, finished producer, trusted source and
policy before using a continuation. A continuation only guides discovery:
independent owner authentication, unique immutable claims and apply-time custody
still authorize every write. Existing history with no authenticated candidate
fails closed; malformed or duplicate windows cannot pretend the history is empty.
Do not start an event-driven scan for cancelled admissions. The scheduled sweeper
still handles interrupted owners, and failed or successful real deliveries still
trigger scans.

Regressions replay a thousand checkpoints, retain the authenticated continuation,
fall back from an invalid newest candidate, reject wholly untrusted or malformed
windows, and exercise the actual workflow condition. No test writes to GitHub.


## Reuse evidence-based recovery for package publication propagation

Both Agent CLI legs failed during installation, before tests, when official npm
metadata resolved `ai@7.0.152` but its tarball returned HTTP 404. The Docker build
failed on the same URL. The full job logs have these SHA-256 digests:

- Agent scenarios, job `120613341313`: `799a92d0a0231bb667686e6ac02e5b6621c4144ad5e9fdc357a442e5ba67f0b1`.
- Agent generalization, job `120613341485`: `14d217f2062a141880ebf26bbcf02e48a87ab7cccdf9c06fd8505fab0a6ffc22`.
- Docker build, job `120613340569`: `dd1eb568c8fa26d9f604d14a8f28c76bf92e7e82a7605003c75c434cdc0b1ddb`.

A subsequent metadata-only probe observed the exact tarball returning HTTP 200.
Its Last-Modified timestamp was 19:41:17 UTC on 2026-10-23; the agent installation
failed at 19:41:34 UTC. This is evidence of publication propagation, rather than
an unavailable package version that needs a dependency override.

Reuse `install-client-registry-recovery.mjs` for the Agent CLI jobs and both
container runtime installers. It permits only one identical retry after exact
package/version/tarball/integrity metadata and an HTTP 200 HEAD from official npm.
Both attempts and probes share the existing ten-minute deadline. Unknown errors,
authentication failures, unavailable tarballs, timeouts, overflow, and uncertain
process cleanup remain failures. Project installs preserve the existing manifest,
provider pins, and lifecycle trust; global E2E clients use the observed official
Agent CLI version `0.27.0`. No test or runtime probe is skipped.

Container tests stage the actual Docker COPY declarations and execute the
installer with a failing stand-in package manager. This checks its complete
source import closure and exit propagation without building an image or
installing dependencies. The fixture also exposed a CLI entry guard that silently
did nothing through macOS's temporary-directory symlink; compare real paths so
that invocation executes and preserves a failed install. A non-file import host,
including JavaScript stdin, must return false without an install or path error.
The stdin regression fails against the unguarded real-path lookup.
No product downloads or
local Rust builds are needed for these controls.


## Allow complete public app transfers their own bounded deadline

The produced v0.355.5 native packages and public signing metadata verified, but
[public Intel Mac verification](https://github.com/link-assistant/formal-ai/actions/runs/40017610647/job/120634705715)
failed before renderer startup: copying the application from its read-only DMG
with `ditto` exceeded the helper's generic 30-second subprocess deadline.
The complete 36,491-byte log has SHA-256
`f8eaeba4eb3f07a23af59de48ad989d71d3edef4979d94ec42143860caba7145`.
The signed DMG is 442,462,575 bytes, including a 258,399,440-byte native component.
The successful producer's installed-DMG smoke step took about 48 seconds, whereas
the independent consumer treated this transfer like a small metadata command.
The independently downloaded Mac ARM and browser packages passed.

Give the two `ditto` operations a single 180-second attempt each, while retaining
30-second metadata probes and the existing 15-minute hosted job cap. Do not retry
a partial extraction, change signatures, remove quarantine, or weaken checksum,
native-identity, deep-signature or rendered-UI assertions. Log command identity,
deadline, elapsed time, status, signal and error code without arguments or
credentials. A virtual process driver covers a transfer beyond the old deadline,
permanent failures and deadline exhaustion without copying or executing products
locally. All 63 public-download/Mac/readability controls pass without skips.

The failed v0.355.5 consumer remains failed and receives no full-download
acceptance credit. Verify the actual next release with the corrected consumer;
an unrelated old success cannot stand in for that source-bound result.


## Keep historical assertions complete after ledger rotation

[Specification shard 1](https://github.com/link-assistant/formal-ai/actions/runs/40014890861/job/120636778854)
failed its historical v0.296.0 assertion because it read only the newest ledger
file. Its full 298,826-byte log has SHA-256
`5f0c1741b0917f3f25223a5013838175747a4b46871eb44795b32639a9758676`.
The archive-aware production reader still finds that exact baseline and its zero
percentage; all historical release rows retain metric version 3. This is a test
consumer that did not follow the storage change, rather than deleted evidence.

Include the shared production ledger reader once at the unit crate root, and use
it for the historical specification, all-row metric-version assertion, and
self-hosting evidence assertion. Keep every existing assertion; header-only and
newest-status consumers can still read the small root file. The reader's inline
storage controls are also included in ordinary unit selection. Seven JavaScript
storage and script-selection controls pass, and the maintained JavaScript twin
verifies the baseline and every actual historical row without compiling Rust.
The native assertion changes compile and run only in CI.


## Keep Docker and matrix deadline contracts aligned with executable configuration

[Full shard 6](https://github.com/link-assistant/formal-ai/actions/runs/40014890861/job/120636778744)
reported two contract failures: its Docker assertion still named the old flat
helper COPY layout, and its timeout assertion could not parse the existing
`${{ matrix.capmin }}` job caps once checkouts gained explicit step deadlines.
Its full 487,189-byte log has SHA-256
`e407f467b8cf22073c0b396c334715f26ed4daa8c84b65ea5c7be52155fb9467`.
[Coverage shard 17](https://github.com/link-assistant/formal-ai/actions/runs/40014890534/job/120631351532)
found the same Docker assertion; its full 575,860-byte log has SHA-256
`4c028e380d7415c0a1f7d42fb47b1f64dfd6cb37a6ea97d347e80e9fe0e12afe`.
The real Docker build, runtime and released-to-candidate memory-upgrade job
`120636778212` passed on this exact source.

Update the Docker contract to require the actual recovery script, its complete
source import closure and its installed entry path. Share literal job-cap lookup
and resolve the minimum across every row of the maintained literal include
matrix. Both deadline sweeps retain their 70-percent margin. Missing, zero,
non-numeric, incomplete or unsupported matrix declarations fail closed; no job or
step clock is increased. Native negative fixtures cover misleading quoted fields
and a shorter matrix leg. An independent parsed-YAML control checks every desktop
step against every real leg. All 15 targeted Docker/deadline/readability controls
pass locally; the Rust assertions compile and run only in CI.

Deferred under the release-first scope: a wider exploratory parsed-YAML audit
found existing 25-minute container steps under 30-minute jobs. Those steps use a
different YAML indentation that the historical text-only sweep does not cover.
Their measured deadlines and finite job clocks are retained. A later headroom
review must cover that syntax and choose budgets from actual timings; this PR
does not claim complete parsed-YAML headroom coverage for all workflows.


## Update the compatible browser-tool chain for the live Zod advisory

[The next lint run](https://github.com/link-assistant/formal-ai/actions/runs/40022351344/job/120652744467)
passed its Rust stage but failed the JavaScript dependency audit. Its complete
610,876-byte log has SHA-256
`f536385e53129dceb97ec305456a5080a447de904e1bff36922f2c5b78c20d33`.
The official [GHSA-mrhx-c7xc-m3w5](https://github.com/advisories/GHSA-mrhx-c7xc-m3w5)
metadata marks Zod below 4.6.6 vulnerable to memory exhaustion and names 4.6.6
as the first fixed version. Both desktop and VS Code locks held Zod 3.25.76
through chromium-bidi 17.0.2, which explicitly requires Zod 3.

Do not force Zod 4 into that incompatible parent. Pin Puppeteer and its core to
25.13.0, whose declared mapper supports Zod 4, and regenerate both lockfiles with
npm's lockfile-only resolver. The old and new Puppeteer releases declare the same
Node >=22.12.0 requirement. The selected mapper resolves Zod 4.6.7. Exactly ten
related browser-tool package records change in each project; all other package
records, project versions, unrelated overrides and manifest fields are identical.

Both regenerated locks return zero findings from the live npm audit. Official
registry HEAD requests confirm all ten new tarball URLs return HTTP 200. Resolution
used isolated temporary projects with package-lock-only and ignore-scripts: no
node_modules, package archive fetches, browser downloads or lifecycle execution.
The temporary npm caches were removed after preserving metadata and audit proofs.
The advisory threshold remains enforced, without suppression or an audit waiver;
actual updated desktop/browser/VS Code runtime validation runs in CI.


## Refresh reviewed source fixtures without changing governed assertions

The archived-ledger assertion and the security manifest updates also change
source inputs pinned by the governed specification witness. A local run caught
this missing regeneration before the next full native cycle finished. Invoke the
maintained explicit reviewed-baseline generator with the exact preceding witness
hash and unchanged reader hash. Do not use the narrower version-only refresh for
these source changes.

The complete object comparison permits only exact full-file input text and
fixture identities for the reviewed desktop/VS Code manifests and unit module,
plus the changed specification source identity. Case IDs, asks, checks, counts,
unsupported reasons, reader source and every other object field remain exact.
The initial over-narrow one-hash check rejected the additional captured manifest
inputs and restored the preceding witness before review; no partial snapshot was
committed. The final strict comparison accepts only those declared source inputs.

The maintained witness changes from SHA-256
`2b3a94e7819d64e9ac2f81e5e5949698452394379a00fb5e0a4e7ccad7ae2de2`
to `10629c27ecdf419b632188de6a74cae41c602431561362e78f68ccb2801aa1ba`.
All 43 governed-source and version-only maintenance controls pass without skips.
The version-only guard still rejects changes to cases, assertions, unsupported
scope and native source; this explicit review does not relax that guard.

## Accept equivalent quoted workflow references in the native contract

[Coverage shard 11](https://github.com/link-assistant/formal-ai/actions/runs/40027608553/job/120672887040)
failed the desktop cold-build contract because it searched for an unquoted YAML
reference after checkout settings were compacted into a flow mapping. The actual
checkout still pins `${{ github.workflow_sha }}`. The complete 598,734-byte log
has SHA-256 `7804c4342b2beaa87f1479dd44e607000a9f361621f61106ad9ce64c31c30057`.
Accept that same reference in plain or double-quoted scalar form. Keep the
independent parsed-YAML checks for the exact reference, disabled credential
persistence, shallow history and protocol capture before source checkout.
Those workflow and deadline controls pass locally without Rust compilation;
the native assertion remains enabled and runs in CI.

## Resolve the additional live JavaScript security findings together

[The fresh lint audit](https://github.com/link-assistant/formal-ai/actions/runs/40032107333/job/120686777005)
reported two more advisories. Its complete 613,965-byte log has SHA-256
`4a7dd2355f3a5de741ba60e692a60784b7dabca583bec6be87b59b948f219fbd`.
[GHSA-27r3-jxmh-p828](https://github.com/advisories/GHSA-27r3-jxmh-p828)
requires body-parser 2.4.0 on the existing major version.
[GHSA-jr5w-w6x2-84p9](https://github.com/advisories/GHSA-jr5w-w6x2-84p9)
requires Electron 44.9.1 or newer on the existing Electron 44 line. The official
npm resolver selects body-parser 2.4.0, Electron 44.9.3 and qs 6.16.1 without
changing either package manifest or any engine requirement.

Review the full lockfile objects, not only the audit summary. The first candidate
was rejected because body-parser's new qs constraint also needs its locked
dependency updated. The final candidate changes three desktop records and two
VS Code records; every other lockfile field remains identical. Published metadata
authenticates the exact versions, tarball URLs, integrity values and dependency
constraints. Every direct body-parser dependency satisfies its published range.
Both live lockfile-only audits report zero vulnerabilities, and all three
official tarball HEAD requests return HTTP 200. No package archives, Electron
binaries, browsers, lifecycle scripts or Rust builds run locally. The audit
threshold stays enforced; real installation and runtime validation remain in CI.

## Retain exhausted macOS runner failures without acceptance credit

[The preceding ARM Mac package job](https://github.com/link-assistant/formal-ai/actions/runs/40027608647/job/120677625775)
exhausted its existing three attempts with disk-image service failures:
`hdiutil` reported `Device not configured`, followed by dmgbuild's empty
`Unable to shrink` diagnostic. The complete 3,736,740-byte log has SHA-256
`4173056a2342322fbee09a98a80e304537f21fa6837be99288456afe757a8dd5`.
The maintained wrapper already recognizes those exact signatures, preserves the
job deadline and removes only incomplete top-level DMGs before another attempt.
Do not increase attempts or relabel the failed job. The corrected source receives
a fresh normal packaging job, and that job must pass before merge.

## Avoid full protocol-only checkout transfers on native consumers

[The next Intel Mac package job](https://github.com/link-assistant/formal-ai/actions/runs/40032107489/job/120693006129)
timed out while fetching the workflow protocol commit, before package installation
or compilation. Its complete 14,701-byte log has SHA-256
`068e7b46a3ee9b3c9f4eaa69a1dd496b9820797643a8d992e3b5a002f8520905`.
The ARM Mac package job on that same source passed, so the preceding exhausted
disk-image attempt is retained as a failed run rather than a current-source waiver.

Capture protocol scripts through a shallow, blob-filtered sparse checkout in
native producers and desktop, CLI and VS Code consumers. Include both script
directories required by the unchanged capture commands. Leave the following
full source checkout unchanged: checkout v7 disables the earlier sparse selection
when no sparse input is supplied. Bound stalled Git HTTP transfers with the same
low-speed settings already used by the release preflight. Keep every checkout
deadline, immutable reference, credential setting and release assertion intact.

Apply the change through parsed CST ranges, preserving surrounding source and
comments, and compare the complete parsed workflow object against the declared
delta. A small local Git fixture proves protocol files remain available while
build inputs are omitted initially, then restored byte-for-byte after disabling
sparse checkout. Workflow and deadline controls pass without compiling Rust,
building images or contacting GitHub from the fixture.

## Audit every lockfile before publishing a security batch

[The next audit](https://github.com/link-assistant/formal-ai/actions/runs/40036580050/job/120700635386)
confirmed the npm fixes but found the same body-parser advisory in the root
`bun.lock`. Its complete 612,562-byte log has SHA-256
`7eccfc2663cb579323d8b4313a9fa20798036e26689c8c6c2b6d5ab093a9b1bf`.
The preceding local validation covered the two edited npm locks and missed this
corresponding Bun resolution. Run the maintained whole-repository audit gate
before publishing security changes rather than inferring coverage from individual
package audits.

Use Bun's lockfile-only resolver with scripts disabled. Review only the root
body-parser 2.4.0, qs 6.16.1 and nested content-type 2.1.0 records against official
package metadata. Every other lock field and the package manifest remain exact;
all direct dependency ranges are satisfied. The unchanged maintained audit gate
then passes for both committed Bun locks and all four npm locks. All governed
source controls also pass without changing their witness. No package archive,
product execution or Rust compilation is needed locally.
