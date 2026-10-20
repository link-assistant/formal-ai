# Issue 1328 / PR 1329 handoff

Continue in [the existing PR 1329](https://github.com/link-assistant/formal-ai/pull/1329).
Its four original bot commits and public session logs must be retained by normal
history updates. The integration started from `98f7808d49583e1e7932d133355445d8fdfb91f9`
(v0.355.0). Do not create a replacement PR or rewrite its branch history.

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
locally. Installed rustfmt may format text without compiling it. No additional
subagents are needed after the completed collection and source-selection work.


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

The current original Agent job ([118768617314](https://github.com/link-assistant/formal-ai/actions/runs/39462450498/job/118768617314)) was cancelled by the 30-minute enclosing job limit: preparation took 10 minutes 30 seconds, the actual solve then ran for 19 minutes 43 seconds, and its final response arrived seconds before cancellation. The independent source compiler/output oracle and evidence upload were never reached, so this is not a pass. Delivery now has a 60-minute enclosing budget covering cold preparation and the unchanged independent verification, while the solve itself is capped at 25 minutes. There are no workflow reruns, extra task attempts, changed task inputs or relaxed output assertions. Native client requests still observed 65–99 second latency; the original address-only profile does not identify that remaining CPU hotspot, so no complete native speedup is claimed.

The remaining source shard found browser command-capture drift and a covered 277 KB exhaustion replay exceeding its unchanged five-second limit. The browser and JavaScript planner now call one source-owned structural capture contract with their existing lexical adapters; its worker factory is generated. The retry guard proves whether earlier tool calls exist before doing request identity/file-read classification; when none precede the current user request, that classification cannot affect the counted window and is avoided. Fresh file requests and continuations retain their existing controls. The old dogfood assertion’s literal `origin HEAD` command is updated to require the actual branch observation and original message/remote guard; real disposable Git commit/push tests continue to verify the effect.

Native coverage shard 3 retained an original unknown-command passthrough test and exposed provider-owned uppercase `OBJECT`/`STRING` schema types being rejected as unknown. Both validators now normalize only the declared type keyword while preserving the argument value and refusing real type/missing-value errors. Native lint also identified new prose; native and JavaScript diagnostics now use the same metadata text. The existing native wording helper reparsed that immutable table for every lookup; its shared seed reader now parses once and fills slots in one pass, matching JavaScript’s cached table. This removes repeated work but does not establish the unsymbolized production CPU hotspot by measurement.
