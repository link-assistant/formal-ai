# Plan 03 — Repository workspace protocol (bottleneck B3 of #1138)

Status: design recorded before any test or code edit. Baseline: `main` at
`09294f25d`, worktree `.claude/worktrees/issue-1138`, 2026-09-16.

**Contract ownership.** This plan owns contract 4.4 of
[plan 00](00-root-causes-and-integration.md) — `trait Workspace`, "one protocol
for any repository". Every type below is that contract's implementation; where
an earlier draft of this plan used a different name for a contract type, the
contract name wins (plan 00 §8). It consumes contract 4.1 (`need`), produces
contract 4.3 (`evidence`), and reuses the default-deny command policy in
`rust/src/agentic_coding/shell_command_policy.rs` rather than duplicating it.

**Order.** Plan 00 §5 places this plan **after**
[plan 06 — prerequisite discovery](06-prerequisite-discovery.md):
`01 → 04 → 02 → 05 → 06 → 03`. The two are mutually referential by design and
the split is clean: this plan owns the trait and every step except recovery;
plan 06 adds prerequisite recovery *inside* `Workspace::run` (plan 00 §4.4).
Landing 06 first means a clone whose toolchain is absent recovers instead of
stopping; landing this plan first means it stops honestly. Both are acceptable
states of the branch; neither plan may claim the other's result.

## Issues addressed — issue numbers with one line each on what they ask and which part this plan delivers

