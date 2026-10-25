# Issue 1328 / PR 1329 handoff

Current work follows the [critical delivery scope](critical-delivery-scope.md):
unblock usable published releases and the three original HelloWorld tasks first,
while preserving all issue 1364 requirements and recording broader work as
unverified follow-up. Existing evidence, mandatory production/critical delivery
checks and all source proof oracles remain authoritative. The independent live
concurrency fixture is now manual-only and explicitly unproven; see the
[recorded deferral](critical-delivery-scope.md#recorded-follow-up). Its prior
strict-final Unknown remains a failed attempt, not a skipped check or a pass.

## Latest continuation: 25 October 2026

PR 1410 is now merged and v0.355.10 passed all main, publisher and public-download
checks, including independently checked Docker manifests. The fresh actual
three-client owner failed: Claude left Hive's 174-byte bootstrap placeholder, while Agent and
Codex exceeded their finite first-pass budgets while progressing. The next
batched [PR 1417](https://github.com/link-assistant/formal-ai/pull/1417) fixes placeholder scope and repeated native registry parsing,
with captured-request regressions. See the
[current causes, fixes and acceptance contract](released-client-progress.md).
The broad vision and issue 1364 are not complete; fresh released-client
acceptance and terminal authenticated cleanup remain mandatory.

PR 1417's first CI batch exposed file-size limits and a stale governed source
witness after registering the native regression. The correction moves request
timing into the existing logging module and reformats the captured Claude JSON
without changing parsed content. The witness refresh changes only the reviewed
`rust/tests/unit/mod.rs` source hash; cases and assertions remain identical.
Both JavaScript failures share that witness cause. The PR description retains
issues 1328 and 1364 using the exact pending-acceptance disposition. All tracked
files satisfy the size gate, 35 focused JavaScript tests pass and the native
specification generator remains current. Require fresh exact-head CI after the
batch push; native latency and released-client acceptance remain unverified.

The next batch also corrects a diagnostics dependency: the logging helper takes
the caller's trace flag instead of reading the CLI surface. Clock creation is
lazy and native-only; browser WASM never calls the unsupported standard-library
monotonic clock. Preserve the architecture ceiling and verify the actual native
request tests in CI. These fixes are manual and grant no autonomous authorship
credit.

The older H4 native run passed pure captured-request planning, but its real HTTP
regression timed out after 30 seconds despite planning completing in 716 ms.
The follow-up fixes a Rust/JavaScript parity gap: prediction-hit bookkeeping
must skip classification when memory has no prediction events. Rust invoked
the full offline solver there; JavaScript already short-circuits. Retain memory
and log recording, genuine prediction matching and the original HTTP deadline.
The drafted guard and captured-prompt test require fresh native CI; dispatch,
memory and dialog timings identify any remaining downstream bottleneck.

PR 1407 is merged; v0.355.9 and all configured release/public-download gates
passed. Real released-image acceptance then verified the native memory
lifecycle and default-token access, but exposed a post-admission proposal race
and missing Claude/Codex/MCP runtime prerequisites. The fixes were delivered by
[`PR 1410`](https://github.com/link-assistant/formal-ai/pull/1410) on
`fix/published-ci-proposal-admission`; see the
[exact causes, verification contract and evidence](released-client-prerequisites.md).
Issue 1364 remains open until a fresh real three-client execution and trusted
cleanup pass. No local Rust compilation or product download/execution is
permitted. The private untracked session export must never be read or staged.

PR 1410 also replayed the recent public Hive Mind client failures:
stdout-only source receipts, raw Git status whitespace, and missing hosted
checks for explicitly staged workflows. PR readiness in the staged transport
grants no CI credit; the original independent workflow execution and all
acceptance assertions remain required. The default-token hosted read-only
audit now proves owner discovery. Existing uncertain cleanup claims must not
be retried automatically. PR 1410's final reviewed head and main publication
passed; the next follow-up requires its own complete exact-head checks.

Its final blocker batch corrected an order-dependent Docker COPY test and cited
the existing Rust staged-readiness twins. It also preserved the solve budget
after the actual 28-minute image setup: only the delivery parent receives a
documented temporary 55-minute allowance, while the solve remains bounded at
21 minutes. Public-timestamp regressions cover the parent/child budget. Require
fresh final-head CI; neither earlier image success nor hermetic replay grants
the still-missing released three-client acceptance.

Keep native regressions outside production source. The staged-readiness
regression now uses the repository's supported external fixture registration,
preserving access to private helpers without shipping inline test bodies.

## Earlier release history

The original integration and subsequent release fixes are merged. Most recently,
[PR 1406](https://github.com/link-assistant/formal-ai/pull/1406) merged normally as
`fd721a2277c58932fa49c65652db06b29a8993f4` after all its checks passed.
Its version-preparation child is `55e2f7ea83ad127c6f77c820bee3fdb996498943`
(`0.355.8`). Release producers and public-download verification passed, including
full/slim multiarchitecture containers and both macOS architectures. Original
three-client acceptance is still unverified: hosted run `40139076853` failed
before client tasks because its lifecycle verifier expected an assistant message
instead of the actual linked task record. The focused follow-up on
`fix/released-lifecycle-and-ci-recovery` fixes that verifier and uses the real
JavaScript memory writer for its regression. See
[the cause, diagnostics and release evidence](released-memory-lifecycle.md).
The same [PR 1407](https://github.com/link-assistant/formal-ai/pull/1407) also fixes
the subsequent candidate run `40152800096`: all three clients selected the default
token but stopped before their claims because a false repository role flag was
treated as proof of denied push access despite the jobs' declared write scope.
A bounded, read-only Git receive-pack probe now establishes actual access when
metadata is not positive. No new credentials are required. This privileged
protocol executes from the default branch, so its hosted validation must follow
the corrected PR's green checks and merge; do not rerun the old effectful run.

[PR 1404](https://github.com/link-assistant/formal-ai/pull/1404) merged normally as
`af6e30e7828c0dfd1eb844235756949c610f5467` after all 16 workflows passed.
It removes the mandatory external credential requirement,
following the latest user direction and Hive Mind's default-token approach.
See [the updated cause and contract](hive-pre-effect-finalization.md). The default
token creates run-owned orphan branches, issues and PRs in this repository and
uses Hive's staged `ci/` workflow transport with `act`. Original task hashes,
two-pass/no-op and independent output verification remain mandatory. Cleanup
runs only after the complete owner workflow is terminal. The released
three-client requirement remains unverified until the actual hosted run passes.
Preserve all original bot commits and public evidence in normal history.
Before another merge, require green exact-head CI and let any active guarded
release finish; current `main` remains required throughout publication.

The merged PR 1405 follow-up used `fix/hive-cleanup-policy-history`. Cleanup preparation
after PR 1404 exposed a historical-policy rollout failure; see the
[captured cause and compatibility proof](hive-pre-effect-finalization.md#historical-policy-rollout).
The fix reads each policy's root list from its original immutable source and
caches failed lookups. The actual older CI progress hash is reproduced, and
249 related JavaScript regressions pass. Monitor main pipeline
[40097546818](https://github.com/link-assistant/formal-ai/actions/runs/40097546818)
through publication before merging this follow-up, then require its hosted
cleanup recovery and the actual three-client acceptance.
The same follow-up also fixes the observed three-minute native test artifact
download timeout in that pipeline by publishing a selected self-coding artifact
with the original receipt. The exact native replay remains required. The failed
main run has one bounded transport retry; no further retry is authorized by
this record. The permanent workflow change is projected from the canonical
staged-release source with publication operations and bindings unchanged.
That bounded main retry passed all agent-CLI legs, but automatic source
preparation then failed on an unavailable governed source read before changing
`main`. The same follow-up adds real HTTP status, redacted process diagnostics
and finite recovery for discovery reads under the unchanged deadline. Live
writer/job observations remain single-attempt and every source/lease gate is
preserved. Do not rerun the unchanged failed main run again; validate the fixed
source through the follow-up's CI and next default-branch run.
PR #1405 merged as `9739704f886447ef05f586be942b73e00e512ff5` after all 18
workflows passed (152 successful checks and 24 conditional skips). The paused
finalizer (`392761102`) was re-enabled immediately and verified `active`.
Multiple hosted discovery runs passed, and a read-only live historical-cursor
check authenticated the journal in 20 requests. A scheduled full sweep remains
to be observed; no workflow is intentionally left disabled.

The historical main run `40109825719` exposed a separate Codex TUI startup race:
the harness sent Enter while its model was still loading and inside the pinned
client's paste-suppression window. See
[`codex-tui-startup-race.md`](codex-tui-startup-race.md) for the captured evidence
and regression, delivered in merged PR 1406. Its hosted PTY regression and main
real grounded-client test passed, followed by release `v0.355.8`. Validate the
current lifecycle follow-up, self-merge only its exact green head after prior
publishers/effect owners are terminal, and monitor publication plus the original
three-client acceptance. Do not rerun the failed original-Hive workflow: its
first-attempt requirement remains authoritative. Work solo under the latest
owner instruction, without local Rust builds or product execution/downloads.

## Findings and implementation

| Failure | Implemented recovery / acceptance |
| --- | --- |
| The same `_web_run {}` declaration repeated with reused IDs and unknown errors | Preserve namespace identity and count actual declarations, canonical arguments, and the objective budget. Stop after three identical actions or 128 total actions. Unknown tools and absent results still consume attempts. |
| Search schema expects `input` while the planner supplies `query` | Project semantic aliases, validate required and explicit values, and emit a terminal incomplete result before sending an invalid call. Both wire routes share the boundary. |
| MCP `isError` disappeared during Responses ingestion | Retain the complete structured result object; failed/empty search consumes its repair rung and never becomes usable research evidence. |
| Prose `branch so` became a local refspec | Existing owned binding validation remains; current-branch recovery observes Git in the client workspace, refuses detached HEAD, and preserves explicit branch/remote intent. |
| PR 1329 claimed completion by executing a nested `solve` reproduction and reading a version-named file | Command-output capture now requires an owned imperative and a bound clause in the first paragraph. The narrative negative and genuine imperative positive are regression controls. |
| Unwritable logs produced repeated errors and memory IO appeared as schema incompatibility | Probe and disable the failed sink once, retain stderr, expose the real error through health, and keep original memory IO categories. |
| Repeated long-context normalization | Exact-input bounded cache: native process-wide and JavaScript source-host scoped. The early transcript stop avoids invoking expensive planning after exhaustion. The production address-only profile does not prove normalization was the sole CPU cause. |
| Other persistent retry paths | Three consecutive failed Telegram polls, dreaming runs and interrupted reads stop; real success resets the failure budget. Builder download settings, Pages polls and source fetching have finite configuration/deadlines. Healthy service loops remain cancellable. |
| CI evidence constructed `origin/` | Validate `FORMAL_AI_EVIDENCE_BASE_REF`; never override GitHub's reserved environment variable. Original authoring checks remain. |
| Native projection dispatch rejected PR 1329 | Authenticate the actual run and open same-repository PR through GitHub APIs; bind current head/base/merge witnesses and physical clean checkout before and after generation. Forged selectors still fail. |
| A fixture lease-start upload finalized with HTTP 404 | Three original uploads, unchanged names/paths/bytes/receipt IDs, no overwrite; a one-minute caller deadline fits the original five-minute job. Producer recovery also retains the original output ID binding. |

The CPU diagnosis must remain precise: the original native binary lacked a
published symbolic profile. Do not claim a measured 130-second normalization
speedup from an address-only profile. The new native long-context/reused-call
regression is compiled and timed by CI, not locally.

## Validation and automation

Source checks execute through the maintained JavaScript Formal AI planner and
real driver, with Node memory bounded. Rust compilation, native products,
containers and installers run only in CI. Generated worker factories,
JavaScript-to-Rust projections and TypeScript twins use their maintained
translation tools. Existing worker ceilings and translation ratchets remain;
no expected-output oracle was relaxed to produce a green result.

The worker manifest initially omitted the new generated cache factory. Formal AI
itself completed a real Read/Edit/Read insertion; an independent check compared
the entire manifest to the worker directory using the canonical normalizer rule.
This is a literal maintenance task, not autonomous invention of an algorithm.
The subagent also completed a derived trailing-whitespace task. Independent
semantic module creation reads its declared acceptance source and then honestly
reports `SearchToolMissing` in the source driver. Supplied controllers/bootstrap
code receive zero autonomous-authorship credit; do not attach false authoring trailers.

The full source duplication inventory still reports existing independent clone
candidates. The new modules have no independent clone candidates and generated
factory/twin proofs are checked without exemptions. This task does not establish
that the entire pre-existing repository is duplication-free.

See [CI observations and collector usage](ci-failure-collection.md). In the two
inspected Hive Mind runs, log publication recovered and the actual final outcome
was `Finished without changes`, exit 5. The first-error permission annotation
was misleading; do not claim a token/permission fix delivered source changes.
Formal AI's improvements here do not demonstrate delivery of those unrelated
Hive Mind issues by an autonomous source author.

## Resume / CI

Push the completed bulk to PR 1329 with normal Git commands, retaining the bot
history. Use automatic CI triggered by that push. Inspect every failure's actual
job log and fix source causes through the JavaScript Formal AI runner. Do not
manufacture a green result by dispatching unrelated runs, rerunning failures,
skipping original checks, or fabricating artifacts. Record exact final head and
actual CI conclusions when known; local source success alone is not native success.

Keep local disk use bounded. Do not read or publish unrelated local session
exports. Do not run Cargo, rustc, Docker, Electron or downloaded native products
locally. Installed rustfmt may format text without compiling it. Use up to three subagents under the latest owner instruction. Keep a dedicated
JavaScript Formal AI change lane and continuous CI monitoring. Coordinate source
ownership, freeze completed batches, regenerate projections once, and push the
reviewed integrated batch. Do not create child agents beyond this limit.


First automatic CI on `f2c8974a39437ae2bb7d2809a6f63552f5ffc736` caught
an omitted embedded package-data regeneration and new compiled Git-observation
literals above the existing ratchet. Recovery uses `mirror-package-data.mjs`
and shared observation rules in `data/meta/work-item-steps.lino`; it does not
raise the reviewed ceiling or relax the source-content check.


The first live original-Hive Claude run on `8249aa89960a8d54960a8c954a4232509dfe5850`
([job 118749500606](https://github.com/link-assistant/formal-ai/actions/runs/39456423351/job/118749500606))
proved successful publisher discovery, workspace installation and postcondition
probing of Kotlin, followed by the exact `Hello, World!` output. Delivery then
failed because the changed double-quoted recipe scalar retained `\"` bytes;
Git received quote characters in the branch refspec. The correction uses a
single-quoted meta scalar with actual shell double quotes and a real disposable
Git commit/push regression for both named and current-branch targets. The original
Hive task, setup discovery, and original output oracles remain unchanged.


Browser coverage shard 6 on `c3b9dab103f3d6bd743f60abdbab18422342076d`
([job 118765975389](https://github.com/link-assistant/formal-ai/actions/runs/39462450445/job/118765975389))
reported 16 failures with no IDs. The actual census was 205 extracted cases,
all 205 passing: 16 source-qualified cases had been refused because the generated
native binding registry still named the prior `rust/src/lib.rs` content hash.
Regenerate with `node scripts/lib/generate-native-specification-programs.mjs`,
then require its `--check` and all 221 original worker cases. Do not manually
replace hashes, lower the recorded passing floor or relabel the lost cases as gaps.
The diagnostic now distinguishes extraction/source-qualification loss from a
worker assertion failure and prints IDs even when the prior gap list was empty.

The next browser shards exposed two declarations whose differently named native twins already exist: `memoizedNormalization` mirrors `normalization_cache::normalized`, and `gitPushRecovery` mirrors `git_push_recovery::recover`. Formal AI adds path-qualified citations verified by the original citation and planner gates; the zero-missing ceiling remains. The release advertisement is regenerated from the complete source closure, adding the new native-session authentication dependency without changing publication targets. Record/scalar qualification refusals share the regenerated native registry fix above. The live original Hive Claude and Codex jobs on `c3b9dab` both passed with the corrected recipe, without changes to their original tasks or success oracles.

Shard 12 additionally found that the quoted-reproduction ownership guard rejected a genuine imperative task preface, allowing a literal-write fallback to capture the words “its exact stdout”. Command capture now accepts a preceding unquoted imperative task sentence in the same paragraph, while narrative, quoted-command and cross-paragraph prefixes remain refused. The original parent-directory test is preserved, with matching native and JavaScript ownership controls. New retry constants/local bindings are spelled in full words rather than raising abbreviation ceilings. Moving unchanged response types inward removed four solver-to-surface references; the dependency ceiling is tightened to the measured 34.

Shard 9 and the JavaScript tier require all new user-facing validation, repetition and refspec wording to use the existing message data tables. Their text is preserved byte-for-byte and all zero-prose ceilings remain zero. A command-output clause also refuses another run verb in its suffix, preventing a prior quoted reproduction from claiming the later command’s output.

The current original Agent job ([118768617314](https://github.com/link-assistant/formal-ai/actions/runs/39462450498/job/118768617314)) was cancelled by the 30-minute enclosing job limit: preparation took 10 minutes 30 seconds, the actual solve then ran for 19 minutes 43 seconds, and its final response arrived seconds before cancellation. The independent source compiler/output oracle and evidence upload were never reached, so this is not a pass. The initial mitigation used a 60-minute enclosing budget with a separate 25-minute solve cap; subsequent CI rejected that mitigation under the existing 30-minute speed policy. The final arrangement below supersedes it. There are no workflow reruns, extra task attempts, changed task inputs or relaxed output assertions. Native client requests still observed 65–99 second latency; the original address-only profile does not identify that remaining CPU hotspot, so no complete native speedup is claimed.

The remaining source shard found browser command-capture drift and a covered 277 KB exhaustion replay exceeding its unchanged five-second limit. The browser and JavaScript planner now call one source-owned structural capture contract with their existing lexical adapters; its worker factory is generated. The retry guard proves whether earlier tool calls exist before doing request identity/file-read classification; when none precede the current user request, that classification cannot affect the counted window and is avoided. Fresh file requests and continuations retain their existing controls. The old dogfood assertion’s literal `origin HEAD` command is updated to require the actual branch observation and original message/remote guard; real disposable Git commit/push tests continue to verify the effect.

Native coverage shard 3 retained an original unknown-command passthrough test and exposed provider-owned uppercase `OBJECT`/`STRING` schema types being rejected as unknown. Both validators now normalize only the declared type keyword while preserving the argument value and refusing real type/missing-value errors. Native lint also identified new prose; native and JavaScript diagnostics now use the same metadata text. The existing native wording helper reparsed that immutable table for every lookup; its shared seed reader now parses once and fills slots in one pass, matching JavaScript’s cached table. This removes repeated work but does not establish the unsymbolized production CPU hotspot by measurement.

The first current-head ratchet on `bd38849` ([job 118803310383](https://github.com/link-assistant/formal-ai/actions/runs/39473478587/job/118803310383)) found one new compiled paragraph predicate, measuring 440 against the unchanged ceiling of 439. Command capture now identifies completed empty source lines structurally, both before the owned command and after its quoted body. This also refuses cross-paragraph output capture with CRLF and whitespace-only blank lines, and preserves indentation on a legitimate next-line command. The shared JavaScript/browser contract and native reader retain matching controls; no ceiling or original five-second replay bound is raised.

A related schema-boundary review found that JavaScript compared object enum/const values by insertion order, and that the shared canonicalizer could lose an own `__proto__` key when copying into an ordinary object. Formal AI performs the literal source edits to use the shared canonical serializer, retain all own keys in a null-prototype dictionary, and resolve property schemas only from explicitly declared keys. The new controls cover object order, array order, distinct reserved-key arguments, and both declared and forbidden `__proto__`, `constructor`, and `toString` fields. All 64 targeted wire/retry/schema controls pass. These are bootstrap-directed source repairs; no autonomous semantic authorship credit is claimed.

Browser shard 11 on `62f8a9c` ([job 118818994067](https://github.com/link-assistant/formal-ai/actions/runs/39476386275/job/118818994067)) caught 26 newly introduced underscore message IDs above the existing notation ceiling. The correction derives exactly the new IDs against the real base commit, converts them to hyphenated names, updates their JavaScript/native consumers as a closed set, and regenerates the embedded and translated copies. Rendered message text remains the original contract and the notation ceiling is not increased. The original Hive Claude delivery on that head passed its independent verification; Agent and Codex were still running when this failure arrived.

The remaining Clippy findings on revision `62f8a9c` are repaired without suppressions: the normalization cache releases its lock before branching, private-module functions use consistent visibility, retry bookkeeping is const-capable, log health borrows its redacted input, and JSON array bounds compare in the unsigned 64-bit domain without truncating schema limits on 32-bit targets. Generated requirements and status surfaces are refreshed with the maintained JavaScript generators. All three original Hive delivery lanes passed on `62f8a9c`; the next revision still requires its own complete hosted checks.

A final retry audit found a real exception to the earlier listener-loop classification: repeated `accept` errors and connection-thread allocation failures could spin indefinitely. The TCP transport now uses the shared three-failure budget separately for accepting and dispatching connections. Healthy progress resets only its own budget, and exhaustion returns the final original IO error. Fault-injection controls run the production iterator/dispatch loop with bounded generators; native execution remains in CI.

The same listener gap also existed in WebSocket, WebRTC signaling and the logging proxy. All four entry points now delegate to the shared bounded iterator/dispatcher in `bounded_retry`; thread allocation uses fallible `Builder::spawn` rather than panic-on-failure. The four fault-injection tests exercise that production helper. No per-adapter retry implementation is duplicated.

Native shard 8 on `18a4287` found a missing generated module-map row for the new shared retry module and named-branch assertions that still expected the prior literal push spelling. The row is derived from the module declaration and its own documentation header. The branch tests retain their exact canonical command comparison and now explicitly require the named ref binding, its actual quoted push operand, and detached-HEAD refusal; disposable real Git commit/push controls remain mandatory. The same shard found nested redundant edit aliases crossing a strict schema: recursive Rust/JavaScript projection is repaired while preserving the original integration oracle and invalid-value refusal.

All three original Hive delivery lanes passed on published `18a4287`: [Claude](https://github.com/link-assistant/formal-ai/actions/runs/39482864340/job/118838409331), [Codex](https://github.com/link-assistant/formal-ai/actions/runs/39482864340/job/118838409190), and [Agent](https://github.com/link-assistant/formal-ai/actions/runs/39482864340/job/118838409154), with their original independent output oracles. All eight full source-content builds and three lint lanes also passed. This does not grant a pass to a later head.

The final Hive workflow separates binary build, one same-source image build, three original solves, and three independent compiler/output verifications. Every job is capped at 30 minutes, and each solve at 21, preserving the existing 70% step-to-job headroom rule. A streamed compressed image artifact is checked against the same run, source/tree, binary, image and adapter identities on load; complete staged evidence is hashed before transfer and checked before the original independent oracle. Final artifact names remain the original names. Hidden files remain excluded, credentials are resolved after image loading, and task compiler discovery is unchanged. Hosted transfer time and disk usage require the new head's actual CI result.

Native shard 5 found required command descriptions that could be grounded in the actual request but were refused. Both schema projectors now use that exact request only after their original default/identity/enum guards, preserving invalid explicit values for refusal. Agent CLI self-CST E2E separately found roughly 1.6 MiB `tool_call_bound` diagnostic records: complete CST arguments exceeded the maintained 1 MiB diagnostic reader budget. The repair bounds diagnostic metadata after redaction, with explicit size/digest/preview summaries; it does not widen the reader, skip malformed lines or truncate the actual CST/dialog artifacts. The JavaScript tier's other observed failure repeats the same Hive CI-speed violation.


The remaining native and coverage failures on `18a4287` repeat the required-description and named-branch causes above, plus two older generic-commit assertions and a held-out unmade-choice assertion. Generic commit controls must require the current branch observation, quoted selected ref and detached-HEAD refusal while retaining their canonical whole-command check. The held-out task must still refuse to invent visibility, and now also require no invalid client call and an explicit incomplete terminal response; returning a call missing a required field contradicts the current boundary requirement. Explicit-default and valid-call controls remain intact.

Server parity on `18a4287` had only one content divergence: the two isolated health responses reported their different actual temporary log directories. Each runtime now gets an explicit owned log directory. The comparator normalizes only that exact directory at the top-level `logs.directory` location; foreign directories, child paths, nested payloads, sink state, failures and protocol differences remain observable. The zero-divergence ceiling and original corpus remain unchanged. Source-only regression checks pass; native parity and the complete new-head workflow results still require hosted CI.


The final unsharded native suite exposed an upload step capped at four minutes within a five-minute fixture job, violating the existing 70% headroom invariant. Its step cap is tightened to three minutes; upload attempts and identity checks remain unchanged. The split Hive solve likewise uses 21 minutes within its 30-minute job. The three successful preceding-head solve steps measured 5m07s (Claude), 8m40s (Codex), and 15m26s (Agent); 21 minutes retains measured headroom without introducing a CI-speed exception. Matching JavaScript source controls now check both constraints before native CI.


The first automatic checks on `48f4aabb` found unsupported job-level `runner.temp` expressions for the transferred Hive image directory. As with the existing evidence directory, the bundle directory is now exported from the initial step through `GITHUB_ENV`; job-level environment declarations retain only supported context. Matching source controls preserve isolated paths and the original phase graph.

Cargo audit also received two newly published trust advisories: RUSTSEC-2026-0382 (rustls-webpki) and RUSTSEC-2026-0381 (webpki-roots). Official RustSec advisories and crates.io sparse metadata identify patched stable versions 0.103.16 and 1.0.10. Both tracked lockfiles are refreshed where affected, using registry checksums; dependency edges and features are unchanged and the declared parent requirements accept both patches. This is a registry-authenticated text-only update through JavaScript Formal AI, with zero autonomous-authorship credit. No local Cargo/native command is run, and no audit ignore is added; hosted Cargo audit and locked native builds must independently confirm the result.


## Critical coding delivery continuation, 2026-10-21

The owner requires actual delivery of the recent critical coding requirements,
not only requirement drafts. The existing Pending doctrine and parity rows remain
binding; no percentage over CI checks represents implementation completeness.

The JavaScript source-composition planner now interprets compact complete-source
requests and import aliases through general grammar/frame rules. The original
request and two archived compact requests pass the unchanged independent oracle.
A controlled original preimage fails an alias task with the same corrected
harness setup. Native counterparts and refusal tests are supplied for hosted
compilation. See the finite, lossless [composition evidence packet](../../../experiments/formal_ai_subagent/evidence/formal-ai-only-1188/compact-source-composition-1329/README.md).
This is a small qualified task family; retries are not independent cohort members,
and supplied repair controllers receive zero autonomous-authorship credit. Full
model input, token use, development savings and cost remain Unknown.

Complete source models now support a persisted declaration-qualified parameter
transformation contract, including match, captures, preconditions, refusal, exact
inverse and language facets. The shared rule preserves every unrelated concrete
node and independently reparses the result. The natural-language planner observes
the exact requested source and delegates to an installed authorized host. A real
repository self-edit and a hand-authored TypeScript edit in another Git repository
have full before/after CSTs, rules, inverses and physical-write receipts. The host
refuses generated owners, wrong reads, stale preimages and unsupported scopes.
Bounded candidate writes preserve the original on short/failed-write errors before
an optimistic preimage check and atomic replacement. Compiler validity, comment
meaning, arbitrary semantic equivalence and native planner parity are not inferred
from this finite alpha-renaming result. Refer to the existing complete doctrine
for the broader requirements still awaiting their acceptance evidence.

JavaScript deep concept grounding no longer throws when a supplied lookup returns
senses. It derives attributed concepts and structural meanings using the native
seed rules, emits only relations between grounded terms, and carries visited-need,
depth and source-span evidence through recursive glosses. Captured native sense
identities and original agentic recipe controls pass. The default offline registry
walker remains a separate delivery gap; this repair does not claim live discovery
or complete runtime parity.

Source duplication cleanup uses an AST-validated reusable transformation for six
stateless unary forms, preserving receiver/getter behavior, Unicode trim policy,
comments and binding semantics. The reviewed JavaScript batch replaces repeated
callbacks with canonical bindings; two byte-identical Rust escaping helpers reuse
the existing canonical implementation. Neither these extractions nor generated
projection proofs certify zero independent duplicates throughout the repository.
Fresh complete inventories and the strict zero acceptance remain required.

The Node log CLI now uses descriptor metadata before reading its bounded input.
The shared reader detects growth/replacement and refuses invalid read progress;
public stable symlinks retain their prior policy while private provenance paths
remain protected. Regression controls exercise the real CLI as well as injected
IO failures. Diagnostic truncation limits and custody rules are not weakened.

Hosted lint on f3d853b found stale static wall-clock metadata after the Hive DAG
split and five source Clippy findings. The source fixes preserve the existing
225-minute ceiling and original behavioral oracles. The discovered timeout path is
120 minutes; it is not observed elapsed time or publication evidence. Hosted
native validation, original three-client delivery and final release verification
must bind the final published revision before claiming completion.


The final integrated source checks pass, including readability, exact maintained
JS/TS/worker projections, the unchanged translation ratchet, embedded data,
requirements/status, release advertisement, CI speed and the original governed
baseline. The combined focused JavaScript regression run passes 174 checks with
no skips. A complete current JavaScript/TypeScript duplication inventory has zero
projection gaps and 729 independent candidate groups; strict acceptance remains
false. This is not a claim of repository-wide zero duplication or native parity.

Hosted full-test shard 6 found the new connection retry tests inline under src/.
The existing four test bodies and helper are moved without semantic edits into
registered tests/unit/bounded_retry.rs. The production implementation and public
API are unchanged. Native compilation and this placement check run only in CI.

On published f3d853b, the new Hive phase split completed all three original
client deliveries and all three independent verification jobs successfully in
[run 39494552135](https://github.com/link-assistant/formal-ai/actions/runs/39494552135).
That evidence belongs to that revision; the next published head still needs its
own automatic checks. Client and job durations do not identify the old native
CPU hotspot, and nominal provider cost fields do not establish actual spending.

The release readiness audit found the existing changelog selects a patch release
on a merge to main. Explicit workflow calls cover Docker publication and the
release-event recursion fallback, with source-bound desktop assets, exhaustive
macOS signed/ad-hoc branches and public download verification. Repository-level
optional registry credentials and the npm trusted-publishing variable were absent
when inspected; inherited organization/environment configuration was not audited.
Do not claim npm, extension marketplace or Docker Hub publication from a green
asset/GHCR pipeline. Require actual post-merge publishing and download receipts.


## Qualified coding changes after the first integrated push

The production source-model path preserves complete readable CST content and now
also transforms a uniquely attached JSDoc parameter tag with its proven function
parameter. The persisted transformation contains the original complete source,
canonical installed grammar and an exact source-free inverse. It preserves the
comment delimiters, type spelling, description, whitespace and unrelated comments.
References in arbitrary prose remain source-linked needs. Unsupported or ambiguous
bindings refuse the change. The original parameter transformation, independent
reparse, generated-owner protection and guarded physical write remain active.
Rust Markdown parameter meaning, arbitrary natural-language semantics and compiler
validity are not certified by this operation. The documentation controls and both
actual persisted inverse restorations pass; this is a qualified implementation of
the readable recursive source/comment doctrine, not its full acceptance.

A 155-byte natural-language request now makes the maintained JavaScript planner
read an actual production file, invoke its installed stateless-unary AST operation,
and read back the exact requested owner. The original route and a controlled
preimage route produced no plan and no effects. The repaired route consolidated
one real callback into its canonical helper. Four independently fixed lexical
output cases pass before and after, with parsing and comment preservation controls.
The consumer oracle is retained in CI. This is a narrow operation grammar, with
zero autonomous-authorship credit for the supplied algorithm and route repairs.
The source grows from 3085 to 3160 UTF-8 bytes: 75 net bytes for the 155-byte task.
That is useful refactoring, not amplification acceptance or token savings. Full
model input, model usage and cost remain Unknown. A separate larger historical
source task still fails its original oracle after bounded discovery; these attempts
must remain failures and cannot inflate a representative success cohort.

The mutable source-cache policy is shared data, with a 64 MiB aggregate byte cap,
512 URL metadata entries, 2048 regular files, 8 MiB response bodies and 16 KiB
metadata records. Admission counts retained objects and interrupted-write remnants
as well as peak staging space. The Node adapter uses bounded regular-file reads;
writers use one fail-fast shared-lock attempt. Full caches refuse new captures
without evicting historical evidence or touching unrelated directories. Empty,
warm, corrupt, deleted and unavailable-source outcomes remain distinct. Legacy
injected storage adapters retain caller-owned policies; their existence is not a
proof of bounded storage. Native source and controls require hosted compilation;
browser caches and a complete minimal-bootstrap discovery cohort remain separate
requirements. The native transport must bound actual captured output independently
of the installed curl version, not only inspect length after full buffering.

Automatic CI on `33f11949` found an obsolete hardcoded 115-minute assertion and six
new diagnostic phrases outside the existing seed mechanism. The test now derives
the current declared workflow DAG and checks its recorded duration against the
unchanged 225-minute ceiling. The six original diagnostics and the new unary-route
diagnostics use the existing installed message seed; the zero-literal gate passes.
The fixes preserve original behavior and ratchets. Final-head hosted checks and
actual post-merge releases still determine merge and production readiness.

The broader requirements remain binding: representative small-task success,
compositional self-coding, complete JavaScript/TypeScript/Rust behavior parity,
strict zero independent source duplication, complete source/comment meaning,
four-language documentation synchronization, dependency-aware fact checking,
formal requirement acceptance and contradiction repair. Qualified fixtures,
source projections, a green individual job or a short task cannot establish these
broader results.

The [complete unary attempt summary](evidence/natural-unary-source-task.json)
corrects an acceptance overclaim. The original request does not restrict coverage
to the existing six helper shapes; an inline `.kind` projection remains. The fixed
behavior oracle cannot establish consolidation coverage. Whole-task acceptance
is zero, despite the one useful actual checkout effect. The engine's finding and
pending independent validation remain accurate; these attempts cannot enter the
small-task success numerator. The earlier failure evidence and correction are
retained. Subsequent native source counterparts require independent hosted
compilation and configured-capability acceptance.


## Integrated repair batch before the next automatic CI

The native planner now has real counterparts for source preflight, stateless unary
consolidation and qualified CST parameter changes. The actual native server accepts
an explicit `--source-command-host` installation descriptor. It validates canonical
regular-file identities for Node and the two fixed maintained entrypoints; task
text cannot grant an executable or resource root. Both runtimes render the same
installed command seed. The optional native route delegates the transformation
to the installed JavaScript bundle. This is not an independent Rust parser or
proof that a standalone binary contains that bundle. The resource home and its
transitive dependencies remain caller-trusted, beyond the pinned entrypoints.

The added hosted acceptance runs the existing CI binary against tiny disposable
source consumers, with an explicit installation descriptor, actual HTTP planning,
physical writes and independent consumer oracles. Unary completion additionally
requires its own fresh requested readback. The CST route's independent physical
read is recorded separately from a planner-requested read. Local checks validate
source, descriptor and command-admission controls only; native execution remains
exclusive to CI. Fixture acceptance does not establish representative autonomous
coding success or acceptance of the larger production consolidation task.

The callback counterparts are used by real native consumers. Borrowed token text,
seed values, ECMAScript whitespace, strict empty comparison and positive length
have source-level regression controls, including preserved getter/method behavior.
Their typed domains do not claim the full JavaScript object/value model. The
native cache transport bounds actual captured bytes and has finite interruption,
process termination and cleanup paths. Node cache controls also cover filesystem
faults and strict metadata decoding. Full browser storage and bootstrap discovery
acceptance remain separate requirements.

The automatic checks on `33f11949ecea2659021619303692798956066b58` exposed
stale quote-owner citations, missing native counterparts, ungrounded grammar forms,
new abbreviated names, one Clippy suggestion and governed witness drift in addition
to the earlier timing/seed diagnostics. Repairs use actual native owners, grounded
operation meanings, binding-safe full names and the existing canonical quote
policy. Abbreviation, literal, twin and closure ceilings are unchanged. Genuine
translation, worker, embedded-data and specification generation follow the source
changes; no expected program is substituted to suppress failures.

The governed witness refresh was explicitly reviewed against all original
qualified cases and program/fixture content. All 221 qualified cases and their
order remain unchanged. Only source-identity metadata for `rust/src/lib.rs` and
`rust/tests/unit/mod.rs` changes, corresponding to the reviewed module and test
registrations. The original witness test passes after the refresh. Source success
cannot substitute for final-head hosted compilation, the original Hive delivery
checks, or post-merge publication and public-download verification.


## Follow-up to hosted checks on `4769993f`

The lightweight no-std worker includes the maintained seed parser directly. Its
first hosted compiler rejected the newly used strict-empty callback because that
worker did not have the native callback module. The correction extracts the
core-only operation into one canonical Rust source, re-exported by the native
module and included directly by the worker. It does not copy the algorithm or
pull JSON, native seed owners or the meta reasoner into the lightweight worker.
The worker registration preserves unrelated source comments and whitespace.
Desktop, CLI and extension artifact-download failures on that revision were
consequences of the missing source-producer artifact; they did not reach signing.

Hosted native test compilation also exposed a private type path, an incorrectly
typed byte-slice literal and a nonexistent locked-rustix FIFO function. The tests
now use the existing public type owner, an explicitly typed byte slice and the
actual `mkfifoat` API. Assertions and production visibility are unchanged. Two
plain-text incomplete-search markers move to the canonical message seed shared
by JS and Rust. Both exact markers, embedded occurrences and case-sensitive
near-misses retain public-planner regression controls; structured receipt checks
remain in addition to those guards, and the predicate ceiling is unchanged.

A further retry/deadline review reproduced blocked stdin and descendant-held
output pipes defeating ExecutionBox deadlines. The repair covers input writes,
output EOF, child status and bounded cleanup as one operation. Native asynchronous
process I/O uses only features of already locked dependencies, enabled for native
targets. Startup calibration itself has a finite probe; an unsuccessful probe
uses the existing startup floor rather than claiming a measured result. Excess
output produces an explicit observed failure with bounded partial output.
Missing EOF or reap evidence produces an incomplete-cleanup failure. On Windows,
an escaped descendant can retain an underlying Tokio blocking I/O worker even
though the caller returns within its bound; that outcome remains a refusal,
without a claim of complete process containment. Healthy execution and the
original error/result controls remain requirements.

The CI collector's process deadline likewise remains active until output closure.
It owns a process group where supported and has a finite final pipe-cleanup
window. Forced stream closure cannot certify a complete tail, successful command
or complete cleanup. Real Node-child regressions cover inherited pipes, including
an escaped descendant; fixture processes are explicitly cleaned up. These local
controls execute no Rust products. Hosted compilation and final-head runtime,
release publication and authenticated public-download checks remain necessary.


The locked native feature additions change the Cargo manifest's source identity.
Its governed fixture witnesses are refreshed only after a complete serialized
inventory comparison proves that all original qualified cases, program fields,
other fixture fields and oracle sources remain unchanged. The reviewed difference
is limited to that manifest's identity; the lockfile and baseline reader are
unchanged. This review does not certify the new native runtime behavior. The
unchanged original governed controls and hosted native checks remain required.

## Finite external-command boundary continuation

A finite retry count does not bound a stalled individual process. The continuation
therefore adds a shared binary-safe native command supervisor and a JavaScript
async supervisor whose synchronous Worker adapter preserves synchronous host APIs.
One command deadline includes startup, stdin writes, child status and output EOF;
cleanup has its own short finite settlement. Capture overflow, broken readers,
failed writes and incomplete cleanup are failures. The original primary error,
its I/O category and bounded partial bytes remain inspectable. Successful parent
exit alone cannot certify output held open by descendants. The explicit cleanup
receipt concerns the owned child and its I/O, not every escaped process on every
platform. Windows escaped descendants remain outside full tree containment.

Production callers now use these shared boundaries for prerequisite probes and
setup, prover/exporter invocations, Git/history/fact observations, authoring,
benchmark helpers, diagnostic transports and container state. Existing long
solve and transfer budgets are retained. Sequential SWE evaluation derives its
finite outer budget from the unchanged per-case allowance with checked arithmetic.
Foreground client launches retain operator-owned terminal signal cancellation
only when both stdin and stdout are actual terminals; unattended launches use
the finite supervised path. This is an explicit interactive policy, not a claim
that a terminal session has a fixed elapsed-time limit.

ExecutionBox preparation is supervised separately from its existing program
observer. Docker open stages share one remaining preparation budget; workspace
archives preserve binary bytes and have a finite capture limit. The original
execution observer and its behavioral assertions remain intact. JavaScript
history storage stays host agnostic and requires an explicitly supplied bounded
runner; the Node adapter supplies the shared host. An unconfigured legacy runner
is refused before Git execution. Original history assertions remain unchanged;
their setup selects the bounded adapter. The pre-existing census alignment skip
(PR1196-SDK1-native-JS-Rust-census-alignment) remains recorded debt, not a new pass.

The standalone command-capture program is generated from the actual maintained
async JavaScript supervisor. Babel source spans produce readable UTF-8 meta
source rows, and both JavaScript and Rust consumers join those exact rows. The
projection check rejects drift. This is a generated executable-source projection;
it does not establish the broader recursive CST, comment semantics or complete
language parity requirements. Strict UTF-8 decoding, original command chunking,
receipt fields and nonzero exit statuses remain; incomplete cleanup cannot yield
a successful wrapper exit.

The hosted web suite runs the production subprocess-boundary scan and canonical
capture projection check through the bounded supervisor. The structural scan
recognizes local process imports, constructors, typed receivers and opaque macro
call shapes. It does not prove arbitrary dynamic aliases, macro expansion,
interprocedural flow or cancellation of every asynchronous operation. Native
lifecycle controls exercise the real public supervisor in CI; no Rust compiler,
product binary, container or installer is run locally. Supplied repair
controllers receive zero autonomous-authorship credit, and usage/cost stays
Unknown.

Automatic CI on `f737adb6e5e05634d0bfe7416bac7f4ead4fa556` exposed notation debt
in newly introduced message keys and comment-grammar atoms, plus two Clippy
documentation-markup errors. The repairs preserve diagnostic text and external
JSDoc tag spelling while updating the actual consumers; notation ceilings and
original behavioral oracles are retained. Source success is not a substitute
for the next exact revision's hosted native tests, full CI, merge or publication
receipts. Broader retained Pending requirements remain binding.


The integrated local replay passes bounded-command, ExecutionBox, standalone
capture, history, attributed dispatch and notation controls. The original census
alignment skip remains unchanged. The first governed replay was retained as a
failure; complete comparison then proved all original case IDs, order, program
content and oracle sources unchanged, with only the actual library and unit-root
module-registration source hashes differing. Its explicit maintained witness
refresh passes the original controls. The translation closure uses one worker
and the already pinned local meta-language source, without fetching or compiling
Rust. The final formatting repair preserves every script token and the existing
readability limit. Current-source local checks cannot replace the next exact
head's automatic native, Hive, packaging or release acceptance.


The next automatic head, `69dcae569e03ce4b089c5f6ce694278148ba2112`,
compiled the library but exposed three binary-root E0433 references in
cli_benchmark, cli_context and cli_report. They now explicitly reuse
`formal_ai::bounded_process`; no second supervisor module is introduced. A
read-only module-tree audit found no further such reference in the binary,
test or bench roots. Native compilation of this correction remains hosted.

The continued retry audit also covers Formal AI's actual JavaScript driver.
Grep, shell capture and exclusive creation use the shared finite supervisor;
retained raw captures have explicit byte limits independently of their previews.
No subprocess inherits writable capture-file descriptors. The common raw writer
rejects invalid or zero byte progress immediately. Original complete-byte,
Unicode, nonzero-status and physical-creation assertions remain mandatory, and
the hosted structural scan includes the maintained driver directory. Caller-owned
creation input retains its existing contract while sharing the operation deadline.
This does not admit unowned source-discovery plans or grant implementation,
mutation or completion authority to lexical candidates.

The driver uses a visible Node stdio preload to preserve the prior regular-file
behavior of abrupt Node diagnostic writes while its shared supervisor captures
bounded pipes. Existing NODE_OPTIONS/security hooks remain visible. A strict
accepted-node source provider constructed inside such a captured Node child
continues to reject the extra NODE_OPTIONS; no guard exception or hidden preload
is supplied. The normal direct accepted-provider supervisor retains its own
constructor-owned environment. The original production-research controls are
checked separately; no universal provider/environment compatibility is claimed.


## Babel import-shape compatibility repair

Browser coverage shard 8 on `79248fdb760077d5d2f56e7ae9c5762799060ed9`
([job 118981517770](https://github.com/link-assistant/formal-ai/actions/runs/39526042339/job/118981517770))
found that the subprocess checker missed an awaited destructured dynamic import.
The repository declares Babel 8, whose default dynamic-import node is an
`ImportExpression`; the installed local Babel 7 produced its older
`CallExpression` shape. The checker now recognizes both structural forms,
retaining CommonJS and static-import handling and the strict startup policy.
The unchanged three-call rejection fixture reproduces the missing classification
with the modern parser option before repair. The original ten controls and an
explicit two-shape regression pass afterward. No package pin, original assertion,
source ceiling or CI limit changes. This is a supplied-controller repair through
actual JavaScript Formal AI, with zero autonomous-authorship credit. The new
published head still requires its own complete hosted CI and release evidence.


The same revision's first lint lane
([job 118982172431](https://github.com/link-assistant/formal-ai/actions/runs/39526042534/job/118982172431))
reported seven Clippy findings and a two-line core-boundary overrun. Four
21-minute durations now use the corresponding minute constructor. Both reader
join loops take ownership while retaining an empty vector for subsequent
completion checks. The reader error remains higher priority than a deadline
error. Each affected theorem-export handler removes one blank separator; complete
token and comment content is unchanged, and the original ledger stays untouched.
Installed rustfmt checks text only. The maintained JavaScript boundary twin and
original source controls validate the integrated source; hosted Clippy and native
lifecycle execution remain necessary. No warning suppression, test assertion,
retry ceiling, command deadline or cleanup allowance changes.


## Qualified property-callback continuation

The original 155-byte natural request is unchanged. Earlier attempts consolidated
only the text callback and remain recorded as incomplete. A general named-property
factory now derives its captured name from the actual member AST; there is no
owner-path or `kind` answer branch. Exact helper AST, authenticated plans, optimistic
source preimages, comments and unsupported/captured-projection refusals remain.
The maintained JavaScript Formal AI driver performed Read, trusted Bash operation,
and Read on the actual checkout, without a supplied target patch. Independent
whole-owner coverage finds no remaining stateless unary callback, while the
capturing callback remains unchanged. The original five consumer controls pass.

[Completion evidence](evidence/unary-property/completion.json) links full readable
recursive CST models for both source versions. These own actual UTF-8 source,
comments and whitespace and reconstruct without the original source file. This
structural fidelity does not certify comment meaning or general equivalence. New
factory and emitted-consumer controls exercise arbitrary names, Unicode, reserved
properties, getter receiver/one-read behavior, exact thrown errors, refusal and
source drift. Existing text helpers, worker factory and line ceiling are unchanged.
The unused Node-only helper worker projection is removed by the canonical import
closure rather than adding an artificial worker root or budget. The real native
typed receiver and token-kind consumer have hosted controls; native agentic route
parity and execution remain pending CI. This is one qualified task identity, not
a 60–80% cohort. Supplied repairs have zero autonomous-authorship credit; usage,
total model input and cost remain Unknown.

## Test-only Clippy continuation

Lint lane 1 on `6e85ddbbdd4767cd890ea5b528d29535d1dbefdc`
([job 118998286172](https://github.com/link-assistant/formal-ai/actions/runs/39529409842/job/118998286172))
passed the earlier production fixes and exposed duplicate unit-source inclusion,
empty-assert diagnostic forms, unchecked source-span conversions, local item
placement and needless generic borrows. Unit descendants now import one root-owned
production template reader. Existing data, assertions, deadlines and runtime order
remain; source spans use checked conversion. The new property test's local types
are likewise declared before statements. Installed rustfmt formats text only;
hosted Clippy and runtime results remain necessary. The unit-root witness requires
a strict full-inventory comparison before its maintained refresh.


## Shared image dependency compatibility repair

All four image validation cells on `6e85ddbbdd4767cd890ea5b528d29535d1dbefdc`
failed during dependency installation, before runtime checks. The unpinned global
install selected Azure/Bedrock versions whose exact OpenAI dependency had not
been published. Both Dockerfiles now consume one private source-controlled
compatibility manifest for start-command, the agent and agent-commander, plus
compatible Azure/Bedrock pins. Other OpenAI consumers retain their own resolution.
An owned per-user project puts the same tools on PATH. One finite install attempt
preserves Bun's existing script-trust policy; metadata verification inspects the
actually installed tool versions and exact affected SDK edges without executing
dependency code. All three original runtime probes remain byte-identical.

Five new controls and the three original Node capability controls pass with
metadata/fake-package-manager fixtures; these are not actual installation claims.
No Bun/npm install, dependency archive, image, Rust build or product execution ran
locally. Both architectures and both image variants still require the next exact
head's hosted installation and runtime validation. The manifest is a compatibility
policy, not a full transitive lockfile or blanket reproducibility guarantee.

The strict governed-witness review proves all original 221 qualified cases, their
order, all program/fixture fields and original oracle source content unchanged.
Only the exact reviewed unit-root module-registration identity is normalized for
comparison before the maintained witness writer runs; the reader stays unchanged.
Historical failed replays remain retained.


## Fresh-request cost and latest hosted failures

The existing five-second long-context control replays an already exhausted
transcript; it does not measure a fresh large request. The complete available
pre-first-assistant prefix is retained privately, but the original 277,529-byte
wire envelope and tool definitions are unavailable. Never label a reconstructed
or synthetic request as the original. [Sanitized measurements](evidence/fresh-request-latency.json)
separate inputs, entrypoints, variants, cold/warm observations and failed deadlines.
No original task text or CPU profile is published.

A measured JavaScript profile identified repeated immutable read-mode grammar
parsing/expansion/compilation. The source host now caches only compiled forms;
native code uses its static OnceLock projection. Prompt decisions remain fresh,
with original quote, path, domain, ambiguity, form-order and host-reset guards.
A separate source audit found quadratic Unicode lookaround: materializing a
whole string suffix merely to read its first scalar. Two-unit UTF-16 windows
preserve astral and unpaired-surrogate behavior without allocating the suffix.

A separately declared public 277,529-byte synthetic case timed out at 45 seconds
before its first result on both the baseline and grammar-cache-only internal
entrypoints. The combined repair completes three valid planner/outcome requests
within that process boundary. The public JavaScript completion API then measures
11.145, 9.368 and 9.322 seconds with independently checked owned issue fetch and
required arguments. Its matching public API baseline was not measured. The
45-second guard is a new observation boundary, not an earlier fresh-request SLA.
CPU deltas include process workers; they are not single-thread CPU time.

The new hosted native control uses the complete public request entrypoint,
supervised child processes and separate synthetic 21,000/277,529-byte cases.
Each case must finish all three cold/repeated calls within one 45-second bound.
Its first call must fetch the requested issue, every call must bind valid arguments,
and the completion cannot echo the entire user input. Native CPU is explicitly
unavailable. Compilation and timing remain CI-only; do not claim a measured
130-second native improvement or an exact original-wire replay.

On `625355ee588db7122323077a0cbc5a414252cc2e`, all four full/slim image
architecture validations and the main binary/test archive build passed. Lint
lane 1 found a redundant must-use attribute, an empty-shell-assignment warning,
and two missing publisher snapshots. Repairs remove only the redundant attribute,
spell the empty variable explicitly and append the two authoritative package
records without rewriting historical snapshot timestamps or weakening the gate.

The macOS x64 desktop job failed before packaging/signing: node-gyp timed out
downloading headers for the installed Node version. The macOS host dependency
install opts into verified matching headers already supplied by setup-node.
Explicit environment or effective npm configuration keeps its authority. The
preferred node-gyp package-config option applies only to that npm invocation;
later Electron rebuilds retain their own target/header selection. Unexpected
diagnostics and unreviewed lifecycle scripts remain errors. This is a source
repair, not proof of a successful hosted desktop package or public download.

All supplied repair controllers retain zero autonomous-authorship credit.
Use actual next-head CI, then normal merge and actual default-branch release
receipts; earlier-head successes cannot substitute for those acceptances.


Coverage shard 1 on `625355e` found two new public modules missing from the
architecture module map. A JavaScript source projection reads the canonical Rust
generator's actual override/section tables and module headers, and adds only
those missing rows. All existing map content is byte-identical after removing
the insertions. The generator and original native module-map oracle are unchanged;
pre-existing prose differences remain explicit rather than being silently rewritten.
The next exact head still requires the original hosted coverage check.


The fresh native observation forwards its bounded child stderr through the
standard IO writer directly. The ordinary successful libtest print-macro capture
would otherwise discard those measurements in both hosted execution lanes.
Child arguments, request bytes, quality assertions, 45-second case boundary and
one-MiB stream caps are unchanged. Actual hosted observations remain required.


Lint lane 1 on `49f6dc8` found a restricted production helper included under a
private test-root module and two nonempty assertion diagnostics in the new fresh
request control. Only the test-root module becomes public; production visibility
and behavior remain byte-identical. Equivalent inequality assertions improve
failure diagnostics without changing accepted values, requests, clocks or bounds.
The test-root witness requires another strict full-inventory comparison before
its maintained refresh; do not overwrite the baseline as an automatic repair.


### Coverage follow-up on the published 49f6dc8 head

Three independent coverage failures are retained and repaired as one batch. The
native fresh-request child timed out at its unchanged 45-second boundary. The
uninstrumented full-test archive completed the first synthetic 21 KB request in
33.226737 seconds, then timed out during a later request; neither run reached the
277,529-byte case. Seeded role matching recompiled immutable surface regexes for
every sentence. Both source counterparts now cache only compiled role grammar,
preserving source order, Unicode/quote boundaries and host reset behavior without
retaining prompts, matches or answers. Constructor-count instrumentation on the
separate JavaScript 21 KB fixture observes 466,875 compilations per old request,
versus 145 total for all three repaired requests. Its timing overhead and runtime
scope remain explicit in the latency evidence; this is not a measured native
speedup. The hosted test keeps every original input, output oracle and deadline,
and adds readable stage/partial-stderr diagnostics for future failures.

The original native unary-edit authority oracle also failed: after an identical
successful source operation, the global repetition guard blocked its required
fresh Read. The repair invalidates prior typed source-read evidence only for a
planned Read after a qualified successful completed Run. It does not authorize
another Run or reset the three-action/objective counters. Failed, unknown and
incomplete receipts retain their prior boundary. All 21 original JavaScript unary
controls pass on both variants; the additional shared Progress primitive control
fails before the repair, and all 23 controls pass afterward. The additional public
planner control already passes the baseline, so it is not a local reproduction
of the native public-route failure. The original native authority oracle remains
unchanged and requires hosted revalidation.

The Docker source test still required the obsolete global Bun installation. Its
replacement checks the actual shared manifest, helper copy, installer invocation
and owned PATH; every other original native assertion and all three runtime probes
are unchanged. All four actual image validation cells, both macOS packaging jobs
and all three original Hive clients passed on 49f6dc8. Those results do not confer
a pass on the next published head. No native products were run locally, and no
original deadline, oracle or ceiling was relaxed.


### Issue 1364 and Hive Mind E2E follow-up

The complete eleven-obligation scope and actual three-client release acceptance
are recorded in `docs/requirements/issue-1364-three-client-release.md`. Source
fixes and local replays do not complete the hosted and released-image acceptance.
The original nested Agent tool-use exit-127 frame is reproduced by a sanitized
fixture and now retains the terminal provider status without trusting arbitrary
source JSON as a process receipt.

[Hive Mind issue 3630](https://github.com/link-assistant/hive-mind/issues/3630)
was created and independently verified open on 2026-10-21. It requires equivalent
owned E2E issue cleanup, final-CI admission, successful-PR evidence reuse, finite
failed retries, one real merged/released-source execution, and faithful sanitized
log replays. The observed 78 open generated issues belong to Formal AI; the Hive
Mind report requires an audit and does not assert that same count there.

Live memory startup now uses the existing lossless migration for validated older
schemas. Unknown/incompatible bytes remain untouched beside an explicitly logged
private current-schema sibling; IO/permission failures remain errors. Health reads
the selected store without migrating it. Thirty JavaScript controls pass, including
all twenty original memory tests. Native live-volume and SIGTERM controls are
registered for hosted CI only. Native and Docker PID1 shutdown have not been run
locally, and filesystem calls or a blocked JavaScript event loop do not acquire a
portable hard-preemption guarantee from the bounded shutdown policy.


## Owned E2E lifecycle and exact issue 1364 replay continuation

[Hive Mind issue 3630](https://github.com/link-assistant/hive-mind/issues/3630)
is open and records the analogous lifecycle audit. It distinguishes the confirmed
Formal AI issue buildup from an analogous Hive Mind risk that still needs audit.
The original issue 1364 tasks, issue bodies and prepared branches remain binding;
a green generic Hello World run does not satisfy their stronger acceptance.

The integrated protocol admits real client effects only after a complete census
of applicable source-bound CI results. Successful clients can be reused within a
pull request while retaining the actual tested source. A source-free Git metadata
ref consumes each task/policy/input key before effects; Actions artifact expiry
does not reset it. Results can only advance the exact claim by a normal,
non-forced child commit. Lost producer proof or uncertain write outcomes stop
further effects. Retain these metadata refs. Public ledger content contains only
declared binding metadata, never product source, raw conversations or credentials.

The hosted replay uses the exact three public issue/PR pairs, keeps one task
checkout across both passes, and independently requires the original delivered
stdout, task CI, readiness, comments/build instructions and clean Git state.
The second pass must make no source writes or commits. Sanitized complete dialog
captures exercise the production protocol adapters and preserve explicit missing
or truncated evidence. Raw captures stay outside published evidence directories;
privacy export retains numeric types and compares the retry/write oracle before
and after redaction. Supplied integration controllers receive zero autonomous
source-authorship credit.

Local emulations and source checks do not establish hosted acceptance. Default
branch workflow activation, target-repository credentials, exact-head native CI,
all three actual released-image client runs and public release/download checks
remain acceptance steps until their actual receipts are observed. No local Rust
compilation or product download is authorized. All eight maintained generation steps were refreshed after the coding-planner
repair was integrated; final source and hosted checks still determine acceptance.


### Three-client recovery integration, 2026-10-21

The maintained historical protocol suite now exercises the authentic Kotlin, Rust
and Scala task bodies and prepared-branch source identities through the actual
Messages, Responses and Chat adapters. Each first run completes its supporting
changes; a second run on the same checkout succeeds without source writes, a new
commit or a changed HEAD. The fixture controls GitHub and toolchain results, so
this is source-level acceptance, separate from actual hosted services, native
execution and released Hive images. Original task bodies remain unchanged; source
fixtures derive from the existing program grammar and independently verify the
authentic Git blob, SHA-256 and UTF-8 byte identity before use.

Recovery binds actual command aliases and exact custom patch frames, revalidates
source evidence after mutations, and handles the original continuation before
optional PR feedback. Guarded full-source replacement requires the complete
observed preimage, matching file identity and digest, and a new post-write probe.
It preserves ordinary files, foreign task evidence and physical compiled outputs
while removing only qualified owned artifacts from the Git index. The Scala
header correction is an intentional authorized edit, not whitespace equivalence.
Tool-call identities now include the normalized request-history length, retaining
same-request determinism while distinguishing later fresh observations. Duplicate
identifier and unknown-receipt refusals remain mandatory.

The two-failure budget applies to the proposed qualified operation, including its
actual schema-bound arguments. A distinct verification action can observe the
state after two failed writes; a third identical failed operation is blocked.
The independent three-attempt and 128-action limits remain unchanged. Replay
metadata stores the original body hashes without duplicating request bodies in
production knowledge data; live acceptance still fetches and verifies the exact
public requirements. Neither ratchet ceilings nor original output oracles were
raised or bypassed.

The provenance-only job is shared through a read-only reusable workflow. Its
caller remains bound to the original release event, and the called source is
included in policy identity. Mechanical generation preserves all 52 original
release operations and 104 output bindings, every original comment and whitespace
outside the new job, and the staged workflow bytes. The retained original stays
within its unchanged 1500-line limit. The PR static timeout maximum is 115 minutes
through the release workflow, below the unchanged 225-minute ceiling; these are
static allowances, not runtime observations.

Use at most three active subagents for this PR, with one JavaScript Formal AI
delivery lane and one CI failure monitor. Lease one bounded Node workload across
the team, draft independent work in bulk, commit coherent fixes, and push once
all known causes have committed repairs. Execute Rust only in CI. Supplied
controllers receive zero autonomous source-authorship credit. Preserve failed
receipts and sanitize public evidence; never stage personal session exports.

Exact-head hosted native and image checks, normal merge, publication and public
download verification, and the three actual released-image Hive runs remain
required. Source-level replay success does not establish these outcomes.


### Follow-up from actual hosted compilation

The next hosted head exposed duplicated PR-target predicates and two obsolete
private Rust helpers left behind by the source-mutation and inventory refactor.
The repair shares the existing validated target classification across recipe
consumers, removes only the unreferenced internal definitions, and preserves the
active mutation/inventory implementations and public JavaScript API. The original
debt ceiling remains unchanged. Include the maintained JavaScript debt checker
with an explicit comparison base in local source qualification; native compilation
and runtime acceptance remain CI-only.


Native regression tests must call existing public protocol entry points and match
current constructor signatures. The captured nested exit-status test now uses
the public Chat API with genuine assistant/tool frames; its original exit-code
and missing-status assertions stay intact. Explicit empty-slice types preserve
nonempty-call checks without ambiguous Rust inference. Static source checks and
JavaScript API equivalence do not replace hosted native unit execution.

### Owner-authorized manual release blocker repair

The owner explicitly permits direct manual fixes to prioritize unlocking the
next release. Preserve actual Formal AI authorship receipts without attributing
manual changes to it. Keep one CI monitor and one source reviewer when useful;
one bounded Node workload remains shared across the team, and Rust execution
remains hosted only. Draft all actual failures in bulk and commit coherent
repairs, then publish one ordinary push after the current complete CI census.

On published head `f9d8e3d14ac4636d2405fce51232ea5fd0c2b8fd`, the Linux ARM64
desktop job returned HTTP 500 after native rebuilds and Electron archive progress
completed. The 7zip prefetch succeeded. The failed endpoint is absent from the
full mixed log, so do not describe the checksum hypothesis as observed evidence.
The pinned builder already has server-error retries. The repair isolates download
recovery before packaging instead of multiplying those retries: the same nested
Electron downloader retains checksum validation, individual requests share a
three-attempt ceiling and deadline, and only a manifest matching the exact archive
can become the builder's supported versioned checksum seed. Request diagnostics
exclude credentials and query strings. Pure JavaScript controls use tiny fake
payloads; a separate integration control exercises the installed nested downloader
in hosted packaging. Neither replaces successful actual packaging, release or
the three original released-client deliveries.

On `8b2df74a1c74d5d0977854acd063a73a11894468`, the Linux ARM64 desktop job
passed its installed-downloader controls and actual cached Electron packaging.
The next actual blocker was [coverage shard 22](https://github.com/link-assistant/formal-ai/actions/runs/39699421816/job/119572291769):
the live authoring fixture inherited Git's stdout, which split another test's
libtest result line. Its batch exited successfully but lacked one exact
completion. Capture the bounded display subprocess before printing through
Rust's test-aware output, including lossy display of non-UTF-8 Git messages;
display decoding must not introduce an error after a successful commit. Keep
the coverage parser strict. The child-harness regression runs the original
authoring fixture and requires an intact exact completion with no leaked
commit trailers. Local JavaScript controls reproduce the refusal, while native
execution and complete next-head CI remain required before merge.

The next browser-engine package job stalled inside locked `npm ci`, before
Playwright started, until the existing 295-second setup deadline. The log does
not identify its request. npm's default five-minute request timeout exceeded
that outer limit. Install requests now have a shorter timeout and one bounded
retry, with timing output and operation phase/exit/elapsed records. Keep locked
installation and disabled lifecycle scripts; do not retry the whole install or
increase the setup/job limits. The exact remote setup must still succeed in CI.

CodeQL also identified two assertion diagnostics that printed the session-ID
evidence. Both marker assertions remain intact, while their failure messages
are now static and do not disclose session contents. Actual CodeQL and native
test success on the next head remain required.

The dependency gate also exhausted its outage budget without answers for the
root Bun and E2E npm lockfiles. npm audit now bounds individual requests below
the existing outer attempt deadline; only the existing outer loop retries.
Live per-attempt tool, lockfile, status and elapsed records survive isolated
output capture. All lockfiles still need real audit verdicts. Bun keeps its
existing process bound; its transport cause remains unknown. Neither an
unanswered audit nor an advisory is treated as a successful audit.

### Current coverage-only release debt

The complete CI census for `4ab04ffa3eb5cfd0b48597a6cd2e3033abe2ee71`
finished with all noncoverage workflows successful. All native targets and
desktop packaging passed, CodeQL completed with no new alerts, canonical
requirements matched, and all six dependency audits returned clean first-attempt
verdicts. Native full/SPEC jobs may reuse an identical-input green-ledger result;
that is not fresh exact-head execution. Coverage shard 13 freshly completed its
previously interrupted multilingual case and every selected obligation.

The remaining real failure was the offline how-to service-opt-out HTTP case in
[coverage shard 26](https://github.com/link-assistant/formal-ai/actions/runs/39722357450/job/119645561072).
The server accepted the request, then its caller reached the unchanged
30-second response deadline. `WouldBlock` is the final socket timeout error;
it does not prove thread/resource exhaustion or a framing bug. No computation
hotspot is established. Preserve the complete mixed log, SHA256
`5fc19d674768e70e3a80a0966c9076b224c8c24bdd5459975eb5d16ccd32793b`,
526,046 bytes, and the correctly failed coverage aggregate.

The owner's release-first authorization permits temporary deferral of this
noncritical instrumented scenario. The existing coverage selection now excludes
only `issue_991_how_to_http::chat_completions_honours_a_service_opt_out`;
ordinary native full CI retains that original test, all assertions and its
30-second deadline. The original ordinary case
[passed on source 829](https://github.com/link-assistant/formal-ai/actions/runs/39720271433/job/119635342874),
but that historical result is not instrumented acceptance. Library and browser
opt-out controls remain required. This deferral does not claim the slow path is
fixed. The stable debt ID and restoration condition are recorded in
[the critical scope](critical-delivery-scope.md#temporary-instrumented-http-measurement-debt).
All remaining selected coverage, profile/ratchet and production/released-client
gates remain required. Qualify the complete known failure batch and publish one
ordinary push, then require actual latest-head CI before normal merge. No local
Rust build, extra E2E attempt or timeout increase is needed for this deferral.


### Released memory and PID1 measurement

Source tests and original-three-client task/no-op receipts leave two distinct
released-runtime observations unproven: live-memory handling and native PID1
shutdown with an active connection. The published-only Hive image step now
measures those outcomes on the authenticated immutable release parent before
real task effects, without another client or GitHub test attempt. Both maintained
memory fixtures and their canonical parser participate in the policy identity.
The probe checks supported lossless migration and exact backup, preserved
incompatible bytes with an actual chat write to the reported current store,
idempotent restarts, the PID1 executable hash, held-connection responsiveness
and successful bounded SIGTERM exit. It keeps a small JSON observation artifact
and refuses incomplete cleanup. Dreaming and live fetch are explicitly disabled
for isolation; default-dind behavior and original-volume reconstruction are not
claimed. Pure Node controls use injected Docker/network observations and cannot
establish hosted or released acceptance. Require the real published result.


### Agent CLI install artifact availability

On source `c5f27d6b014b0019942aba3c6cc2c8893d011368`,
[Agent CLI setup](https://github.com/link-assistant/formal-ai/actions/runs/39732636164/job/119672417632)
failed before any model execution. The full 33,279-byte log has SHA256
`68225921c12e78984c8b22ed3d2a4c0e1133f35d5393712384e877843961f248`.
Bun resolved the pinned client dependency graph, then received a 404 for
`vscode-languageserver-types-3.18.5.tgz`. Official exact-version metadata and a
HEAD-only observation subsequently returned 200 from this observer. Preserve
the failed attempt; that later availability is not global-recovery proof.

The repair keeps client pins and normal integrity verification. It permits one
unchanged install retry only after the first command exits cleanly with the
sole qualifying official tarball 404 and fresh, bounded exact-version metadata
and nonredirecting HEAD checks establish eligibility. Both commands and probes
share one finite deadline, newly tighter than the existing job limit. Other
errors, persistent missing bytes, timeouts, capture overflow and uncertain
cleanup cannot reopen the retry. Bounded captured output is replayed after each
attempt with explicit phase records; live child-output streaming is not claimed.
Source/emulated recovery controls cannot replace actual hosted installation and
the original model/E2E or released-client acceptance.


### Current Agent model acceptance and prover setup failure

On `c28e28d4fa71e1cc0ddcd35c12d1ce97eb223c59`,
[Agent E2E](https://github.com/link-assistant/formal-ai/actions/runs/39736536093/job/119688639024)
succeeded under hosted Bun 1.4.2. The pinned client installed on its first
attempt; the original greeting, real tool file-read, summary, interactive,
persistent-config undo and relocated-config oracles all executed successfully.
The normal installation/model path is accepted on that source. No live outage
retry occurred. Its full 38,622-byte log has SHA256
`03c5e7c8852cad7a9e1c5f4526581a71e9b4514626b2dbd2561ffb87c760a501`.
Two genuine browser Worker tests and clean CodeQL also passed on the same
authenticated source tree. These are candidate observations, not a released
three-client or all-eleven acceptance.

The next actual blocker was
[prover setup](https://github.com/link-assistant/formal-ai/actions/runs/39736536224/job/119691094928).
Its complete 31,707-byte log has SHA256
`022474bff847c92d94e3459479c47deae81b7646abb0bfed1c9312e83705d3ba`.
Both existing APT attempts hit their 120-second deadlines through the same
Azure Ubuntu archive; neither Lean nor Rocq compilation executed. The official
runner-image source confirms a mirror list and signed Ubuntu source settings.
Current HEAD-only observations find the official alternate index and exact Coq
archive available, but prove neither throughput nor hosted recovery.

The prover-only opt-in uses its existing second attempt to change the exact
known Azure URI to the official HTTPS Ubuntu archive after a deadline failure.
It preserves source trust settings, package choices, completed download cache,
all attempt/stage/job bounds and both real compiler oracles. Other callers do
not opt in. Retain the original failure, exercise the offline shell controls,
and require actual current-head setup and compiler success before merge. This
manual repair receives zero autonomous Formal AI authorship credit.

### Pending seed in the multilingual memory-reset test

On `c0ba2074f876429176ce74a16d1dcaa413169504`,
[local web shard 2](https://github.com/link-assistant/formal-ai/actions/runs/39743045641/job/119720443946)
failed before a reset command while waiting for an arbitrary synthetic seed to
finish. The full log is 119,955 bytes, SHA256
`9f4de1f8e33ac20aa83c8c252d27c896bbc0ef50ade92a6ddcadddcb1d90fd05`.
Two diagnostic Markdown members were read using bounded HTTPS ranges from the
authenticated `playwright-report-local-2` artifact, without downloading its
complete archive, trace, products or executing a browser locally. The snapshots show
pending `Memory reset seed en` and `Memory reset seed ru`; the latter follows
a completed English reset. Whole-archive digest verification is not claimed.

Seed each reset through a real `2 + 2` exchange and require both persisted user
and assistant messages before exercising each original reset phrase. All reset,
confirmation, empty-memory and deadline oracles remain. Keep unknown-prompt
coverage elsewhere and track `PR1329-browser-unknown-seed-latency` as unresolved
profiling debt, not a product latency fix. This direct fixture repair receives
zero autonomous Formal AI authorship credit. Require actual hosted acceptance
on the next source and retain every other pending check as unaccepted.

### Merged-source prover setup follow-up

PR 1329 merged normally as
`9b800dc33a2c6ec8bed1e56a48d8c7c8a5934268` after its complete check inventory
was green. The subsequent default-branch
[prover installation](https://github.com/link-assistant/formal-ai/actions/runs/39764481959/job/119776755681)
failed before either compiler ran. The complete 30,164-byte mixed log has
SHA256 `098fc63823d20ea9e3187d0a146501fff7ed47dcca7524f7d30f1efc9b905b63`.
Both existing attempts reached their 120-second deadlines; the second did
exercise the official HTTPS mirror alternate. Its progress is insufficient
to establish that the mirror recovery solves cold-install throughput.

The follow-up uses the owner's temporary noncritical-test deferral authority:
only the real Lean/Rocq math integration requires explicit repository variable
`FORMAL_AI_REAL_PROVER_INTEGRATION=true`. Keep its skipped result unaccepted.
When enabled, install mandatory packages without recommendations and run every
original compiler oracle under the same bounds. This is release debt, not a
fixed network root cause. The stable debt and restoration conditions are in
[the critical scope](critical-delivery-scope.md#temporary-real-prover-installation-debt).
No production, coding, released-client or public-download gate is deferred.
Manual changes receive zero autonomous Formal AI authorship credit.

Continue collecting all default-branch failures into one repair batch. Require
actual follow-up-head CI before normal merge, then monitor its production
release and the original three-client acceptance; the merged candidate and
historical compiler successes cannot substitute for those observations.

The follow-up is [PR 1394](https://github.com/link-assistant/formal-ai/pull/1394).
Its first lint lane failed only `check-pull-request-link`: the description
omitted an explicit issue disposition. Preserve the complete 396,505-byte log
from [job 119788589006](https://github.com/link-assistant/formal-ai/actions/runs/39767617743/job/119788589006),
SHA256 `0d2f2b0a95fa9ebec09264722539b6ff8fccd26bdce5cda5a1d538dffe483016`.
Add the required whole-line pending-acceptance reference to issue 1364 rather
than a false closing claim. Released-client acceptance remains unfinished.
The compiler and production gates themselves need no further change for this
description failure. The next ordinary source update records these PR/debt
links and lets CI check the corrected description without a manual rerun.


### Public release and read-only admission follow-up

[Release v0.355.1](https://github.com/link-assistant/formal-ai/releases/tag/v0.355.1)
is public. Its crate publication, registry-content audit, installed-crate
smoke test, full/slim GHCR images and Docker Hub mirror passed their actual
hosted steps. Installer, signing, public-download and remaining surface
verification still require their own results. The original released three-client
and all-eleven issue-1364 acceptance is not established by those release steps.

The subsequent [admission failure](https://github.com/link-assistant/formal-ai/actions/runs/39790340888/job/119864406820)
happened before client execution: the fourth GitHub CLI read exited 1 with an
empty response and completed cleanup. Preserve its full 28,367-byte log, SHA256
`6eede8918b85174d2a35779480b1a5dcc0ace05c3451563487e4a66dba19fdfd`.
The exact cause of the suppressed 29-byte diagnostic remains unknown.

Batch one unchanged clean REST-read retry and privacy-safe attempt diagnostics
into PR 1394. Keep the original global request, byte and time limits; GraphQL,
known denials, partial bodies and uncertain processes stay unretried. A second
failure stops without client effects. The
[critical scope](critical-delivery-scope.md#bounded-read-only-github-admission-recovery)
records the precise policy and evidence limits. The preceding head's green CI
cannot accept this additional change: require fresh current-head results
before the normal merge. This manual repair has zero autonomous Formal AI
authorship credit. Continue monitoring actual release consumers and the original
three task repositories; do not close issue 1364 prematurely.

### Browser dependency setup and checksum cascade

The main v0.355.1 release pipeline completed successfully. Independent data-only
checks verified anonymous immutable full/slim GHCR and DockerHub manifests
against the released source and executable receipts; these contain the merged
PR 1329 changes. Installer/browser/download and original-client acceptance
remain separate requirements.

Browser publication then exhausted its existing setup deadline downloading
Ubuntu fonts, before any browser smoke assertion. Desktop checksum publication
subsequently refused the missing browser publication receipt. Preserve both
complete failure logs and the precise limits in the
[critical scope](critical-delivery-scope.md#browser-system-dependency-publication-blocker).
Batch the shared validated HTTPS mirror preparation, immutable protocol/package
dependency, HTTP diagnostic delimiter correction and regression controls into
PR 1394. Keep every existing deadline, signature check and browser/checksum
oracle. Earlier current-head green CI does not accept these added changes;
require a fresh exact-head census before the normal merge. The suppressed
admission diagnostic's underlying cause remains unknown.
