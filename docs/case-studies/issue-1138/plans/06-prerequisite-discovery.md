# Plan 06 — Prerequisite and environment discovery (bottleneck B6 of #1138)

Status: design recorded before any test or code edit. Baseline: `main` at
`09294f25d`, worktree `.claude/worktrees/issue-1138`, 2026-09-16.
Predecessor: [issue-710 plan 07](../../issue-710/plans/07-prerequisite-discovery-bridge.md),
whose `## Test order` checklist stands at **one of seven boxes checked**
(`07-prerequisite-discovery-bridge.md:65-81`). This plan is its continuation
with the surrounding machinery (#930, #937, the browser) attached.
**Contract alignment.** [Plan 00](00-root-causes-and-integration.md) fixes five
shared contracts and this plan implements against them rather than inventing
parallel types (plan 00 §8 renames any duplicate to the contract name):

| Contract | This plan's use |
| --- | --- |
| §4.1 `need` (link record, `kind … prerequisite …`, `state open \| planned \| satisfied \| unsatisfiable`) | a missing program **is** a `need` of kind `prerequisite`. `PrerequisiteNeed` below is its in-memory projection, not a second record. |
| §4.2 `trait SourceLookup` — `lookup(&mut self, need, bounds) -> LookupOutcome`, one client over `data/seed/sources-registry.lino` | the publisher search calls it; it does **not** call `UnknownConceptLookup` directly (plan 01 decides whether the coding trait survives as a thin adapter). |
| §4.3 `evidence` — `for_need`, `produced_by`, `command`, `exit`, `output_hash`, `source_ids`, `recorded_at` | every probe, every setup step and every re-probe emits one; a `prerequisite` need reaches `satisfied` only when an `evidence` row references it. |
| §4.4 `trait Workspace` | owned by plan 03. This plan adds recovery **inside** `Workspace::run` (plan 00 §4.4: "plan 06 adds prerequisite recovery inside `run`"). |
| §4.5 generated status table | the probe table and the toolchain ledger are rows it reads; no document restates them. |

**Order.** Plan 00 §5 places this plan **before**
[plan 03 — repository workspace protocol](03-repository-workspace-protocol.md):
`01 → 04 → 02 → 05 → 06 → 03`. Recovery therefore exists by the time the
workspace protocol calls `run`, and plan 03's honest stop at
`kotlinc: command not found` becomes this plan's starting observation.
Depends on: plan 01 (the `SourceLookup` implementation) and plan 04 (a
formalizer that emits needs), named below at the exact seams; until either
lands, the flows here work from an observed exit code and say so.

## Issues addressed — issue numbers with one line each on what they ask and which part this plan delivers

| Issue / PR | What it asks | What this plan delivers |
| --- | --- | --- |
| [#1138](https://github.com/link-assistant/formal-ai/issues/1138) B6 | "A failure such as `command not found` is itself a requirement fed back into B1/B4 (search the trusted publisher for the install procedure, install workspace-scoped, retry), with the same forget-and-rediscover proof as procedures." | The whole plan. |
| [#8](https://github.com/link-assistant/formal-ai/issues/8) (closed) | Never give a code example without compiling and running it (via `link-foundation/start`); 1-minute timeout → halve iterations and report the N boundary; 10-minute non-compiling → total failure with a verbose log; every surface must know its environment limits and say "this code was not tested, not compiled, not checked". | The probe-before-answer contract, the honest-limit statement per surface, and the timeout-as-failure rule. The maintainer's own allowance — *"intelligence may be not smart enough at the moment, but all the interface though telegram bot should be all in place"* — deferred the backend, not the requirement. |
| [#930](https://github.com/link-assistant/formal-ai/issues/930) (**open**, 0 comments, nothing delivered) | E78: wire the Telegram code path through `link-foundation/start` docker; compile and run before answering; halve-and-retry on timeout; hard-fail with a verbose log past ten minutes. | `ExecutionBox` + the Telegram surface's execution tool; the halving policy expressed as an honest iteration ladder, not a budget. |
| [#937](https://github.com/link-assistant/formal-ai/issues/937) (**open**, 0 comments, nothing delivered) | E85: per-conversation detached `link-foundation/box` container, reattachable, idle-stopped; snapshot restoration by default with command-replay as a settings-selectable fallback. | `ConversationContainer` lifecycle, `SnapshotPolicy::{Snapshot, Replay}`, idle stop, reattach. |
| [#331](https://github.com/link-assistant/formal-ai/pull/331) (merged, closes #330) | Its "honest scope note" explicitly deferred *"dockerized execution with snapshots/replay (`box`/`start`)"* and *"browser Linux VM (`rust-web-box`)"*, recorded in that case study §9 and re-filed as #937. | Both deferrals, taken up: the container pipeline here, the browser runtime as an explicitly-scoped option. |
| [#710](https://github.com/link-assistant/formal-ai/issues/710) / PR [#888](https://github.com/link-assistant/formal-ai/pull/888) | PR #888's own list of what stays open names *"automatic trusted prerequisite installation/recovery"*. Plan 07's seven-item test order has six unchecked boxes. | Items 2-7 of that test order, with the plan-01/plan-04 seams made explicit. |
| [#1085](https://github.com/link-assistant/formal-ai/issues/1085) / PR [#1086](https://github.com/link-assistant/formal-ai/pull/1086) | R1085-2: seed and meta become link data; migrate handlers smallest-first; no new Rust routing. | The toolchain table stops being a `const` array in `src/coding/catalog/languages.rs` and becomes seed rows with probed state. |
| [#699](https://github.com/link-assistant/formal-ai/issues/699) → [#959](https://github.com/link-assistant/formal-ai/issues/959) | Ratchet the handler ledger down; hard-coded predicates move to seed. | 14 hard-coded `setup_hint` strings and 5 hard-coded `environment` strings leave Rust; `src/solver_handlers/installation_conversion.rs` stops being the only "install" surface. |
| [#924](https://github.com/link-assistant/formal-ai/issues/924) / PR [#1007](https://github.com/link-assistant/formal-ai/pull/1007), [#1021](https://github.com/link-assistant/formal-ai/issues/1021) / PR [#1027](https://github.com/link-assistant/formal-ai/pull/1027) | Self-development each release; refuse-by-default publishing ladder (`src/contribution_write_path.rs`). | The same default-deny shape governs installs: nothing is installed without an explicit grant, and never system-wide. |
| [#1091](https://github.com/link-assistant/formal-ai/issues/1091), [#655](https://github.com/link-assistant/formal-ai/issues/655) / PR [#679](https://github.com/link-assistant/formal-ai/pull/679) | The first self-authored task; the replayable Hive-Mind-dispatched loop. | The replay discipline PR #679 established (183 stream-JSON events, byte-for-byte) is the model for replaying a discovered setup procedure offline. |
| [#670](https://github.com/link-assistant/formal-ai/issues/670) (E51) | Browser multi-language execution via WebVM. | Added by the 2026-09-16 reconciliation to match plan 13's coverage table: this plan delivers the docker backend and honest browser probing (L15, L16); the WebVM spike itself is named as `ExecutionBackend::BrowserRuntime { runtime }` and stays open. |
| hive-mind [#2059](https://github.com/link-assistant/hive-mind/issues/2059) | Asks for a separate docker image reachable on the docker network as `link-assistant-formal-ai`, and notes *"we don't store any memory of formal AI system, so once server restarted - it will be reset to its initial state."* | The toolchain ledger is durable and workspace-scoped, so a restart does not lose a discovered procedure; only the installed bytes are disposable. |

## Current state — evidence with file:line, tests, ledgers

### (a) One SWE-bench Lite instance

`src/external_benchmarks/grade.rs:333-362` is the **only** prerequisite probe in the entire Rust tree:

```rust
fn ensure_swebench_runtime() -> Result<(), String> {
    let module = Command::new("python3").args(["-c", "import swebench.harness.run_evaluation"]).output()…;
    if !module.status.success() {
        return Err("the pinned official `swebench` Python harness is not installed".to_string());
    }
    let docker = Command::new("docker").arg("info").output()…;   // grade.rs:346 — the only Command::new("docker") in src/
```

It reports honestly and stops. Its failure becomes `benchmark_unavailable` (`src/external_benchmarks/mod.rs:159-170`, rendered through `vocabulary::render("external_benchmark_swe_unavailable", …)`), which `docs/benchmarks.md:317-320` describes as *"never counted as a solver failure and never replaced by an exact-diff proxy"*. That is correct honesty and the wrong outcome: the missing harness is never turned into a requirement, never searched for, never installed. `.github/workflows/external-benchmarks.yml:113-117` installs it, once, in CI, from a pinned URL a human wrote:

```yaml
python -m pip install "git+https://github.com/SWE-bench/SWE-bench.git@f7bbbb2ccdf479001d6467c9e34af59e44a840f9"
```

So the knowledge "SWE-bench needs this package at this revision" exists in the repository — in a workflow file, unreachable from the runtime.

### (b) A request to add a regression test to this repository

The named tests would be `cargo test --test unit <module>` (the criterion `data/meta/ladder-ratchet.lino:4` already states). `cargo` is not in `src/agent.rs:642-657`'s allowlist, so the call is `AgentError::UnsupportedCommand("cargo")` — a *refusal*, not a "command not found", and refusals carry no publisher, no install procedure and no retry. `src/agentic_coding/command_reroute.rs:163-176` has the only structure that could receive a real 127:

```rust
struct RecipeProgress { files_written, commands_done, command_outputs, failure: Option<StepFailure> }
struct StepFailure { reported, exit_code: Option<i32>, from_run: bool }
```

`StepFailure::report` (`command_reroute.rs:185-210`) emits `agentic_step_failed_with_exit_code` / `agentic_step_failed`. Issue-710 plan 07's first observed limit (`07-prerequisite-discovery-bridge.md:7-9`) is exactly this: *"`command_reroute::RecipeProgress` stops at the first failed recipe step. It does not interpret later recovery evidence or retry that step."* The retry half was fixed during PR #888 (`07:195-199`: "Keep scanning only for the same pending step; clear its failure only on bound successful evidence"); the *interpretation* half — 127 means "a program is missing" — was not.

`grep -rn "command not found\|CommandNotFound" src` returns **zero hits**. Nothing in the tree recognizes the string that ends Kotlin and Scala sessions.

### (c) A Kotlin task on a machine without `kotlinc`

1. `src/coding/catalog/languages.rs:186-201` declares the row:

   ```rust
   ProgramLanguage {
       slug: "kotlin", name: "Kotlin", code_fence: "kotlin",
       execution: ProgramExecution {
           status: ExecutionStatus::Unavailable,
           environment: "…toolchain is not configured in this repository runtime",
           check_command: Some("kotlinc Main.kt -include-runtime -d Main.jar"),
           run_command: "java -jar Main.jar",
           notes: "The Kotlin seed is returned with this warning until a …-backed execution profile is available.",
       },
       setup_hint: "the Kotlin compiler from https://kotlinlang.org/docs/command-line.html (a JDK is required as well)",
       …
   }
   ```

   `status` is a **constant written by a human**. `ExecutionStatus` has exactly two variants (`src/coding/catalog/types.rs:202-206`), `Verified` and `Unavailable`, with `label()` returning `"compiled and ran"` / `"not compiled or run"` (`types.rs:208-214`). There is no third state for "we have not looked".
2. `check_command` is **never executed**. Its fourteen consumers all render it: `src/engine.rs:959-962`, `src/coding/guidance.rs:305-311`, `src/solver.rs:711-714`, `src/coding/program_contract.rs:93-96`, `src/coding/synthesis_runtime.rs:226-252`, `src/agentic_coding/narration.rs:76-92`, `src/solver_handlers/mod.rs:710-739`.
3. `src/engine.rs:893-925` composes the answer; `src/engine.rs:939-951` relabels the output block to `"Expected output after verification"` when `status != Verified`; `src/coding/guidance.rs:291-297` unconditionally prepends `format!("Install {setup_hint}.")` — in en/ru/hi/zh only, with **no Spanish branch**, which is already a five-language doctrine gap in this exact code path.
4. Live evidence: `docs/case-studies/issue-710/plans/06-repository-task-generalization.md` records Kotlin session `ses_f5a282912ffeOLFh21bbx9Dqsl` — *"now writes correct Kotlin and both supporting files, but ends honestly at `kotlinc: command not found` after five rounds"* — and *"The Scala CI job log confirms `scalac: command not found`, exit 127"*. Scala's row is the twin at `languages.rs:170-185`.
5. The knowledge that would fix it exists, in three places none of which the runtime reads:
   - `data/meta/box-language-projects.lino` (199 lines): seven `box_language_project` records with an image, an optional `shell_prelude` and ordered `init_step_N` / `init_command_N` pairs (rust `:15-32`, python `:33-54`, javascript `:55-77` with `shell_prelude "source /home/box/.nvm/nvm.sh"` at `:63`, typescript `:78-101` with `network_required "true"` at `:87`, go `:102-121`, java `:122-139`, ruby `:140-159`), plus three `box_language_project_deferred` rows (c `:160-167`, cpp `:168-175`, csharp `:176-183`) each carrying a `reason` such as *"no dedicated box-c image, the toolchain ships only in the 5.91GB full box image"*. **Kotlin and Scala are in neither list.**
   - `data/meta/box-image-survey.lino` (15 lines): `published` = `("box" "box-rust" "box-python" "box-js" "box-go" "box-java" "box-ruby")`, `published "false"` = `("box-c" "box-cpp" "box-csharp" "box-dotnet")`, `pinned_tag "2.4.0"`, `surveyed_at "2026-08-14T12:16:42Z"`.
   - `src/box_language_projects.rs:315-323` exposes `box_language_contract()` and `box_image_survey()`. **Nothing under `src/` calls either outside tests.**
6. `data/seed/shell-intents.lino` / `src/seed/shell_intents.rs:92-102` is the nearest existing shape:

   ```rust
   pub struct WorkspaceCommands { pub marker: String, pub test: String, pub install: String, pub build: String }
   ```

   with `marker` documented at `:94` as *"File whose presence identifies the workspace toolchain."* It identifies an ecosystem by a marker file; it never checks whether the binary exists.

### (d) A Telegram "run this code" request

1. `src/main.rs:663-682` routes `Command::Telegram` to `run_telegram`.
2. `data/seed/environments.lino:67-75` declares the telegram environment's tools: `("intent_routing" "write_program" "concept_lookup" "fact_lookup" "summarize_conversation" "brainstorm" "coreference" "roleplay" "html_replies")` — **no execution tool of any kind**.
3. The runtime that could execute exists and is unused. `Dockerfile:59` `FROM konard/box-dind:2.1.1`; `Dockerfile:66-67`

   ```dockerfile
   FORMAL_AI_START_ISOLATION=docker \
   FORMAL_AI_START_RUNNER="$ --isolated docker --auto-remove-docker-container --" \
   ```

   `Dockerfile:79` `RUN bun install -g start-command @link-assistant/agent agent-commander`. `scripts/verify-docker-runtime.sh:25-32` asserts both variables. `tests/unit/docker_runtime.rs:6-40` pins the whole Dockerfile shape. **`grep -rn "FORMAL_AI_START_RUNNER\|FORMAL_AI_START_ISOLATION" src` returns nothing.** The image knows how to isolate; the program does not know the image knows.
4. `data/seed/environments.lino:76-82` describes `docker_microservice` with `tools ("telegram_polling" "telegram_webhook" "start_command" "docker_isolation" "inner_docker_daemon" "bundle" "memory")` — names, not capabilities. `src/solver_config.rs:8-41` has `ExecutionSurface::DockerMicroservice` with slug `"docker_microservice"`, used only for self-description prose (`src/solver_handlers/self_awareness.rs:115,128,139,165`).
5. `data/seed/environments.lino:51` and `:64` list `code_exec_box_dind` among desktop and VS Code tools. `grep -rn "code_exec_box_dind" src` finds it **only** in that seed file — there is no implementation.
6. Browser: `data/seed/environments.lino:5-10` lists `eval_js` and no Python. `src/web/worker/formal_ai_worker_14.js:519-535` runs JavaScript in `new Function("console", '"use strict"; ' + code)`; `:536-537` sends everything else to `i18n.noToolchain(language)` = *"the browser sandbox cannot invoke a `${language}` toolchain"*, and `:538-545` prints the **catalog's** expected output under the "not run" label. `src/web/app/main.jsx:1149` carries the standing TODO: `// iteration will wire it to docker / WebVM execution.`
7. `docs/case-studies/issue-710/plans/02-dynamic-discovery-design.md` §7 and `#1138`'s B6 row both record this as "browser answers stay unverified".

### Ledgers, gates, existing containers

- The only container invocation in the repository is shell, CI-only, and for a fixed hello-world corpus: `scripts/verify-box-language-projects.sh:42-47` probes `command -v docker` and `docker info` and *skips* if absent; `:85-87` `docker pull --quiet "$image"`; `:94-97` `network=(--network none)` unless `NETWORK_REQUIRED=true`; `:99-105` streams the corpus in as a tar on stdin (rationale at `:91-93`: bind mounts inherit host uids the unprivileged `box` user cannot write). `scripts/run-box-language-project.sh:38-41` relaxes `set -u` around the prelude because *"Toolchain managers (nvm, rvm) are not `set -u` clean"*.
- `.github/workflows/release.yml:1254-1310` runs that as job `box-language-projects` over a seven-language matrix, after `scripts/free-runner-disk.sh` (`:1288-1291`), whose `DISPOSABLE_PATHS` (`free-runner-disk.sh:20-26`) delete `/usr/share/dotnet` — CI removes the .NET SDK to make room for the box image that would supply it.
- `.github/workflows/agentic-cli-matrix.yml:140-171` is the only `apt-get` path, bounded by `scripts/apt-install-with-retry.sh` with `FORMAL_AI_APT_ATTEMPTS: 3`.
- `scripts/check-wasm-worker-size.rs:25,30`: `MAX_WASM_BYTES: u64 = 512 * 1024`; the shipped worker measured 291,074 bytes (`docs/case-studies/issue-710/plans/07-prerequisite-discovery-bridge.md:381`).
- `data/seed/sources-registry.lino` declares 13 trusted source kinds with licences, APIs and cache paths — the retrieval substrate a publisher lookup needs, used today by the how-to handler and not by any install path.
- `src/coding/concept_discovery.rs:81-91` defines `trait UnknownConceptLookup` whose only implementation is `struct NoLookup` returning `None`; `discover()` at `:189-191` uses it. That is plan 01's subject and this plan's upstream dependency.
- `src/meta_frame.rs:644-700`: `NeedLedger::resolve` gives every need `Planned` (route present) or `Blocked` (no route), with the doc at `:632-641` stating *"Runtime validation must provide per-need evidence before a result can become `Satisfied`."*

## Root causes — numbered

**RC1. Toolchain availability is a human-written constant, not an observation.**
`src/coding/catalog/languages.rs:9` `pub const PROGRAM_LANGUAGES: &[ProgramLanguage]` with `ExecutionStatus::{Verified, Unavailable}` fixed per row (`types.rs:202-206`). *Mechanism:* the system cannot be wrong about its environment because it never looks; and it cannot become right, because becoming right would be a source edit. A machine with `kotlinc` installed still gets "not compiled or run".

**RC2. `check_command` is rendered, never run.**
Fourteen consumers, all formatting (`engine.rs:959-962`, `guidance.rs:305-311`, `solver.rs:711-714`, `program_contract.rs:93-96`, `synthesis_runtime.rs:226-252`, `narration.rs:76-92`). *Mechanism:* the one datum that would answer "does this toolchain work here?" is treated as documentation. #8's "never give a code example without compiling and running it" is therefore structurally unreachable.

**RC3. Exit code 127 has no meaning anywhere in the tree.**
Zero hits for `command not found` / `CommandNotFound` in `src/`. `StepFailure` (`command_reroute.rs:172-176`) carries `exit_code: Option<i32>` and reports it, but nothing classifies it. *Mechanism:* a missing prerequisite is indistinguishable from a compile error, so no recovery can be selected. Issue-710 plan 07's test-order item 3 (`07:69`, unchecked) is exactly this classification.

**RC4. The install knowledge in the repository is unreachable from the runtime.**
`data/meta/box-language-projects.lino` (init commands per language), `data/meta/box-image-survey.lino` (which images exist), `.github/workflows/external-benchmarks.yml:113-117` (the pinned SWE-bench harness), `Dockerfile:66-67` (the isolation runner), `src/coding/catalog/languages.rs:23-240` (14 `setup_hint` URLs) all encode "how to get X". Not one is read at answer time — `box_language_contract()` (`src/box_language_projects.rs:315`) has no non-test caller. *Mechanism:* the system has memorized fourteen answers and can use none of them, which is the worst of both worlds: it is memoization *and* it does not work.

**RC5. There is no installer, only a document converter.**
`src/solver_handlers/installation_conversion.rs` (registered at `src/solver_dispatch.rs:368` as `("installation_conversion", try_installation_conversion)`) converts README install prose between shell and PowerShell. `src/solver_handler_how.rs:47,712` computes `is_install_procedure` only to steer the *source gate* toward official docs. Issue-710 plan 07 states it (`07:20-22`): *"`installation_conversion` extracts command-like steps, but is a document converter, not a safe platform-aware installer or dependency resolver."* *Mechanism:* even a perfectly retrieved procedure has nothing that can execute it under a policy.

**RC6. No workspace-scoped install convention exists.**
The repository has `.formal-ai` for memory (`src/shared_memory.rs:7`), `target/formal-ai-benchmarks` for benchmark payloads (`manifest.rs:143`), `$FORMAL_AI_SOURCE_CACHE_DIR` for retrieved bytes (`solver_handler_how_synthesis.rs:34`), a temp agent sandbox (`agent.rs:60`), and one XDG use (`client_integrations/completion_learning.rs:20`). *Mechanism:* nothing picks a writable directory *in order to install a toolchain into it*, so "workspace-scoped, never system-wide" has no implementation to point at, and the only precedent the system could copy is CI's `apt-get`, which is system-wide.

**RC7. The container pipeline is shell, CI-only, and fixed-corpus.**
`scripts/verify-box-language-projects.sh` + `scripts/run-box-language-project.sh` + `release.yml:1254-1310`. *Mechanism:* the capability exists as a workflow artefact, so #930 and #937 read as unbuilt even though two thirds of the machinery is written. Nothing in Rust can start, reattach to, snapshot or stop a container, and `code_exec_box_dind` is a name in a seed list with no code behind it.

**RC8. Each surface states its limit as a constant instead of measuring it.**
`data/seed/environments.lino` gives each environment a `tools (…)` list written by hand; `src/web/worker/formal_ai_worker_14.js:536` returns a fixed `noToolchain` string; `src/engine.rs:948` relabels output to "Expected output after verification" from a constant. *Mechanism:* #8's "every surface must know its environment limits" is satisfied in prose and violated in fact — the browser says it cannot run Python whether or not a Python runtime is loaded, and the Docker-in-Docker image says it can run nothing at all.

**RC9. The formalizer cannot say what it lacks.**
`src/coding/concept_discovery.rs:85-91` `NoLookup` returns `None`; the memory-contract probe reports *"zero concepts or procedures"* (`07:26-28`, session `ses_f598e46aaffe38H1DJrykzOPCF`, `2 of 9 protocol primitives`). *Mechanism:* "a missing compiler becomes a prerequisite problem to solve" (the #710 plan-06 model, step 5, `06:57-60`) requires the formalizer to emit *"I need a Kotlin compiler"* as a node. It cannot, so the recursion B6 describes has no first step. This plan does not fix that; it defines the interface (`PrerequisiteNeed`) that plan 04 must emit and plan 01 must satisfy, and it works from an observed exit code in the meantime.

## Solution options — at least 3 distinct options for the whole plan

### Option A — Probe → need → publisher lookup → workspace-scoped setup → retry, with containers as one backend among several

*Description.* Five pieces: (1) a `ToolchainProbe` that actually runs `check_command` and records a `ProbeVerdict`; (2) a `PrerequisiteNeed` emitted on a missing-program verdict or a 127, routed through the existing `NeedLedger`; (3) a publisher lookup over `data/seed/sources-registry.lino` producing a `SetupProcedure` with provenance; (4) a `WorkspaceToolchain` root under `.formal-ai/toolchains/<slug>/<content-id>/` into which the procedure installs, under a default-deny grant; (5) a retry of the original step and a durable `ToolchainLedger` row. Execution backends — host sandbox, `ExecutionBox` (docker), `ConversationContainer` (#937), browser runtime — are selected by an `ExecutionBackend` enum, and **absence of any backend is an honest refusal, never a silent skip**.

*Architecture sketch.*

```
data/seed/toolchains.lino            language → program, probe argv, ecosystem, publisher host
data/seed/setup-publishers.lino      program → the trusted publisher its procedure must come from
data/meta/prerequisite-recipe.lino   the ordered recovery sequence (plan 07 §Recovery sequence, as data)
data/meta/toolchain-ledger.lino      what was discovered, from where, with what content id
        │
src/prerequisite/mod.rs      PrerequisiteNeed, RecoveryOutcome, recover()
        ├── probe.rs         ToolchainProbe, ProbeVerdict, probe_command()
        ├── publisher.rs     SetupProcedure, discover_setup_procedure()   ← plan 01's lookup
        ├── install.rs       WorkspaceToolchain, install_scoped()
        └── ledger.rs        ToolchainLedger, record/replay/forget
src/execution_box/mod.rs     ExecutionBox, ExecutionBackend, ConversationContainer, SnapshotPolicy
        │
consumers: coding catalog answers, agentic recipe steps, repository workspace verify (plan 03),
           Telegram (#930), desktop/vscode `code_exec_box_dind`, browser runtime
```

*Pros.* Every piece is reachable from every surface. The probe alone (piece 1) already fixes RC1/RC2/RC8 and is independently shippable. Containers become an optimisation, so a laptop keeps the capability. The ledger gives the forget-and-rediscover proof B6 asks for. Seed data replaces 14 `setup_hint` constants and 5 `environment` constants, which moves the #959 ratchet down.

*Cons.* Five new subsystems. Executing a retrieved procedure is the single most dangerous thing this repository would do; the policy must be airtight. `.formal-ai/toolchains/` can become large and needs a disk policy (`scripts/check-disk-usage-policy.rs`).

*Doctrine fit.* Strong. Associative stack (seed rows + a recipe document), generalization (one recovery sequence for every program, not one branch per language), default-deny, workspace-scoped, forget-and-rediscover, honest failure.

*Effort.* ~14 commit-sized leaves. *Risk.* Medium-high, concentrated in `install.rs`.

### Option B — Container-first: every execution goes through `link-foundation/box`

*Description.* Do not install anything. Select the `konard/box-<language>` image for the task, run the whole compile-and-run inside it, and treat a missing image as the honest limit. Extend `data/meta/box-language-projects.lino` with kotlin and scala rows and get the images published.

*Architecture sketch.* `src/execution_box/` only; `BoxLanguageContract` (`src/box_language_projects.rs:113`) gains a runtime caller; `scripts/run-box-language-project.sh` is reimplemented in Rust; `data/meta/box-image-survey.lino` becomes the availability oracle.

*Pros.* Two thirds written already (RC7 says the machinery exists). No installer, so RC5's danger disappears entirely. Reproducible: the same image, the same bytes, everywhere. Directly delivers #930 and #937, and the #8 "compile before answering" requirement for every language with an image.

*Cons.* `data/meta/box-image-survey.lino:13-16` already records four missing images, and kotlin/scala are in *neither* list — so the two languages B6 names by name are exactly the ones this does not fix, unless someone publishes images, which is not a capability the system can discover. A full box is 5.91 GB (`box-language-projects.lino:167`) and CI already deletes SDKs to fit one. Docker becomes mandatory: no macOS lane, no browser, no plain-laptop path. And nothing is *discovered* — the image list is another memorized table, the same failure as RC4 wearing a container.

*Doctrine fit.* Weak on "everything discoverable must be forgettable and rediscoverable": the image mapping cannot be rediscovered from a trusted publisher; it is surveyed by hand (`box-image-survey.lino:6` `surveyed_at`).

*Effort.* ~7 leaves. *Risk.* Medium, but it buys the wrong thing.

### Option C — Ask the human: detect, explain precisely, and stop

*Description.* Add the probe (piece 1 of Option A) and nothing else. When a program is missing, say exactly which program, which command failed, with which exit code, and what the trusted publisher's documented install command is — retrieved live, shown, **never executed**.

*Architecture sketch.* `src/prerequisite/probe.rs` + `publisher.rs`; no `install.rs`, no ledger, no containers.

*Pros.* Small, safe, honest, and strictly better than today: `setup_hint` becomes retrieved-and-cited rather than hard-coded, which fixes RC1, RC2, RC4 (partly) and RC8. Zero new execution surface. Ships in three leaves.

*Cons.* Does not deliver B6's sentence — "install workspace-scoped, retry" is the requirement, and this stops one step before it. The Kotlin session still ends at `kotlinc: command not found`; it just ends more informatively. #930's "compile before answering" stays unreachable.

*Doctrine fit.* Honest and deterministic, but it is a deferral, and "no deferral" is standing doctrine.

*Effort.* ~3 leaves. *Risk.* Low.

### Option D — Declare prerequisites up front from a manifest and refuse tasks whose manifest is unmet

*Description.* Extend `src/seed/shell_intents.rs:92-102`'s marker idea: each task class declares its required programs in seed; a pre-flight resolves them all; an unmet manifest refuses the task before any work.

*Pros.* Cheap, fully declarative, fits `WorkspaceCommands` exactly, no runtime surprises.

*Cons.* Prerequisites are discovered *by failing*, not by declaration — a transitive one (Kotlin needs a JDK; Scala needs a JDK; the SWE-bench harness needs Docker *and* a Python of the right version) only shows up when a step runs. A declared manifest is a fifteenth memorized table. And refusal before trying contradicts "a hard task is split until each leaf is directly solvable": the split is what discovers the need.

*Doctrine fit.* Weak. *Effort.* ~4 leaves. *Risk.* Low, value low.

## Decision — selected option and reasons

**Option A is selected, with Option B folded in as one backend and Option C as its first three leaves.**

Reasons:

1. Only Option A delivers B6's sentence literally. B stops at the two languages it cannot reach, C stops before installing, D refuses before discovering.
2. Option A is strictly incremental: leaves L1-L3 *are* Option C and are independently valuable (a real probe, a retrieved publisher, an honest message). Leaves L8-L11 *are* Option B (the box backend, #930, #937). If the installer turns out to be too dangerous to enable by default, the plan still leaves the repository better on RC1, RC2, RC4, RC7 and RC8.
3. It resolves the thing that makes RC4 galling: the repository already contains the answers, in `setup_hint` strings, workflow steps and the box contract. Option A *deletes* them as constants and *rediscovers* them from the publisher, which is the forget-and-rediscover proof rather than a claim about it.
4. It keeps default-deny. `src/agent.rs:642-657` already default-denies with the `other =>` arm; `src/contribution_write_path.rs` already refuses publishing by default (PR #1027). Installing joins that ladder as a third grant, not as a new philosophy.
5. It gives B5 (obligations satisfied by evidence) a concrete first customer: a `PrerequisiteNeed` is a need that *cannot* be `Satisfied` by route selection, because `Satisfied` requires the probe to pass afterwards. `src/meta_frame.rs:632-641` already says so; this plan is the first code that honours it.

**Rejections.**

- **Option B** rejected as the whole plan (kept as a backend): the two languages B6 names have no image, image availability is a hand-surveyed table that cannot be rediscovered, and making Docker mandatory deletes the capability on macOS and in the browser.
- **Option C** rejected as the whole plan (kept as leaves L1-L3): it is a deferral of the install step, and the doctrine forbids deferral. Its value is real and is captured by shipping it first.
- **Option D** rejected: it replaces discovery with a fifteenth declared table and refuses work before the split that would find the need.

## Architecture — exact

### New module tree

```
src/prerequisite/mod.rs        PrerequisiteNeed, RecoveryOutcome, recover, RecoveryStep
src/prerequisite/probe.rs      ToolchainProbe, ProbeVerdict, probe_command
src/prerequisite/publisher.rs  SetupProcedure, SetupStep, discover_setup_procedure
src/prerequisite/install.rs    WorkspaceToolchain, InstallGrant, install_scoped
src/prerequisite/ledger.rs     ToolchainLedger, ToolchainRecord
src/execution_box/mod.rs       ExecutionBox, ExecutionBackend, BoxSession
src/execution_box/container.rs ConversationContainer, SnapshotPolicy, ContainerLifecycle
data/seed/toolchains.lino
data/seed/setup-publishers.lino
data/meta/prerequisite-recipe.lino
data/meta/toolchain-ledger.lino
```

Collision check (`grep -rn` over `src tests scripts data` at the baseline): `ToolchainProbe` 0, `ProbeVerdict` 0, `PrerequisiteNeed` 0, `prerequisite_discovery` 0, `SetupProcedure` 0, `setup_procedure` 0, `WorkspaceToolchain` 0, `ToolchainLedger` 0, `toolchain_ledger` 0, `ExecutionBox` 0, `ContainerSession` 0, `ConversationContainer` 0, `SnapshotPolicy` 0, `BoxSession` 0, `MissingPrerequisite` 0, `RuntimeAvailability` 0, `probe_command` 0, `Provisioner` 0. **Do not use** `ProbeOutcome` (taken: `src/reasoning_standard/refutation.rs:59`), `RecipeProgress` (taken: `src/agentic_coding/command_reroute.rs:163`), `NeedLedger` (taken: `src/meta_frame.rs:644`), `ExecutionStatus`/`ProgramExecution` (taken: `src/coding/catalog/types.rs:194,203`), `ExecutionSurface` (taken: `src/solver_config.rs:8`), `WorkspaceCommands` (taken: `src/seed/shell_intents.rs:92`), `install_procedure` (near-collision with the local `is_install_procedure` at `src/solver_handler_how.rs:47,712`), `BoxLanguage*` (taken: `src/box_language_projects.rs`), `execution_environment` (35 existing hits).

### Probing

```rust
// src/prerequisite/probe.rs

/// One executable the system may need, and how to find out whether it is here.
///
/// Read from `data/seed/toolchains.lino`; never written in Rust.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolchainProbe {
    /// The program name as a shell would resolve it (`kotlinc`, `scalac`, `docker`).
    pub program: String,
    /// The argv that proves it works, e.g. `["-version"]`. Not a compile.
    pub argv: Vec<String>,
    /// Substring the successful output must contain, when the publisher documents one.
    pub expect: Option<String>,
    /// Toolchains this one needs first (`kotlinc` → `java`).
    pub requires: Vec<String>,
}

/// What a probe observed. There are three states, not two: today's
/// `ExecutionStatus` (`src/coding/catalog/types.rs:202-206`) has no way to say
/// "we have not looked", which is why every row is a human's guess.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProbeVerdict {
    /// Ran, exit 0, `expect` satisfied. Carries the version line observed.
    Present { version: String },
    /// The program is not on the path, or exited 127.
    Missing { exit_code: Option<i32>, stderr: String },
    /// Present but unusable (wrong version, broken install, permission denied).
    Unusable { exit_code: Option<i32>, stderr: String },
    /// Not probed in this environment, and the reason why.
    NotProbed { reason: String },
}

/// Run `probe` in `root` and report exactly what happened.
///
/// Deterministic in its verdict shape; the observed version string is recorded,
/// never asserted against a hard-coded value.
pub fn probe_command(probe: &ToolchainProbe, root: &Path) -> ProbeVerdict;
```

`src/coding/catalog/types.rs` gains a third `ExecutionStatus` variant and a derivation, which is the minimal change to RC1 across all fourteen consumers:

```rust
pub enum ExecutionStatus { Verified, Unavailable, NotProbed }

impl ExecutionStatus {
    /// Derive the status from a live probe rather than from a constant.
    #[must_use]
    pub fn from_verdict(verdict: &ProbeVerdict) -> Self { … }
}
```

`src/engine.rs:939-951`'s `execution_output_label` gains the `NotProbed` case: the label becomes *"Output, not observed in this environment"* and the report names the probe that was not run. The four-language match in `src/coding/guidance.rs:291-311` gains its missing Spanish branch in the same commit (it is a five-language doctrine gap today, independent of this plan).

### Need emission — the B4/B1 seam

```rust
// src/prerequisite/mod.rs

/// A missing executable, stated as a requirement rather than as an error.
///
/// This is the in-memory projection of a plan 00 §4.1 `need` record whose
/// `kind` is `prerequisite` and whose `subject` is the missing program; the
/// durable form is the link record, not this struct. It is what plan 04's
/// formalizer emits and what plan 01's `SourceLookup` satisfies. It is also
/// produced directly from an observed `ProbeVerdict::Missing` or from a failing
/// `Workspace::run` (plan 03), so recovery works before either of those lands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrerequisiteNeed {
    /// The missing program.
    pub program: String,
    /// The requirement that needed it, verbatim, in its original language.
    pub source_span: String,
    /// The exact call that failed: command, exit code, stderr.
    pub observed: ProbeVerdict,
    /// Host platform as observed, never inferred from the requested language
    /// (issue-710 plan 07 recovery step 2, `07:44-46`).
    pub platform: Platform,
    /// Needs this one depends on, discovered recursively; cycles are detected.
    pub requires: Vec<String>,
}
```

`PrerequisiteNeed` is written into the existing `NeedLedger` as a row whose status starts `Blocked` (`src/meta_frame.rs:660-668` already assigns `Blocked` when a leaf has no route). It may become `Planned` when a `SetupProcedure` is selected, and **`Satisfied` only when a re-probe returns `Present`**. That is `src/meta_frame.rs:632-641`'s stated contract, honoured.

### Publisher lookup — plan 01's seam

```rust
// src/prerequisite/publisher.rs

/// An install procedure, with the provenance that makes it trustable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetupProcedure {
    pub program: String,
    /// The trusted publisher the procedure came from, by `sources_registry` id.
    pub source_id: String,
    /// The exact URL fetched and the content id of the bytes retrieved.
    pub source_url: String,
    pub content_id: String,
    /// Platform this procedure is valid for.
    pub platform: Platform,
    /// Ordered steps, each with its own postcondition.
    pub steps: Vec<SetupStep>,
    /// The probe that must pass afterwards. Without it the procedure is refused.
    pub postcondition: ToolchainProbe,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetupStep {
    pub command: String,
    /// Where the step is allowed to write. A step outside the workspace root is
    /// refused before execution, whatever the fetched text says.
    pub writes_under: PathBuf,
    /// Expected artifact digest, when the publisher documents one.
    pub digest: Option<String>,
}

/// Find `program`'s official setup procedure through the trusted-source
/// registry, deepest-first over the source kinds declared in
/// `data/seed/sources-registry.lino`.
///
/// Bounded by evidence, not by a budget: the search ends when a procedure with
/// a postcondition is found, when the publisher list is exhausted, or when a
/// cycle is detected. Exhaustion returns `None` and is reported as such.
///
/// This calls the plan 00 §4.2 contract, `SourceLookup::lookup(need, bounds)`,
/// with the need's `kind` set to `prerequisite`. It does not call
/// `crate::coding::concept_discovery::UnknownConceptLookup`
/// (`src/coding/concept_discovery.rs:81-83`) directly — that trait's only
/// implementation today is `NoLookup` (`:85-91`), and plan 01 decides whether
/// it survives as a thin adapter over `SourceLookup` or is replaced. Until
/// plan 01 lands, `discover_setup_procedure` returns `None` and says so.
pub fn discover_setup_procedure<L: SourceLookup>(
    need: &PrerequisiteNeed,
    lookup: &mut L,
    bounds: &LookupBounds,
) -> Option<SetupProcedure>;
```

`data/seed/setup-publishers.lino` pins *which host is authoritative for which program*, because ranking is not authority — issue-710 plan 07's observed limit (`07:10-12`): *"`.gov`/`.edu` preference also does not identify a compiler's official source."* A lookalike host is refused, and the refusal is a recorded event.

### Workspace-scoped installation and the grant

```rust
// src/prerequisite/install.rs

/// A toolchain installed for this workspace only.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceToolchain {
    pub program: String,
    /// `.formal-ai/toolchains/<program>/<content-id>/` beneath the workspace root.
    pub prefix: PathBuf,
    /// Environment bindings subsequent steps must carry. Shell state does not
    /// persist between tool calls (issue-710 plan 07 recovery step 6, `07:55-57`),
    /// so these are explicit and replayable, never `export`ed and hoped for.
    pub environment: BTreeMap<String, String>,
    pub content_id: String,
}

/// Permission to install. Default-deny, matching `src/agent.rs:642-657`'s
/// `other =>` arm and `src/contribution_write_path.rs`'s refuse-by-default ladder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallGrant {
    /// Nothing is installed. The need is reported. This is the default.
    Refused,
    /// Only the programs named here, only under the workspace root.
    Allowed { programs: Vec<String>, root: PathBuf },
}

/// Execute `procedure` under `grant`, then verify its postcondition.
///
/// # Errors
/// Refuses, before running anything, when: the grant does not name the program;
/// any step writes outside `grant.root`; a documented digest does not match;
/// free disk is below the procedure's stated requirement; or the procedure has
/// no postcondition probe. A successful command is not success — only the
/// postcondition probe returning `Present` discharges the need
/// (issue-710 plan 07 recovery step 7, `07:58-60`).
pub fn install_scoped(
    procedure: &SetupProcedure,
    grant: &InstallGrant,
) -> Result<WorkspaceToolchain, PrerequisiteError>;
```

No step may run `sudo`, write outside `grant.root`, or modify `PATH` outside the returned `environment` map. Fetched text cannot widen the grant — the grant is constructed by the caller from operator configuration and is immutable for the duration of the install. `--allow-install <program>` on the CLI and a `pkg_prerequisite_install` associative package (the shape `src/associative_package.rs:510-551` already uses for permissions) are the only two ways to move it off `Refused`.

### Recovery sequence as data

`data/meta/prerequisite-recipe.lino` encodes issue-710 plan 07's eight-step recovery sequence (`07:42-61`) as ordered `meta_step` records with `precondition`/`postcondition`, grounded by `tests/unit/specification/prerequisite_recipe.rs` exactly as `data/meta/agentic-coding-recipe.lino` is grounded by `tests/unit/specification/agentic_meta_algorithm.rs` (`docs/meta-algorithm.md:255-266`):

```
prerequisite_recipe
  record_type "meta_recipe"
  issue "1138"
  topic "prerequisite_discovery"
prerequisite_step_bind
  record_type "meta_step"
  order "1"
  id "bind_failure"
  detail "Bind the failure to the actual prior call and recipe step. A nonzero exit is not always a missing compiler; permission denial is not installation consent."
  source_file "src/prerequisite/mod.rs"
prerequisite_step_observe
  order "2"
  id "observe_platform"
  detail "Observe the missing executable, platform and available runtime or manager in the client workspace. Do not infer the host from the requested language."
… (look_up_publisher, formalize_procedure, prefer_workspace_scope, lower_to_tools, retry_and_recheck, retain_experience)
```

The recovery function walks that document:

```rust
/// Recover from `need`, or report precisely why recovery is not possible.
///
/// Every step — the first probe, each setup step, the re-probe — appends a plan
/// 00 §4.3 `evidence` record referencing `need`, so the need's transition to
/// `satisfied` is backed by observations rather than by having chosen a method.
/// Called from inside `Workspace::run` (plan 00 §4.4) as well as directly.
pub fn recover<L: SourceLookup>(
    need: &PrerequisiteNeed,
    grant: &InstallGrant,
    lookup: &mut L,
    bounds: &LookupBounds,
    ledger: &mut ToolchainLedger,
) -> RecoveryOutcome;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecoveryOutcome {
    /// Installed and re-probed `Present`; the original step may be retried.
    Recovered { toolchain: WorkspaceToolchain, retry: String, evidence: Vec<Evidence> },
    /// A procedure was found but the grant refused it. Names the procedure and the grant.
    NotPermitted { procedure: SetupProcedure },
    /// No trusted publisher procedure was found. Names every source consulted.
    /// Maps to the need's `unsatisfiable` terminal state (plan 00 §4.1) and is
    /// reported to the user, never converted into completion prose (§6.6).
    NotFound { consulted: Vec<String> },
    /// Installed but the postcondition still failed. Names both observations.
    StillMissing { before: ProbeVerdict, after: ProbeVerdict },
}
```

Every variant is a statement the answer can make verbatim. There is no variant meaning "probably fine".

### The toolchain ledger and the forget-and-rediscover proof

`data/meta/toolchain-ledger.lino`, append-only, one record per discovered procedure:

```
toolchain_ledger
  record_type "toolchain_ledger"
  policy "append_only"
toolchain_kotlinc
  record_type "toolchain_record"
  program "kotlinc"
  source_id "kotlin_official"
  source_url "https://kotlinlang.org/docs/command-line.html"
  content_id "<sha256 of the retrieved bytes>"
  platform "darwin-arm64"
  postcondition_program "kotlinc"
  postcondition_argv ("-version")
  rediscover "https://kotlinlang.org/docs/command-line.html"
  observed_version "kotlinc-jvm <version as observed>"
  installed_prefix ".formal-ai/toolchains/kotlinc/<content-id>/"
```

The installed *bytes* are disposable; the *record* is durable — the same distinction `docs/case-studies/issue-710/plans/06-repository-task-generalization.md:230-238` requires of the source cache: *"retain a compact reconstruction record with the original cache ID, source URL/recipe, and provenance; never carry its disposable payload."*

Forget-and-rediscover proof (`formal-ai learn forget --toolchain kotlinc` then re-run):

1. Delete the record **and** the installed prefix.
2. Re-run the task. The probe returns `Missing`, the lookup returns the same publisher, the procedure's `content_id` equals the deleted one.
3. Replay offline from the retained URL + content id and assert the same `content_id` again.
4. Assert `observed_version` matches whatever was observed, not a hard-coded string.

### The docker execution pipeline (#930 and #937)

```rust
// src/execution_box/mod.rs

/// Where code actually runs. Selected per task; absence is a refusal, not a skip.
///
/// **This enum is plan 00 §4.4's single "where does this run" vocabulary. It
/// absorbs plan 03's `VerifyBackend`, which declared the same idea a second
/// time: `Sandbox` becomes `HostSandbox`, `BoxImage` becomes `Box`, and
/// `SweBenchImage` joins here. This plan lands before plan 03 in plan 00 §5's
/// order, so plan 03 consumes it (plan 00 §9 R7).**
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionBackend {
    /// The existing allowlisted sandbox (`src/agent.rs:177`).
    HostSandbox,
    /// A one-shot container for one compile-and-run (#930).
    Box { image: String },
    /// The pinned upstream SWE-bench instance image, when Docker is present.
    /// Optional: absence downgrades to `HostSandbox`, never to success (plan 03).
    SweBenchImage { instance_id: String },
    /// A per-conversation detached container (#937).
    Conversation { conversation_id: String },
    /// The browser runtime, when one is loaded.
    BrowserRuntime { runtime: String },
}

/// One container lifetime.
pub struct ExecutionBox {
    backend: ExecutionBackend,
    /// `--network none` unless the task's contract declares network is required,
    /// mirroring `scripts/verify-box-language-projects.sh:94-97`.
    network: NetworkPolicy,
    /// Wall-clock deadline. Exceeding it is a reported failure, never a truncation.
    deadline: Duration,
}

impl ExecutionBox {
    /// Start (or reattach to) the box. Inputs travel as a tar stream on stdin,
    /// not a bind mount — the uid and mount-namespace rationale is recorded at
    /// `scripts/verify-box-language-projects.sh:91-93`.
    pub fn open(backend: &ExecutionBackend, policy: &BoxPolicy) -> Result<Self, BoxError>;

    pub fn run(&mut self, script: &str, inputs: &[(String, Vec<u8>)]) -> Result<BoxObservation, BoxError>;

    /// Detach without destroying (issue #937). The container stops when idle.
    pub fn detach(self) -> Result<BoxHandle, BoxError>;
}
```

```rust
// src/execution_box/container.rs

/// A container bound to one conversation, reattachable across restarts (#937).
pub struct ConversationContainer {
    pub conversation_id: String,
    pub image: String,
    pub handle: Option<BoxHandle>,
    pub idle_after: Duration,
    pub restore: SnapshotPolicy,
}

/// How a stopped container's state comes back (#937's explicit choice).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnapshotPolicy {
    /// Default: `docker commit` / export the filesystem and restore it.
    Snapshot,
    /// Settings-selectable fallback: replay the recorded command log into a
    /// fresh container. Divergence between replay and snapshot is reported.
    Replay,
}

impl ConversationContainer {
    pub fn attach(&mut self) -> Result<&mut ExecutionBox, BoxError>;
    /// Stop when idle longer than `idle_after`, preserving state per `restore`.
    pub fn stop_if_idle(&mut self, now: Instant) -> Result<(), BoxError>;
    pub fn restore(&mut self) -> Result<(), BoxError>;
}
```

**Lifecycle.** `attach` → reuse a running handle, or restore a snapshot, or (on `Replay`) start a fresh container and replay the recorded command log, comparing the resulting filesystem digest with the snapshot digest when both exist and reporting any divergence. `stop_if_idle` snapshots then stops. A restart of the process re-reads handles from `data/meta/toolchain-ledger.lino`'s sibling `conversation_containers` records, which answers hive-mind#2059's *"once server restarted - it will be reset to its initial state"*.

**Snapshot vs replay.** Snapshot is the default because it is the only one that preserves state the system did not produce (a downloaded dependency cache, a compiler's own scratch). Replay is exact and cheap but only reproduces what was recorded; it is offered as the settings-selectable fallback #937 asks for, and when both are available the plan *measures* whether they agree rather than asserting it.

**Timeouts are honest failures, not budgets.** `ExecutionBox::run` carries a `deadline`. Exceeding it produces `BoxObservation { timed_out: true, elapsed, deadline, partial_output }` and the answer says *"the program did not finish within N seconds; here is what it printed before the deadline"*. It never silently truncates and never reports a pass. #930's "halve the iteration bound and retry, reporting which N timed out and which N stopped timing out" is implemented as an explicit descending ladder recorded in full: every N tried, every outcome, published in the answer. That is a measurement, not a budget — the distinction `src/agent.rs:40-49` already draws for `PYTHON_TIME_BUDGET_FLOOR` (*"One minute is a backstop, not the expected cost … The floor only has to be wide enough that start-up latency never decides whether a command that would have succeeded is instead reported as a timeout"*). #930's ten-minute hard fail becomes a `RecoveryOutcome::StillMissing`-shaped report with the full verbose log attached, not a silent give-up.

**Telegram wiring (#930).** `data/seed/environments.lino:67-75` gains `"code_execution"` in the telegram `tools` list and a new `execution` facet naming the backend; `src/telegram_runtime.rs` routes a code answer through `ExecutionBox` before composing the reply; when no backend is available the reply carries #8's exact honest sentence — *"this code was not tested, not compiled, not checked"* — localized in all five languages from seed, not from a Rust `match`.

**Which image.** `src/box_language_projects.rs:315-323`'s `box_language_contract()` and `box_image_survey()` gain their first non-test callers. `data/meta/box-language-projects.lino` gains kotlin and scala rows — as `box_language_project_deferred` with an honest `reason`, because `data/meta/box-image-survey.lino:13-16` shows no `box-kotlin` or `box-scala` image exists. So for those two languages the box backend reports unavailable and control falls to the prerequisite path, which is the point: the two languages B6 names are the ones the container option cannot serve.

### Browser Python runtime option

`ExecutionBackend::BrowserRuntime { runtime }` with two candidate runtimes, neither bundled:

- **Pyodide** (CPython on WebAssembly, ~10 MB core plus per-package wheels). It is a *lazily fetched* asset, never part of `src/web/formal_ai_worker.wasm`, whose ceiling is `MAX_WASM_BYTES: u64 = 512 * 1024` (`scripts/check-wasm-worker-size.rs:30`) against a current 291,074 bytes. Bundling would blow that ceiling by twenty times; the loader is a separate, user-initiated fetch with the download size stated up front, exactly as the OCR bundle already does (`src/web/app.js` declares *"Downloads about 6 MB on first use: OCR wrapper, worker, WebAssembly core, and English traineddata"*).
- **WebVM / `rust-web-box`** (a full Linux image in the browser) — recorded as the deferred option in PR #331's scope note and in `VISION.md:269` (*"Browser-only mode can start with JavaScript evaluation and later experiment with WebVM"*). Out of scope for this plan; the `runtime` field makes it addable without a redesign.

**Honesty behaviour in the browser.** Today `src/web/worker/formal_ai_worker_14.js:536-537` returns a constant `noToolchain(language)` string whether or not a runtime is present. It becomes a probe: with no runtime loaded, the answer says which runtime *could* be loaded, how large it is, and that the output shown is the catalog's expectation and was not observed; with a runtime loaded, the program is actually run and the observed output is shown with `ExecutionStatus::Verified`. The user-initiated download is the grant — nothing is fetched without a click, which is the browser's form of default-deny. The five-language strings live in `src/web/i18n-catalog-messages.lino`, not in a JS `match`.

### Default-deny summary

| Action | Default | How it is granted | Where enforced |
| --- | --- | --- | --- |
| Run a program | deny | seed allowlist row (plan 03) | `src/agent.rs:642-657` `other =>` arm |
| Fetch a publisher page | allow (read-only, trusted registry only) | `data/seed/sources-registry.lino` | `publisher.rs` refuses non-registry hosts |
| Install a toolchain | **deny** | `InstallGrant::Allowed { programs, root }` via `--allow-install` or `pkg_prerequisite_install` | `install.rs::install_scoped` |
| Write outside the workspace | **deny, unconditionally** | never | `install.rs`, before any step runs |
| Start a container | deny | operator config naming the backend | `ExecutionBox::open` |
| Network inside a container | **deny** (`--network none`) | the task contract's `network_required` | `ExecutionBox`, mirroring `verify-box-language-projects.sh:94-97` |
| Download a browser runtime | deny | an explicit user click, size shown | `src/web/` loader |

### Failure and honesty behaviour

- `ProbeVerdict::NotProbed` is a real answer. "We did not look" is never rendered as "it does not work".
- `RecoveryOutcome::NotFound { consulted }` names every source consulted, so the gap is attributable to the registry rather than to the system's mood.
- `RecoveryOutcome::NotPermitted` names the procedure it found and the grant that refused it, so a maintainer can grant it deliberately.
- A successful setup command with a failing postcondition is `StillMissing`, never success (`07:58-60`).
- A timeout is reported with the deadline, the elapsed time, and the partial output.
- `docs/case-studies/issue-710/plans/07-prerequisite-discovery-bridge.md:80-81` is binding: *"Do not install a compiler manually and count that as the system's recovery. Do not count source-token coverage or a read-back plan as executed semantics."* Every test below is written so that a manual install fails it.

## Tests first — held-out cases in en/ru/hi/zh/es with actual prompt text

The held-out toolchain must be one **no seed file, workflow, catalog row or ledger mentions today**. `kotlinc` and `scalac` appear in `src/coding/catalog/languages.rs:177,193` and would leak. The held-out programs are therefore **`zig`** (first family) and **`gleam`** (second family); `grep -rn "zig\|gleam" src data scripts .github` must return nothing before these tests are written, and the tests assert that.

**Family 1 — a missing compiler becomes a requirement, is discovered, installed workspace-scoped, and the task retried.**

| Lang | Prompt (verbatim) |
| --- | --- |
| en | `Write a program in Zig that prints the sum of the numbers from one to ten, then actually compile and run it here and show me the real output.` |
| ru | `Напиши на Zig программу, которая печатает сумму чисел от одного до десяти, затем действительно скомпилируй и запусти её здесь и покажи мне настоящий вывод.` |
| hi | `Zig में एक प्रोग्राम लिखो जो एक से दस तक की संख्याओं का योग छापे, फिर उसे यहीं सचमुच संकलित करके चलाओ और मुझे असली आउटपुट दिखाओ।` |
| zh | `用 Zig 写一个打印一到十之和的程序，然后在这里真正编译并运行它，把真实的输出给我看。` |
| es | `Escribe un programa en Zig que imprima la suma de los números del uno al diez, luego compílalo y ejecútalo realmente aquí y muéstrame la salida real.` |

Expected, with `InstallGrant::Refused` (the default): the answer names `zig`, quotes the observed `command not found` and exit 127, names the trusted publisher it found and the exact install command it would run, and states that nothing was installed because no grant was given. **The catalog's expected output must not be presented as observed output.**

Expected, with `InstallGrant::Allowed { programs: ["zig"], root: <workspace> }`: `zig` lands under `.formal-ai/toolchains/zig/<content-id>/`, the postcondition probe returns `Present` with the observed version, the program compiles and runs, and the shown output is the observed one. Nothing outside the workspace root changed — asserted by a filesystem digest of `$HOME` and `/usr/local` before and after.

**Family 2 — the same recovery, a different program, no new code.**

| Lang | Prompt (verbatim) |
| --- | --- |
| en | `The build for this project needs a compiler that is not on this machine. Find out which one, get it, and then run the project's own test command.` |
| ru | `Для сборки этого проекта нужен компилятор, которого нет на этой машине. Выясни, какой именно, установи его и затем запусти собственную тестовую команду проекта.` |
| hi | `इस प्रोजेक्ट को बनाने के लिए एक ऐसे कंपाइलर की ज़रूरत है जो इस मशीन पर नहीं है। पता करो कौन-सा, उसे लाओ, और फिर प्रोजेक्ट की अपनी टेस्ट कमांड चलाओ।` |
| zh | `这个项目的构建需要一个本机没有的编译器。查出是哪一个，把它装好，然后运行项目自己的测试命令。` |
| es | `La compilación de este proyecto necesita un compilador que no está en esta máquina. Averigua cuál, consíguelo y luego ejecuta el propio comando de pruebas del proyecto.` |

Passing requires the *same* `recover()` to serve `gleam` with no Rust change — one seed row in `data/seed/toolchains.lino` and one publisher row, or, better, neither: the program name is read from the observed failure and the publisher from the registry.

**Family 3 — the honesty case, no grant, no runtime.** The Telegram and browser surfaces answering "run this code":

| Lang | Prompt (verbatim) |
| --- | --- |
| en | `Run this and tell me exactly what it prints: print(sum(range(1, 11)))` |
| ru | `Запусти это и скажи точно, что оно печатает: print(sum(range(1, 11)))` |
| hi | `इसे चलाओ और मुझे ठीक-ठीक बताओ कि यह क्या छापता है: print(sum(range(1, 11)))` |
| zh | `运行这个并准确告诉我它打印了什么：print(sum(range(1, 11)))` |
| es | `Ejecuta esto y dime exactamente qué imprime: print(sum(range(1, 11)))` |

Expected with no execution backend: the reply contains the honest sentence in the prompt's language (`"this code was not tested, not compiled, not checked"`, #8's wording) and **does not** contain `55`. Expected with a backend: the reply contains `55` and the observation record. A reply containing `55` without an observation is the specific failure this test exists to catch.

### Unit / integration / specification tests

| Path | Name | Asserts |
| --- | --- | --- |
| `tests/unit/issue_1138_toolchain_probe.rs` | `probe_reports_missing_with_the_observed_exit_code` | a probe for a program that does not exist yields `Missing { exit_code: Some(127), .. }` with non-empty stderr. |
| same | `probe_reports_present_with_the_observed_version` | a probe for `python3` yields `Present { version }` where `version` is whatever was printed, not a pinned string. |
| same | `an_unprobed_toolchain_is_not_reported_as_unavailable` | `ExecutionStatus::from_verdict(&NotProbed { .. })` is `NotProbed`, and `execution_output_label` says so rather than "Expected output after verification". |
| same | `catalog_status_comes_from_a_probe_not_a_constant` | no `ExecutionStatus::Verified` or `::Unavailable` literal remains in `src/coding/catalog/languages.rs`. |
| `tests/unit/issue_1138_prerequisite_need.rs` | `exit_127_becomes_a_need_not_an_error` | a `StepFailure { exit_code: Some(127), .. }` (`src/agentic_coding/command_reroute.rs:172-176`) produces a `PrerequisiteNeed` naming the program. |
| same | `a_permission_denial_is_not_installation_consent` | exit 126 / `Permission denied` yields `Unusable`, never `Missing`, and never triggers an install (plan 07 recovery step 1, `07:42-44`). |
| same | `an_ordinary_compile_error_is_not_a_missing_prerequisite` | a nonzero exit with a compiler diagnostic yields no need. |
| same | `the_platform_is_observed_not_inferred_from_the_language` | a Kotlin task on darwin yields `Platform::Darwin`, not a JVM-implied platform (plan 07 step 2, `07:45-46`). |
| same | `a_prerequisite_need_is_blocked_until_the_reprobe_passes` | the `NeedLedger` row is `Blocked`, then `Planned`, and is `Satisfied` only after `Present` (`src/meta_frame.rs:632-641`). |
| `tests/unit/issue_1138_setup_publisher.rs` | `a_lookalike_host_is_refused` | a fixture ranking `kotlin-lang.example.com` above the registry host selects neither and records the refusal. |
| same | `a_procedure_without_a_postcondition_is_refused` | `install_scoped` returns `PrerequisiteError::NoPostcondition` before running anything. |
| same | `exhausted_search_names_every_source_consulted` | `NotFound { consulted }` lists the registry ids, in order. |
| same | `a_cycle_is_detected_and_reported` | `a` requires `b` requires `a` terminates with a named cycle. |
| same | `two_dependents_share_one_setup` | `kotlinc` and `scalac` both needing a JDK install it once (plan 07 test-order item 5, `07:73-74`). |
| `tests/unit/issue_1138_install_scope.rs` | `install_refuses_without_a_grant` | default `InstallGrant::Refused` installs nothing and changes no bytes. |
| same | `a_step_writing_outside_the_root_is_refused_before_execution` | a fetched procedure containing `/usr/local/bin` is refused; no process is spawned. |
| same | `a_digest_mismatch_is_refused_without_data_loss` | wrong checksum → refusal, the prior tree byte-identical (plan 07 item 6, `07:75-76`). |
| same | `insufficient_disk_is_refused_before_download` | stated requirement above free space → refusal with both numbers. |
| same | `a_successful_command_with_a_failing_postcondition_is_still_missing` | `StillMissing { before, after }`, never success (`07:58-60`). |
| same | `the_environment_is_returned_explicitly_not_exported` | `WorkspaceToolchain::environment` is non-empty and the process env is unchanged (`07:55-57`). |
| `tests/unit/issue_1138_toolchain_ledger.rs` | `forget_and_rediscover_reproduces_the_content_id` | delete record + prefix, re-run, identical `content_id`. |
| same | `the_ledger_retains_the_recipe_not_the_payload` | after forgetting, the record's `rediscover` URL survives and the installed bytes do not. |
| same | `a_restart_reattaches_from_the_ledger` | a fresh process finds the installed toolchain without re-probing the publisher. |
| `tests/unit/issue_1138_execution_box.rs` | `a_timeout_is_a_reported_failure_with_both_numbers` | `timed_out: true`, `elapsed`, `deadline`, partial output; no pass. |
| same | `the_halving_ladder_records_every_n_it_tried` | #930's ladder reports each N and its outcome; nothing is hidden. |
| same | `network_is_denied_unless_the_contract_requires_it` | `--network none` by default (`verify-box-language-projects.sh:94-97`). |
| same | `a_missing_docker_daemon_is_a_refusal_not_a_skip` | no backend → the honest sentence, never a silent pass. |
| `tests/unit/issue_1138_conversation_container.rs` | `an_idle_container_stops_and_restores_its_state` | write a file, idle past `idle_after`, reattach, file present. |
| same | `replay_and_snapshot_are_compared_when_both_exist` | divergence is reported, not hidden. |
| `tests/unit/issue_1138_surface_honesty.rs` | `an_unverified_answer_says_so_in_five_languages` | the honesty sentence resolves in en/ru/hi/zh/es from seed; `55` absent. |
| same | `guidance_has_a_spanish_branch` | `src/coding/guidance.rs:291-311` no longer falls through to English for Spanish. |
| `tests/unit/specification/prerequisite_recipe.rs` | `recipe_matches_the_live_source` | every `meta_step.source_file` exists; `order` contiguous; each `id` has an arm. |
| same | `recipe_is_rediscoverable` | delete-and-regenerate reproduces the committed content id. |
| `tests/unit/issue_1138_held_out_toolchain.rs` | `the_held_out_program_is_absent_from_the_repository` | `grep` for `zig`/`gleam` over `src`, `data`, `scripts`, `.github` finds nothing. |
| `tests/integration/issue_1138_recovery_live.rs` *(`#[ignore]`, network)* | `a_missing_held_out_compiler_is_discovered_installed_and_retried` | the whole family-1 flow, all five languages. |
| `tests/unit/docs_requirements/issue_1138.rs` | `requirements_shard_matches_the_plan` | the shard names every R1138-6-x. |

### Gates and ratchets

1. `data/meta/debt-ratchet.lino` (22 lines, issue #1126) gains `hardcoded_setup_hints` at the current **14** and `hardcoded_execution_environments` at the current **5**, both shrink-only, reaching `0` when RC1/RC4 close. **Both are added through the strict two-sided checker plan 09 leaves 1-5 install, so they land after those leaves; `check-debt-ratchet.rs` is at-or-below today and would not catch an improvement that was never recorded (plan 00 §9 X6).**
2. `data/meta/ci-gates/check-prerequisite-recipe.lino` registers the specification test with its justification (R1085-16's requirement that every gate carry one).
3. `scripts/check-hardcoded-language.rs`'s 1,286-literal allowlist may not grow for any string this plan adds; the honesty sentences are seed rows.
4. `scripts/check-wasm-worker-size.rs`'s `MAX_WASM_BYTES = 512 * 1024` is **not raised**. Any browser runtime is a lazily fetched asset outside that budget, and a test asserts the worker binary is unchanged.
5. `data/meta/toolchain-ledger.lino` is append-only, enforced the way `data/meta/self-hosting-ledger.lino` is (`scripts/self-hosting-metric.rs:734-786` appends and never rewrites).
6. The existing `benchmark ratchet` floors are untouched; nothing here may lower a recorded score.

### Benchmark commands — measure and record whatever it is

```sh
# Probe every catalogued language on this machine and record the honest table.
cargo run --bin formal-ai -- environments --probe

# The held-out recovery, refused (default) and granted.
cargo run --bin formal-ai -- chat "Write a program in Zig that prints the sum of the numbers from one to ten, then actually compile and run it here and show me the real output."
cargo run --bin formal-ai -- chat --allow-install zig "<same prompt>"

# Forget and rediscover.
cargo run --bin formal-ai -- learn forget --toolchain zig
cargo run --bin formal-ai -- chat "<same prompt>"      # content id must match

# The box backend, the seven languages that have images.
scripts/verify-box-language-projects.sh                 # unchanged, still the CI reference
cargo run --bin formal-ai -- chat --backend box "Write a hello world in Go and run it."

# SWE-bench with the harness prerequisite recovered rather than preinstalled.
cargo run --bin formal-ai -- benchmark run --suite swebench_lite --slice 1 --online --append
```

Record the probe table, the recovery outcome for each of the five languages, and the number of catalogued languages whose status is `Verified` by observation rather than by constant — whatever those numbers are. A run in which zero languages probe `Verified` is a publishable result; a run whose numbers are not recorded is not.

## Implementation leaves — ordered, each individually verifiable and commit-sized

- [x] **L1.** Add `src/prerequisite/probe.rs` with `ToolchainProbe`, `ProbeVerdict`, `probe_command`, and `data/seed/toolchains.lino` carrying the probe argv for every catalogued language. Test: missing/present/unusable/not-probed.
- [ ] **L2.** Add `ExecutionStatus::NotProbed` and `from_verdict` (`src/coding/catalog/types.rs:202-214`); make `src/engine.rs:939-951` and `src/coding/guidance.rs:291-311` render it; add the missing Spanish branch. Test: an unprobed toolchain is not reported as unavailable.
- [ ] **L3.** Delete the 14 `setup_hint` and 5 `environment` constants from `src/coding/catalog/languages.rs` in favour of seed rows; add both to `data/meta/debt-ratchet.lino` as shrink-only ceilings at their current values first, then drive them down in this commit. Test: no `ExecutionStatus::Verified`/`::Unavailable` literal remains in that file.
- [x] **L4.** Add `src/prerequisite/mod.rs` `PrerequisiteNeed` and the classifier that turns a `StepFailure` (`src/agentic_coding/command_reroute.rs:172-176`) or a `WorkspaceError::MissingPrerequisite` (plan 03) into one. Test: 127 vs 126 vs compile error; platform observed.
- [x] **L5.** Write the need into `NeedLedger` as `Blocked`, and forbid `Satisfied` without a passing re-probe. Test: the three-state lifecycle.
- [x] **L6.** Add `src/prerequisite/publisher.rs` `SetupProcedure`, `SetupStep`, `discover_setup_procedure` over `data/seed/sources-registry.lino` + a new `data/seed/setup-publishers.lino`, calling the plan 00 §4.2 `SourceLookup` contract (plan 01 owns its one implementation; `src/coding/concept_discovery.rs:81-91` is the trait it supersedes or adapts). Test: lookalike host, no postcondition, exhaustion, cycle, shared dependency.
- [x] **L7.** Add `src/prerequisite/install.rs` `WorkspaceToolchain`, `InstallGrant`, `install_scoped`, default `Refused`, `.formal-ai/toolchains/<program>/<content-id>/`. Test: the five refusal cases and the explicit-environment case.
- [x] **L8.** Add `data/meta/prerequisite-recipe.lino` + `recover()` + `tests/unit/specification/prerequisite_recipe.rs`. Test: grounding and rediscovery.
- [x] **L9.** Add `src/prerequisite/ledger.rs` `ToolchainLedger` and `data/meta/toolchain-ledger.lino`, append-only, with `formal-ai learn forget --toolchain <program>`. Test: forget-and-rediscover content id; restart reattach.
- [ ] **L10.** Run the family-1 held-out prompt in all five languages, refused and granted, and record the outcome table. No tuning in this commit.
- [ ] **L11.** Add `src/execution_box/mod.rs` `ExecutionBox`, `ExecutionBackend`, `BoxPolicy`, tar-on-stdin input, `--network none` default, honest deadline. Give `box_language_contract()` (`src/box_language_projects.rs:315`) its first non-test caller. Test: timeout reporting, network denial, missing-daemon refusal.
- [ ] **L12.** Add kotlin and scala to `data/meta/box-language-projects.lino` as `box_language_project_deferred` rows with the honest `reason` that `data/meta/box-image-survey.lino:13-16` records no such image, so the box backend reports unavailable for exactly the two languages B6 names.
- [ ] **L13.** Wire Telegram (#930): add `"code_execution"` to `data/seed/environments.lino:67-75`, route `src/telegram_runtime.rs` through `ExecutionBox`, implement the descending-N ladder with every N recorded and the ten-minute verbose hard fail. Test: family-3 prompts with and without a backend.
- [ ] **L14.** Add `src/execution_box/container.rs` `ConversationContainer`, `SnapshotPolicy`, idle stop, reattach, restart recovery (#937). Test: idle-and-restore; replay/snapshot divergence reported.
- [ ] **L15.** Make `src/web/worker/formal_ai_worker_14.js:519-545` probe instead of assert: state which runtime could be loaded and its size, keep the worker binary under `MAX_WASM_BYTES`, move the five-language strings into `src/web/i18n-catalog-messages.lino`. Test: the worker binary is byte-unchanged; the honesty sentence resolves in five languages.
- [ ] **L16.** Add the lazily-fetched Pyodide loader behind an explicit user click with the download size shown, and run the family-3 prompt in the browser with it loaded. Record whether the observed output is `55`.
- [ ] **L17.** Recover the SWE-bench harness prerequisite through `recover()` rather than through `.github/workflows/external-benchmarks.yml:113-117`, and record whether the run still reaches the evaluator.
- [ ] **L18.** Update `REQUIREMENTS.md` shard, traceability, `VISION.md`, `ROADMAP.md`, `GOALS.md`, `docs/benchmarks.md`, `docs/meta-algorithm.md` per the next section; tick the six open boxes of `docs/case-studies/issue-710/plans/07-prerequisite-discovery-bridge.md:65-81` that this plan actually closes, and leave the rest unticked with the reason.


**Leaf L6 note (wave I6).** `search_setup_procedure` takes the plan 00 §4.2
`SourceLookup` contract by generic parameter and is exercised against a fixture
lookup; plan 01's `RegistrySourceLookup` is the one implementation it receives
live. Nothing here constructs a second implementation.

**Leaf L7 note (wave I6).** `install_scoped` performs every refusal *before*
anything runs, prepares the workspace-scoped prefix and returns the explicit
environment; it does **not** execute the fetched commands. Executing a retrieved
procedure happens in `recover` through a declared `ExecutionBackend`, which is
also where `StillMissing` is produced, so the most dangerous operation in this
repository is never a side effect of preparing a directory.

## Docs to update — exact statements, quoted, with replacement

The exact quoted statements and their replacement text moved to plan 11's
findings table on 2026-09-16, so there is one docs authority and no document
is described in two places (plan 00 §8). This plan's entries are rows
**D198-D209** of
[`11-docs-consistency-audit.md`](11-docs-consistency-audit.md) §"Issue #1138
plan doc replacements", and plan 11's leaves apply them after the ledger rows
they cite exist (plan 00 §7).

| row | document |
| --- | --- |
| D198 | `VISION.md:163` |
| D199 | `VISION.md:269` |
| D200 | `ROADMAP.md:126` |
| D201 | `GOALS.md:96` |
| D202 | `GOALS.md`, Self-Evolution list |
| D203 | `docs/meta-algorithm.md:185-262` |
| D204 | `docs/benchmarks.md:356-358` |
| D205 | `docs/requirements/issue-0008-telegram-bot-requirements.md` |
| D206 | `docs/requirements/issue-0195-docker-in-docker-telegram-runtime.md` |
| D207 | New shard `docs/requirements/issue-1138-prerequisite-discovery.md` |
| D208 | `docs/requirements-traceability.md` |
| D209 | `docs/case-studies/issue-710/plans/07-prerequisite-discovery-bridge.md:65-81` |

Any further document this plan's implementation touches is added as a new
plan 11 row, never as a second copy here.

## Risks and open questions

1. **Executing retrieved text is the most dangerous thing in this repository.** The mitigations are structural (default-deny grant, workspace-root confinement checked before any spawn, digest verification, no `sudo`, no `PATH` mutation outside the returned map, mandatory postcondition), but a publisher page that has been compromised upstream is not defended against by any of them. Open: whether `SetupProcedure` should additionally require a maintainer-reviewed procedure the first time a program is seen, with the ledger row acting as the review record — which would make the first install of each program human-gated and every later one automatic.
2. **"Trusted publisher" is not a solved predicate.** Issue-710 plan 07 names this (`07:14-16`): *"A source must be authoritative for the particular dependency, not merely well-ranked."* `data/seed/setup-publishers.lino` pins it per program, which is a table — the thing RC4 criticises. Open: whether the publisher can be derived (from a package index's declared homepage, from the language's own registry) rather than pinned, and whether that derivation is itself rediscoverable.
3. **This plan depends on plans 01 and 04.** `discover_setup_procedure` calls the `SourceLookup` contract, whose one implementation plan 01 owns; today the nearest thing in the tree is `UnknownConceptLookup` with only `NoLookup` behind it (`src/coding/concept_discovery.rs:85-91`). Until plan 01 lands, family-1 and family-2 tests pass only against a fixture lookup, and the live integration test stays `#[ignore]`. That must be stated in the commit message and in the traceability row; it must not be presented as a working recursion.
4. **Disk.** `.formal-ai/toolchains/` can hold multiple JDKs and compilers. `scripts/check-disk-usage-policy.rs` and `scripts/check-cache-budget.rs` need a policy for it, and `scripts/free-runner-disk.sh:20-26` already shows CI fighting for space. Open: eviction rules — the ledger record is durable, but which installed prefixes may be reclaimed under pressure, and how "only positive net savings may be selected for pressure relief" (`06-repository-task-generalization.md:241-245`) applies here.
5. **Kotlin and Scala have no box image.** `data/meta/box-image-survey.lino:9-16` lists seven published and four missing repositories, and neither language is in either list. The container option cannot serve exactly the two languages B6 names, which is why the install path is load-bearing rather than optional. Open: whether to request `box-kotlin` / `box-scala` upstream, noting that doing so would make the fix a memorized table rather than a discovery.
6. **Pyodide size versus the worker ceiling.** `MAX_WASM_BYTES = 512 * 1024` (`scripts/check-wasm-worker-size.rs:30`) against a current 291,074 bytes; Pyodide's core is roughly twenty times the whole ceiling. The lazy-load design keeps the gate green, but it means the browser's honest state is "a runtime is available to download", not "a runtime is present" — and a user on a metered connection will never see observed output. Open: whether a much smaller Python (RustPython on wasm, MicroPython) buys enough of the semantics to be worth a second runtime.
7. **Snapshot versus replay divergence.** #937 asks for both. They will disagree whenever a step touched something the command log did not record. The plan reports divergence rather than choosing a winner; open: whether divergence should invalidate the replay path for that conversation, and whether `docker commit` is available in every deployment (`Dockerfile:59` is Docker-in-Docker with `DIND_STORAGE_DRIVER="vfs"`, where commits are slow and large).
8. **`#930`'s halving ladder versus "no budgets".** Halving an iteration bound on timeout *is* a budget unless every N and every outcome is published, which is what makes it a measurement. Open: whether the maintainer reads the descending ladder as a budget regardless; if so, the alternative is one deadline, one honest failure, and no retry — which loses #930's "report which N timed out and which N stopped timing out".
9. **Probing costs time on every answer.** Probing fourteen toolchains per coding answer is unacceptable; probing lazily and caching in the ledger risks a stale `Present` after the user uninstalls something. Open: the cache invalidation rule — probably re-probe on any `Missing`-shaped failure and trust the cache otherwise, which is self-correcting but means the first failure after an uninstall is reported as a failure before it is reported as a need.
10. **Held-out program leakage.** `zig` and `gleam` must stay absent from `src`, `data`, `scripts` and `.github` for the held-out tests to mean anything, and a well-meaning later commit adding either to the language catalog would silently void the evidence. `tests/unit/issue_1138_held_out_toolchain.rs::the_held_out_program_is_absent_from_the_repository` exists to fail loudly when that happens, and the two names must be recorded in `data/meta/` as reserved.
11. **The `0.15 %` figure and every other number cited around this work.** Issue #1138's table cites "self-hosting share 0.15 % (58 of 38,373)"; that number occurs exactly once in the repository, as prose at `docs/case-studies/issue-710/plans/04-final-requirements-release-and-self-improvement.md:460-461`, and it predates the failed strict measurement at `07-prerequisite-discovery-bridge.md:427-432`. Nothing in this plan may cite it as a ledger value; the ledger's newest row is `tag "v0.350.0"` at `data/meta/self-hosting-ledger.lino:1214-1230`.