| Issue / PR | What it asks | What this plan delivers |
| --- | --- | --- |
| [#1138](https://github.com/link-assistant/formal-ai/issues/1138) B3 | "One workspace protocol shared by SWE-bench, the #848 ladder and self-coding: clone at base commit, locate the files the requirement names …, read, edit, run the named tests, produce the diff." | The whole plan: `rust/src/repository_workspace/` plus the three call sites. |
| [#848](https://github.com/link-assistant/formal-ai/issues/848) (closed by PR [#897](https://github.com/link-assistant/formal-ai/pull/897)) | A 130-task L1–L4 coding ladder over real open issues, verified "by observed effect … never by narration"; L1 must succeed or fail honestly. | The ladder's L1/L2 rungs get a repository workspace instead of the ambient checkout, so `issue_to_pr 0/16` becomes a measurable number rather than a structural zero (`experiments/issue_847_coding_ladder/results.json` summary: `{"total":130,"passed":65,…,"by_level":{"L1":{"passed":0,"total":16}…}}`). |
| [#1021](https://github.com/link-assistant/formal-ai/issues/1021) / PR [#1027](https://github.com/link-assistant/formal-ai/pull/1027) | R1021-22: "a pull request opened by Formal AI from a real `solve` run, green without a human editing the branch." Recorded **Not achieved** (`REQUIREMENTS.md:2405`; `docs/requirements-traceability.md:800`). | The attributed authoring path: `formal-ai solve --model formal-ai` as a first-class subcommand that drives the same protocol and emits the four self-hosting trailers. |
| [#1085](https://github.com/link-assistant/formal-ai/issues/1085) / PR [#1086](https://github.com/link-assistant/formal-ai/pull/1086) | R1085-9: ladder leaves must compile, composites apply both children's diffs to one tree, the deepest passing level ratchets. `data/meta/ladder-ratchet.lino` records `leaf_nodes_passing 15`, `deepest_passing_level none`. | The 32 leaves stop needing a pre-authored `experiments/issue_1028_agent_cli_ladder/rules/L*.lino` per leaf; the protocol locates and edits from the requirement, so leaf passing is a capability number rather than a rule-authoring number. |
| [#655](https://github.com/link-assistant/formal-ai/issues/655) / PR [#679](https://github.com/link-assistant/formal-ai/pull/679) | An end-to-end `solve <issue-url> --tool agent --model formal-ai` loop from plan to draft PR, replayable in CI. PR #679 delivered the *inner* Agent CLI ↔ Formal AI replay (183 stream-JSON events); the outer live entry stayed gated on hive-mind#2059. | The outer entry, owned in this repository rather than upstream: `solve` reads an issue URL, clones, works, and emits an attributed commit. |
| [#710](https://github.com/link-assistant/formal-ai/issues/710) / PR [#888](https://github.com/link-assistant/formal-ai/pull/888) | Plan 06 P6: "use the rebuilt Formal AI via external Agent CLI for … a regression-test/source modification, and complete project tasks from fresh state" — still `- [ ]` (`docs/case-studies/issue-710/plans/06-repository-task-generalization.md:96-102`). | The "fresh state" half: the workspace is a clone, not the ambient repository, so a repository task has a defined starting tree. |
| [#924](https://github.com/link-assistant/formal-ai/issues/924) / PR [#1007](https://github.com/link-assistant/formal-ai/pull/1007) | One reviewable repository change authored by Formal AI each release cycle, recorded in the ledger. | The mechanism that makes a release-cycle change routine instead of a bespoke `scripts/author-change-with-formal-ai.sh` invocation per task. |
| [#1091](https://github.com/link-assistant/formal-ai/issues/1091) | The first self-authored task ("count `Gemfile.lock` as a lockfile"), landed in `d060b00` via PR #1103. | The generalization: the same shape of task lands without a hand-written `--produces`/`--into`/`--contains` contract. |
| [#699](https://github.com/link-assistant/formal-ai/issues/699) → [#959](https://github.com/link-assistant/formal-ai/issues/959) | No new specialized handler Rust; migrate to seed rows and registry methods. | The protocol's step list is `data/meta/repository-workspace-protocol.lino`; the Rust is one executor, and no `try_*` dispatch arm is added. |
| [#8](https://github.com/link-assistant/formal-ai/issues/8), [#930](https://github.com/link-assistant/formal-ai/issues/930), [#937](https://github.com/link-assistant/formal-ai/issues/937) | Execution before answering; per-conversation containers. | Owned by Plan 06; this plan only declares the boundary at which the protocol asks for an execution environment. |
| [#838](https://github.com/link-assistant/formal-ai/issues/838) | "Find hive-mind on my desktop" — a local filesystem search must reach the filesystem. | Added by the 2026-09-16 reconciliation to match plan 13's coverage table: plan 10 owns the routing, this plan owns the workspace the located path is read and edited in. |
| [#1066](https://github.com/link-assistant/formal-ai/issues/1066) (closed by #1067 with five of six acceptance items undelivered) | run the self-authoring harness; land a qualifying attributed pull request; cut a release on it. | Added by the reconciliation: carry-over C83 assigns three of the five undelivered items to this plan (L13, L17) and the depth-5 decomposition ladder to plan 12. |

## Current state — evidence with file:line, tests, ledgers

### (a) One SWE-bench Lite instance

1. `rust/src/external_benchmarks/manifest.rs:298-311` declares the suite: `id: "swebench_lite"`, `task_family: "agentic_repository_patch"`, `source: SuiteSource::ParquetRows { url: ".../dev-00000-of-00001.parquet", … }`, `grading: Grading::SweBenchTests`, `availability: Availability::Runnable`.
2. `rust/src/external_benchmarks/mod.rs:124-137` fetches and parses the records into `BenchmarkCase`s.
3. `rust/src/external_benchmarks/cases.rs:152-166` builds the case. The **entire** repository grounding is a sentence:

   ```rust
   prompt: format!(
       "Repository {repository}. Resolve this issue and reply with the fix as a unified diff patch.\n\n{statement}"
   ),
   ```

   There is no `git clone`, no `base_commit` read (the field exists in the upstream record and is passed through untouched inside `Expectation::SweBench { record: value.to_string() }` at `cases.rs:161-163`), and no file list.
4. `rust/src/external_benchmarks/mod.rs:151-156` runs the solver: `let workspace = cache_root.join("run").join(manifest.id);` then `.map(|case| solver.solve(&case.prompt))`. `workspace` is only ever handed to the grader; the solver never receives it.
5. `rust/src/external_benchmarks/grade.rs:172-192`: `grade_swebench` extracts a diff from each answer, and

   ```rust
   if patches.iter().all(Option::is_none) {
       return Ok(cases.iter().map(|case| failure(case, "empty patch rejected by the official SWE-bench criterion")).collect());
   }
   ```

   So the recorded `0 / 1` (`docs/benchmarks.md:294`) is produced **before Docker is contacted**: `ensure_swebench_runtime()` (`grade.rs:193`, defined at `grade.rs:333-362`) is never reached. The official harness at `grade.rs:277-290` and the report reader at `grade.rs:321` never run for this suite today.
6. `.github/workflows/external-benchmarks.yml:23-24,126-127,146-148` bounds the scheduled slice at `SWE_BENCH_SLICE: ${{ inputs.swebench_slice || '1' }}` — one instance.

**Net:** `0/1` is structural. A solver that cannot see the repository cannot emit a patch against it; the honest empty-patch path then closes the case. The `0 / 1` row is not a near miss, and growing the slice to 23 (the Lite dev split) would produce `0/23` for the same reason.

### (b) A request to add a regression test to this repository

1. Routing: `rust/src/agentic_coding/general_planner.rs:153` `compose_general_change_plan(full_request)` builds a `GeneralChangePlan` whose steps are persisted to `PLAN_PATH = ".formal-ai/general-change-plan.lino"` (`general_planner.rs:21`).
2. Target resolution: `rust/src/agentic_coding/requirement_resolution.rs:24-26`

   ```rust
   pub fn resolve_requirement_target(requirement: &str) -> Option<RequirementTarget> {
       resolve_in(workspace(), requirement)
   }
   ```

   and `rust/src/self_ast_census.rs:478-481`

   ```rust
   pub fn workspace() -> &'static WorkspaceCensus {
       static WORKSPACE: OnceLock<WorkspaceCensus> = OnceLock::new();
       WORKSPACE.get_or_init(|| WorkspaceCensus::compile(owned_source_files()))
   }
   ```

   `owned_source_files()` is the **compile-time** manifest generated by `build.rs`. The census therefore describes the sources the running binary was built from, not the tree on disk, and not any other repository.
3. Editing: `rust/src/agentic_coding/structured_edit.rs:105` `member_insertion(task)` and `rust/src/agentic_coding/workspace_change.rs:318` `grounded_rewrite(task)` transform bytes for *literal-into-list* and *identifier rename* shapes; `rust/src/agentic_coding/link_edit_rules.rs:32-53` names the three link-edit rules.
4. Execution: `rust/src/agentic_coding/driver.rs:39` advertises exactly four tools, `DRIVER_TOOLS = ["web_search", "web_fetch", "write_file", "run_command"]`, bounded by `MAX_TURNS = 12` (`driver.rs:43`). `run_command` lands in `rust/src/agent.rs:242` → `run_command_inner` (`agent.rs:338`) → `resolve_allowed_program` (`agent.rs:642-657`), whose whole allowlist is

   ```rust
   "cat" | "ls" | "printf" | "env" | "python3" | "rustc"  // everything else: AgentError::UnsupportedCommand
   ```

   **`git` is not on it. Neither is `cargo`.** The driver cannot clone, cannot diff, cannot run `cargo test`.
5. Measured outcome: `experiments/issue_847_coding_ladder/results.json:1-28` records `{"measurement":{"dataset_total":130,"measured_total":130,"complete":true,…},"summary":{"total":130,"passed":65,"failed":65,"not_measured":0,"by_level":{"L1":{"passed":0,"total":16},"L2":{"passed":5,"total":12},"L3":{"passed":10,"total":28},"L4":{"passed":50,"total":74}}}}` — with `test_authoring` at 1/8 and `issue_to_pr` at 0/16 in the PR #897 family table. `experiments/issue_847_coding_ladder/README.txt:44-47` records the harness reverting the *real repository* with `git checkout -- .` between tasks, i.e. the tasks edit the ambient checkout rather than a workspace. The pass predicate (`run_coding_ladder.sh:284-288`) is `ok = verified and answered and not timed_out and not refused and not not_measured and not expectation_error`, with the contract stated at `run_coding_ladder.sh:25-28`: *"Verification is by observed effect, never by the agent's narration — 'I created the file' with no file is a FAIL."* The harness `exit 0`s unconditionally (`run_coding_ladder.sh:30`) and **no workflow runs it**: `docs/case-studies/issue-957/raw-data/verified-776-928.md:57` records that "the coding ladder … is not referenced by any workflow; only #842's 24-node task ladder (24/24) runs in CI (task-ladder.yml)."
6. `docs/case-studies/issue-710/plans/06-repository-task-generalization.md:96-102` leaves P6 unchecked; `docs/case-studies/issue-710/plans/07-prerequisite-discovery-bridge.md:175-183` records the open-ended refactor probe: *"Formal AI read the file and answered with its contents in two model rounds. The resulting file is byte-identical to the seed: **no edit was authored**."*

**Net:** a regression-test request either resolves to a known literal-insertion shape and succeeds, or reads a file and stops. There is no clone, no `cargo test <name>`, no diff.

### (c) A Kotlin task on a machine without `kotlinc`

`rust/src/coding/catalog/languages.rs:186-201` declares `kotlin` with `status: ExecutionStatus::Unavailable`, `check_command: Some("kotlinc Main.kt -include-runtime -d Main.jar")`, and a **hard-coded** `setup_hint: "the Kotlin compiler from https://kotlinlang.org/docs/command-line.html (a JDK is required as well)"`. No code executes `check_command`; `rust/src/engine.rs:959-962` only renders it as prose, and `rust/src/coding/guidance.rs:291-296` unconditionally prepends `format!("Install {setup_hint}.")`. Live evidence: `docs/case-studies/issue-710/plans/06-repository-task-generalization.md` records Kotlin session `ses_f5a282912ffeOLFh21bbx9Dqsl` *"ends honestly at `kotlinc: command not found` after five rounds"*, and the Scala CI job log *"confirms `scalac: command not found`, exit 127"*. Under this plan a Kotlin repository task reaches the same wall one step later: the clone succeeds, the edit succeeds, and the named test cannot run. **This plan stops there and hands the failure to Plan 06 as a need.** It does not install anything.

### (d) A Telegram "run this code" request

`rust/src/main.rs:663-682` routes `Command::Telegram` into `run_telegram`. `data/seed/environments.lino:67-75` declares the telegram environment's tools as `("intent_routing" "write_program" "concept_lookup" "fact_lookup" "summarize_conversation" "brainstorm" "coreference" "roleplay" "html_replies")` — no execution tool at all. The docker-in-docker image exists (`Dockerfile:59` `FROM konard/box-dind:2.1.1`, `Dockerfile:66-67` `FORMAL_AI_START_ISOLATION=docker`, `FORMAL_AI_START_RUNNER="$ --isolated docker --auto-remove-docker-container --"`), and `scripts/verify-docker-runtime.sh:25-32` asserts those variables, but **no file under `rust/src/` reads either variable**. The answer is composed from the static catalog and labelled "Expected output after verification" (`rust/src/engine.rs:939-951`). This is Plan 06's trace; here it matters only because a repository task arriving over Telegram gets the same protocol and the same honest refusal surface.

### Ledgers and gates as they stand

- `data/meta/ladder-ratchet.lino`: `leaf_nodes_selected 32`, `leaf_nodes_passing 15`, `deepest_passing_level none`, checked by `.github/workflows/issue-1028-agent-ladder.yml`.
- `data/meta/self-hosting-ledger.lino:3-6`: the four trailers `Formal-AI-Session` / `Formal-AI-Model` / `Formal-AI-Evidence` / `Formal-AI-Pull-Request`; `current_metric_version "3"`.
- `scripts/author-change-with-formal-ai.sh:159` is the only attributed authoring path today: `"$AGENT" --model formalai/formal-ai --permission-mode auto …`, with the trailers written at `scripts/author-change-with-formal-ai.sh:243`. It requires the caller to supply `--task`, `--produces`, `--into`, `--contains`, `--evidence` and `--pull-request` by hand.
- `docs/benchmarks.md:294` SWE-bench Lite `0 | 1`; `docs/benchmarks.md:311` "`0 / 1` on SWE-bench Lite. The ratchet makes every number a floor that may never fall below."

## Root causes — numbered

**RC1. The benchmark case constructor throws the repository away.**
`rust/src/external_benchmarks/cases.rs:152-166` reduces an instance to one prompt string. The `repo`, `base_commit`, `environment_setup_commit`, `FAIL_TO_PASS` and `PASS_TO_PASS` fields present in the upstream record survive only inside the opaque `record: String` that the grader re-parses (`grade.rs:200`). *Mechanism:* the solver's input type is `&str`, so nothing downstream can be repository-aware even if it wanted to be. Every future repository suite would have to repeat the same loss.

**RC2. The solver's only entry point takes a prompt, not a task with a workspace.**
`rust/src/external_benchmarks/mod.rs:155` is `solver.solve(&case.prompt)`. *Mechanism:* there is no seam at which a working tree could be attached, so repository grounding cannot be added without changing the call, which is why the `workspace` variable at `mod.rs:151` exists purely for grading.

**RC3. File location is compile-time and self-only.**
`rust/src/self_ast_census.rs:478-481` memoizes `WorkspaceCensus::compile(owned_source_files())` in a `OnceLock`; `requirement_resolution.rs:24-26` is hard-wired to it. *Mechanism:* the one general "find the file a requirement names" capability the repository owns is physically incapable of describing a clone. Note the seam already exists — `WorkspaceCensus::compile(files: &[(&str, &str)])` (`self_ast_census.rs:293`) and `resolve_in(census, requirement)` (`requirement_resolution.rs:36`) are both parameterized — so this is a missing constructor, not a missing design.

**RC4. The sandbox allowlist excludes every tool a repository task needs.**
`rust/src/agent.rs:642-657`: six programs, default-deny via the `other =>` arm. *Mechanism:* `git clone`, `git diff`, `cargo test` and `pytest` are `AgentError::UnsupportedCommand`. The doctrine (default-deny with explicit allowlists) is right; the allowlist is simply empty of repository verbs, and it is a `match` in Rust rather than seed data, so widening it is a source edit rather than a data edit.

**RC5. Three consumers each invented their own working tree.**
SWE-bench uses `target/formal-ai-benchmarks/run/<suite>` (`manifest.rs:143` `CACHE_DIR`, `mod.rs:151`). The #848 ladder edits the **real checkout** and reverts with `git checkout -- .` (`experiments/issue_847_coding_ladder/README.txt:44-47`). The #1085 ladder applies a pre-authored `.lino` rule per leaf (`experiments/issue_1028_agent_cli_ladder/rules/L01.lino` … `L32.lino`) against the checkout. The agentic driver uses a fresh temp dir (`rust/src/agent.rs:60`, `AgentWorkspace::for_prompt`, `agent.rs:186-209`). *Mechanism:* four incompatible notions of "where the work happens" means no capability proven in one transfers to the others — exactly the "capability partial by its own number" note in `docs/case-studies/issue-710/plans/01-requirements-audit-coding-and-benchmarks.md:44`.

**RC6. The #1085 ladder measures rule authorship, not capability.**
`experiments/issue_1028_agent_cli_ladder/leaves.tsv` has 32 rows; each names its file, its literal and its symbol (`L01 … src/web_search_core.rs "wikiquote" WEB_SEARCH_PROVIDERS`), and each has a committed `rules/L<nn>.lino`. `data/meta/ladder-ratchet.lino` ratchets `leaf_nodes_passing`. *Mechanism:* the rule is the memoized answer. A leaf that passes proves the rule interpreter works, not that the requirement was understood; and no leaf outside the 32 can be attempted. This is the memoization the doctrine forbids, wearing a ratchet.

**RC7. Attributed authoring is a shell script with a hand-written contract per task.**
`scripts/author-change-with-formal-ai.sh:22-33` documents mandatory `--task`, `--produces`, `--into`, `--evidence`, `--pull-request`, `--message`. *Mechanism:* the human supplies the file the model must write and the text it must contain; the model supplies the bytes. That is why #1091 needed a bespoke contract and why `REQUIREMENTS.md:2405` still reads **Not achieved** for R1021-22. There is no `solve` subcommand at all: `rust/src/main.rs:596-698` enumerates `Chat`, `Agent`, `Serve`, `Telegram`, `Benchmark`, … and no `Solve`.

**RC8. Nothing turns a produced tree into a reviewable unified diff.**
`grade.rs:365+` reads the official harness report and `extract_diff(answer)` (`grade.rs:175`) pulls a diff out of *prose*. The system can recognize a diff it was handed; it cannot compute one from two trees. *Mechanism:* the SWE-bench pass criterion is "apply this patch", so without diff production the whole suite is unreachable regardless of edit quality.

## Solution options — at least 3 distinct options for the whole plan

### Option A — One `RepositoryWorkspace` in Rust, protocol steps in seed data, three call sites converted

*Description.* Add `rust/src/repository_workspace/` owning a checked-out tree, a base commit, a locator, an editor bridge, a named-test runner and a diff producer. The **order** of the steps is `data/meta/repository-workspace-protocol.lino`, walked by the existing recipe/rule machinery, not by a Rust state machine. Convert SWE-bench, the #848 ladder and the self-coding authoring path to it.

*Architecture sketch.*

```
data/meta/repository-workspace-protocol.lino   (the ordered steps + their checks)
data/seed/repository-task-verbs.lino           (clone/locate/read/edit/test/diff vocabulary, 5 languages)
          │
src/repository_workspace/mod.rs   RepositoryWorkspace, WorkspaceProtocol, ProtocolOutcome
          ├── clone.rs            WorkspaceSpec, clone_at_base()
          ├── locate.rs           locate_targets (census for Rust, ripgrep-free literal scan otherwise)
          ├── edit.rs             bridges to structured_edit / link_edit_rules / workspace_change
          ├── verify.rs           Command (cargo test <name> | python -m pytest <node-id>)
          └── diff.rs             unified_diff(base, head) -> String
          │
consumers: external_benchmarks::run_suite_with_online  (swebench branch)
           experiments/issue_847_coding_ladder/run_coding_ladder.sh (via `formal-ai solve`)
           src/cli_solve.rs  `formal-ai solve --model formal-ai`
```

*Pros.* One protocol, three consumers, a single place to measure. Reuses `WorkspaceCensus::compile` and `resolve_in` unchanged (RC3 is a constructor away). The step order is data, so #699/#959 sees a new data file, not a new handler. `RepositoryWorkspace` composes over the existing `AgentWorkspace` default-deny policy instead of replacing it.

*Cons.* Touches four subsystems at once. `git` must join the allowlist, which is a real widening of the sandbox surface. Diff production in pure Rust (no `similar` / `diff` crate; `Cargo.toml` deliberately keeps a thin dependency tree — `.github/workflows/stock-rust-install.yml:36-43` even rejects `openssl-sys`) is new code.

*Doctrine fit.* Strong. Seed data + registry method, no `try_*` arm, default-deny preserved, nothing hard-coded per task or per benchmark.

*Effort.* ~9 commit-sized leaves, ~1,400 net source lines plus tests.

*Risk.* Medium. The main risk is `git` in the allowlist; mitigated by a subcommand allowlist (`clone`, `-C <root> diff`, `-C <root> checkout`, `-C <root> rev-parse`) declared in seed, not by trusting `git` wholesale.

### Option B — Delegate the whole workspace to the official SWE-bench container and to `git worktree` for self-coding

*Description.* Do not own a tree. For SWE-bench, run the model *inside* the harness container the evaluator already builds; for self-coding, use `git worktree add` on the ambient repository; for the #848 ladder, keep `git checkout -- .`.

*Architecture sketch.* `grade.rs` gains a pre-pass that starts the instance image and pipes prompts/commands into it over `docker exec`; `cli_solve.rs` shells out to `git worktree`.

*Pros.* No diff engine needed (the container has `git diff`). Environment prerequisites for SWE-bench instances come free from the upstream images, which is exactly what B6 is otherwise expensive to solve. Matches `docs/benchmarks.md:314-320` ("the pinned official harness … applies a candidate patch in the upstream container").

*Cons.* Docker becomes a hard requirement for any repository capability, so a laptop or the macOS CI lane (`.github/workflows/macos-core-tests.yml`) loses it entirely; `ensure_swebench_runtime` already returns `Err` for a missing daemon (`grade.rs:346-358`) and that is recorded as `benchmark_unavailable`, which would now swallow the *capability*, not just the score. Two different mechanisms for foreign and own repositories means B3's "one protocol" is not delivered. `git worktree` on the ambient repository shares the stash and index with the operator's checkout.

*Doctrine fit.* Weak on "a hard task is split until each leaf is directly solvable" — the leaf becomes "have Docker". Honest, but the honesty is an unavailability row rather than a capability.

*Effort.* ~5 leaves. *Risk.* High (platform coupling; no macOS path).

### Option C — Retrieval-only: keep prompts, add a repository-context retriever

*Description.* Leave `solve(&str)` alone. Before solving, fetch the repository at the base commit into the content-addressed source cache (the same machinery `rust/src/solver_handler_how_synthesis.rs:34` `FORMAL_AI_SOURCE_CACHE_DIR` already uses) and inject the top-N relevant file excerpts into the prompt. Grade the emitted diff as today.

*Pros.* Smallest change; no allowlist widening; no new execution surface; works identically in the browser. Directly improves the "locate" half of B3 with no new failure modes.

*Cons.* Cannot run the named tests, so RC8 and the #848 `verification`/`test_authoring` families are untouched; the model writes a patch it never applied. It reproduces exactly the failure mode the maintainer named in #848 — "substring assertions on agent output are not verification". It also grows the prompt without bound, which collides with `MAX_TURNS` and the context budget.

*Doctrine fit.* Fails "deterministic and honest": a diff produced without applying it is narration.

*Effort.* ~3 leaves. *Risk.* Low, but it buys a number that does not mean what it says.

### Option D — Push the protocol upstream into Hive Mind / Agent CLI

*Description.* Treat `hive-mind solve --model formal-ai` as the owner of cloning, editing and PR opening; Formal AI only answers.

*Pros.* Zero new surface here; matches the existing `formal-ai with agent` topology.

*Cons.* hive-mind#2229 is closed and PR #1086 still records that path as "not an authoring path" because commits carry no trailers or evidence bundle; hive-mind#2059's own thread recommends **gating** real dispatch until writes succeed. Depending on an external repository for our own measured capability is precisely what made R1021-22 stall for a year of releases. Also violates "trusted sources primarily … everything discoverable must be forgettable and rediscoverable": the protocol would be unrecoverable from this repository's data.

*Effort.* ~2 leaves here, unbounded upstream. *Risk.* High and not ours to manage.

## Decision — selected option and reasons

**Option A is selected.**

Reasons:

1. It is the only option that literally delivers B3's sentence: one protocol shared by all three consumers. B, C and D each leave at least one consumer on a different mechanism.
2. The two hardest pieces already have seams: `WorkspaceCensus::compile(files)` (`self_ast_census.rs:293`) takes an explicit target set, and `resolve_in(census, requirement)` (`requirement_resolution.rs:36`) takes a census. Option A adds `WorkspaceCensus::of_directory` and changes nothing else in resolution. That is generalization of an existing boundary, which the doctrine prefers over a new mechanism.
3. It keeps default-deny: the sandbox stays an allowlist, and the new entries are *subcommand-scoped* rows in seed data (`git clone <url> <dir>`, `git -C <root> diff`, …), reviewable as data, exactly as `data/seed/shell-intents.lino` already declares pre/post conditions for `mv`/`cp` (`rust/src/agentic_coding/mutating_action.rs:29-33`).
4. It makes both ladders honest at once: the #848 ladder gets a defined tree instead of the operator's checkout (RC5), and the #1085 ladder can delete its 32 pre-authored rules and re-measure (RC6). A ratchet over *authored rules* is a memoization ratchet; a ratchet over *located-and-edited leaves* is a capability ratchet.
5. Docker stays optional. Option B's coupling would make the capability disappear on macOS, and `ensure_swebench_runtime` (`grade.rs:333-362`) already proves how easily a missing daemon converts a capability question into an availability row.

**Rejections.**

- **Option B** rejected: it splits the protocol in two (container for foreign repos, worktree for our own), makes Docker a hard dependency for all repository work, and turns capability failures into `benchmark_unavailable`. Its one genuine advantage — free instance environments — is recovered as an *optional* verification backend in Option A's architecture (see `ExecutionBackend::SweBenchImage` below), without making it load-bearing.
- **Option C** rejected: it produces an unapplied patch, which is narration, and #848's own record already names substring assertions on agent output as non-verification.
- **Option D** rejected: it makes our measured capability depend on a repository we do not control, and the two hive-mind issues (#2059, #2229) show that dependency costing a year with nothing attributable at the end.

## Architecture — exact

### Contract alignment (plan 00 §4.4)

Plan 00 fixes the shape:

```rust
pub trait Workspace {
    fn open(spec: &WorkspaceSpec) -> Result<Self, WorkspaceError>;   // clone at base commit, or the own repo
    fn locate(&self, need: &Need) -> Vec<Location>;                   // self-AST census for own repo, search otherwise
    fn read(&self, loc: &Location) -> Result<String, WorkspaceError>;
    fn edit(&mut self, change: &Change) -> Result<Evidence, WorkspaceError>;
    fn run(&mut self, cmd: &Command) -> Evidence;                     // named tests, build, program
    fn diff(&self) -> Result<UnifiedDiff, WorkspaceError>;
}
```

This plan implements that trait once, as `RepositoryWorkspace`. The mapping from
the contract names to this plan's module layout is fixed here so plan 14's
reconciliation leaf has nothing to rename:

| Contract (plan 00 §4.4) | Implementation | Module |
| --- | --- | --- |
| `WorkspaceSpec` | `WorkspaceSpec { origin, base_commit, sparse_paths }` | `rust/src/repository_workspace/clone.rs` |
| `Workspace::open` | `RepositoryWorkspace::open` / `::adopt` | `rust/src/repository_workspace/mod.rs` |
| `Location` | `Location { relative_path, symbol, how: LocationEvidence }` | `rust/src/repository_workspace/locate.rs` |
| `Workspace::locate` | `locate_targets(workspace, need)` | `rust/src/repository_workspace/locate.rs` |
| `Change` | reuses `structured_edit` / `link_edit_rules` / `workspace_change` shapes | `rust/src/repository_workspace/edit.rs` |
| `RunCommand` | `RunCommand { line, names }` — a named-test invocation or a build (plan 00 §9 R6) | `rust/src/repository_workspace/verify.rs` |
| `Evidence` | plan 05's single record in `rust/src/execution_evidence.rs`, emitted by `edit` and `run`; this module adds no fields (plan 00 §9 R2) | `rust/src/execution_evidence.rs` |
| `UnifiedDiff` | `UnifiedDiff(String)`, computed from the tree | `rust/src/repository_workspace/diff.rs` |
| `Need` | plan 00 §4.1 link record; consumed, never redefined here | `rust/src/meta_frame.rs` + plan 05 |

`Workspace::locate` takes a `Need`, not a `&str`: the requirement arrives as the
`need.subject` span in `need.language`, which is what makes the five held-out
languages below one code path rather than five.

### New module tree

```
src/repository_workspace/mod.rs          (RepositoryWorkspace, WorkspaceProtocol, ProtocolOutcome)
src/repository_workspace/clone.rs        (WorkspaceSpec, clone_at_base)
src/repository_workspace/locate.rs       (locate_targets, Location, LocationEvidence)
src/repository_workspace/edit.rs         (apply_change)
src/repository_workspace/verify.rs       (RunCommand, run_named_tests -> Evidence; ExecutionBackend from plan 06)
src/repository_workspace/diff.rs         (UnifiedDiff, unified_diff)
src/repository_workspace/outcome.rs      (plan 06 L13/L15: the five-language surface honesty sentences
                                          a surface may say about a command it ran)
src/repository_workspace/world_model.rs  (plan 15: evidence-backed current-to-goal deltas for
                                          repository work items)
src/repository_workspace/protocol-header.txt (plan 03 L8: the header the regenerated protocol
                                          document carries, so rediscovery hits the same content id)
src/cli_solve.rs                         (`formal-ai solve`)
data/meta/repository-workspace-protocol.lino
data/seed/repository-task-verbs.lino
data/seed/repository-command-allowlist.lino
```

Every one of `RepositoryWorkspace`, `WorkspaceProtocol`, `WorkspaceSpec`, `UnifiedDiff`, `repository_workspace`, `of_directory`, `locate_targets`, `base_commit` returns **zero** hits from `grep -rn` over `src`, `tests`, `scripts`, `data` on the baseline, so no name collides. `NeedLedger` (`rust/src/meta_frame.rs:644`), `RecipeProgress` (`rust/src/agentic_coding/command_reroute.rs:163`), `ExecutionStatus` (`rust/src/coding/catalog/types.rs:203`), `ProbeOutcome` (`rust/src/reasoning_standard/refutation.rs:59`) and `AgentWorkspace` (`rust/src/agent.rs:177`) are taken and are *reused*, not shadowed. `Location`, `Change`, `RunCommand` and `Evidence` are contract names owned by plan 00; this plan defines them in `rust/src/repository_workspace/` only if plan 05 has not already placed them, and adopts plan 05's definitions otherwise.

### Types and signatures

```rust
// src/repository_workspace/clone.rs

/// Where a repository task's tree comes from, and at which commit.
/// Contract name: `WorkspaceSpec` (plan 00 §4.4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceSpec {
    /// `owner/name` for a hosted repository, or an absolute path for a local one.
    pub origin: String,
    /// The exact commit the task is defined against. Never a branch name.
    pub base_commit: String,
    /// Paths to fetch. Empty means the whole tree.
    pub sparse_paths: Vec<String>,
}

/// Materialise `spec` under `root`. Deterministic: same spec, same bytes.
///
/// # Errors
/// Returns the observed command, exit code and stderr when `git` is refused,
/// missing, or exits non-zero. Never falls back to a different commit.
pub fn clone_at_base(spec: &WorkspaceSpec, root: &Path) -> Result<PathBuf, WorkspaceError>;
```

```rust
// src/repository_workspace/mod.rs

/// A checked-out tree a repository task may read, edit, test and diff.
pub struct RepositoryWorkspace {
    root: PathBuf,
    spec: WorkspaceSpec,
    /// Byte snapshot of every file the protocol has read or written, so a diff
    /// is computed from observed bytes rather than from a second `git` call.
    baseline: BTreeMap<String, Vec<u8>>,
    sandbox: AgentWorkspace,            // reuses src/agent.rs default-deny
}

impl RepositoryWorkspace {
    /// Clone `spec` into a fresh directory under `base_dir`.
    pub fn open(spec: &WorkspaceSpec, base_dir: &Path) -> Result<Self, WorkspaceError>;

    /// Adopt an existing directory (the ambient checkout, a `git worktree`)
    /// without cloning. `base_commit` is read with `git -C <root> rev-parse HEAD`.
    pub fn adopt(root: &Path) -> Result<Self, WorkspaceError>;

    #[must_use] pub fn root(&self) -> &Path;
    #[must_use] pub fn base_commit(&self) -> &str;

    /// Every source file in the tree as `(relative_path, contents)`, in path
    /// order — the argument shape `WorkspaceCensus::compile` already takes.
    pub fn source_files(&self) -> Result<Vec<(String, String)>, WorkspaceError>;

    pub fn read(&self, relative: &str) -> Result<String, WorkspaceError>;
    pub fn write(&mut self, relative: &str, contents: &str) -> Result<(), WorkspaceError>;

    /// The unified diff between the base commit and the current tree.
    pub fn diff(&self) -> Result<String, WorkspaceError>;
}
```

```rust
// src/self_ast_census.rs  — one added constructor, nothing else changes

impl WorkspaceCensus {
    /// Census every Rust module under `root`, in path order.
    ///
    /// The existing `compile(files)` already takes an explicit target set
    /// (`self_ast_census.rs:293`); this reads that set off disk so a clone can
    /// be addressed the same way the owned manifest is.
    pub fn of_directory(root: &Path) -> std::io::Result<Self>;
}
```

```rust
// src/repository_workspace/locate.rs

/// One file (and optionally one declaration) a requirement names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Location {
    pub relative_path: String,
    pub symbol: Option<String>,
    /// Which mechanism found it, for the honesty trace.
    pub how: LocationEvidence,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocationEvidence {
    /// The self-AST census resolved a declaration (Rust trees).
    Census,
    /// A literal named in the requirement occurs in exactly one file.
    LiteralOccurrence,
    /// A path named verbatim in the requirement exists in the tree.
    NamedPath,
}

/// Resolve the files `need` names inside `workspace`.
///
/// `need` is the plan 00 §4.1 record: its `subject` is the requirement text as
/// written, its `language` is one of en/ru/hi/zh/es, so the five held-out
/// prompts below are one code path rather than five. Rust trees go through `WorkspaceCensus::of_directory` +
/// `agentic_coding::requirement_resolution::resolve_in` unchanged. Every other
/// language goes through a deterministic literal/path scan. Ambiguity resolves
/// to `Vec::new()` — a guess is never returned.
pub fn locate_targets(
    workspace: &RepositoryWorkspace,
    need: &Need,
) -> Result<Vec<Location>, WorkspaceError>;
```

```rust
// src/repository_workspace/verify.rs

/// One named test, build or program invocation. Contract name: `RunCommand`
/// (plan 00 §4.4).
///
/// **reconciled: was `Command`; now `RunCommand`, because `Command` is already
/// taken twice — by `std::process::Command` and by clap's `Command` enum in
/// `rust/src/main.rs` — and this plan's own draft used `line` in the struct and
/// `command` at the SWE-bench call site (plan 00 §9 R6).**
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunCommand {
    /// The command line, lowered from seed data for the tree's ecosystem.
    pub line: String,
    /// The test names that must pass (SWE-bench `FAIL_TO_PASS` ∪ `PASS_TO_PASS`).
    pub names: Vec<String>,
}

// Where the named tests actually execute is `crate::execution_box::ExecutionBackend`,
// declared by plan 06 (`rust/src/execution_box/mod.rs`) and consumed here.
//
// **reconciled: was `VerifyBackend { Sandbox, SweBenchImage, BoxImage }` declared
// in this module; now plan 06's `ExecutionBackend { HostSandbox, Box { image },
// SweBenchImage { instance_id }, Conversation { conversation_id },
// BrowserRuntime { runtime } }`, because two enums for "where does this run" is
// the same defect in two plans, and plan 06 lands first in plan 00 §5's order
// with the superset of variants (plan 00 §9 R7).**

/// Run `tests` in `workspace` on `backend` and report what was observed.
///
/// A missing interpreter, compiler or container is returned as
/// `WorkspaceError::MissingPrerequisite { program, exit_code, stderr }` — the
/// exact shape Plan 06 turns into a requirement. It is never a pass and never a
/// silent skip.
pub fn run_named_tests(
    workspace: &RepositoryWorkspace,
    tests: &RunCommand,
    backend: &crate::execution_box::ExecutionBackend,
) -> Result<Evidence, WorkspaceError>;

// `Evidence` is plan 05's single definition in `rust/src/execution_evidence.rs`
// (plan 00 §4.3). This module declares no record of its own.
//
// **reconciled: this plan's draft declared a second `Evidence` struct carrying
// `stdout`, `stderr`, `passed`, `failed` and `timed_out`. Now: the deterministic
// half lives in `Evidence::detail` as
// `EvidenceDetail::Tests { passed, failed, timed_out }`, which is hashable and
// enters the ledger; the raw `stdout` / `stderr` are hashed into `output_hash`
// and returned beside the record as the non-persisted
// `ObservedOutput { stdout, stderr }`, so a caller can show them without the
// ledger storing them (plan 00 §4.3, §9 R2).**
```

```rust
// src/repository_workspace/mod.rs — the protocol itself

/// The ordered repository protocol, read from
/// `data/meta/repository-workspace-protocol.lino`.
pub struct WorkspaceProtocol {
    steps: Vec<ProtocolStep>,
}

/// One step: what it does, what it must observe before the next step runs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolStep {
    pub order: usize,
    pub id: String,                 // clone | locate | read | edit | verify | diff
    pub precondition: Vec<String>,
    pub postcondition: Vec<String>,
}

impl WorkspaceProtocol {
    /// Parse the committed protocol document.
    #[must_use] pub fn load() -> Self;
    #[must_use] pub fn parse(document: &str) -> Self;

    /// Execute the protocol for `task` against `workspace`.
    ///
    /// Every step records an execution record (command, exit code, observed
    /// output) into the `NeedLedger` row for its obligation before the next
    /// step is planned — the B5 contract, honoured here rather than restated.
    pub fn execute(
        &self,
        workspace: &mut RepositoryWorkspace,
        task: &RepositoryTask,
    ) -> ProtocolOutcome;
}

/// What a repository task is, independent of where it came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepositoryTask {
    /// The requirement text, verbatim, in whatever language it arrived in.
    pub requirement: String,
    pub clone: WorkspaceSpec,
    /// Named tests, when the source supplies them (SWE-bench does; an issue may not).
    pub tests: Option<Command>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolOutcome {
    pub located: Vec<Location>,
    pub edited: Vec<String>,
    pub observations: Vec<Evidence>,
    pub diff: String,
    /// The first step whose postcondition was not observed, if any.
    pub stopped_at: Option<ProtocolStep>,
    /// Requirements the protocol could not satisfy, stated plainly.
    pub open: Vec<String>,
}
```

### The protocol as data

`data/meta/repository-workspace-protocol.lino`:

```
repository_workspace_protocol
  record_type "meta_recipe"
  issue "1138"
  topic "repository_workspace"
  summary "One protocol for SWE-bench, the #848 ladder and self-coding. Grounded by tests/unit/specification/repository_workspace_protocol.rs against the live source."
repository_step_clone
  record_type "meta_step"
  order "1"
  id "clone"
  detail "Materialise the origin at the exact base commit. A branch name is refused."
  precondition "task carries an origin and a 40-character base commit"
  postcondition "git -C <root> rev-parse HEAD equals base_commit"
  source_file "src/repository_workspace/clone.rs"
repository_step_locate
  record_type "meta_step"
  order "2"
  id "locate"
  detail "Resolve the files the requirement names: census for Rust trees, literal and path occurrence otherwise. Ambiguity resolves to nothing."
  precondition "clone postcondition observed"
  postcondition "at least one Location, or an explicit unresolved need"
  source_file "src/repository_workspace/locate.rs"
… (read, edit, verify, diff)
```

The step order is therefore data. Adding "run the linter before the tests" is a `.lino` edit, not a Rust edit — the #699/#959 rule.

### The command allowlist as data

`data/seed/repository-command-allowlist.lino` declares each permitted **program plus subcommand plus argument shape**:

```
repository_command_allowlist
  record_type "command_allowlist"
  default "deny"
  command git_clone
    program "git"
    subcommand "clone"
    arguments ("--no-checkout" "--filter=blob:none" "{origin}" "{root}")
    mutating "true"
    precondition "root does not exist"
    postcondition "root/.git exists"
  command git_rev_parse
    program "git"
    subcommand "rev-parse"
    arguments ("-C" "{root}" "HEAD")
    mutating "false"
  command cargo_test_named
    program "cargo"
    subcommand "test"
    arguments ("--manifest-path" "{root}/Cargo.toml" "--test" "{suite}" "{name}")
    mutating "false"
```

`rust/src/agent.rs:642-657` `resolve_allowed_program` gains one arm that consults this table instead of growing a hard-coded list:

```rust
fn resolve_allowed_program(program: &str) -> Result<PathBuf, AgentError> {
    let candidates: &[&str] = match program {
        …existing six…
        other => return seed::repository_command_allowlist()
            .program_paths(other)
            .ok_or_else(|| AgentError::UnsupportedCommand(other.to_owned())),
    };
    …
}
```

The `other =>` default-deny arm survives verbatim; what changes is that the allowlist is reviewable data with per-command pre/post conditions, exactly as `data/seed/shell-intents.lino` already declares them for `mv` and `cp` (`rust/src/agentic_coding/mutating_action.rs:29-33`, `rust/src/seed/shell_intents.rs:92-102`).

### SWE-bench conversion

`rust/src/external_benchmarks/cases.rs:152-166` stops manufacturing a prompt and starts manufacturing a task:

```rust
"swebench_lite" => Ok(BenchmarkCase {
    id: instance.clone(),
    prompt: statement,                       // the issue text, unmodified
    repository: Some(WorkspaceSpec {
        origin: repository,
        base_commit: string_field(value, "base_commit", manifest, index)?,
        sparse_paths: Vec::new(),
    }),
    tests: Some(RunCommand {
        line: String::new(),                 // lowered from the tree's ecosystem at run time
        names: json_string_array(value, "FAIL_TO_PASS")?
            .into_iter()
            .chain(json_string_array(value, "PASS_TO_PASS")?)
            .collect(),
    }),
    expectation: Expectation::SweBench { record: value.to_string() },
}),
```

`BenchmarkCase` gains `pub repository: Option<WorkspaceSpec>` and `pub tests: Option<RunCommand>`; every existing suite leaves both `None`, so `cases.rs:60-151` is untouched.

`rust/src/external_benchmarks/mod.rs:150-156` gains one branch:

```rust
let responses = cases
    .iter()
    .map(|case| match &case.repository {
        Some(spec) => repository_workspace::solve_repository_case(&solver, case, spec, &workspace),
        None => solver.solve(&case.prompt),
    })
    .collect::<Vec<_>>();
```

`solve_repository_case` opens a `RepositoryWorkspace`, runs `WorkspaceProtocol::execute`, and returns a response whose `answer` **is** `ProtocolOutcome::diff` wrapped in a fenced block, so `grade::extract_diff` (`grade.rs:175`) keeps working byte for byte and `grade_swebench` is unchanged. The empty-patch short circuit at `grade.rs:186-192` then fires only when the protocol genuinely produced nothing — which is the honest case, not the structural one.

Docker remains optional: `ExecutionBackend::SweBenchImage` is chosen only when `ensure_swebench_runtime()` (`grade.rs:333`) succeeds; otherwise `ExecutionBackend::HostSandbox` runs the named tests with the repository's own interpreter, and a missing interpreter surfaces as `WorkspaceError::MissingPrerequisite` — Plan 06's input.

### `#848` ladder conversion

`experiments/issue_847_coding_ladder/run_coding_ladder.sh` stops reverting the operator's checkout (`README.txt:44-47`) and instead invokes `formal-ai solve --repository . --base-commit $(git rev-parse HEAD) --task "<prompt>"`, which opens a `RepositoryWorkspace::open` on a clone of the checkout at that commit. `new_branch_for.sh` (the L1 verify) then checks the branch inside the workspace. The recorded score moves from "did the ambient tree change" to "did the workspace produce a diff that applies and whose named tests pass".

### `#1085` ladder conversion

`experiments/issue_1028_agent_cli_ladder/run.sh` keeps `leaves.tsv` (the 32 requirements) and **deletes `rules/L01.lino` … `rules/L32.lino`**. Each leaf becomes `locate_targets(workspace, need)` followed by the existing `link_edit_rules::apply_link_edit` (`rust/src/agentic_coding/link_edit_rules.rs:177`) selected by shape, not by a pre-authored rule id. `data/meta/ladder-ratchet.lino` gains a second ratcheted number so the change is visible and honest:

```
  leaf_nodes_selected 32
  leaf_nodes_passing 15
  leaf_nodes_passing_without_authored_rules 0     # new; may only rise
  deepest_passing_level none
```

The first number stays as the historical record; the second is what this plan moves. Recording the new number as 0 on the first run is required, whatever it turns out to be.

### `solve --model formal-ai` as an attributed authoring path

New subcommand in `rust/src/main.rs` (`Command::Solve` — clap's own `Command` enum at `rust/src/main.rs:596-698`, unrelated to the contract type above) and `rust/src/cli_solve.rs`:

```rust
/// Arguments for `formal-ai solve`.
pub struct SolveArgs {
    /// A GitHub issue URL, or `-` to read the requirement from stdin.
    pub issue: Option<String>,
    /// Literal requirement text, when no issue is given.
    pub task: Option<String>,
    /// Repository to work in (default: the current checkout).
    pub repository: String,
    /// Base commit (default: `HEAD` of `--repository`).
    pub base_commit: Option<String>,
    /// Which model authors the change. Only `formal-ai` is an authoring path.
    pub model: String,
    /// Where the run's raw traces are committed.
    pub evidence: PathBuf,
    /// The pull request the commit belongs to.
    pub pull_request: Option<String>,
    /// Refuse to commit; print the diff instead. Default-deny for mutation.
    pub commit: bool,
}

/// Run one repository task end to end and, when `--commit` is given, land it
/// with the four self-hosting trailers.
///
/// # Errors
/// Any protocol step whose postcondition was not observed aborts before the
/// commit; a partial tree is never committed.
pub fn run_solve(args: &SolveArgs) -> Result<SolveOutcome, Box<dyn Error>>;
```

Trailer recording reuses what already exists rather than inventing a second format. `scripts/author-change-with-formal-ai.sh:243` writes

```
Formal-AI-Session: %s
Formal-AI-Model: %s
Formal-AI-Evidence: %s
Formal-AI-Pull-Request: %s
```

and `scripts/self-hosting-metric.rs` reads them (`SESSION_TRAILER`/`EVIDENCE_TRAILER`/`PULL_REQUEST_TRAILER` at `:30-32`, `MODEL_TRAILER` via `scripts/self-hosting-attribution.rs`). `run_solve` emits the identical four lines, writes the session id and the raw protocol trace under `--evidence`, and — because `commit_has_formal_ai_evidence` (`scripts/self-hosting-metric.rs:251-296`) requires each session id to appear *inside* an evidence file and the model string to be named there — writes `formal-ai/<CARGO_PKG_VERSION>` into the evidence bundle verbatim. The `--no-commit` default means mutation is opt-in, matching `rust/src/contribution_write_path.rs`'s refuse-by-default ladder (PR #1027, R1021-x) and `data/meta/self-development-pull-request-contract.lino`.

`scripts/author-change-with-formal-ai.sh` is then reduced to a thin wrapper over `formal-ai solve`, keeping its CLI for the workflow in `.github/workflows/self-authored-pull-request.yml` while the logic lives in Rust and is unit-testable.

### Failure and honesty behaviour

- A protocol step whose postcondition is not observed sets `ProtocolOutcome::stopped_at` and lists every unmet requirement in `open`. The answer says which step stopped and what was observed; it never says "done".
- `run_named_tests` returning `MissingPrerequisite` is **not** a test failure and **not** a pass. It is recorded as an unsatisfied need, handed to Plan 06, and the run reports "the named tests did not run because `<program>` is not available", with the exit code and stderr quoted.
- A timeout is an `Evidence { timed_out: true, .. }` with the elapsed time and the deadline stated. It is a failure of the run, not a budget that silently truncates work: `rust/src/agent.rs:338-395` already reaps the child and reports `timed_out` honestly, and `PYTHON_TIME_BUDGET_FLOOR` (`agent.rs:49`) documents why a floor exists — it is a backstop against start-up latency deciding an outcome, not an allowance.
- `locate_targets` returning empty on ambiguity is the existing `requirement_resolution::resolve_in` contract (`requirement_resolution.rs:28-34`: "a remaining tie is ambiguity and resolves to nothing rather than to a guess"); the protocol reports the ambiguity and the candidates.
- A diff that does not apply cleanly to the base commit is refused before it is offered as an answer.

### Forget-and-rediscover

The protocol document and the command allowlist are both data with content ids. `formal-ai learn cycle` already replays a frontier; the new gate is: delete `data/meta/repository-workspace-protocol.lino`, run the protocol reconstruction command, and assert the regenerated document's content id equals the committed one — the same shape `rust/src/self_ast_census.rs:413` `content_id()` and `drift_report` (`:542`) already use for the census.

## Tests first — held-out cases in en/ru/hi/zh/es with actual prompt text

All five prompts below describe the **same** requirement against the same
repository, and none of them names a file. They are held out: no seed file, test
fixture or ladder rule may contain their wording, and
`scripts/check-hardcoded-language.rs` must not gain an allowlist entry for them.

| Lang | Prompt (verbatim) |
| --- | --- |
| en | `In the repository at commit {base}, the list of trusted search providers is missing the encyclopaedia of quotations. Add it, keep the file valid, and run the tests that cover that list.` |
| ru | `В репозитории на коммите {base} в списке доверенных поисковых провайдеров нет энциклопедии цитат. Добавь её, не сломай файл и запусти тесты, которые покрывают этот список.` |
| hi | `कमिट {base} पर मौजूद रिपॉज़िटरी में भरोसेमंद खोज प्रदाताओं की सूची में उद्धरणों का विश्वकोश नहीं है। उसे जोड़ो, फ़ाइल को वैध रखो, और उस सूची को कवर करने वाले परीक्षण चलाओ।` |
| zh | `在提交 {base} 的仓库里，可信搜索来源的列表缺少引语百科。请把它加进去，保持文件有效，并运行覆盖该列表的测试。` |
| es | `En el repositorio en el commit {base}, la lista de proveedores de búsqueda de confianza no incluye la enciclopedia de citas. Añádela, mantén el archivo válido y ejecuta las pruebas que cubren esa lista.` |

The target is `WEB_SEARCH_PROVIDERS` in `rust/src/web_search_core.rs` — the same
declaration ladder leaf `L01` names (`experiments/issue_1028_agent_cli_ladder/leaves.tsv:1`)
— but the prompts name neither the file nor the constant, so passing requires
`locate_targets` via the census, not a pre-authored rule.

A second held-out family targets a **foreign, non-Rust** tree so `LocationEvidence::LiteralOccurrence` is exercised rather than `Census`:

| Lang | Prompt (verbatim) |
| --- | --- |
| en | `Clone this project at the given commit, find where the default timeout is defined, raise it to sixty seconds, and run only the test that asserts the default.` |
| ru | `Склонируй проект на указанном коммите, найди, где задан таймаут по умолчанию, подними его до шестидесяти секунд и запусти только тот тест, который проверяет значение по умолчанию.` |
| hi | `दिए गए कमिट पर इस प्रोजेक्ट को क्लोन करो, पता करो कि डिफ़ॉल्ट टाइमआउट कहाँ परिभाषित है, उसे साठ सेकंड करो, और केवल वही परीक्षण चलाओ जो डिफ़ॉल्ट की जाँच करता है।` |
| zh | `在给定提交处克隆这个项目，找到默认超时在哪里定义，把它改成六十秒，只运行断言默认值的那个测试。` |
| es | `Clona este proyecto en el commit indicado, localiza dónde se define el tiempo de espera por defecto, súbelo a sesenta segundos y ejecuta solo la prueba que comprueba ese valor por defecto.` |

### Unit / integration / specification tests

| Path | Name | Asserts |
| --- | --- | --- |
| `rust/tests/unit/issue_1138_repository_workspace.rs` | `clone_at_base_refuses_a_branch_name` | `WorkspaceSpec { base_commit: "main", .. }` → `WorkspaceError::NotACommit`, no directory created. |
| same | `clone_at_base_checks_out_the_exact_commit` | after `open`, `base_commit()` equals the spec and `git rev-parse HEAD` in the tree agrees. |
| same | `workspace_is_isolated_from_the_ambient_checkout` | writing in the workspace leaves `CARGO_MANIFEST_DIR` byte-identical. |
| same | `diff_is_empty_for_an_untouched_clone` | `diff()` == `""`. |
| same | `diff_round_trips_through_git_apply` | the produced diff applies to a second clone at the same base and reproduces the edited tree byte for byte. |
| `rust/tests/unit/issue_1138_locate_targets.rs` | `census_locates_a_declaration_the_prompt_never_names` | each of the five prompts above resolves to `rust/src/web_search_core.rs` / `WEB_SEARCH_PROVIDERS`. |
| same | `ambiguity_resolves_to_nothing` | a requirement matching two declarations returns `vec![]` and names both candidates. |
| same | `literal_occurrence_locates_in_a_python_tree` | a three-file Python fixture with one occurrence of `DEFAULT_TIMEOUT` resolves; two occurrences do not. |
| `rust/tests/unit/issue_1138_named_tests.rs` | `missing_interpreter_is_a_prerequisite_not_a_failure` | a fixture whose runner is `kotlinc` yields `MissingPrerequisite { program: "kotlinc", exit_code: Some(127), .. }`, and no `Evidence` row is written. |
| same | `timeout_is_reported_with_the_deadline_and_elapsed` | `timed_out: true`, both numbers present, no pass claimed. |
| `rust/tests/unit/issue_1138_command_allowlist.rs` | `an_unlisted_program_is_still_refused` | `resolve_allowed_program("curl")` → `UnsupportedCommand`. |
| same | `an_unlisted_git_subcommand_is_refused` | `git push` is refused although `git clone` is allowed. |
| same | `allowlist_rows_come_from_seed_not_from_rust` | the set of allowed programs equals the seed table's programs ∪ the six legacy ones. |
| `rust/tests/unit/issue_1138_solve_cli.rs` | `solve_refuses_to_commit_by_default` | without `--commit`, the tree is unchanged and the diff is printed. |
| same | `solve_writes_all_four_trailers` | the commit message contains the four trailer lines and the evidence bundle names `formal-ai/<version>`. |
| same | `solve_with_a_hosted_model_is_not_an_authoring_path` | `--model claude-…` refuses to write the `Formal-AI-Model` trailer. |
| `rust/tests/unit/specification/repository_workspace_protocol.rs` | `protocol_document_matches_the_live_source` | every `meta_step.source_file` exists, `order` is contiguous 1..n, every `id` is matched by a `ProtocolStep` arm — the same grounding contract `rust/tests/unit/specification/agentic_meta_algorithm.rs` applies to `data/meta/agentic-coding-recipe.lino`. |
| same | `protocol_document_is_rediscoverable` | delete-and-regenerate reproduces the committed content id. |
| `rust/tests/integration/issue_1138_swebench_case.rs` | `swebench_case_carries_a_clone_spec` | the parsed case has `repository: Some(_)` with a 40-char base commit and non-empty `FAIL_TO_PASS`. |
| same *(`#[ignore]`, network)* | `one_lite_instance_runs_the_whole_protocol` | clone → locate → edit → verify → diff, with the diff recorded whatever it is. |
| `rust/tests/unit/docs_requirements/issue_1138.rs` | `requirements_shard_matches_the_plan` | the new `docs/requirements/issue-1138-…` shard names every R1138-3-x id used here. |
| `rust/tests/source/source_tests/repository_workspace/tests.rs` | `module_files_stay_under_the_size_ceiling` | the existing `scripts/check-file-size.rs` contract for the new directory. |

### Gates and ratchets

1. `data/meta/ladder-ratchet.lino` gains `leaf_nodes_passing_without_authored_rules`, checked by the existing step in `.github/workflows/issue-1028-agent-ladder.yml`; it may only rise.
2. `data/meta/debt-ratchet.lino` gains `authored_ladder_rules` with the current value `32`; it may only **fall**, reaching `0` when RC6 is closed.
3. A new `data/meta/ci-gates/` row `repository_workspace_protocol` with its justification (the gate-justification requirement of R1085-16).
4. `.github/workflows/external-benchmarks.yml` `swebench_slice` default rises from `1` to `23` (the whole Lite dev split) once one instance completes the protocol without `benchmark_unavailable`, and the ledger records whatever the score is.
5. The existing `benchmark ratchet` (`rust/src/external_benchmarks/ratchet.rs`) keeps `0/1` as the floor until a run beats it; nothing in this plan may lower a recorded floor.

### Benchmark commands — measure and record whatever it is

```sh
# One instance through the new protocol, honest result recorded.
cargo run --bin formal-ai -- benchmark run --suite swebench_lite --slice 1 --online --append

# The whole Lite dev split. Record the number, whatever it is; do not tune to it.
cargo run --bin formal-ai -- benchmark run --suite swebench_lite --slice 23 --online --append

# Ratchet check without running anything.
cargo run --bin formal-ai -- benchmark ratchet --base-ref origin/main

# The #848 ladder through the workspace protocol.
experiments/issue_847_coding_ladder/run_coding_ladder.sh          # records results.json

# The #1085 ladder with the 32 authored rules deleted.
experiments/issue_1028_agent_cli_ladder/run.sh --no-authored-rules

# One real authoring run, no commit.
cargo run --bin formal-ai -- solve --issue https://github.com/link-assistant/formal-ai/issues/1091 \
  --model formal-ai --evidence docs/case-studies/issue-1138/evidence/first-solve
```

Every one of these writes its number to a ledger before anything is tuned. A
`0/23` on SWE-bench Lite is an acceptable, publishable outcome of this plan; a
`0/23` that is *not recorded* is not.

## Implementation leaves — ordered, each individually verifiable and commit-sized

- [x] **L1.** Add `WorkspaceCensus::of_directory(root)` to `rust/src/self_ast_census.rs` beside the existing `compile(files)` (`:293`). Test: a three-file fixture directory censuses identically to `compile` on the same `(path, source)` pairs.
- [x] **L2.** Add `rust/src/repository_workspace/clone.rs` with `WorkspaceSpec` and `clone_at_base`. Test: exact-commit checkout, branch-name refusal, deterministic tree.
- [x] **L3.** Add `data/seed/repository-command-allowlist.lino` and route `rust/src/agent.rs:642-657`'s `other =>` arm through it, keeping the default-deny arm. Test: `curl` refused, `git push` refused, `git clone` allowed, allowlist set equals the seed table.
- [x] **L4.** Add `rust/src/repository_workspace/mod.rs` with `RepositoryWorkspace::{open, adopt, root, base_commit, source_files, read, write}`. Test: isolation from the ambient checkout.
- [x] **L5.** Add `rust/src/repository_workspace/diff.rs` `unified_diff` and `RepositoryWorkspace::diff`. Test: empty diff for an untouched clone; `git apply` round trip.
- [x] **L6.** Add `rust/src/repository_workspace/locate.rs` `locate_targets`, delegating Rust trees to `requirement_resolution::resolve_in` unchanged. Test: the five held-out prompts and the ambiguity case.
- [x] **L7.** Add `rust/src/repository_workspace/verify.rs` `Command`, `ExecutionBackend`, `run_named_tests`, `Evidence`, and `WorkspaceError::MissingPrerequisite`. Test: missing interpreter, timeout, pass/fail split.
- [x] **L8.** Add `data/meta/repository-workspace-protocol.lino` and `WorkspaceProtocol::{load, parse, execute}` plus `rust/tests/unit/specification/repository_workspace_protocol.rs`. Test: source-file grounding, contiguous order, rediscovery content id.
- [x] **L9.** Wire the protocol's per-step observations into `NeedLedger` rows so no step is `Satisfied` without an execution record (`rust/src/meta_frame.rs:644-700`). Test: a step that did not run leaves its need `Planned`, never `Satisfied`.
- [x] **L10.** Widen `BenchmarkCase` with `repository` / `tests`; convert `cases.rs:152-166`; add the `solve_repository_case` branch at `mod.rs:150-156`. Test: parsed case carries a 40-char base commit; every other suite still has `None`.
- [ ] **L11.** Run one SWE-bench Lite instance end to end and append the honest row to `data/benchmarks/external-results.lino`. No tuning in this commit.
- [x] **L12.** Add `rust/src/cli_solve.rs` + `Command::Solve` in `rust/src/main.rs`, refusing to commit by default. Test: the three `solve_cli` cases.
- [x] **L13.** Emit the four trailers and the evidence bundle from `run_solve`; reduce `scripts/author-change-with-formal-ai.sh` to a wrapper. Test: `scripts/self-hosting-attribution.rs::model_attribution` accepts the produced commit; a hosted model is refused.
- [x] **L14.** Convert `experiments/issue_847_coding_ladder/run_coding_ladder.sh` (today `cmd = [binary, "with", "agent", "--non-interactive", "-p", task["prompt"]]` at `run_coding_ladder.sh:196`) to `formal-ai solve --repository . --base-commit …`; rerun; record the new 130-task number whatever it is. **This commit must also update `rust/tests/unit/issue_848_coding_ladder.rs:532-560`, which pins nine exact substrings of that script** (`"[\"rustc\", \"--edition=2024\""`, `"rust_target_existed[created]"`, `"\"dataset_total\": len(all_tasks)"`, `"\"complete\": not only"`, `"results-partial-$FILTER_SLUG.json"`, `"expect_from_file"`, `"re.MULTILINE"`, and both lines of the `server_started` / `not_measured` predicate at `run_coding_ladder.sh:284-288`). Every pinned semantic must survive; only the invocation line changes.
  **Closed (2026-09-23). The conversion and pin halves landed 2026-09-19; the
  measurement half landed with run 35783073283 (branch worktree-issue-1138,
  9d98e28ad), the first complete post-conversion run — 20/130, L1 0/16, no
  `NOT MEASURED` rows, reproduced at the same score by the df2701415 round.
  This leaf's own instruction was to record the discontinuous number whatever
  it is, so 20/130 replaces the 65/130 agent-transport row as the committed
  canonical result and the ratchet floor, which may only rise from here. The
  deterministic `solve` protocol's edit derivation currently covers one shape
  (adding quoted members to a located declaration) and has no answer channel
  for read-family tasks, which is where the 45-task difference lives; raising
  the number is protocol capability work, not harness change. The v0.320.0
  65/130 result stays recorded in
  `docs/case-studies/issue-848/README.md`.**
- [x] **L14b.** Wire the #848 ladder into CI — it has never run there (`docs/case-studies/issue-957/raw-data/verified-776-928.md:57,62`: "recorded score 65/130 with L1 = 0/16 — the exact 'honest attempt at L1' bar konard set is still failing, and nothing ratchets it"). Add `.github/workflows/coding-ladder.yml` modelled on `.github/workflows/task-ladder.yml` (weekly + `workflow_dispatch` + path-filtered), a `data/meta/ci-gates/coding-ladder.lino` row with its justification, and a floor in `data/meta/ladder-ratchet.lino` for the 130-task score that may only rise. This closes the R848-1 "ladder runs in CI with recorded score" clause that `docs/case-studies/issue-957/raw-data/verified-all.ndjson:880` marks `PARTIAL`.
- [ ] **L15.** Add `leaf_nodes_passing_without_authored_rules` to `data/meta/ladder-ratchet.lino`, run `issue_1028_agent_cli_ladder` with rules disabled, record the number, and add `authored_ladder_rules: 32` to `data/meta/debt-ratchet.lino` as a shrink-only ceiling.
- [ ] **L16.** Raise `swebench_slice` to `23` in `.github/workflows/external-benchmarks.yml` and record the full-split row. **A slice is a measurement width, not a ceiling: widening it can only lower the recorded score, and `historical_floor_violations` groups by `(suite, slice)`, so the recorded `0/1` floor is untouched. This is not a loosened gate (plan 00 §9 X7).**
- [ ] **L17.** Author one real repository change through `formal-ai solve --commit` on a bot branch and attach it to the release cycle, closing R1021-22 or recording precisely why it is still open.
- [x] **L18.** Update `REQUIREMENTS.md` shard, traceability, `docs/benchmarks.md`, `docs/meta-algorithm.md`, `VISION.md`, `ROADMAP.md`, `GOALS.md` per the next section.

### L1-L18 reconciliation checkpoint — 2026-09-17

This checkpoint records the live tree before the final Plan 03 close-out. A
checkbox is not changed merely because code exists: the focused test named by
the leaf must pass in this worktree first.

| leaf | live-tree state before close-out | remaining proof or work |
| --- | --- | --- |
| L1 | `WorkspaceCensus::of_directory` exists and `issue_673_self_ast_census::directory_census_matches_the_same_explicit_source_set` supplies the three-file fixture equivalence test (two Rust sources plus one excluded non-source). | Focused test verified green on 2026-09-18; leaf checked. |
| L2-L5 | The exact-commit clone, seed command policy, isolated workspace and round-trip diff are implemented with Plan 03 unit coverage. | Re-run the focused workspace and command-policy tests. |
| L6 | `locate_targets` uses the live workspace census plus seed meanings, reports ambiguity, and has held-out en/ru/hi/zh/es and foreign-tree tests. | Focused locator tests verified green on 2026-09-18; leaf checked. |
| L7-L8 | Named-test execution and the data-owned protocol are implemented; the specification tests ground the source files, order and reconstructed content id. | Re-run the focused named-test and protocol-specification tests. |
| L9 | `WorkspaceProtocol::execute` starts every step `Planned` and only projects `Satisfied` from an attached `Evidence` record; stopped and skipped steps are tested. | Focused workspace test verified green on 2026-09-18; leaf checked. |
| L10 | `BenchmarkCase` carries optional `WorkspaceSpec` / `RunCommand`; only SWE-bench populates them and the runner selects `solve_repository_case`. | Pinned integration parser test verified green on 2026-09-18; leaf checked. |
| L11 | The ignored live one-instance test exists. No new network/container execution has been observed in this close-out, so no result may be appended. | Live clone, official evaluator run, and honest external-results row. |
| L12 | `formal-ai solve` is registered and defaults to an isolated, non-committing run; the three required CLI behaviours have unit coverage. | Focused solve tests verified green on 2026-09-18; leaf checked. |
| L13 | `run_solve` writes four trailers and commits the same model-bearing evidence bundle as its source edit, and the produced commit is now proved against the canonical attribution parser (`solve_attribution::solve_commit_payload_is_accepted_by_the_canonical_attribution_parser`, green 2026-09-18). The wrapper reduction landed 2026-09-19: `scripts/author-change-with-formal-ai.sh` is a thin translator onto `formal-ai solve`, the loop itself lives in `rust/src/authoring_loop.rs` (behaviorally proved by `ci_cd::authoring_effects` and `solve_attribution::the_live_authoring_loop_lands_a_commit_the_canonical_parser_accepts`, green 2026-09-19 — four trailers in one commit, producer-naming evidence, framed-events-only capture, bounded readiness probe, seed replays refused as authorship), and the wrapper shape stays pinned by `ci_cd::issue_1069::the_authorship_route_is_a_wrapper_over_solve_and_never_publishes`. | Leaf checked. |
| L14 | The coding ladder invokes `formal-ai solve`, validates stdout as a patch, then applies it before its existing judges. | Closed 2026-09-23 by run 35783073283 (9d98e28ad): 20/130 complete, zero `NOT MEASURED`, recorded as the discontinuous baseline. |
| L14b | The path-filtered scheduled workflow, gate record and 65/130 ratchet exist. | Re-run its hermetic checker. |
| L15 | The `--no-authored-rules` run mode, the disabled-rule result field, the `not_measured` ratchet row and the `authored_ladder_rules: 32` shrink-only ceiling all exist. | Only a real 32-leaf Agent CLI run may set the passing value. |
| L16 | Width-23 semantics and tests are encoded; historical floors remain keyed by `(suite, slice)`. | A full live run is still required before a `0/23` or better result row may be claimed. |
| L17 | No close-out evidence proves a real `formal-ai solve --commit` bot-branch contribution attached to a release cycle. The draft flow exists with hermetic coverage (`issue_1138_draft_pull_request`, four tests green 2026-09-18). | External authoring, pull-request and release evidence. |
| L18 | The issue-1138 requirement shard and broad documentation edits exist in the shared worktree; the plan 03 requirements suite passes all five tests (2026-09-18). Plan 11 owns D170-D180 and their final consistency proof. | Leaf checked; leave the cross-document close-out to Plan 11 rather than duplicate it here. |

### Prefix-slot routing fix — 2026-09-18

The held-out Spanish repository prompt derailed before the capability table:
its word for "providers" (*proveedores*) embeds the English "prove", and
`spelled_surface_present` read a prefix surface such as the seeded
`prove …` as a raw substring, so `proof_directive` was evidenced, the
`proof_request` promotion fired, and the promoted handler preempted the typed
`shell` handoff every other language received. A prefix surface's lead half is
a phrase, so it now must occur as complete words — a word boundary is the text
edge or a non-alphanumeric character, which keeps "¿Cuántos litros …" matching
while "proveedores" does not (`rust/src/rule_interpreter.rs`).
`issue_1138_handler_promotions::a_prefix_surface_does_not_match_inside_an_embedding_word`
pins the promotion side; `issue_1138_self_use_repository_workspace::chat_hands_repository_location_to_a_workspace_capable_client`
pins the five-language handoff it restores.

### L14/L14b implementation checkpoint — 2026-09-17

The runner now observes `HEAD`, calls `formal-ai solve --repository .
--base-commit <observed HEAD> --task <prompt> --evidence <per-task temp dir>`,
captures stdout as the sole diff transport, checks it with `git apply --check`,
and applies it explicitly before the unchanged effect, compiler and reset
judges run. `rust/tests/unit/issue_848_coding_ladder.rs` pins that invocation and
transport while retaining every earlier semantic pin.

The standalone, path-filtered `.github/workflows/coding-ladder.yml` runs weekly
and on dispatch, and its temporary full result is checked against the 65/130
floor. The fast registered `coding_ladder` gate only compares workflow,
committed full result and ratchet; it never executes the 130 tasks. The
committed evidence remains the honest 2026-08-02 result, including L1 0/16.
Therefore L14 itself remained unchecked until 2026-09-23, when run 35783073283
on 9d98e28ad completed the measurement half: 20/130, L1 0/16, zero
`NOT MEASURED` rows, at the same score the df2701415 round had measured. That
run is now the committed canonical result and the ratchet floor, and the
pre-conversion 65/130 stays recorded in `docs/case-studies/issue-848/README.md`.

## Docs to update — exact statements, quoted, with replacement

The exact quoted statements and their replacement text moved to plan 11's
findings table on 2026-09-16, so there is one docs authority and no document
is described in two places (plan 00 §8). This plan's entries are rows
**D170-D180** of
[`11-docs-consistency-audit.md`](11-docs-consistency-audit.md) §"Issue #1138
plan doc replacements", and plan 11's leaves apply them after the ledger rows
they cite exist (plan 00 §7).

| row | document |
| --- | --- |
| D170 | `docs/benchmarks.md:311` |
| D171 | `docs/benchmarks.md:314-320` |
| D172 | `docs/meta-algorithm.md:185-262` |
| D173 | `ROADMAP.md:145` |
| D174 | `VISION.md:343` |
| D175 | `GOALS.md:96` |
| D176 | `GOALS.md:111` |
| D177 | `docs/requirements/issue-1085-the-links-network-is-not-the-system-that-reasons.md` |
| D178 | `docs/requirements/issue-1021-full-range-coding-and-contribution-artifacts.md` |
| D179 | New shard `docs/requirements/issue-1138-repository-workspace-protocol.md` |
| D180 | `docs/requirements-traceability.md` |

Any further document this plan's implementation touches is added as a new
plan 11 row, never as a second copy here.

## Risks and open questions

1. **`git` in the sandbox.** Even subcommand-scoped, `git clone` fetches arbitrary remote content and `git` reads user config. Mitigation: `--no-checkout --filter=blob:none` plus an explicit checkout of the base commit, `GIT_CONFIG_COUNT=0` and `.env_clear()` (already the case at `rust/src/agent.rs:361`), and a seed-declared origin allowlist for the authoring path. Open: whether SWE-bench's 12 upstream origins should be enumerated in seed or accepted as "any `https://github.com/` origin named by the dataset".
2. **Disk.** SWE-bench Lite's 23 dev instances are large Python repositories; `scripts/free-runner-disk.sh:20-26` already deletes SDKs to make room for box images. Open: whether `sparse_paths` can be derived from the issue text well enough to avoid full clones, and what `scripts/check-disk-usage-policy.rs` should say about the clone cache.
3. **Diff production without a dependency.** The repository keeps a deliberately thin dependency tree (`.github/workflows/stock-rust-install.yml:36-43`). A hand-written Myers diff is ~200 lines and must produce output `git apply` accepts exactly. Alternative: shell out to `git -C <root> diff` under the allowlist, which is simpler but makes `git` load-bearing for *reading* as well as cloning. Open; the tests (`diff_round_trips_through_git_apply`) are the same either way.
4. **Non-Rust location quality.** `LocationEvidence::LiteralOccurrence` is deliberately weak. Every SWE-bench Lite instance is a Python repository, so the census path never helps there. This plan does not claim Python location works; it claims the *number* becomes measurable. Whether B1's concept lookup and B4's formalization are needed before the number moves is an open empirical question — record the number first.
5. **Ladder discontinuity.** Converting the #848 ladder from the ambient checkout to a clone will change its score for reasons unrelated to capability (tasks that relied on the operator's dirty tree). The first post-conversion run must be recorded as a new baseline with the reason stated, not compared to 65/130 as if the measurement were the same.
6. **Deleting the 32 authored rules.** `data/meta/ladder-ratchet.lino`'s `leaf_nodes_passing 15` is a committed floor. This plan does not lower it; it adds a second number. Open: whether the maintainer wants the authored-rule number retired entirely once the second number exceeds it.
7. **Interaction with Plan 06.** Trace (c) stops at `kotlinc: command not found`. If Plan 06 lands first, the protocol's `MissingPrerequisite` becomes a requirement and the run continues; if this plan lands first, the run stops honestly. Both orders are acceptable; what is not acceptable is either plan claiming the other's result.
8. **`solve` name.** `formal-ai solve` collides conceptually with `hive-mind solve` and with `UniversalSolver::solve`. `grep` shows no `Command::Solve` and no `"solve"` subcommand string in `rust/src/main.rs`, so there is no code collision, but the documentation must be explicit that `formal-ai solve` is the repository-task entry point and `UniversalSolver::solve` remains the prompt entry point.
9. **Brittle literal pins.** Three test modules assert exact substrings of the artifacts this plan edits: `rust/tests/unit/issue_848_coding_ladder.rs:532-560` pins nine substrings of `run_coding_ladder.sh` and two `prompts.json` task shapes; `:669-722` pins the score strings `"38/130"`, `"61/130"`, `"62/130"`, `"65/130"` in `docs/case-studies/issue-848/README.md` (the `"65/130"` literal is at `rust/tests/unit/issue_848_coding_ladder.rs:681`); `rust/tests/unit/docs_requirements/issue_924.rs:82-97` pins five header substrings of `data/meta/self-hosting-ledger.lino`. Every one of these must be updated in the same commit as the artifact it pins, and none may be weakened to let a new number through. `REQUIREMENTS.md:1725-1745` and `docs/requirements/issue-0848-executable-coding-tasks.md:10` carry the same `65/130` line and must move together.
10. **The 0.15 % figure is prose, not data.** Issue #1138's table cites "self-hosting share 0.15 % (58 of 38,373)". That number occurs exactly once in the repository, at `docs/case-studies/issue-710/plans/04-final-requirements-release-and-self-improvement.md:460-461`, as the output of a local `rust-script scripts/self-hosting-metric.rs` run; no ledger row has `percentage_basis_points "15"` or `changed_lines "38373"`, and it predates the failed strict measurement recorded at `docs/case-studies/issue-710/plans/07-prerequisite-discovery-bridge.md:427-432`. Any claim this plan makes about moving the share must cite a ledger row in `data/meta/self-hosting-ledger.lino` (the newest is `tag "v0.350.0"`, `percentage_basis_points "171"`, `trailing_percentage_basis_points "389"` at lines 1214-1230), not that prose.
11. **Attribution correctness.** PR #888's post-push audit (`docs/case-studies/issue-710/plans/07-prerequisite-discovery-bridge.md:427-441`) shows a trailer declaring `formal-ai/0.350.0` whose evidence never named that identifier, requiring an append-only retraction. `run_solve` must write the model identifier into the evidence bundle in the same commit, and L13's test must assert exactly that, or the same retraction will be needed again.
