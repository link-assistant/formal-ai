# Formal AI full-repository audit — final issue manifest

Repo: `link-assistant/formal-ai` @ `de61602f` (v0.326.0). Audit date 2026-08-04.
Sources consolidated: `needs-issue.ndjson` (27 rows), `arch-review.md` (25 findings), `test-report.md` ("broken things", 7 items), `handchecks.ndjson` (49 rows), `open-issues.txt` (61 open issues) + `verified-all.ndjson` tracked_by map.

Planning umbrella: **#651**. Last used epic number: **E77 = #924**. New epics start at **E78**.

Standing process clauses (konard, applied to every issue body below): collect data to `docs/case-studies/issue-{id}`; add debug/verbose output when root cause is undeterminable; file upstream issues with repros where relevant; deliver in a single PR until every requirement is fully addressed; state exactly what to do and how to test it.

---

## Part A — New issues to file

### E78: Telegram docker execution pipeline for code examples (issue #8)

**Body:**

**Problem statement.**
Source: [#8](https://github.com/link-assistant/formal-ai/issues/8) (issue body). konard: "Bot should not provide any code example without compiling and running it. For that it should use http://github.com/link-foundation/start to spawn docker images" and "If it timeouts in 1 minute, the bot should try to reduce number of iterations in half and so on" and "if code does not compile, and reasoning of the bot takes more than 10 minutes — the task should totally fail, and verbose log ... recorded". konard later allowed interface-first delivery ("intelligence may be not smart enough at the moment, but all the interface though telegram bot should be all in place"), so the execution backend was explicitly deferred, not waived.
Current-code evidence: no compile/execute pipeline exists in `src/telegram.rs` / `src/telegram_runtime.rs`; `data/seed/environments.lino` telegram tool entries exclude execution; no iteration-halving timeout retry exists anywhere in `src/` (only an unrelated question-generation tier-halving mechanism); no 10-minute hard-fail-with-verbose-log path exists on the telegram surface.

**What to do.**
1. Wire the telegram code-example path through `link-foundation/start` to spawn a docker container (reuse the container lifecycle work from E79/E-worker below if landed first; otherwise a single-shot container per compile/run is sufficient for this issue).
2. Never emit a code example to the user without a successful compile+run in that container; on failure, report the failure honestly (no silent fallback to an unverified example).
3. On a 1-minute execution timeout: halve the iteration/loop-bound parameter and retry; keep halving until either success or the retry floor is hit. Report to the user which N (iterations/variables) triggered the timeout and the N at which it stopped timing out, so users can reason about performance.
4. If the code does not compile and total bot reasoning exceeds 10 minutes, fail the task outright (do not return a best-effort answer) and persist a verbose log of every reasoning step and action taken, for later improvement.

**How to test.**
- Automated: unit tests for the iteration-halving retry loop (mock timeout injector, assert halving sequence and reported N); integration test that a telegram code-example prompt only returns code after a `verified compiled and ran` marker (reuse the pattern in `chat --format chat`'s `thinking_steps`/execution-status field); a test asserting a forced 10-minute-reasoning + non-compiling case fails the task and writes a verbose log file.
- Manual: run the bot against a Rust/Python "hello world" prompt in a private chat and confirm the returned code was actually compiled and run; force a pathological slow case and confirm halving + reporting behavior; force a permanently-non-compiling case and confirm hard failure with a verbose log path printed.
- Multilingual: repeat the "give me code for X" prompt in en/ru/hi/zh; the compile-verify gate must apply identically regardless of prompt language.
- Standing clauses: collect data to `docs/case-studies/issue-{id}` (timeline of #8, full requirement list including the interface-first allowance, per-requirement solution plan, survey of link-foundation/start's current API, online research on docker-in-docker sandboxing); add verbose/debug output per requirement 4 above; deliver in one PR until every sub-requirement here is addressed.

**Source refs:** #8 (issue body). **Dedup:** merges R8-1, R8-2, R8-4 (identical `proposed_issue_title` in needs-issue.ndjson — three requirements from the same issue, one deliverable).

---

### E79: Local WebSocket/WebRTC memory server with the CLI as both server and client (issue #107)

**Body:**

**Problem statement.**
Source: [#107 comment](https://github.com/link-assistant/formal-ai/issues/107#issuecomment-4481573815). konard: "make sure we have local server, that will be available at localhost at WebSocket and WebRTC protocols ... So CLI should be a server and client" — i.e. fully local agent-storage server reachable the same way the CLI itself is used, giving one simple interface to Formal AI.
Current-code evidence: no WebSocket or WebRTC code exists anywhere in `src/` (grep clean); `src/network_endpoint.rs` only implements plain HTTP. No delivery evidenced in #114 or later.

**What to do.**
1. Add a local WebSocket server mode to `formal-ai serve` (or a new subcommand) that speaks the same request/response shape as the existing HTTP/OpenAI-compatible surface, bound to localhost by default.
2. Add a WebRTC data-channel mode for the same local-first use case (peer-to-peer agent storage access without a central relay).
3. Make the CLI itself capable of acting as both server (spin up the local WS/WebRTC endpoint) and client (connect to a running local endpoint) through one binary, so `formal-ai` is "a simple interface to Formal AI" per the requirement.
4. Reuse the existing permission/memory model — this is a transport addition, not a new storage engine.

**How to test.**
- Automated: integration test that starts the WS server, connects a WS client (Rust test harness), round-trips a chat request, and asserts parity with the HTTP path on the same prompt; a WebRTC data-channel smoke test (loopback offer/answer) exercising one full request/response.
- Manual: `formal-ai serve --ws` then connect with a generic WebSocket client (e.g. `websocat`) and confirm a working chat exchange; confirm CLI-as-client mode connects to a separately-running server instance.
- Multilingual: run one en/ru/hi/zh prompt each over the WS transport and confirm answers match the HTTP-transport answers byte-for-byte (determinism doctrine).
- Standing clauses: `docs/case-studies/issue-{id}` with full requirement list, WebRTC-in-Rust library survey (e.g. `webrtc-rs`), and solution plan; verbose logging on connection lifecycle if debugging is needed; single PR.

**Source refs:** #107. **Dedup:** none (unique, not covered by any open issue).

---

### E80: Exercise generated per-language projects inside matching link-foundation box images in CI (issue #119)

**Body:**

**Problem statement.**
Source: [#119 PR comment](https://github.com/link-assistant/formal-ai/pull/119#issuecomment-4484885013). konard: "we should prefer test each software project of a specific language inside such version of link foundation box docker image, that matches the language" — to reduce test size/flakiness by using each language's own traditional repo-init tooling inside the right box image.
Current-code evidence: box DinD (`konard/box-dind` Dockerfile) is already the CI runtime, but no CI leg tests generated language projects (from `installation_conversion.rs` / `program_synthesis.rs` outputs) inside a matching per-language link-foundation box image using that language's native init commands.

**What to do.**
1. For each language the project-generation/installation-conversion handlers support, add a CI matrix leg that pulls the matching `link-foundation/box` image variant.
2. Inside that container, run the language's traditional init/build commands (`cargo new` + `cargo build`, `npm init` + `npm install`, `pip`/`poetry`, etc.) against a Formal-AI-generated project to verify it actually builds/runs, not just that the generator emitted plausible text.
3. Wire this as a new CI job (or extend an existing coding-catalog job) in `release.yml`, gated the same way other slow legs are.

**How to test.**
- Automated: the new CI job itself is the test; additionally add a local `cargo test` harness that can invoke the same box-image check via `docker run` when Docker is available, skipping gracefully otherwise (mirrors `verify-docker-runtime.sh` pattern).
- Manual: run the new CI job locally with `act` or direct `docker run` against 2-3 sample generated projects (Rust, Python, JS) and confirm each builds inside its box image.
- Multilingual: not directly language-of-prompt relevant, but ensure the generated-project corpus includes projects produced from en/ru/hi/zh prompts.
- Standing clauses: `docs/case-studies/issue-{id}`; survey existing `link-foundation/box` image tags; single PR.

**Source refs:** #119. **Dedup:** none.

---

### E81: CI check enforcing ≥5 wording variations per language for every conversational test case (issues #103/#123/#134)

**Body:**

**Problem statement.**
Source: [#123 comment](https://github.com/link-assistant/formal-ai/issues/123#issuecomment-4485896648). konard: "we should not stop until each test will have at least 5 variations per 4 languages, also I think we should have CI/CD checks to enforce that." PR #124 fixed the specific reported prompts, but the CI/CD enforcement was never delivered.
Current-code evidence: `demo.spec.js:324` resolves example prompts through the real worker; `check:language-parity` and `check:intent-coverage` CI checks exist — but no check enforces "≥5 wording variations per language" as a per-test-case floor.

**What to do.**
1. Define a machine-checkable convention for "wording variation" (e.g. a naming/tagging scheme in test fixtures, or a manifest listing prompt-variant groups per test case).
2. Write a CI script (in the style of `check-language-parity`) that walks the conversational test corpus and fails if any test case has fewer than 5 variations in any of en/ru/hi/zh.
3. Backfill variations for existing under-covered test cases until the new check passes.
4. Wire the check into `release.yml`.

**How to test.**
- Automated: the new CI script itself, plus a unit test on the script's counting logic using fixture data engineered to trip the floor.
- Manual: intentionally reduce one test case to 4 variations in one language and confirm the CI check fails locally.
- Multilingual: the check is inherently en/ru/hi/zh scoped; confirm coverage counts print per-language.
- Standing clauses: `docs/case-studies/issue-{id}`; single PR; verbose output listing exactly which test cases are under the floor.

**Source refs:** #123 (comment), follow-up #124. **Dedup:** none — `check:language-test-coverage` (in test-report.md's CI gate list) is a related but distinct existing check; confirm no overlap before implementing (note this in the case study).

---

### E82: Restart E39 — shrinking JS-worker budget, hard-fail WASM fallback, and slice-2 absorption (issue #658 continuation)

**Body:**

**Problem statement.**
Source refs: [#658](https://github.com/link-assistant/formal-ai/issues/658) (E39, CLOSED but incomplete per doctrine), needs-issue R658-1, R134-2 ("still a lot of non-UI logic in JavaScript" — [#134 comment](https://github.com/link-assistant/formal-ai/pull/134#issuecomment-4489651616)), arch-review 2.1/8.1 (HIGH), REQUIREMENTS.md R536 ("JavaScript must be used only as interfacing glue and for JSX (React) UI components ... same WASM web engine ... reused"), konard's 2026-08-04 hard doctrine statement (JS = glue/UI only, all logic in Rust).
Current-code evidence: `scripts/check-worker-line-budget.rs:27,63` — `TARGET_TOTAL_LINES = 3_000`, `CEILING_TOTAL_LINES = 27_705`; the doc comment (`:34-58`) records **eight upward re-baselines** since 2026-07-14 and never one downward. `src/web/worker/formal_ai_worker_00..23.js` totals 27,705 lines (grew ~1,000 lines since `docs/case-studies/issue-658/capability-inventory.md` recorded 26,708). Only migration slice 1 (CI guards) of 6 planned landed. `formal_ai_worker_20.js:1332-1339` silently falls back from WASM to the full JS mirror on any instantiation failure with no user-visible signal and no engine-provenance in the answer trace. `ROADMAP.md:421` incorrectly states "#658 closed" as if absorption finished.

**What to do.**
1. Change `check-worker-line-budget.rs`'s contract: `CEILING_TOTAL_LINES` may only decrease between releases; CI fails if a PR raises it. Require any PR adding worker lines to remove at least as many elsewhere.
2. Land migration slice 2 (extraction/parsing → WASM per the original inventory) and delete the JS fallback code at each migrated delegation site rather than keeping both paths (model: `formal_ai_worker_23.js`, an 18-line true adapter).
3. Replace the silent `catch { wasm = null; mode = "js fallback" }` at `formal_ai_worker_20.js:1332-1339` with a visible "engine unavailable" error state; keep a diagnostic-only override for local development; record `engine: wasm|js` in every answer's trace.
4. Correct `ROADMAP.md:421`'s "#658 closed" claim to reflect actual absorption status.
5. Reopen tracking under this new epic since #658 is closed but the underlying requirement (R536, R380) is not satisfied.

**How to test.**
- Automated: CI assertion that a PR cannot raise `CEILING_TOTAL_LINES`; parity-fixture growth test (`data/parity/cross-runtime-synthesis.json`) covering each migrated function with held-out inputs; e2e test that stubs a 404 `.wasm` fetch and asserts the demo surfaces a loud error, not a silent JS-mode answer; a trace-inspection test asserting `engine=wasm` on ordinary runs.
- Manual: run `bun run build:web`, corrupt/rename the built `.wasm`, reload the demo, and confirm a visible error banner instead of silent degraded answers.
- Multilingual: re-run the existing en/ru/hi/zh parity fixtures after slice 2 lands and confirm no regression.
- Standing clauses: `docs/case-studies/issue-{id}` (full requirement list incl. R536/R380 lineage, per-slice solution plan, existing-library survey for WASM loader patterns); verbose diagnostic logging for WASM instantiation failures; single PR per landed slice, but the epic itself tracks to full absorption across PRs.

**Source refs:** #658 (E39, closed-but-incomplete), #134, R536, R380. **Dedup:** this is the restart/continuation of closed #658 — do not file as a fresh unrelated issue; cross-link explicitly. Also subsumes needs-issue R658-1 and R134-2 (identical underlying problem).

---

### E83: File the two promised upstream relative-meta-logic issues (library usability + WASM compilation) — issues #185, #209

**Body:**

**Problem statement.**
Source: [#185 comment](https://github.com/link-assistant/formal-ai/issues/185#issuecomment-4500644364) — konard required proof requests to use `link-foundation/relative-meta-logic` as a Rust library, and to report an upstream issue if it's not usable as a library or lacks needed features. [#209 comment](https://github.com/link-assistant/formal-ai/issues/209#issuecomment-4529053709) — konard required prime-infinitude proofs to compile to WebAssembly via relative-meta-logic, and to file an upstream feature request if RML lacks wasm-to-wasm compilation (if technically possible).
Current-code evidence: `relative-meta-logic` is absent from `Cargo.toml`; `src/relative_meta_logic.rs` re-models the concept in-repo instead of consuming the library (PR #199 delivered a proof engine without it). `src/proof_engine/library.rs` + `tests/e2e/tests/issue-209.spec.js` deliver multilingual prime-infinitude proofs but zero `wasm` references exist in `src/proof_engine/`. `gh` search of `link-foundation/relative-meta-logic` issues shows no filing from either thread.

**What to do.**
1. Evaluate `link-foundation/relative-meta-logic` as a Rust library dependency today (versions may have moved since #185/#209); determine concretely what's missing for (a) library consumption from `src/relative_meta_logic.rs` and (b) wasm-to-wasm proof compilation.
2. File one upstream issue on `link-foundation/relative-meta-logic` per concrete gap found, each with a minimal repro from Formal AI's actual use case.
3. If RML is usable as-is for either gap, land the integration in this PR instead of only filing an issue.
4. If wasm-to-wasm compilation is technically impossible for RML's design, record that explicitly in `docs/case-studies/issue-{id}` rather than leaving it silently unresolved.

**How to test.**
- Automated: if integration lands, a test asserting `src/relative_meta_logic.rs`'s behavior is now backed by the crate (e.g. via a re-export or delegation call, not a parallel re-implementation); if only filing, a case-study record checked into the repo documenting the filed issue URLs.
- Manual: confirm the filed upstream issue(s) exist and link back to this issue's case study.
- Standing clauses: `docs/case-studies/issue-{id}`; existing-library survey of RML's current API surface; online research on wasm-to-wasm compilation feasibility; single PR.

**Source refs:** #185, #209 (needs-issue R185-1, R209-1). **Dedup:** merged — both are the same class of debt (promised-but-never-filed upstream issue on the same dependency).

---

### E84: Compile links substitution rules to Rust, JavaScript, and WebAssembly (issue #331)

**Body:**

**Problem statement.**
Source: [#331 PR comment](https://github.com/link-assistant/formal-ai/pull/331#issuecomment-4577374430). konard: "When we have substitution rules that are Turing complete we should be able to convert them to Rust/JavaScript/WebAssembly — all options should be available."
Current-code evidence: no substitution-rules-to-Rust/JavaScript/WebAssembly compiler exists anywhere. `src/substitution.rs` is a pure matcher/instantiator; `src/skill_compiler` lowers natural language to associative packages only. No `emit_rust`/`to_js`/wasm-compile path exists in `src`.

**What to do.**
1. Design an IR-to-target-language emitter for Turing-complete substitution rules (`replace x y`, `when n do m` and their compositions).
2. Implement a Rust code-emission backend first (aligns with the JS=glue/all-logic-in-Rust doctrine — this is the canonical target).
3. Implement JS and WASM emission as secondary backends, explicitly for interop/embedding use cases (e.g. shipping a standalone compiled rule as a browser snippet) — not as a parallel logic implementation that violates the JS doctrine.
4. Wire this into the existing rule-synthesis / program-plan handlers so a proven substitution-rule program can be exported in any of the three forms on request.

**How to test.**
- Automated: unit tests compiling a small Turing-complete rule set (e.g. a counter/loop construct) to each of the three targets and executing the emitted code, asserting identical output to the interpreted rule.
- Manual: run `formal-ai` against a substitution-rule prompt, request Rust/JS/WASM export, and execute each emitted artifact outside the process.
- Multilingual: emission target language is independent of prompt language, but confirm the prompt that produces the rule set works in en/ru/hi/zh.
- Standing clauses: `docs/case-studies/issue-{id}`; single PR; verbose trace of the compilation steps if correctness is hard to verify statically.

**Source refs:** #331 (needs-issue R331-3). **Dedup:** none.

---

### E85: Per-conversation detached docker execution containers with snapshot/replay state restoration (issue #331)

**Body:**

**Problem statement.**
Source: [#331 PR comment](https://github.com/link-assistant/formal-ai/pull/331#issuecomment-4577374430). konard: server/telegram execution should use docker via `link-foundation/box`/`link-foundation/start`; support a detached container per conversation, reattachable multiple times, stopped when idle to save CPU/RAM; restore a resumed conversation's container state — zip-archive/snapshot restoration by default, command-replay as a user-selectable fallback in settings; support container snapshots.
Current-code evidence: `src/cli_environments.rs` has no detach/reattach/snapshot logic; `compose.yaml`/`Dockerfile` only start the bot itself, not per-conversation execution containers.

**What to do.**
1. Add a per-conversation container lifecycle manager: spin up a detached `link-foundation/box` container on first execution need, keep a stable handle keyed to the conversation id.
2. Stop (not destroy) the container when idle past a configurable timeout to save CPU/RAM; support reattaching to a still-alive container on the next turn.
3. Implement zip-archive/snapshot-based state restoration as the default resume path when a conversation's container was stopped or evicted; implement command-replay as a fallback, selectable in settings.
4. Add container-snapshot support (save/restore a point-in-time filesystem+process state) as the underlying primitive for the above.
5. This is a natural shared foundation for E78's telegram execution pipeline — sequence accordingly (E85 blocks/feeds E78 if both are picked up together).

**How to test.**
- Automated: integration tests using a lightweight test container image: (a) detach/reattach round-trip preserves filesystem state; (b) idle-stop-then-resume via snapshot restores state; (c) command-replay fallback reproduces equivalent state when snapshot restoration is disabled in settings.
- Manual: run a multi-turn conversation that writes a file in turn 1, let the container idle-stop, resume in turn 2, and confirm the file is still present via both restoration modes.
- Standing clauses: `docs/case-studies/issue-{id}`; survey `link-foundation/box`/`start` for existing snapshot primitives before building bespoke ones; file upstream issues if `start` needs an API extension for detach/reattach; single PR.

**Source refs:** #331 (needs-issue R331-5). **Dedup:** none as a standalone execution-container epic, though it overlaps thematically with E78 (telegram) and R716-2/R331-6 handchecks — cross-link in the case study rather than merging, since this is infra and E78 is the telegram-specific consumer.

---

### E86: Unify coding-task handlers under one shared meta-algorithm-builder module (issues #412/#413)

**Body:**

**Problem statement.**
Source: [#412](https://github.com/link-assistant/formal-ai/issues/412) (issue body) — konard: prefer an ALGORITHM BUILDER over templates, "meta algorithm, building algorithm that builds algorithms", starting with coding tasks, scope is the whole codebase. [#413 comment](https://github.com/link-assistant/formal-ai/pull/413#issuecomment-4681963053) — konard reiterated the repo-wide mandate applies beyond the specific fix, and required nothing be delayed or deferred; the agent's direct question about expanding scope to the full meta-builder was **never answered in-thread** before merge.
Current-code evidence: `src/dreaming.rs`/`dreaming_application.rs` implement a meta-algorithm-amendment mechanism; `src/solver_handlers/installation_conversion.rs:845-936` has a concrete `meta_algorithm`/`algorithm_construction` trace (`render_meta_algorithm`, pinned by `tests/unit/installation_conversion.rs:179`); `meta-language` crate (`Cargo.toml:53`) is wired for CST/AST manipulation. But no other coding handler (`program_synthesis.rs`, `coding_catalog.rs`, `rule_synthesis.rs`, numeric-list handling) imports or shares this trace/module — it is one handler-family's local meta-builder, not a repo-wide unifying one. `ROADMAP.md:122` (as of the audited HEAD) still lists a task-agnostic meta-builder as tracked future work.

**What to do.**
1. Extract the `installation_conversion.rs` meta_algorithm/algorithm_construction mechanism into a shared module usable by any coding handler.
2. Migrate `program_synthesis.rs`, `coding_catalog.rs`, `rule_synthesis.rs`, and the numeric-list handler onto the shared meta-builder, replacing their bespoke construction logic.
3. Explicitly answer, in this issue's case study, konard's unresolved #413 question (expand-in-PR vs land-incrementally) with a documented decision and rationale, closing the loop that was left open since #413 merged.
4. Update `ROADMAP.md`'s meta-builder tracking line to reflect the unification once landed, or to state clearly what remains if only partially unified.

**How to test.**
- Automated: existing `installation_conversion.rs` tests stay green; new tests assert `program_synthesis`/`coding_catalog`/`rule_synthesis` share the same meta-algorithm trace module (e.g. via a shared-symbol usage test, not just behavioral parity).
- Manual: run one coding prompt through each of the four handler families and confirm each answer's trace shows the shared meta-algorithm construction steps in the same shape.
- Multilingual: re-run the coding-prompt suite in en/ru/hi/zh and confirm unification doesn't regress any language.
- Standing clauses: `docs/case-studies/issue-{id}` (timeline of #412/#413/#423/#424/#433/#448 recurrence, full requirement list, per-handler migration plan); single PR (or a small sequence of PRs from this epic, each fully addressing one migrated handler family, since konard's "nothing delayed" clause applies retroactively here).

**Source refs:** #412, #413 (needs-issue R412-2, R413-1 — merged, same underlying gap). **Dedup:** none — #412/#413 have no open tracking issue; ROADMAP.md only narrates the gap.

---

### E87: Grow the installation-guide ↔ script conversion corpus to 50+ top-GitHub-project cases (issue #423)

**Body:**

**Problem statement.**
Source: [#423](https://github.com/link-assistant/formal-ai/issues/423) (issue body). konard: "at least 50 test cases for most popular GitHub project ... Even better to take top 50 most popular GitHub projects that have both installation scripts and manual installation guides."
Current-code evidence: bidirectional README↔script conversion is delivered (`src/solver_handlers/installation_conversion.rs`; `tests/unit/installation_conversion.rs` including bash+PowerShell round trips), but `data/seed/projects.lino` holds only ~15 projects — well short of the 50+ acceptance floor.

**What to do.**
1. Identify 50+ of the most popular GitHub projects that have both a README install guide and an install script (sh/PowerShell).
2. Add them to `data/seed/projects.lino` (respecting the 1500-line-per-file / 128-record conventions from E88 below if that lands first).
3. Extend the regression corpus so each project round-trips through the existing extraction→IR→render pipeline with a held-out-paraphrase style test per project family, not just literal fixture replay.

**How to test.**
- Automated: `tests/unit/installation_conversion.rs` (or a new corpus test) asserts ≥50 projects round-trip README↔script; CI fails if the corpus count drops below 50.
- Manual: spot-check 5 projects' generated scripts against their real upstream install scripts for structural fidelity.
- Standing clauses: `docs/case-studies/issue-{id}`; single PR.

**Source refs:** #423 (needs-issue R423-1), follow-up #424. **Dedup:** none.

---

### E88: Generate downloadable research documents (PDF/DOCX) from research prompts (issue #425)

**Body:**

**Problem statement.**
Source: [#425](https://github.com/link-assistant/formal-ai/issues/425) (issue body, Russian original: "Сделай мне пдф файл со списком стран..."). konard required research + document generation combined — an actual downloadable PDF/DOCX, not a chat-only table. [#432](https://github.com/link-assistant/formal-ai/pull/432) required full document manipulation (TXT/Markdown/PDF/DOCX) through `meta-language`; the agent found meta-language lacked PDF/DOCX/formatting concepts, filed `meta-language#83-86`, and paused per konard's own instruction — the PR nevertheless merged with only a localized formal-plan increment.
Current-code evidence: `src/document_formats.rs` only recognizes PDF/DOCX profiles; there is no generation path. `research_table` answers stay chat-only.

**What to do.**
1. Check the current status of `link-foundation/meta-language#83-86` (filed from #432) — if the needed PDF/DOCX/formatting primitives have landed upstream, adopt them.
2. If still missing, implement a minimal Rust-native PDF/DOCX writer (or a vetted crate) sufficient for research-table/document output, keeping `meta-language` as the eventual target per doctrine.
3. Wire research prompts that ask for a document artifact (PDF/DOCX/Markdown) end-to-end: research → synthesize → render → downloadable file, across CLI/server/web surfaces.
4. If meta-language's gaps are still blocking, file/update the upstream issues with a concrete current repro before landing a Rust-native stopgap.

**How to test.**
- Automated: an e2e test that a "make me a PDF of X" prompt produces a real, parseable PDF file with the researched content (not a stub).
- Manual: re-run the original #425 Russian prompt and confirm a real PDF downloads with correct content.
- Multilingual: repeat the document-generation prompt in en/ru/hi/zh and confirm each produces a correctly-localized document.
- Standing clauses: `docs/case-studies/issue-{id}` (include the meta-language#83-86 status check); single PR.

**Source refs:** #425 (needs-issue R425-1), #432, `meta-language#83-86`. **Dedup:** none.

---

### E89: Gemini protocol surface — expose the thinking trace as thought parts (issue #608)

**Body:**

**Problem statement.**
Source: [#608](https://github.com/link-assistant/formal-ai/issues/608) (issue body) — "Our reasoning transparency — a core selling point per VISION.md — is invisible to every external client," requiring the thinking trace exposed through each protocol's standard reasoning channel (streaming and non-streaming) for OpenAI Chat, OpenAI Responses, Anthropic, and Gemini.
Current-code evidence: 3 of 4 channels delivered — OpenAI `reasoning_content`+delta (`tests/unit/specification/openai_compatibility.rs:120-137`), Responses reasoning summary events (`src/responses_stream.rs:102,237`), Anthropic thinking blocks+`thinking_delta` (`src/anthropic.rs:66-106`). Gemini is missing: `src/gemini.rs:285-309` emits only `text`/`functionCall` parts, no `thought: true` parts anywhere in `src`.

**What to do.**
1. Add Gemini `thought: true` parts to both streaming and non-streaming response shapes in `src/gemini.rs`, mirroring the existing thinking-trace content used for the other three protocols.
2. Gate verbosity by the existing diagnostic/thinking-level config, matching the other three implementations' behavior.
3. Keep `thinking_steps` as the vendor-extension field alongside the new standard channel.

**How to test.**
- Automated: e2e test per `tests/e2e` protocol-suite convention (mirroring the existing OpenAI/Anthropic/Responses thinking-trace tests) asserting Gemini responses include `thought: true` parts in both streaming and non-streaming modes.
- Manual: call `formal-ai serve`'s Gemini-compatible endpoint with a real Gemini client library and confirm thinking content renders.
- Standing clauses: `docs/case-studies/issue-{id}`; single PR.

**Source refs:** #608 (needs-issue R608-1), follow-up #613. **Dedup:** none — the other 3 channels are already done; this is the missing leg only.

---

### E90: Add an automated redaction skill/handler for issue-report publishing (issue #771 item d)

**Body:**

**Problem statement.**
Source: [#771 comment](https://github.com/link-assistant/formal-ai/issues/771#issuecomment-5012134308). konard required (among other report-flow fixes, items a-c of which are delivered): "Formal AI should also do his best to redact all personal data" via a dedicated skill/handler that reasons through redacting personal/sensitive data before publishing a report — assume unpublished personal info must be redacted, public-figure knowledge is exempt.
Current-code evidence: report format rebuilt (#822/#839 suites), gist upload delivered (`cli_report.rs:82-101`, secret by default), confirmation flow before filing (`src/agentic_coding/report_issue.rs`) — but no dedicated redaction skill/handler exists; only manual "redact it" advice appears in docs.

**What to do.**
1. Build a redaction handler that runs over report content before publishing: reasons about candidate personal/sensitive spans (names, emails, addresses, credentials, private conversation content) and redacts them, using the public-figure exemption from the requirement.
2. Integrate it into the report-issue flow (both GitHub-issue and gist targets) as a mandatory pre-publish pass, not optional.
3. Make redaction decisions visible/auditable (e.g. a diff or a list of redacted spans) so the user confirmation step (already delivered) is meaningful.

**How to test.**
- Automated: unit tests feeding synthetic report content containing planted PII (email, phone, name adjacent to private context) and public-figure mentions, asserting the former are redacted and the latter are preserved.
- Manual: trigger a real report flow on a conversation containing a fabricated personal detail and confirm it's redacted before the confirmation prompt shows the final content.
- Multilingual: test redaction on en/ru/hi/zh report content, since PII patterns differ across languages/scripts.
- Standing clauses: `docs/case-studies/issue-{id}`; existing-library survey (PII-detection crates/models); single PR.

**Source refs:** #771 item (d) (needs-issue R771-2). **Dedup:** none — items (a)-(c) already delivered, this is the residual gap only.

---

### E91: Self-coding harness creates GitHub issues despite explicit no-issue-creation instructions (issue #790)

**Body:**

**Problem statement.**
Source: [#790 comment](https://github.com/link-assistant/formal-ai/issues/790#issuecomment-5013825362). konard: issues #784, #786, #789, #790, #791 were "created accidentally by an automated self-coding retry for #762 despite explicit instructions not to create issues" — a routing/permission defect.
Current-code evidence: no guard/permission artifact preventing harness-created issues exists; the pattern recurred across all five listed issues with no fix issue referenced.

**What to do.**
1. Locate the self-coding retry path that creates GitHub issues (likely in the agentic-coding harness's failure-recovery logic) and identify why an explicit "do not create issues" instruction is not honored.
2. Add a hard guard: issue-creation calls check an explicit deny-flag/instruction context before firing, defaulting to deny when the caller's instructions include a no-issue-creation clause.
3. If the defect traces to the upstream self-coding harness (hive-mind or similar) rather than formal-ai's own code, file the issue upstream with a repro instead of/in addition to a local guard.

**How to test.**
- Automated: a regression test that runs the self-coding retry path with an explicit "do not create issues" instruction and asserts zero `gh issue create` (or equivalent API) calls occur, using a mocked GitHub client.
- Manual: reproduce the original #762-retry scenario in a sandboxed/dry-run mode and confirm no issue is created.
- Standing clauses: `docs/case-studies/issue-{id}` (timeline of #784/#786/#789/#790/#791 recurrence); file upstream if the harness itself is at fault; add verbose logging of why an issue-creation attempt was allowed/denied; single PR.

**Source refs:** #790 (needs-issue R790-1), recurring on #784/#786/#789/#791. **Dedup:** none.

---

### E92: Task ladder — add mutating-action rungs (824.L1-L4) with sandbox-reset semantics (issue #824)

**Body:**

**Problem statement.**
Source: [#824](https://github.com/link-assistant/formal-ai/issues/824) (issue body) — "Move <dir> to <dir>" filesystem requests must be performed (verify pre-conditions, idempotent `mkdir -p` + `mv`, verify post-conditions, confirm), not refused, matching the laguna-s-2.1-free reference behavior. Comment 5073614623 explicitly records this is **not** covered by #840/#842 and that mutating-action ladder rungs 824.L1-L4 with sandbox-reset semantics were deliberately deferred — no follow-up issue was ever filed.
Current-code evidence: no mutating-action intent exists; the deferral is recorded but unfollowed-up.

**What to do.**
1. Design ladder rungs 824.L1-L4: progressively more complex mutating filesystem actions (single move, move-with-conflict, multi-step move+cleanup, move requiring created intermediate directories), each with defined pre/post verification.
2. Implement sandbox-reset semantics so each rung starts from a clean, known filesystem state (needed for deterministic ladder scoring).
3. Implement the actual "move X to Y" capability: verify source exists, verify/create destination parent (`mkdir -p`), perform the move, verify the result, and confirm to the user — matching the cited reference behavior instead of refusing.

**How to test.**
- Automated: ladder tests 824.L1 through L4 in the existing task-ladder test style, each asserting pre/post filesystem state via sandbox reset.
- Manual: re-run the original #824 "move .../hive-control-center to ~/Code/Archive/link-assistant" prompt and confirm successful move with pre/post verification output, matching the expected Links Notation log format from the issue.
- Standing clauses: `docs/case-studies/issue-{id}`; single PR.

**Source refs:** #824 (needs-issue R824-1), comment 5073614623. **Dedup:** tracked_by #824 already (existing open issue), but #824 itself does not implement the ladder rungs — file this as the concrete follow-up the comment promised but never got; cross-link to #824 rather than treating as covered-by-existing.

---

### E93: Report export writes session `.lino` files into the caller's CWD instead of a scratch/session directory (issues #838/#838-family)

**Body:**

**Problem statement.**
Source: arch-review 7.1 (MEDIUM) + needs-issue R838-1 ([#838](https://github.com/link-assistant/formal-ai/issues/838), issue body) — the report flow must never produce an unusable artifact; a live defect was found during this audit: `ReportTarget::HarnessLog`/`ServerLog` build `formal-ai context export --output formal-ai-{harness,server}-<id>.lino` with a **bare relative path** (`src/agentic_coding/report_issue.rs:367-379`), so runs from a repo checkout drop session dumps at the repo root. Two such files are sitting untracked in this repo right now: `formal-ai-harness-latest.lino` (311 lines) and `formal-ai-server-latest.lino` (157 lines, base64-inlined message bodies including a full third-party OpenCode system prompt).
The `GithubIssue` target already does this correctly via a mktemp scratch dir (`report_script.rs:29-32`); two of four report destinations pollute the CWD, two don't.

**What to do.**
1. Make `HarnessLog`/`ServerLog` (and any other CWD-relative targets) write into the same scratch/session directory pattern already used by `GithubIssue`.
2. Add root-anchored `.gitignore` rules (`/formal-ai-harness-*.lino`, `/formal-ai-server-*.lino`) mirroring the existing `.log` policy at `.gitignore:66-77` (ignore globally, un-ignore under `docs/case-studies/**`), as a defense-in-depth measure even after the code fix.
3. Resolve the two stray files currently in the repo root: either file them under `docs/case-studies/issue-838/` after running `scripts/check-secrets.sh` (the server capture embeds a third-party system prompt — verify no genuine secrets), or delete them if not evidentially useful.

**How to test.**
- Automated: a test asserting no `ReportTarget` variant's output path is CWD-relative (e.g. a path-shape assertion across all four targets).
- Manual: run the report flow with CWD set to the repo root and confirm no new files appear at the repo root afterward.
- Standing clauses: `docs/case-studies/issue-{id}`; single PR; run `scripts/check-secrets.sh` on the two stray files before any commit decision.

**Source refs:** #838 (needs-issue R838-1), arch-review 7.1. **Dedup:** tracked_by #838 (existing open issue) but #838 itself doesn't cover this specific export-path bug — file as the concrete follow-up; cross-link to #838.

---

### E94: Versioned recoverable memory — snapshot/rollback with immutable baseline tests (issue #873)

**Body:**

**Problem statement.**
Source: [#873](https://github.com/link-assistant/formal-ai/issues/873) (issue body) — "each state of memory should be recoverable, so if compilation of next version of itself fails for formal AI debugging continues from previous stable and tested version"; most tests immutable as a baseline; never switch to a version that does not pass all tests.
Current-code evidence: no snapshot/rollback/revision mechanism exists in `src/memory*`, `memory_sync.rs`, `associative_persistence.rs`, or elsewhere per the #914 audit.

**What to do.**
1. Add versioned memory snapshots: each self-coding compile attempt records the prior stable memory state before mutation.
2. On compile failure of the next self-authored version, automatically roll debugging back to the last stable, fully-tested version rather than continuing from a broken state.
3. Mark a designated baseline test set as immutable (never weakened to make a failing version pass); gate version-switch on 100% baseline pass.
4. Integrate with the existing gated-promotion protocol (`src/promotion.rs`, #656) as the storage/rollback layer underneath it.

**How to test.**
- Automated: a test that simulates a failing self-compiled version and asserts the system automatically reverts to the prior recorded snapshot; a test asserting baseline tests cannot be edited/weakened as part of an adoption commit (e.g. a hash-pin on the baseline test file set).
- Manual: force a broken self-authored version through the promotion pipeline and confirm rollback occurs with no manual intervention.
- Standing clauses: `docs/case-studies/issue-{id}`; single PR.

**Source refs:** #873 (needs-issue R873-2). **Dedup:** none — related to but distinct from #656 (gated promotion, already closed) and #705/#887 (anticipatory dreaming); this is the rollback/versioning layer specifically.

---

### E95: Bounded autonomy — configurable stuck-recovery time limit, full-trust mode, per-command permission mode (issue #873)

**Body:**

**Problem statement.**
Source: [#873](https://github.com/link-assistant/formal-ai/issues/873) (issue body) — fully automate recovery from any error: on multiple resolution options, ask the user (driving seat) unless full-trust mode is configured (auto-select by weighted advantages/disadvantages); "it should be impossible for the system to get stuck and fail," bounded by a default 1-hour (configurable) limit after which the current plan is presented and permission to continue is requested; support both full-autonomous and per-command-permission modes.
Current-code evidence: no stuck-recovery time limit, full-trust mode, or per-command permission mode exists anywhere in `src`.

**What to do.**
1. Add a configurable stuck-recovery time limit (default 1 hour) to the agentic execution loop: when exceeded, halt, present the current plan, and request permission to continue rather than looping indefinitely or silently failing.
2. Add a "full-trust" mode: on multiple viable resolution options, auto-select by a weighted advantages/disadvantages heuristic instead of asking the user.
3. Add a per-command permission mode as the alternative to full-autonomous mode, letting the user gate each command.
4. Ensure the two modes (full-autonomous, per-command-permission) are both selectable in configuration, with full-trust as an explicit opt-in per konard's opt-in doctrine.

**How to test.**
- Automated: a test harness that forces a pathological "stuck" scenario (unresolvable loop) and asserts the 1-hour (test-scaled) limit triggers a plan-presentation + permission-request instead of indefinite looping; tests for both full-trust auto-select and per-command permission gating.
- Manual: run a genuinely ambiguous multi-option task under full-trust mode and confirm auto-selection with recorded rationale; run the same under per-command mode and confirm each command is gated.
- Standing clauses: `docs/case-studies/issue-{id}`; single PR.

**Source refs:** #873 (needs-issue R873-3). **Dedup:** none — distinct requirement from E94 within the same source issue (memory versioning vs. execution-loop bounding).

---

### E96: Memoized-answer-surface burndown — canned summaries, kupi_slona-style idiom handlers, identity/greeting duplication

**Body:**

**Problem statement.**
Source: arch-review 1.1 (HIGH), 1.2 (HIGH), 1.3 (HIGH), 3.3 (HIGH). Doctrine: VISION.md ("prefer deep understanding ... over answer memoization"), NON-GOALS.md ("a memoized answer cache is not a substitute for reasoning from source data").
Current-code evidence:
- `data/seed/summary-topics.lino:9-18` + `src/solver_handlers/benchmark_prompts.rs:32-84` (`try_summarization_request`, `try_brainstorming_request`) return seeded English paragraphs verbatim for exactly 3 topics; a Russian "резюме Rust" triggers but returns the English canned body.
- `src/solver_handlers/research_table.rs:416,432,448` — three `contains("machine learning algorithm"|"deep learning"+"traditional ml"|"neural network")` blocks return hardcoded English prose comparison cells; column labels (`:26-33`) are hardcoded English Rust consts.
- `src/solver_dispatch.rs:343` (`kupi_slona`) + `src/solver_handlers_policy.rs:39,75-80` — per-idiom Rust handlers each pairing one seed trigger-role with one seed response-key plus a hardcoded in-code NL fallback string, on the hardcoded-language lint's allowlist.
- `data/seed/identity.lino:1-15` + `data/seed/greetings.lino` — 19 prompt→answer pairs with byte-identical `answer` strings duplicated up to 3×, bypassing the existing `response_link` indirection mechanism.

**What to do.**
1. Replace `try_summarization_request`/`try_brainstorming_request` with derivation from cached Wikidata/Wikipedia concept/meaning links, rendered in the prompt's language; demote seed `body` fields to test fixtures.
2. Delete the three `contains(...)` topic blocks in `research_table.rs`; compose comparison content from cached source records with provenance; localize column labels through the seed lexicon.
3. Collapse `kupi_slona` and sibling policy handlers (`physical_action_question`, `shell_refusal`, `punctuation_only_prompt`, etc.) into one seed-role-driven `policy_response` handler walking a `(trigger_role, response_key)` table; delete the hardcoded Rust NL fallback strings.
4. Replace `identity.lino`/`greetings.lino` inline `answer` fields with links to single multilingual response records via the existing `response_link` mechanism, eliminating duplicated literals.

**How to test.**
- Automated: (a) regression test asserting summary/brainstorm answer text is never byte-equal to any seed `body`; held-out-topic test passes the same shape as pinned topics; (b) held-out research-comparison topic (e.g. "compare sorting algorithms") produces the same answer shape as the three previously-pinned ones; (c) one `policy_response` handler passes all existing per-idiom tests unchanged, and `check-hardcoded-language` shows no new allowlist entries; (d) a seed lint failing on byte-identical duplicate `answer` values >1.
- Manual: ask for a summary of a topic not in the seed (e.g. "summarize Python") in en/ru/hi/zh and confirm a derived (non-canned) answer each time.
- Multilingual: repeat 3.2-adjacent checks — "резюме Rust" must return a Russian-derived summary, not the English canned body.
- Standing clauses: `docs/case-studies/issue-{id}`; single PR (may be split into per-family sub-PRs if the epic is broken into rungs, but every requirement above must land before this epic closes).

**Source refs:** arch-review 1.1, 1.2, 1.3, 3.3. **Dedup:** checked against #922 (E75, "method learning for the universal problem-solving algorithm from experience") — #922 is about learning generalized method abstractions from event-log traces, not about removing these specific seeded canned-answer paths; no overlap, this is a new issue.

---

### E97: Close the en/ru/hi/zh parity gap in responses, prompt patterns, and greetings; add a parity lint; fix hi/zh word-operator arithmetic

**Body:**

**Problem statement.**
Source: arch-review 3.2 (HIGH) + test-report.md "broken things" item 2 (MEDIUM, live-verified during this audit). Doctrine: "multilingual en/ru/hi/zh by construction" is a hard standing requirement.
Current-code evidence:
- 55 of 261 intents across `multilingual-responses*.lino` are English-only (`external_benchmark_*` 25 intents, `statement_audit_*` 12, `algorithm_*` 5).
- `prompt-patterns.lino` totals en=61/ru=69/**hi=36**/zh=43; `pattern_concept_prefix` has 24 en/23 ru/7 zh/**0 hi**.
- `greetings.lino` `greeting`/`farewell`/`courtesy_response` have **zero** hi and zh surfaces (`:53-83`) — a Hindi "नमस्ते" has no greeting surface at all in that family (though this audit's live check found the top-level greeting handler does answer "नमस्ते" correctly via a different path — verify and reconcile in the case study).
- `data/cache/wiktionary/` and `data/cache/wordnet/` contain only `en/` (2,002 files, monolingual grounding cache).
- **Live-verified regression:** `2 जमा 2 कितना होता है?` / `2 जोड़ 2 कितना होता है?` (Hindi) and `2 加 2 等于多少?` (Chinese) both fall to the unknown handler, while `2 plus 2` / `2 плюс 2` succeed. Symbolic `2 + 2` works in all languages. `data/seed/operation-vocabulary.lino` has `相加` (zh) but not the `加`/`जोड़`/`जमा` infix forms recognized.

**What to do.**
1. Add a CI lint: for every intent in `multilingual-responses*.lino`, require all four language variants or an explicit waiver record (mirroring `data/overrides/`'s reason-required pattern); backfill the 55 English-only intents.
2. Backfill hi/zh greeting/farewell/courtesy surfaces in `greetings.lino`.
3. Add the missing Hindi/Chinese infix word-operator forms (`जोड़`, `जमा`, `加`) to `operation-vocabulary.lino` so word-operator arithmetic succeeds identically across en/ru/hi/zh (fixes the live-verified regression).
4. Bring `prompt-patterns.lino` hi coverage toward parity (target: counts per language within ±20% per intent).

**How to test.**
- Automated: the new parity lint (fails on any intent lacking a language or waiver); a regression test pinning `2 जोड़ 2`, `2 जमा 2`, `2 加 2` to the correct numeric answer (matching the existing `2 plus 2`/`2 плюс 2` test pattern).
- Manual: ask "नमस्ते" and confirm a Hindi-language greeting; ask each of the four arithmetic phrasings live and confirm identical correct answers.
- Multilingual: this issue *is* the multilingual check — completion criterion is literally en/ru/hi/zh parity across the named files.
- Standing clauses: `docs/case-studies/issue-{id}`; single PR.

**Source refs:** arch-review 3.2, test-report.md "broken things" #2. **Dedup:** none — distinct from E96 (canned-answer removal) and E82 (JS worker); this is seed-data language-coverage specifically.

---

### E98: Rename Graph* public types to link-network vocabulary; widen the terminology lint to identifiers and emitted tokens

**Body:**

**Problem statement.**
Source: arch-review 6.1 (HIGH). Doctrine: associative-only terminology — never "graph"/edges/vertices — stated in REQUIREMENTS.md:1226,1410 and in `data/meta/links-network-terminology-recipe.lino` (#664, "keeping every public surface a links network, not a graph").
Current-code evidence: `scripts/check-associative-terminology.rs` checks only the literal word "graph" in `/v1/`-prefixed routes and module/file names (`:38,82,211-255`); its own doc comment concedes it never reaches internal identifiers. Public API: `src/engine.rs:294-299` `pub struct KnowledgeGraph { pub nodes: Vec<GraphNode>, pub edges: Vec<GraphEdge> }`, 86 references across 12 files. `src/links_query.rs:131,161` emits the literal token `edge` into Links Notation output and uses edge/node vocabulary in user-facing errors (`:274`). The `/v1/graph` alias name leaked into the `graphUrl` client field across `desktop/main.cjs:157,319`, `desktop/lib/local-server.cjs:179`, `vscode/src/lib/config.cjs:82,112`, `vscode/src/extension.node.cjs:356-357`, `src/web/app/main.jsx:4973,5249,9202`. Seed data ships graph vocabulary as user-facing answers (`data/seed/concepts.lino:62,68,80`, Hindi variant keeps English terms untranslated). Vector-`embeddings` appears in user-facing answer text (`research_table.rs:454`, mirrored in `formal_ai_worker_16.js:571`).

**What to do.**
1. Rename `KnowledgeGraph`/`GraphNode`/`GraphEdge`/`SubstitutionGraph` to link-network vocabulary (e.g. `LinkNetwork`/`NetworkLink`) across all 86 references in 12 files.
2. Rename the `graphUrl` client field to `networkUrl` (with a deprecation shim) across desktop/vscode/web configs (5 files).
3. Change `links_query.rs`'s emitted Links Notation token from `edge` to link vocabulary, and update its user-facing error strings.
4. Re-gloss `data/seed/concepts.lino`'s "graph ... vertices and edges" entries through the meaning layer — a "graph" concept may legitimately *describe* graph theory as a subject, but the assistant's own structures must never be described with it; fix the untranslated Hindi variant.
5. Replace "embeddings" in `research_table.rs:454` (and its JS mirror) with associative-technology-correct phrasing.
6. Extend `check-associative-terminology.rs` to scan identifiers, struct/field names, and emitted output tokens for `graph|edge|vertex|embedding`, with a burn-down allowlist modeled on R379's `hardcoded-language-allowlist.txt`.

**How to test.**
- Automated: zero `Graph*`-named public types (grep-based CI check); the widened lint runs in CI with a shrinking allowlist; a test asserting `links_query.rs` output contains no `edge` token; a test asserting `graphUrl` is absent from desktop/vscode/web configs (deprecation shim aside).
- Manual: exercise the deprecated `/v1/graph` alias and confirm it still works with its documented deprecation path only.
- Multilingual: verify the Hindi `concepts.lino` entry no longer leaves English "graph/vertices/edges" terms untranslated.
- Standing clauses: `docs/case-studies/issue-{id}`; single PR (large rename — acceptable as one PR per convention, but may need staged commits within it).

**Source refs:** arch-review 6.1. **Dedup:** none.

---

### E99: main.jsx decomposition — move the 200+ logic functions to WASM/worker calls, keep only components and event wiring

**Body:**

**Problem statement.**
Source: arch-review 8.2 (HIGH). Doctrine: JS = interfacing glue + JSX UI only; all logic in compiled Rust.
Current-code evidence: `src/web/app/main.jsx` is 9,269 lines; 238 top-level lowercase `function`s vs. 15 React components; first component at `:1987` (the preceding ~2,000 lines are pure logic); only ~91 of 9,269 lines contain JSX. Logic living here includes issue-report body construction (`createIssueReportBody`, cited from REQUIREMENTS.md R115 — a documented dual implementation with `src/issue_report.rs`), URL fitting (`fitIssueUrl`), desktop status normalization, memory-bundle handling, evidence-slug construction (`:5249`).

**What to do.**
1. Inventory all 238 top-level functions into: (a) already-in-Rust duplicates (e.g. issue-report rendering vs. `src/issue_report.rs`) — replace call sites with WASM/worker calls, delete the JS copy; (b) UI-adjacent formatting that genuinely touches the DOM — keep; (c) new logic with no Rust home — port to Rust/WASM.
2. Enforce a `main.jsx` line budget (components + wiring only) via `check-file-size.rs` or a dedicated web lint, so this doesn't regress.
3. Retain the existing byte-pinned issue-report-body test but point it at the single (Rust) implementation.

**How to test.**
- Automated: `main.jsx` line count < 2,500 asserted in CI; a lint asserting no new lowercase top-level logic functions land in `app/`; the existing issue-report-body byte-pinned test passes against the Rust-only implementation.
- Manual: exercise the web demo's issue-report flow end-to-end and confirm identical output to before the refactor.
- Standing clauses: `docs/case-studies/issue-{id}`; single PR (may be staged by function-category as sub-commits within it).

**Source refs:** arch-review 8.2. **Dedup:** none. **Depends on:** should land after or alongside E82 (JS worker absorption) since both touch the WASM-call surface — sequence via blocked_by if convenient, not required.

---

### E100: Route browser seed loading through the WASM seed parser; delete the JS parser; enforce seed-manifest parity

**Body:**

**Problem statement.**
Source: arch-review 3.1 (HIGH) + 8.3 (HIGH). Doctrine: "data/seed/ is the canonical knowledge surface for every interface" (VISION.md:213); JS = glue only.
Current-code evidence: `src/web/seed_loader.js` (1,151 lines) hand-parses `.lino` seed files in JS while `src/web/wasm-worker/src/lib.rs:28-30` (`#[path = "../../../seed/parser.rs"] mod seed_parser;`) already compiles the identical parser into the shipped `.wasm`. Separately, the browser loads only 88 of the 117 seed files `src/seed/embedded.rs:433-444` embeds — 29 missing files include all four `meanings-lexicon-import-0*.lino` (832 lexemes dropped from the web runtime), `multilingual-responses-summarization.lino`, `question-generation-lexicon.lino`, `sources-registry.lino`, `model-aliases.lino`, `computer-use-tasks.lino`. Nothing enforces the JS file list against the Rust list.

**What to do.**
1. Replace `seed_loader.js`'s hand-written parser with fetch → WASM-parse calls into the already-shipped `seed_parser` module; delete the JS parsing logic.
2. Drive the fetched-file list from a single shared manifest (e.g. `data/seed/manifest.lino`) consumed by both `src/seed/embedded.rs` and `seed_loader.js`, or add a CI check diffing the two lists and failing on undeclared divergence, with an explicit per-file opt-out + reason for genuinely native-only seeds.
3. Load the 29 currently-missing files in the browser once the manifest/parity mechanism is in place (unless deliberately excluded with a reason).

**How to test.**
- Automated: `seed_loader.js` reduced to <150 lines (fetch + version glue only) — CI-checked; an e2e assertion that browser seed-category counts equal Rust `seed_files()` counts; CI fails on any silent file-list divergence.
- Manual: load the web demo and confirm a prompt that depends on one of the 29 previously-missing files (e.g. a lexeme only in a `meanings-lexicon-import-0*.lino` shard) now resolves correctly in the browser.
- Standing clauses: `docs/case-studies/issue-{id}`; single PR.

**Source refs:** arch-review 3.1, 8.3. **Dedup:** none.

---

### E101: Move desktop tool-router permission/path-confinement logic into the Rust core; stop committing minified web bundles

**Body:**

**Problem statement.**
Source: arch-review 8.4 (MEDIUM, two findings bundled — same "unverified-JS surface" theme). Doctrine: JS = glue only; security-relevant logic belongs in the Rust-owned permission model.
Current-code evidence:
- `desktop/lib/tool-router.cjs` (893 lines) implements "permission-gated tool dispatch for the desktop main process" with mutable grant state (`:165`) and hand-rolled path confinement (`:197-198` `path.relative(...)` + `startsWith("..")`, `:215-218`) — duplicating the confinement/permission concern `src/computer_use/` and the Rust permission model already own, in the one language the project's lint/test doctrine does not cover. A path-traversal bug here is a sandbox escape on the desktop surface.
- `package.json:6`'s `build:web` emits `src/web/app.js`, `vendor.bundle.js`, `web-search-component.bundle.js`, `ocr.bundle.js` — all committed to git as minified, undiffable IIFEs. The seed mirror (`src/web/seed/`) already solved the analogous problem correctly (gitignored, regenerated by `scripts/sync-seed.sh` in the release workflow) — the bundles should follow the same policy.

**What to do.**
1. Move authorization/path-confinement decisions into the Rust core: the desktop process asks the local `formal-ai` binary (which it already supervises) to authorize/execute tool effects; reduce `tool-router.cjs` to IPC dispatch plumbing.
2. Add adversarial confinement tests (symlinks, `..` normalization, UNC paths on Windows) against the single shared Rust implementation.
3. Gitignore `src/web/app.js` and the three `*.bundle.js` files; make the release/pages workflow run `build:web` before artifact upload (mirroring the seed-sync pattern); add a CI check failing if any bundle file is tracked.

**How to test.**
- Automated: adversarial path-confinement test suite in Rust, run in CI; a CI check that fails if `app.js`/`*.bundle.js` are tracked in git.
- Manual: attempt a symlink/`..`-traversal tool call through the desktop app and confirm denial; run a full desktop build and confirm the release artifact still contains a freshly-built bundle.
- Standing clauses: `docs/case-studies/issue-{id}`; single PR.

**Source refs:** arch-review 8.4. **Dedup:** none.

---

### E102: Reorganize src/ into directory modules; document the full module map in ARCHITECTURE.md with a doc-pin test

**Body:**

**Problem statement.**
Source: arch-review 5.1 (MEDIUM) + 5.4 (MEDIUM). The 1000-line file cap (`scripts/check-file-size.rs:18-24`) is being satisfied by mechanical splitting rather than modularization, and ARCHITECTURE.md documents only ~32% of the module surface.
Current-code evidence: 13 files sit in the 950-1000 line band, four at exactly 996-1000; at least 7 files carry a header admitting they exist only to dodge the line cap (`solver_handlers_policy.rs:1-3`, `calculation_word_problem.rs:2`, `solver_handler_units.rs:2`, `solver_handler_how.rs:2`, `solver_helpers/code.rs:2`, `solver_helpers/mod.rs:665`, `solver.rs:294`). 192 top-level `src/` entries, 426 `.rs` files, 116 `mod` declarations in `lib.rs`. Separately, `src/lib.rs` declares 155 modules (+16 binary-only `cli_*`); **105 of 155 (68%) never appear in ARCHITECTURE.md**, including `mcp`, `proxy`, `client_integrations`, the entire `self_*` family, `memory_query_language`, `draft_portfolio`, `external_benchmarks`.

**What to do.**
1. Group the solver family (`solver*`, `meta_method_dispatch`, `method_registry`), world_model family, dreaming family, and translation family into directory modules with `mod.rs` re-exports, so file boundaries follow responsibility instead of the line-count cap.
2. Remove the file headers citing the line cap as their reason to exist, once the real module boundary makes that framing obsolete.
3. Add a generated "Module map" section to ARCHITECTURE.md (one line per module: name, one-sentence responsibility, owning doc section) covering every `lib.rs` module, and pin it with the existing `docs_requirements` doc-pin test pattern so drift fails CI.

**How to test.**
- Automated: top-level `src/` entry count < 60 (CI-checked); a doc-pin test asserting the ARCHITECTURE.md module list equals `lib.rs`'s mod list — new modules fail CI until documented; `cargo public-api` diff empty or explicitly reviewed for the reorganization.
- Manual: `ls src/` visually matches ARCHITECTURE.md's module map after the change.
- Standing clauses: `docs/case-studies/issue-{id}`; single PR.

**Source refs:** arch-review 5.1, 5.4. **Dedup:** none.

---

### E103: Runtime hand-check suite — carry forward the 49 checks this audit could not verify statically

**Body:**

**Problem statement.**
This audit produced 49 findings (`handchecks.ndjson`) that require live/manual verification — running the deployed demo, a real device, a live GitHub Action, an upstream tracker, or a maintainer decision — which cannot be confirmed by static code review alone. Rather than 49 micro-issues, this epic tracks them as one checklist so they get owned and worked through systematically instead of silently lost.

**What to do.**
Work through the checklist below; for each item, either confirm it live and record the result, or file/link the concrete follow-up issue if it turns out broken, then check it off:

1. R1-14 — GitHub Pages demo is published and e2e-tested both in PRs and against the deployed URL.
2. R8-7 — Telegram bot supports 1:1 private messages and public group chats (live bot check).
3. R108-5 — Mobile UI input-section/top-bar layout bug, real-device keyboard focus behavior.
4. R128-1 — Capital-of-country answers for an uncommon country resolve live via Wikipedia/Wikidata/Wiktionary, not a hardcoded fact table.
5. R133-1 — DuckDuckGo default search engine live availability (was reported flaky in #153).
6. R171-2 — `FRAME_POLICY_CHECK_ENDPOINT` reachable from the deployed GitHub Pages app.
7. R180-5 — Diagnostics mode renders expandable raw HTTP request/response for a live web search.
8. R304-1 — External-benchmark suite pass ratios match the recorded ratchet floors on a live run.
9. R312-1 — Unseen coding prompts end-to-end compared against quoted Gemini/DeepSeek answer quality.
10. R331-6 — No expected-output shown before a real verified execution (cross-ref open #905/#908).
11. R353-1 — VS Code extension loads as a web extension in vscode.dev.
12. R439-2 — `agent --model formalai/formal-ai` against `formal-ai serve`, compared with `claude -p` JSON output shape.
13. R444-3 — Live how-to prompt with network access confirms multi-source guide synthesis.
14. R468-5 — Weekly external-benchmarks workflow: honest passed/total rows, ratchet green.
15. R520-1 — Upstream `agent#271/#272`, `agent-commander#39/#40` closed with shipped features.
16. R534-2 — `link-assistant/hive-mind` shared-sccache-container request tracked upstream.
17. R552-3 — `web-capture#141` status; whether formal-ai consumes the meta-language document model.
18. R620-1 — `with-formal-ai gemini`/`--global` re-verified on a machine with cached Google OAuth (cross-ref #909).
19. R635-2 — Standing single-PR-until-complete clause: process check only, no code artifact.
20. R645-2 — Deep-review comment 4939858814 items spot-checked against `dreaming_runtime` tests.
21. R649-3 — `relative-meta-logic` dependent-statement recalculation: code-call integration status (currently narrative-only, `proof_engine/mod.rs:179`).
22. R651-6 — `gh api graphql` sub-issue listing for #651 confirms all E-epics are linked (this consolidation's own file-issues.sh should help satisfy this for E78-E103).
23. R687-3 — Manual sweep: every web/desktop UI control drivable via natural language; gap log.
24. R671-3 — Streamed-capture depth vs. #841 ambitions in a live e2e matrix run.
25. R702-2 — `WorldModel::new()`/`proof_engine`↔`relative_meta_logic` wiring gap, confirm with konard whether narrative-only is acceptable by current design.
26. R708-1 — Tally the ≥15 required NL-memory-query families.
27. R716-2 — Desktop/telegram execution actually lands in a one-shot container when Docker is present; behavior when absent.
28. R717-1 — release.yml/desktop-release.yml green-badge state vs. the four link-foundation pipeline templates, live Actions check.
29. R730-1 — desktop-release workflow failure status, live Actions check.
30. R736-1 — auto-release/desktop-release/docs-generation badge status, live check.
31. R745-2 — "explain/summarize/translate/fix" out-of-box capability probe across languages.
32. R747-2 — Desktop+VS Code tool-set enumeration vs. the #758 shared list at runtime.
33. R753-1 — grok integration functional check; "grok build" subcommand clarification (unanswered by konard in-range — flag for a direct question).
34. R644-1 — PR #644 open/unmerged state — maintainer closure decision needed (see maintainer actions below).
35. R781-8 — Multi-turn action E2E coverage across opencode/agent/claude/codex.
36. R781-10 — Live agentic session shows tool-call explanation before each call.
37. R800-1 — Re-run the amazon.in Russian product-search prompt against current HEAD.
38. R801-1 — Re-run "Search online for Elon Musk" end-to-end.
39. R819-4 — OpenCode re-render bug: confirm upstream report filed.
40. R821-1 — "Search for Elon Musk" quality comparison against Claude/ChatGPT/Google AI-mode.
41. R826-1 — Re-run "ФБС vs ФБО" + "Зарепорти баг" against current HEAD.
42. R827-1 — Re-run "Что такое фуфломицин?" + anaphora follow-up against current HEAD.
43. R841-2 — `command-stream#175/#180`, `agent-commander#43/#46` upstream status; local PTY code deletable yet?
44. R876-1 — Live multi-subagent orchestration with corrective-feedback resume against a provably-wrong statement.
45. R883-2 — `link-foundation/meta-language` issue tracker filings from the #883 window.
46. R887-1 / R888-1 — Maintainer merge decision on CI-green PRs #887/#888 (see maintainer actions below — not an issue action).
47. R904-1 — Confirm whether PR #926 merged remotely after the audit's clone snapshot.
48. R912-1 — `link-assistant/web-search`/`web-capture` post-#912 upstream filings.
49. Live "agent --task ignored" regression (test-report.md item 7) — folded here for a single re-verification pass alongside the dedicated E104 fix issue below, since both need a live check after E104 lands.

**How to test.**
Each checklist line's "how to test" is the live/manual action described in its one-line summary above; record pass/fail + evidence link per line in `docs/case-studies/issue-{id}/handchecks.md`.

**Standing clauses:** `docs/case-studies/issue-{id}` (the checklist itself, plus results); this epic does not need a single PR delivering all 49 — check items off incrementally, but track completion honestly (unchecked items stay visibly open, no marking done without evidence).

**Source refs:** `handchecks.ndjson` (49 rows). **Dedup:** none — this is new tracking, not covered by any existing open issue as a set (individual items reference existing issues #644/#887/#888/#905/#908/#909/#914 where applicable, called out inline above; those existing issues are NOT duplicated by filing this epic, since this is a verification checklist, not a fix).

---

### E104: `agent --task` custom task text is silently ignored — falls back to the default fairy-tale knowledge base

**Body (bug, no E-number... wait, per convention this is a plain bug):**

**Problem statement.**
Source: test-report.md "Broken things" item 7 (LOW/INFO, but a real correctness defect) + handcheck note under R873/agent tests. Live-verified during this audit: `formal-ai agent --silent --task "Formalize «The cat sat on the mat»..."` exits 0, but the output is the **default** fairy-tale knowledge base (`tale:fisherman-and-fish`, header "Formalized «Сказка о рыбаке и рыбке»") — the custom `--task` text is silently not reflected. Current-code evidence: the agent driver routes to its seeded formalize capability regardless of the supplied `--task`, with no warning that the custom text was discarded.

**What to do.**
1. Locate the agent-driver code path that selects the formalize capability and determine why it ignores `--task` in favor of the seeded default.
2. Fix routing so a supplied `--task` is honored; if for some reason a task cannot be routed to a real handler, fail loudly (non-zero exit or a visible warning) instead of silently substituting the default.
3. Add debug/verbose output logging which task text was received and how it was routed, since the root cause is not yet fully determined from static review alone.

**How to test.**
- Automated: a regression test invoking `agent --silent --task "<custom text>"` and asserting the output reflects the custom text, not the fairy-tale default; a negative test confirming an unroutable task produces a visible error rather than silent substitution.
- Manual: re-run the exact repro from this audit (`formal-ai agent --silent --task "Formalize «The cat sat on the mat»..."`) and confirm the output is about the supplied sentence.
- Multilingual: repeat with en/ru/hi/zh task text.
- Standing clauses: `docs/case-studies/issue-{id}`; add verbose/debug output per the root-cause-undeterminable clause; single PR.

**Source refs:** test-report.md "Broken things" #7. **Dedup:** none.

---

### Codify and enforce recorded conventions (bug family, no E-number)

**Title:** Codify and enforce three recorded-but-unenforced conventions: 128-record cache cap, tests-as-docs exact-answer style, and the "Fixes <url>" PR-linking rule

**Body:**

**Problem statement.**
Source: needs-issue R222-1 ([#222 PR comment](https://github.com/link-assistant/formal-ai/pull/222#issuecomment-4513844358)), R234-2 ([#234 PR comment](https://github.com/link-assistant/formal-ai/pull/234#issuecomment-4528554549)), R234-4 (same thread). Three separate konard requirements each landed as a one-time practice but never got the CI/CONTRIBUTING.md enforcement he explicitly asked for:
1. "we should cache not more than 128 the most frequently used words ... each .lino file cannot be larger than 1500 lines." `.lino ≤ 1500` IS enforced (`scripts/check-file-size.rs` in `release.yml:390` + `tests/unit/data_files.rs`), but `check-file-size.rs:57` **excludes `data/cache/wikidata/`** from the gate, and `MAX_SEED_RECORDS_PER_BUCKET=128` (`src/translation/cache.rs:70`) is a documented constant with **no active enforcement** — `data/cache/wikidata/entity` holds 394 entities, over 3× the cap.
2. "I need more detailed examples so tests are like docs ... we need a test or CI/CD rule that will guarantee it." The style is practiced (`tests/unit/assistant_name.rs`) but no test/CI rule enforces it repo-wide.
3. "Word `Addresses` is not recognized by GitHub as explicit link to the issue... will cause it to automatically close on pull request merge." Applied once (`docs/case-studies/pull-request-234/` exists) but codified nowhere — `CONTRIBUTING.md` and `.github/pull_request_template.md` contain no linking guidance.

**What to do.**
1. Include `data/cache/wikidata/` in the `.lino` line-count gate (or state explicitly, with reason, why it's exempted if there's a real constraint).
2. Add active enforcement of `MAX_SEED_RECORDS_PER_BUCKET=128` — a CI check or test that fails if any cache bucket exceeds 128 records (currently `data/cache/wikidata/entity` fails this at 394).
3. Write a CI script that checks conversational/behavioral test files for the "exact example answer" style (not merely contains/not-contains assertions) — flag or fail on tests using only loose assertions where an exact-answer style is expected.
4. Add a CONTRIBUTING.md / `.github/pull_request_template.md` section codifying: PR descriptions must use GitHub's recognized "Fixes #N" / "Fixes <url>" syntax (never "Addresses"), and PR case studies go to `docs/case-studies/pull-request-{id}`.

**How to test.**
- Automated: (a) a test/CI failure demonstrating `data/cache/wikidata/entity`'s 394-entity violation is caught, then fixed to ≤128 (or bucketed); (b) the new tests-as-docs CI rule fails on a deliberately-loose test fixture and passes on an exact-answer one; (c) a CI check (or PR-template lint) confirming a PR body contains "Fixes #N"/"Fixes <url>", not "Addresses".
- Manual: open a test PR with an "Addresses #1" body and confirm the new check flags it.
- Standing clauses: `docs/case-studies/issue-{id}`; single PR.

**Source refs:** #222 (needs-issue R222-1), #234 (needs-issue R234-2, R234-4). **Dedup:** merged three related enforcement-debt requirements into one issue per the grouping instructions.

---

### macOS CI parity (bug family, no E-number)

**Title:** macOS CI parity — fix four portability bugs found running the full suite locally on macOS

**Body:**

**Problem statement.**
Source: test-report.md "Broken things" #1, #3, #4, #5 (all verified locally on macOS 15.7.7 / Darwin 24.6.0, zsh, system bash 3.2.57 during this audit's full local test run). Four distinct macOS-only portability bugs, none affecting product correctness on Linux CI, but all real:

1. **`desktop/scripts/package-macos-with-retry.sh:18`** — `mktemp ".../formal-ai-macos-package.XXXXXX.log"`: BSD `mktemp` does not substitute `XXXXXX` when a suffix follows it, so it creates the literal file `formal-ai-macos-package.XXXXXX.log`; concurrent invocations then collide with "File exists" and the wrapper aborts via `set -e` before ever invoking `npx`. Causes 3 of 4 `ci_cd::macos_package_retry::*` unit tests to fail under the parallel test runner (`cargo test --test unit macos_package_retry -- --test-threads=1` passes 4/4, proving it's the mktemp bug, not test logic). This script runs on macOS GitHub runners in `desktop-release.yml` and only survives there because a single sequential run creates the literal file once — the intended tempfile randomness is silently absent on the very OS the script targets.
2. **`tests/issue_757_session_files.rs:166`** — asserts against the un-canonicalized `FORMAL_AI_PROXY_LOG` path, but `src/client_integrations.rs:486-489` canonicalizes it via `fs::canonicalize`; on macOS `std::env::temp_dir()` (`/var/folders/...`) canonicalizes to `/private/var/folders/...`, so the printed `server log:` line never matches the raw expected path. Manual repro with an already-canonical path prints correctly. Test bug, not product bug.
3. **`tests/integration/issue_819_tui_isolation.rs` and `tests/integration/with_formal_ai.rs`** — fake a PTY with `Command::new("script").args(["-qfec", ...])` (util-linux syntax); BSD `script` has no `-f`/`-e`/`-c` combination, so both tests always fail on macOS. Product code never spawns `script` (verified by grep over `src/`) — test-only issue.
4. **`scripts/sync-seed.sh --check`** — dies with `dests[@]: unbound variable` on macOS's stock bash 3.2 at line 62 (empty-array expansion under `set -u`, fixed in bash 4.4+); the check still exits 1 correctly, but the orphan-detection pass never actually runs on macOS.

**What to do.**
1. Fix the mktemp call in `package-macos-with-retry.sh` to be BSD-portable (put `XXXXXX` at the very end with no suffix after it, or use a portable pattern like `mktemp -t formal-ai-macos-package` + separate suffix handling).
2. Fix `issue_757_session_files.rs:166` to canonicalize its expected path the same way the product does before comparing (or compare canonicalized-to-canonicalized).
3. Fix the two PTY-faking tests to use a portable PTY-spawn approach (e.g. conditionally use BSD `script -q <file> <command>` syntax on macOS, or replace with a Rust PTY crate already used elsewhere in the test suite if one exists).
4. Fix `sync-seed.sh`'s `dests[@]` expansion to be safe under `set -u` with an empty array (`"${dests[@]-}"` or an explicit length check) so it works on bash 3.2.

**How to test.**
- Automated: all four fixes are directly testable by re-running the affected tests/scripts on macOS: `cargo test --test unit macos_package_retry` (parallel, default threading) passes 4/4; `cargo test --test issue_757_session_files` passes on macOS; the two PTY tests pass on macOS; `scripts/sync-seed.sh --check` runs to completion (not crash) on macOS bash 3.2. Add a CI leg (if not already present) that runs the test suite on a macOS runner to catch regressions.
- Manual: run `cargo test --workspace --no-fail-fast` on a real macOS machine post-fix and confirm the previously-6-failing set is down to 0 (accounting for genuinely Linux-only tests staying skipped/ignored on macOS by design, if any).
- Standing clauses: `docs/case-studies/issue-{id}`; single PR covering all four itemized fixes.

**Source refs:** test-report.md "Broken things" #1, #3, #4, #5. **Dedup:** none — grouped per the audit brief's explicit instruction ("ONE macOS CI parity issue with four itemized fixes").

---

### hi/zh word-operator arithmetic parity (bug, no E-number)

**Title:** Hindi/Chinese word-operator arithmetic falls to the unknown handler while English/Russian succeed

**Body:**

**Problem statement.**
Source: test-report.md "Broken things" #2 (MEDIUM, live-verified). Doctrine: "every operation is recognized equally across en | ru | hi | zh" (README/USER-JOURNEYS claim).
Current-code evidence: `What is 2 plus 2?` → `2 plus 2 = 4`; `Сколько будет 2 плюс 2?` → `2 плюс 2 = 4`; but `2 जमा 2 कितना होता है?` / `2 जोड़ 2 कितना होता है?` (Hindi) and `2 加 2 等于多少?` (Chinese) both fall to the unknown handler ("could not determine ... Report issue"). Symbolic `2 + 2` works in all four languages. `data/seed/operation-vocabulary.lino` has the Chinese synonym `相加` recognized but not the infix forms `加`/`जोड़`/`जमा`.

**What to do.**
1. Add the missing Hindi infix operator words (`जोड़`, `जमा`) and Chinese infix operator word (`加`) to `data/seed/operation-vocabulary.lino`'s recognized forms, alongside the already-present `相加`.
2. Audit the full operation-vocabulary table for other infix-vs-standalone-synonym gaps across all four languages while this file is open (holistic pass, not just the two reported words), consistent with the standing "generalization over one-off patching" doctrine.

**How to test.**
- Automated: regression tests pinning `2 जोड़ 2 कितना होता है?`, `2 जमा 2 कितना होता है?`, and `2 加 2 等于多少?` to the correct `= 4` answer, matching the existing `2 plus 2`/`2 плюс 2` test pattern in style (tests-as-docs).
- Manual: re-run all three failing prompts live and confirm correct arithmetic answers.
- Multilingual: this issue IS the multilingual fix; also spot-check other operators (minus/times/divide) in hi/zh for the same infix-word gap while fixing.
- Standing clauses: `docs/case-studies/issue-{id}`; single PR.

**Source refs:** test-report.md "Broken things" #2. **Dedup:** folded the doctrine framing into E97 (parity gap epic) is possible, but this is filed standalone per the audit brief's explicit instruction ("hi/zh arithmetic parity is its own issue (doctrine)").

---

### `.lino` CWD export bug — already filed as part of E93 above

Per the audit brief's instruction ("the .lino CWD export bug is its own issue"), this is delivered as **E93** above (report-flow export path bug), since the underlying defect, its evidence, and its fix are a single coherent unit with the #838 report-flow investigation. Listed here only to confirm it is not missing from the manifest.

---

### `agent --task` ignored — already filed above as E104

Per the audit brief's instruction ("agent --task ignored is its own issue"), this is delivered as **E104** above. Listed here only to confirm it is not missing from the manifest.

---

### E105: Traceability protocol — CI-enforced delivered-version + automated-test + manual-confirmation columns on every REQUIREMENTS.md row

**Body:**

**Problem statement.**
Source: konard's standing requirement, stated directly 2026-08-04 (recorded in project-conventions memory): "every requirement tracked in the repository must have a markdown table row recording (1) when it was delivered in code (version/date/commit), (2) when it was actually tested — BOTH the automated test reference AND a manual test confirmation. Honest 'not yet confirmed' entries are expected where no record exists." This is a new standing requirement as of today's date and is not yet reflected in `REQUIREMENTS.md`'s current column structure (which records requirement text + implementation evidence, but not delivered-version or manual-confirmation columns).

**What to do.**
1. Extend the `REQUIREMENTS.md` table schema with two additional columns: "Delivered" (version/date/commit) and "Tested" (automated-test reference + manual-confirmation date/evidence, or an honest "not yet confirmed" placeholder where no record exists).
2. Backfill the new columns for all existing rows as far as historical data (CHANGELOG.md, git blame, case studies) allows; mark genuinely unknown entries honestly rather than guessing.
3. Add a CI check enforcing that every `REQUIREMENTS.md` row has non-empty values in both new columns (an honest placeholder string counts as non-empty; a truly blank cell fails).
4. Add a doc-pin test (matching the existing `tests/unit/docs_requirements*` pattern) so future requirement rows cannot be added without the new columns.

**How to test.**
- Automated: the new CI check fails on a row missing either column; a doc-pin test asserts the schema (column headers) is present and stable.
- Manual: spot-check 10 backfilled rows against their cited commit/version and confirm accuracy.
- Standing clauses: `docs/case-studies/issue-{id}` (this epic's own timeline, since it's the newest standing requirement); single PR to add the mechanism, with backfill possibly needing a follow-up PR if the row count makes single-PR backfill impractical — if so, document that decision explicitly in the case study rather than silently deferring.

**Source refs:** konard's 2026-08-04 standing requirement (project-conventions memory, not a numbered GitHub issue — this is new doctrine, not a gap in an existing issue). **Dedup:** none — this is brand-new doctrine as of today, no existing tracking possible.

---

### E106: Report external upstream benchmark scores wherever internal curated-slice numbers are cited

**Body:**

**Problem statement.**
Source: arch-review 1.4 (MEDIUM). Doctrine: honest metrics — "0% metrics acceptable, fake floors not."
Current-code evidence (re-verified today against HEAD, since docs commit `f86259fa` "full living-documentation audit refresh" landed 2026-08-04): `VISION.md:235` now reads "the benchmark suite ... grew to a 13-case slice and passes **13/13** with a `minimum_pass_count` ratchet ... all without per-case memorization" — the curated-slice framing from arch-review's original "10/10" quote has since moved to 13/13, but the underlying issue is unchanged: `data/benchmarks/external-results.lino` (the real upstream harness, exemplary in its own honesty policy — "No curated subset, no invented floor: 0 passed is recorded as 0 passed") records HumanEval 0/20, MBPP 0/20, GSM8K 2/20, MATH 0/20, object-counting 0/20, CoEdIT 0/20 as of the most recent recorded runs — and **none of these external passed/total numbers appear anywhere near VISION.md's 13/13 claim**.

**What to do.**
1. Wherever VISION.md (or README) cites the internal curated-slice ratchet number, co-cite the current external per-suite passed/total from `external-results.lino` with a date, so the public claim carries both numbers side by side.
2. Explicitly label the internal slice "curated smoke slice, not a score" per the original arch-review acceptance criterion.
3. Add a doc-pin test keeping the quoted external numbers in sync with `external-results.lino` (same pattern as other `docs_requirements` pins), so this can't silently drift stale again.

**How to test.**
- Automated: the new doc-pin test fails if VISION.md's quoted external numbers don't match `external-results.lino`'s current recorded values.
- Manual: read VISION.md's benchmark paragraph and confirm both the curated-slice number and the honest external numbers are visible together.
- Standing clauses: `docs/case-studies/issue-{id}`; single PR.

**Source refs:** arch-review 1.4. **Dedup:** none — re-verified this is still open against current HEAD despite the 2026-08-04 docs refresh commit; that commit updated the curated-slice number (10→13) but did not add the external co-citation.

---

## Part B — Remaining smaller findings folded into the epics above (not separately filed)

To keep the total near the 15-25 target, these findings are intentionally NOT separate issues — each is explicitly covered inside one of the epics above:

- arch-review 1.5 (hardcoded-NL allowlist size) — folded into 1.7's issue below (same finding, cross-referenced in arch-review itself).
- arch-review 1.6 (routing-layer hardcoded promotion predicates) — related to but distinct from E82/4.1; **filed separately below as its own issue** since it's routing-specific, not worker-JS-specific (see "Data-driven routing" issue).
- arch-review 5.2, 5.3, 8.5, 2.2 — recorded as strengths/low-priority hygiene in arch-review itself; not filed as issues (positive findings + minor nits explicitly called low-value by the reviewer).
- arch-review 3.4 (closure-generated seed shards) and 3.5 (data/README.md gaps) — folded into the "Data-driven routing / seed hygiene" issue below as itemized sub-fixes, since they're small and thematically close to the seed-manifest work in E100.

---

### E107: Data-driven routing — ratchet the handler-migration ledger; move hardcoded promotion predicates into seed; honest closure-audit accounting

**Body:**

**Problem statement.**
Source: arch-review 4.1 (HIGH), 1.6 (MEDIUM), 3.4 (MEDIUM), 3.5 (LOW). Doctrine: "only memory + meta algorithm" (#559 mandate, honestly marked Partial in ROADMAP.md:406).
Current-code evidence:
- `data/meta/handler-migration-ledger.lino` — 4 handlers `migrated`, 50 `pending` of 54 rows (0 `migrated` rows found in this audit's own grep — even more pending than arch-review's snapshot recorded, worth re-confirming in the case study). Ceilings `specialized_handler_files_max`/`try_dispatch_entries_max` mirror the worker line-budget pattern but aren't ratcheted.
- `src/meta_method_dispatch.rs:46,61,94,106,125` — five `if name == "..."` special cases inside the supposedly-uniform executor.
- The browser worker fetches `handler-precedence.lino` (`seed_loader.js:48`) but has zero consumers of it; browser routing is a hand-written ~29-site if-chain (`formal_ai_worker_20.js:231-998`) plus a hardcoded 31-entry `syncHandlers` array.
- `src/intent_formalization.rs:718-835` (`append_prompt_relevants`) hardcodes 18 handler promotions as Rust predicates, including inline `contains("в ")`/`contains(':')` calendar glue; `:333` (`route_for_prompt`) hardcodes a `write_program` bypass before consulting the seed rule book; `src/solver_handlers/user_intent.rs:422-425` special-cases P=NP by name; `src/solver_handlers/shell_command_transform.rs:55,95-124` gates on English `contains("screen")` with inline 4-language cue lists.
- `data/seed/closure-generated-01..08.lino` — 10,228 lines / 2,044 English-only glosses loaded by nothing at runtime, existing solely so `scripts/audit-total-closure.py` reports zero unresolved tokens (a self-satisfying metric).
- `data/README.md` (24 lines) documents only `data/benchmarks/`; `data/seed/` (117 files), `cache/`, `overrides/`, `parity/`, `meta/`, `view/`, `training/` are undescribed.

**What to do.**
1. Add a CI assertion that the handler-migration ledger's `pending` count and both ceilings may only decrease between releases.
2. Migrate at minimum the 11 contextual + 5 prelude hardcoded `match` arms into seed-declared registry rows with data-declared gating cues; eliminate the 5 `if name == "..."` special cases in `meta_method_dispatch.rs`.
3. Make the browser worker's `syncHandlers` order derive from the fetched `handler-precedence.lino` at startup, with a parity test that reorders two seed rows and observes both runtimes change identically — or explicitly document per-handler in `routing-parity.lino` why not.
4. Move `append_prompt_relevants`'s 18 hardcoded promotion predicates into the cue-lexicon/intent-routing seed as `(role, handler)` pairs; delete inline `contains` literals for calendar/`screen`/`p=np`/loop cues, seeding them instead.
5. Either promote `closure-generated-*.lino` into real, 4-language, runtime-loaded lexicon records, or exclude generator output from the closure audit's denominator and let it report the true (non-zero) closure gap.
6. Expand `data/README.md` to describe every top-level `data/` subdirectory.

**How to test.**
- Automated: CI fails if ledger `pending` rises or either ceiling rises; a test asserting zero `if name == "..."` special cases in `meta_method_dispatch.rs`; a parity test proving seed-order changes propagate to browser routing; a test injecting a new seed promotion row and confirming no Rust change was needed; the closure audit's recomputed honest number is asserted in a doc-pin test matching what's quoted in docs.
- Manual: reorder two `handler-precedence.lino` rows and confirm both Rust and browser dispatch order change.
- Multilingual: re-run the calendar/`screen`/P=NP-adjacent prompts in en/ru/hi/zh after seeding their promotion rules, confirming no regression.
- Standing clauses: `docs/case-studies/issue-{id}`; single PR (may be staged by sub-finding, but the epic isn't done until all five "what to do" items land).

**Source refs:** arch-review 4.1, 1.6, 3.4, 3.5. **Dedup:** none — distinct from E82 (JS worker line-count absorption), though both touch `handler-precedence.lino`/worker routing; cross-link rather than merge since 4.1 is about handler *identity/dispatch* being data-driven, while E82 is about worker *line count/logic* being Rust/WASM.

---

## Part C — Covered by existing issues (not filed as new issues)

| Finding / requirement | Covered by | Mapping notes |
|---|---|---|
| R643-6 (extract Chakra+liquid-glass into a separate repo once #643 lands) | #643 (open) | Promised follow-up explicitly deferred until #643 merges; do not file until #643 lands — track as a comment/checklist item on #643 instead of a new issue. |
| R1-14 (Pages demo + e2e against deployed URL) | live/handcheck only | No code gap identified beyond verification; carried in E103 checklist, not a new issue. |
| R520-1, R534-2, R552-3, R883-2, R912-1 (various "file/check upstream issue" verification asks) | live/handcheck only | These ask to *check status* of already-supposedly-filed upstream issues, not to build new functionality; carried in E103 checklist. |
| R635-2, R651-6 (process/convention checks) | konard's standing process + this consolidation's own file-issues.sh | No code artifact to file; R651-6 is satisfied procedurally by this manifest's sub-issue linking. |
| R644-1 (PR #644 unmerged, in limbo) | maintainer action | Not an issue — needs a konard merge/close decision. See "Maintainer actions" below. |
| R887-1 (PR #887 CI-green, ready to merge) | maintainer action | Not an issue — needs a konard merge decision. See below. |
| R888-1 (PR #888 CI-green, ready to merge) | maintainer action | Not an issue — needs a konard merge decision. See below. |
| Full 32-item #710 dropped-requirements re-verification | #710 (E68, open) | Already tracked; this audit's needs-issue items are the *new* gaps found beyond #710's original 32, not a duplicate of #710 itself. |
| Anticipatory dreaming / predictive pre-learning (R887-1 requirement content) | #705 (E63, open) + #887 (PR) | Requirement tracked; PR pending merge decision (see maintainer actions). |
| E68 sub-owners #889-#896 (conversational regressions, coverage ratchet, equation corpus, etc.) | #889-#896 (open) | Already individually tracked; no new issue needed for anything already itemized under these numbers. |
| Self-improvement / method learning from experience | #922 (E75, open) | Checked for overlap with the new E96 (memoized-answer-surface burndown) — confirmed distinct: #922 is about learning generalized method abstractions from event-log traces via the promotion protocol, not about removing the seeded canned-answer handlers found in this audit. No dedup collision. |
| JS = glue/all-logic-in-Rust doctrine (R536) | REQUIREMENTS.md R536 (already recorded), tracked by #658 (closed) | The doctrine requirement itself is already recorded; this audit's E82/E99/E100/E101 are the concrete unfinished-work issues underneath it, filed as the restart of #658 rather than duplicating R536's existence. |
| R412-2 / R413-1 meta-builder mandate | no existing open issue | Confirmed NOT covered by any of #423/#424/#433/#448 (those are narrower, already-closed increments) — filed fresh as E86. |

---

## Part D — Maintainer actions (NOT issues — konard must act directly)

1. **Merge decision on PR #887** ("Add deterministic anticipatory dreaming") — OPEN, CI-green since 2026-08-01 per the audit's handcheck. No code action needed from an issue; konard should merge or explain why not.
2. **Merge decision on PR #888** ("E68: Recover conversational requirements and audit all 32 gaps") — OPEN, CI-green since 2026-08-01 per the audit's handcheck. Same as above.
3. **Closure decision on PR #644** ("Add experimental formalization model fallback") — OPEN with 6 comments, in limbo with no recorded konard decision. Needs an explicit merge/close/request-changes call.
4. **Answer konard's own unresolved #413 question** about expanding scope to the full meta-builder — this audit files E86 as the concrete follow-through, but the *original* thread question still has no reply; note this for konard's awareness even though E86 is now the tracking vehicle.
5. **Clarify the "grok build" subcommand question from #753** — never answered in-thread; carried into E103's checklist (item 33) as a live item, but ultimately needs konard's direct answer, not just code.
6. **Decide the fate of the two stray root `.lino` files** (`formal-ai-harness-latest.lino`, `formal-ai-server-latest.lino`) — E93 covers the code fix preventing recurrence, but the specific two files' disposition (case-study archive vs. delete) is a one-time call the issue's implementer can make, flagged here so it isn't missed.

---

## Summary

- **Epics (E-numbered, sub-issues of #651):** E78 through E107 = **30 epics**.
- **Plain bugs (no E-number):** 3 — "codify and enforce recorded conventions", "macOS CI parity", "hi/zh word-operator arithmetic parity".
- **Total new issues to file: 33** (30 epics + 3 bugs) — see `file-issues.sh` for exact creation order and numbering; note E93 and E104 also satisfy the ".lino CWD export" and "agent --task ignored" line items called out separately in the brief, so no additional issues were filed for those.
- **Deduped-to-existing mappings:** 11 rows in Part C's table (covering roughly 15 individual requirement/finding IDs once #889-#896 and the #710/#705 groupings are unpacked).
- **Maintainer actions (not issues):** 6 items in Part D.

This is above the 15-25 target range; see the note at the top of Part A's design rationale — the audit's own inputs (27 needs-issue rows + 25 arch findings + 7 breakages, after merging duplicates and folding low-value findings into larger epics per Part B) still required 30 distinct epics because most findings are genuinely orthogonal problems (different files, different doctrines, different owners) rather than fragments of the same underlying issue. Further merge candidates considered and rejected: E84/E85 (both from #331 but functionally unrelated — a compiler vs. an execution-container runtime); E94/E95 (both from #873 but memory-versioning vs. execution-loop-bounding are separate subsystems); E99/E100/E101 (each a distinct JS file/subsystem under the same doctrine, kept separate so each has its own clear acceptance criteria and owner).
