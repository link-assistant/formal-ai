Part of the E127 umbrella (#1183). Architect (2026-09-29): the meta algorithm must "be white box instead of black box for LLMs".

## Gap

The umbrella's acceptance #2 requires: *"The answer carries its derivation as links: search queries, fetched URLs with hashes, the formalized page fragments, the parts decomposed from each example, the recomposition, the rendering, the verification output. `formal-ai explain <answer>` prints it. This is the white box."* Today the pieces exist as separate, uncomposed fragments:

- `SymbolicAnswer` (`rust/src/engine.rs:55-65`) carries `evidence_links: Vec<String>` and `thinking_steps: Vec<ThinkingStep>`, but neither is a structured derivation record; `evidence_links` is untyped strings and `thinking_steps` is a human-narrated trace, not a queryable per-stage record.
- `VerifiedAnswer` (`rust/src/solver_handlers/verifiable_task.rs:30-37`) already has the right shape — `derivation_id: String` (content-addressed, `verifiable_task.rs:606`: `let derivation_id = candidate.content_id();`), `fragments: Vec<String>`, `source_urls: Vec<String>`, `source_licenses: Vec<String>`, `checks: Vec<Evidence>` — but it exists **only** for the generic verifiable-task route (quantity/unit-conversion math), not for every answer the solver returns. A second, independent `derivation_id` pattern lives at `rust/src/verifiable_task/ledger.rs:31` and is referenced by `rust/src/obligation_ledger.rs:53` as `check_id = "<VerifiedAnswer::derivation_id>:<check slug>"` — so the naming convention this issue generalizes already has a plan-00 contract citation (`docs/case-studies/issue-1138/plans/00-root-causes-and-integration.md:517`).
- `source_fetch::SourceCapture` (`rust/src/source_fetch.rs:172-179`) already carries exactly "fetched URLs with hashes": `source_url`, `fetched_at`, `sha256`, `cached`, `bytes`, plus a `trace_payload()` renderer. It is used by `concept_lookup.rs`, `coding_research_learning.rs`, `how_to_guide.rs`, `option_network.rs`, `procedure_text.rs`, `anticipation.rs`, `cli_solve.rs` — but nothing collects the captures used by one answer into that answer's own record.
- `EventLog` (`rust/src/event_log.rs`) appends content-addressed events (`web_search:request`, `web_search:query_kind`, `web_search:provider`, `web_search:rank`, `web_search:fused`, `source:http` — see `event_log.rs:412,519-532` and `coding/synthesis_runtime.rs:169-176`: `log.append("source:http", format!("url={};fetched_at={};sha256={};catalog_match={}", …))`). This is a real substrate for a derivation trail, but the log lives per-process-call and is projected only into the `--thinking` narration (`rust/src/thinking.rs`), never persisted and indexed by an answer id for later retrieval.
- The CLI `--thinking` flag (`rust/src/main.rs` `Chat` command) prints `ThinkingStep`s as prose, mirrored in the OpenAI/Anthropic `reasoning` fields and the Telegram bot (`rust/src/thinking.rs:1-20`, `thinking_prose.rs`) and in `js/worker/formal_ai_worker.js` / `js/app.js`'s `naturalizeThinkingStep`. This is narration for a human mid-conversation, not a retrievable, hash-bearing evidence record for a *specific past* answer.
- No `formal-ai explain <answer-id>` command exists. `enum Command` in `rust/src/main.rs:89` lists `Chat, Dataset, Translate, Coding, Context, Report, Serve, Connect, Proxy, Memory, SharedDialog` (and further variants) — no `Explain`. The word `explain` is already taken by `rust/src/agentic_coding/explain.rs`: `EXPLAIN_TASK` routes "how Formal AI itself works" to `self_explanation::canonical_explanation()`, a **different** recipe (source-citation self-inspection of the codebase, not per-answer derivation). Any new command or module must not collide with this existing recipe's keywords (`explain.rs:57-92` lists them).
- Answers carry no stable id exposed to a caller at all outside the verifiable-task path, so there is nothing today a `formal-ai explain <id>` could even key on for an ordinary chat answer.

## Requirements

R1. Every `SymbolicAnswer` the solver returns carries a `derivation_id`: a content-addressed id (`engine::stable_id`) computed the same way `VerifiedAnswer::derivation_id` is computed today, surfaced on every calling surface (CLI, Serve API, Telegram, browser).

R2. A `Derivation` record is built per answer and captures: the search queries issued (from `web_search:*` events), the fetched URLs with their SHA-256 hashes and fetch timestamps (`SourceCapture`), the formalized page fragments consumed (E128/#1163), the parts decomposed from retrieved examples (E129/#1164), the recomposition step (which parts were bound to which parameters), the rendering step (target language emitted, E132/#1167), and the verification evidence (`execution_evidence::Evidence`).

R3. The derivation record is durable and queryable **after** the answer is returned — not only inline in the live transcript — keyed by `derivation_id`, so a later `explain` call does not require replaying the original request.

R4. A new CLI subcommand `formal-ai explain <answer-id>` prints the full derivation for a previously produced answer, reading it from the durable store of R3.

R5. A stage the answering route never populated (for example, a route that has not yet migrated to discovery per E130/#1165) is reported as "not recorded" for that stage — never fabricated or inferred.

R6. The `--thinking` trace and the derivation record are two projections of the *same* `EventLog`, so they cannot disagree (`thinking.rs` and the new `derivation` module both read `EventLog::events()`).

R7. `formal-ai explain` and the existing `explain` self-explanation recipe (`agentic_coding/explain.rs`) are reachable through distinct, non-overlapping trigger words; neither routing regresses the other (existing `EXPLAIN_KEYWORDS` unaffected).

R8. The Serve API's JSON response includes `derivation_id` alongside `evidence_links`, so an external agent (Hive Mind, a PR reviewer) can call `explain` on any answer it received.

R9. Three-roots parity: `derivation.rs` and `explain_answer.rs` are translated with `formal-ai translate --from rust --to js|ts --input <file> --write`; `data/parity/cross-runtime-synthesis.json` gains derivation cases (same event log in, same derivation out) for all three roots.

## Design

- New: `rust/src/derivation.rs` — `pub struct Derivation { pub answer_id: String, pub search_queries: Vec<String>, pub fetches: Vec<FetchRecord>, pub formalized_fragments: Vec<String>, pub decomposed_parts: Vec<String>, pub recomposition: Option<String>, pub rendering: Option<String>, pub verification: Vec<crate::execution_evidence::Evidence> }`, `pub struct FetchRecord { pub url: String, pub sha256: String, pub fetched_at: String }`, and `pub fn record_for(log: &crate::event_log::EventLog, answer_id: &str) -> Derivation`, projecting the same `EventLog::events()` that `thinking.rs` projects, reading existing kinds (`web_search:*`, `source:http`) and new kinds appended as E128/E129/E132/E131 land (`formalize:fragment`, `decompose:part`, `recompose:bind`, `render:emit`).
- Modify `rust/src/engine.rs:55-65`: add `pub derivation_id: String` to `SymbolicAnswer`, computed at construction via `stable_id("answer", &answer)`, generalizing `VerifiedAnswer::derivation_id`'s existing pattern (`solver_handlers/verifiable_task.rs:606`) to every route.
- New: `rust/src/agentic_coding/explain_answer.rs` (distinct name from the existing self-explanation `explain.rs`) — retrieves a durable `Derivation` by id and renders it as Links Notation, following the existing agentic-recipe shape (compare `repair_strategy.rs`, `self_heal.rs`).
- Durable store: extend `rust/src/memory.rs`'s append-only `MemoryEvent` with a `Derivation` variant, or persist content-addressed records under `data/cache/derivations/<answer_id>.lino`, consistent with the `cache_path data/cache/...` convention already used by `data/seed/sources-registry.lino` entries. Example record (Links Notation, styled like `data/seed/sources-registry.lino`):

```
derivation
  answer_id "a1b2c3d4e5f6"
  search_query "how to compile a Kotlin program from the command line"
  fetch
    url "https://kotlinlang.org/docs/command-line.html"
    sha256 "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08"
    fetched_at "2026-09-29T00:00:00Z"
  formalized_fragment "code_block:kotlinc-command"
  decomposed_part "run_command"
  recomposition "bound literal=Hello, Formal AI!"
  rendering "kotlin"
  verification "exit=0"
```

- CLI: add `Explain { answer_id: String, #[arg(long, value_enum, default_value_t = OutputFormat::Text)] format: OutputFormat }` to `enum Command` in `rust/src/main.rs:89`.
- API: Serve response JSON gains `derivation_id` next to `evidence_links` (`rust/src/engine.rs`'s `SymbolicAnswer` serialization already derives `Serialize`).
- JS/TS parity: per R9.

## Tests

- New module `rust/tests/unit/issue_1184_derivation.rs` (register `mod issue_1184_derivation;` alphabetically in `rust/tests/unit/mod.rs`, next to the existing `issue_1138_*` block).
  - `derivation_carries_search_queries_fetches_and_hashes_for_an_online_answer` — offline fixture-backed: builds an `EventLog` with `web_search:*` and `source:http` events, asserts `Derivation::record_for` recovers query text, URL, and sha256.
  - `every_symbolic_answer_carries_a_stable_derivation_id` — two calls with the same prompt produce the same `derivation_id`; a different prompt produces a different one.
  - `explain_reports_not_recorded_for_stages_a_route_did_not_populate` — a route with no fetch stage (e.g., a pure-arithmetic answer) yields `fetches: []`, not a fabricated entry.
  - `explain_and_thinking_trace_agree_on_the_same_event_log` — `thinking::localize_thinking_steps` and `derivation::record_for` built from the same `EventLog` cite the same URLs/queries.
  - `explain_command_does_not_collide_with_the_self_explanation_recipe` — asserts `agentic_coding::explain::is_explain_task` and the new `formal-ai explain <id>` CLI path route to different code, using the existing `EXPLAIN_KEYWORDS` fixture list.
- Online test gated like `rust/tests/unit/issue_991_how_to_synthesis.rs:52-61`'s `live_fetch_requested()` (`FORMAL_AI_LIVE_FETCH=1|true|yes|on`): `explain_of_kotlin_command_line_answer_cites_the_real_kotlinlang_url_and_hash` — runs the E128 (#1163) online scenario, then calls `formal-ai explain <id>` and asserts the printed derivation names `kotlinlang.org/docs/command-line.html` with a real sha256.
- Commands: `RUSTUP_TOOLCHAIN=1.98.1 cargo test --test unit issue_1184_derivation`; live variant `FORMAL_AI_LIVE_FETCH=1 RUSTUP_TOOLCHAIN=1.98.1 cargo test --test unit issue_1184_derivation` (the live test returns early without the variable, like `issue_991_how_to_synthesis.rs`; it is not `#[ignore]`d).

## Definition of done

- [ ] Requirement shard `docs/requirements/issue-1184-white-box-derivation.md` with R1-R9 as rows (format matching `docs/requirements/issue-1138-prerequisite-discovery.md`); `RUSTUP_TOOLCHAIN=1.98.1 rust-script scripts/assemble-requirements.rs --write` run so `REQUIREMENTS.md` includes it.
- [ ] Row for every requirement in `docs/requirements-traceability.md` (ID | Shard | Delivered | Automated test | Manual confirmation).
- [ ] Case-study `docs/case-studies/issue-1184/` with the audit above, a plan, and a survey of what `EventLog`/`SourceCapture`/`VerifiedAnswer` already provide.
- [ ] Changelog fragment `changelog.d/<timestamp>_issue-1184-white-box-derivation.md` (repository root; `bump: minor`, new CLI surface).
- [ ] `RUSTUP_TOOLCHAIN=1.98.1 rust-script scripts/run-ci-gates.rs --stage rust` green.
- [ ] `formal-ai explain <id>` reachable from a fresh build and documented in `docs/status.md` or `ARCHITECTURE.md`.

## Depends on / blocks

- Depends on nothing to start: R1 (`derivation_id`), R3 (durable store), R4 (`formal-ai explain`), R6 and R8 land first, and every stage not yet produced reports "not recorded" (R5).
- Filled in by: #1163 (E128) fetches and formalized fragments, #1164 (E129) decomposed parts and recomposition, #1166 (E131) the obligation graph the recomposition binds against, #1167 (E132) the rendering stage. Each of those issues must append its stage events so this record shows them.
- Blocks umbrella #1183 acceptance item 2 (it names `formal-ai explain <answer>`).
- Used by #1170 (E135) and #1179 (E144) as their evidence records.

