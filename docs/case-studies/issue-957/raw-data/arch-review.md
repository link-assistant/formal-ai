# Formal AI — Deep Architectural Review

Repo: `/Users/konard/Code/Archive/link-assistant/formal-ai` @ `de61602f` (v0.326.0), reviewed 2026-08-04.
Judged against the project's own doctrine: VISION.md / GOALS.md / NON-GOALS.md / REQUIREMENTS.md / ARCHITECTURE.md / ROADMAP.md — associative-only terminology, generalization over memoization, en/ru/hi/zh by construction, determinism, honest metrics, everything opt-in, LLMs never at the steering wheel, and (owner directive received during review) **JS = interfacing glue + JSX UI only; all logic in compiled Rust (native server-side, WASM in web, same WASM engine reused by desktop and other surfaces)**.

Severity scale: **high** = violates a stated doctrine/goal in a way that misleads or blocks the roadmap; **medium** = real deviation, contained; **low** = hygiene.

---

## Dimension 1 — Generalization gaps

### 1.1 (HIGH) Seeded canned "summaries" and "brainstorms" are memoized prompt→answer tables

Evidence:
- `data/seed/summary-topics.lino:9-18` — `topic Rust ... body "Rust is a systems programming language focused on performance, memory safety, and concurrency. ..."`; likewise `topic Wikipedia`, `topic formal-ai`. Three pre-written English paragraphs keyed to topic-detection keywords.
- `src/solver_handlers/benchmark_prompts.rs:32-56` — `try_summarization_request` returns `topic.body.clone()` verbatim; the handler never summarizes anything.
- Same file, `try_brainstorming_request` (`benchmark_prompts.rs:63-84`) returns seeded `category.items` lists.
- The file name itself — `benchmark_prompts.rs` — admits these handlers exist to answer benchmark-style prompts.

Why it violates doctrine: VISION.md ("The system should prefer deep understanding ... over answer memoization"), NON-GOALS.md ("A memoized answer cache is not a substitute for reasoning from source data"). A held-out paraphrase asking to summarize any fourth topic falls to `fallback_body` boilerplate; a Russian "резюме Rust" triggers (multilingual triggers) but returns the **English** canned paragraph — also a multilingual violation.

Draft issue:
> **Title:** Replace seeded per-topic summary/brainstorm bodies with derivation through the meaning network
> **Body:** `try_summarization_request` and `try_brainstorming_request` (src/solver_handlers/benchmark_prompts.rs) return pre-written seed paragraphs from data/seed/summary-topics.lino / brainstorm seeds. This is a memoized prompt→answer table: it covers exactly 3 topics, only in English, and does no summarization. Doctrine requires deriving a summary from the concept/meaning links (Wikidata/Wikipedia cache records already exist under data/cache/) and rendering it in the prompt's language. The seeded bodies should become test fixtures at most, not the production answer path.
> **Acceptance criteria:** (1) "summarize X" for a topic with a cached Wikidata/Wikipedia record produces a derived summary with `source:` evidence links, in en/ru/hi/zh; (2) held-out topics not present in any seed body pass the same test shape; (3) `summary-topics.lino` `body` fields deleted or demoted to fixtures; (4) a regression test asserts the answer text is not byte-equal to any seed `body`.

### 1.2 (HIGH) Canned comparison-table paragraphs keyed to topic substrings

Evidence (from routing audit):
- `src/solver_handlers/research_table.rs:416` — `if normalized.contains("machine learning algorithm")` → four hardcoded English prose paragraphs as table cells.
- `research_table.rs:432` — `contains("deep learning") && contains("traditional ml")` → another canned set.
- `research_table.rs:448` — `contains("neural network")` → same.
- `research_table.rs:26-33` — column labels hardcoded in English: `"Key differences"`, `"Use cases"`, `"Advantages"`, `"Disadvantages"`.

Why: per-topic pinned answers (memoization), English-only content and labels (multilingual violation), inside a handler whose docstring says it serves agent-mode research decomposition — i.e., the "general" research path is stuffed with three special-cased topics.

Draft issue:
> **Title:** Derive research comparison content from cached sources; remove topic-substring canned paragraphs
> **Body:** research_table.rs returns hand-written paragraphs when the prompt contains "machine learning algorithm", "deep learning"+"traditional ml", or "neural network" (lines 416/432/448), and renders English-only column labels from Rust consts. Comparison content must be composed from cached source records (data/cache/, search fusion) with provenance, and labels must come from the multilingual seed lexicon. Any topic outside the three pinned ones currently gets a structurally different (empty) answer, which held-out paraphrase testing would flag.
> **Acceptance criteria:** (1) the three `contains(...)` topic blocks are deleted; (2) comparison cells carry `source:` links; (3) labels localize in all four seed languages; (4) a held-out topic (e.g. "compare sorting algorithms") produces the same answer shape as the pinned ones.

### 1.3 (HIGH) Per-idiom / per-issue named handlers with hardcoded natural-language fallbacks in Rust

Evidence:
- `src/solver_dispatch.rs:343` — `("kupi_slona", try_kupi_slona)`: a top-level solver handler for one Russian children's joke («Купи слона», issue #41).
- `src/solver_handlers_policy.rs:75-80` — hardcoded Russian answer baked into the binary: `"«Купи слона» — это известная русская детская фраза-игра. ..."` as `unwrap_or_else` fallback.
- `src/solver_handlers_policy.rs:39` — `try_physical_action_question` fallback `"Нет. У меня нет физического тела."` hardcoded (issue #39).
- Registry comments pin handlers to single reported dialogs: `src/solver_dispatch.rs:267-272` ("Issue #341: a decomposed agent step like 'test it by scraping wikipedia.org and show me the top 10 most frequent words' must stay bound ..."), `:288-295` (issue #395), `:296-300` (issue #552), `:322-325` (issue #423), `:328-331` (issue #425).

Why: this is per-prompt special-casing at handler granularity. Recognition surfaces moved to seed (`meanings-policy.lino` roles) — good — but the *handler identity* is still one Rust function per joke/issue, and the fallback answers are hardcoded NL in code (the exact thing `scripts/check-hardcoded-language.rs` was built to burn down; these live on its allowlist). One general "idiom/policy response" mechanism driven by seed roles would subsume `kupi_slona`, `physical_action_question`, `shell_refusal`, `punctuation_only_prompt`, etc.

Draft issue:
> **Title:** Collapse per-idiom policy handlers into one seed-role-driven policy responder
> **Body:** `kupi_slona`, `physical_action_question`, and sibling policy handlers each pair one seed trigger-role with one seed response-key plus a hardcoded in-code fallback string (solver_handlers_policy.rs:39, 75-80). The pattern is identical across them: `mentions_role_raw(ROLE_X) → response_for(KEY_X)`. Replace with a single `policy_response` handler that walks a seed table of (trigger_role, response_key) pairs, so adding the next idiom is a data edit — the stated point of "Data Is The Interface". Delete the in-code Russian fallback strings; the seed is already the fallback chain.
> **Acceptance criteria:** (1) one handler replaces ≥4 policy handlers with behavior pinned by existing tests; (2) no NL string literals remain in the replacement (verified by check-hardcoded-language with no new allowlist entries); (3) adding a new idiom in seed alone makes it answer without recompiling; (4) handler-precedence.lino row count drops accordingly.

### 1.4 (MEDIUM) Internal "10/10" benchmark headline vs honest external 2/120 — docs lead with the flattering number

Evidence:
- `data/benchmarks/industry-suite.lino:8,26` — `minimum_pass_count "13"`, but each source has `imported_cases "1"` (1 hand-picked case per benchmark out of e.g. HumanEval's 164).
- `data/benchmarks/external-results.lino:144-260` — the real upstream harness (issue #698) records HumanEval `passed "0"`/20, MBPP `passed "0"`/20, GSM8K `passed "2"`/20, MATH `passed "0"`/20, object-counting `passed "0"`/20, CoEdIT `passed "0"`/20 on both 2026-07-20 and 2026-08-03 runs.
- `VISION.md:235` still headlines: "the benchmark suite ... passes **10/10** with a `minimum_pass_count` ratchet ... all **without per-case memorization**".

Why: the external harness itself is exemplary honesty (`external-results.lino:10` "No curated subset, no invented floor: 0 passed is recorded as 0 passed") — doctrine-compliant. The gap is documentation framing: VISION.md's celebratory 10/10 is a curated 1-case-per-benchmark slice, precisely the "curated subset" the external harness policy forbids, and the honest 0-2/20 numbers appear nowhere in VISION/README. Given the eighth-pass audit's own finding that "the dominant historical failure mode was silent scope-narrowing" (ROADMAP.md:339), leading with the curated number repeats that failure mode in the docs.

Draft issue:
> **Title:** Report external upstream benchmark scores wherever the internal 10/10 slice is cited
> **Body:** VISION.md:235 and related docs cite the internal 10-case curated slice (industry-suite.lino, 1 imported case per benchmark) passing 10/10, while the honest external harness (external-results.lino) records 2 passes out of 120 upstream cases. Both numbers are real; only one is quoted. Every doc that cites the internal ratchet should co-cite the current external passed/total per suite, so the public claim matches the honest-metrics doctrine.
> **Acceptance criteria:** (1) VISION.md/README benchmark paragraphs quote external per-suite passed/total with a date; (2) a doc-pin test keeps the quoted numbers in sync with external-results.lino; (3) the internal slice is explicitly labeled "curated smoke slice, not a score".

### 1.5 (MEDIUM) Hardcoded-NL lint exists but the burn-down has a 1,367-line allowlist

Evidence: `scripts/check-hardcoded-language.rs` + `scripts/hardcoded-language-allowlist.txt` (1,367 lines). Details and worst offenders in Dimension 6 section (shared evidence) and agent findings below.

### 1.6 (MEDIUM) Routing-layer per-prompt promotion predicates hardcoded in Rust

Evidence:
- `src/intent_formalization.rs:718-835` — `append_prompt_relevants`: an 18-entry hardcoded array binding `"handler:<name>"` to Rust predicates, including a 27-line inline calendar conditional with literal `normalized.contains("в ")` / `contains(':')` glue (`:801-828`).
- `src/intent_formalization.rs:311-324` — `select_rule_for_intent` matches 8 literal route slugs.
- `src/intent_formalization.rs:333` — `route_for_prompt` opens with a hardcoded `write_program` bypass before consulting the seed rule book.
- `src/solver_handlers/user_intent.rs:422-425` — the P=NP problem special-cased by name (`contains("p=np") || contains("p = np") || ...`).
- `src/solver_handlers/shell_command_transform.rs:55,95-124` — handler gated on English `contains("screen")`; inline 4-language cue lists (`"loop"|"цикл"|"लूप"|"循环"`) bypassing the cue lexicon.

Why: "Bypassing SolverConfig / seed for hard-coded behavior is not acceptable" (NON-GOALS); the intent-routing rule book (`data/seed/intent-routing.lino`) is the sanctioned mechanism, and these predicates route around it.

Draft issue:
> **Title:** Move append_prompt_relevants promotion predicates into the cue-lexicon / intent-routing seed
> **Body:** intent_formalization.rs:718-835 binds 18 handler promotions to hardcoded Rust predicates, several with inline language literals, and route_for_prompt:333 short-circuits the seed rule book for write_program. The cue-lexicon (data/meta/cue-lexicon.lino) already demonstrates the data shape for cue vocabularies. Each promotion should become a seed rule (cue role → handler name, with combo semantics), leaving Rust only the generic matcher.
> **Acceptance criteria:** (1) append_prompt_relevants reads (role, handler) pairs from seed; (2) inline `contains` literals ("в ", "screen", "p=np", loop cue lists) are deleted or seeded; (3) routing_precedence and existing issue tests stay green; (4) adding a promotion for a new handler requires no Rust change (proved by a test that injects a seed row).

---

## Dimension 2 — Code duplication (Rust ↔ JS worker ↔ surfaces)

### 2.1 (HIGH) ~27.7k-line JS worker mirrors the Rust core; the absorption ratchet only ever moved UP

Evidence:
- 24 shards (not 22): `src/web/worker/formal_ai_worker_00.js` … `_23.js`, **27,705 lines** total, loaded by plain `importScripts` concatenation from `src/web/formal_ai_worker.js:57-86` (global-scope, no modules). Shard headers are stale ("Worker module 20 of 21", `formal_ai_worker_19.js:1`).
- `scripts/check-worker-line-budget.rs:26,60` — `TARGET_TOTAL_LINES = 3_000`, `CEILING_TOTAL_LINES = 27_705` — the ceiling equals today's exact count (zero headroom) and its own doc comment (`:34-58`) is a ledger of eight upward re-baselines (26,807 → … → 27,705, issues #845 #701 #699 #706 #708 #858 #890). Only two entries ever reduced (−2, −1). `git log -S CEILING_TOTAL_LINES`: created 2026-07-14, never lowered.
- `docs/case-studies/issue-658/capability-inventory.md` recorded 26,708 lines in 2026-07; the mirror has **grown ~1,000 lines since the absorption was chartered**. Of its 6 planned migration slices, only slice 1 (CI guards) landed.
- WASM side did grow (real progress): `src/web/wasm-worker/src/lib.rs` (898) + `memory_query_worker.rs` (734) + `proof_translation_worker.rs` (78), 24 `extern "C"` exports, 9 `#[path]`-included shared Rust files, .wasm 275 KiB. But WASM is referenced in only 11/24 shards and **every delegation site keeps a full JS fallback** (`formal_ai_worker_19.js:3-5`, `formal_ai_worker_02.js:1130-1155`) — WASM was added alongside the JS, not instead of it. Only `_23.js` (18 lines) is a true thin adapter.

Concrete Rust↔JS duplicated pairs (sample of 8, ~1,400 lines just in these):
1. Definition merge — `src/definition_merge.rs:157-331` ↔ `formal_ai_worker_09.js:2-104` (8 of 9 functions have name-for-name twins).
2. Program-plan inverse rules — `src/program_plan.rs:81` ↔ `formal_ai_worker_14.js:74-183` (header: "mirror of src/program_plan.rs"; Rust doc comment copied as JS comment).
3. Reciprocal rank fusion — *triplicated*: `src/web_search_core.rs:615` ↔ `src/solver_handlers/web_requests.rs` offline trace ↔ `formal_ai_worker_19.js:2`; Rust doc admits "Equivalent to the JS implementation".
4. Memory-program compiler — `src/memory_program.rs:405-484` ↔ `formal_ai_worker_22.js:48-126`, identical `program_gap:*` tokens.
5. Intent-route matcher — `src/web_engine_core.rs:385-425` ↔ `formal_ai_worker_02.js:1127-1155` (calls WASM, then reimplements the identical cascade as fallback).
6. Relative meta logic — whole shard `formal_ai_worker_21.js` (692) ↔ `src/relative_meta_logic.rs` (434), shared constant `0.6` prior duplicated (`JS:14` / `Rust:43`).
7. Calendar/weekday — `src/solver_handlers/calendar.rs:272-351` ↔ `formal_ai_worker_08.js:305-370`, seven functions name-for-name.
8. Language detection — `src/language.rs:492` (compiled into WASM) ↔ `formal_ai_worker_05.js:148-164`, `_13.js:42-62`, contradicting REQUIREMENTS.md:1580 (R706-6 "share one detection registry").

Why: pillar 18 in ROADMAP ("Rust-to-WebAssembly parity with JavaScript reserved for UI/glue — Built") is **not true today** and the owner doctrine makes this the largest single violation in the repo. Dual maintenance is also the root cause of the parity-fixture apparatus (only 5 cases in `data/parity/cross-runtime-synthesis.json` pin 27.7k lines; most "parity" tests merely regex shard text for presence, e.g. `tests/unit/issue_702_nested_contexts.rs:305`).
- Also: `ROADMAP.md:421` says "shareable packages (#658) closed" — #658 is the WASM-absorption epic and it is not closed; factual error.

Draft issue:
> **Title:** Make the worker line-budget a true one-way ratchet and land absorption slice 2
> **Body:** scripts/check-worker-line-budget.rs was added as a shrink ratchet toward 3,000 lines but has been re-baselined upward eight times and never down; the JS mirror grew ~1,000 lines since docs/case-studies/issue-658/capability-inventory.md was written. Change the CI contract: CEILING_TOTAL_LINES may only decrease (enforce in the script by refusing a ceiling above the committed value), require any PR that adds worker lines to remove at least as many, and land inventory slice 2 (extraction/parsing → WASM) deleting the JS fallbacks at delegation sites rather than keeping both paths. Fix ROADMAP.md:421's incorrect "#658 closed" claim.
> **Acceptance criteria:** (1) CI fails on any ceiling increase; (2) slice 2 functions exist only in Rust/WASM, JS keeps ≤5-line adapters (model: formal_ai_worker_23.js); (3) total worker lines strictly below 26,708 (the chartered baseline); (4) parity fixture grows to cover each migrated function with held-out inputs.

### 2.2 (LOW) Web/desktop/vscode surface duplication is healthy

Evidence: `vscode/src/lib/webview-html.cjs:7,22-28` renders the committed `src/web/` app unchanged; `desktop/scripts/prepare-resources.mjs:55-56` copies `src/web/` + seed wholesale. Only minor drift: `vscode/src/lib/config.cjs:12` mirrors `normalizeDesktopStatus` from `src/web/app.js`. No finding beyond the JS-logic dimension below.

---

## Dimension 4 — Data-driven routing (E44/E57 reality check)

### 4.1 (HIGH) "Only memory + meta algorithm" is ~7% done by the project's own ledger

Evidence:
- `data/meta/handler-migration-ledger.lino` — **4 handlers `migrated`, 50 `pending`** of 56 rows.
- `src/solver_handlers/` = **46 files, 19,621 lines** (largest: web_search_intent.rs 1000, installation_conversion.rs 996, web_requests.rs 993, calendar.rs 981, behavior_rules.rs 953).
- Dispatch is three hardcoded Rust tables + one seed file: `PRELUDE_METHOD_NAMES` (5, `src/solver_dispatch.rs:151`) and `CONTEXTUAL_HANDLER_NAMES` (11, `:131`) both dispatch via literal `match name { ... }` arms (`:167`, `src/meta_method_dispatch.rs:192`); only `HANDLER_FUNCTIONS` (55, `:240`) is order-driven by `data/seed/handler-precedence.lino` with a permutation assertion (`:360-396`) — that part is real and well-guarded.
- Five `if name == "..."` special cases live inside the supposedly uniform executor: `src/meta_method_dispatch.rs:46,61,94,106,125`.
- `SPECIALIZED_HANDLERS` the constant is gone (survives only in pinned snapshots `tests/source/solver.rs:43`); the *concept* — a hardcoded handler registry — is fully alive.
- The JS worker ignores the seed precedence entirely: `src/web/seed_loader.js:48` fetches `handler-precedence.lino`, but no `src/web/` code consumes it; routing there is a hand-written if-chain of ~29 dispatch sites (`formal_ai_worker_20.js:231-998`) plus a hardcoded 31-entry `syncHandlers` array (`:582`). `tests/fixtures/routing-parity.lino` concedes "Full order-parity is impossible on purpose."

Why: E44 (#663) delivered exactly what it scoped — precedence-as-data — but ROADMAP pillar 20 and CHANGELOG.md:651 ("Retired the SPECIALIZED_HANDLERS precedence remnant into data-driven routing") read as if routing is data-driven. Handler identity, gating, and bodies are all still code; the #559 mandate ("only memory + meta algorithm") is honestly marked Partial in ROADMAP.md:406 (19.6k lines / 40 files admitted). The gap between the mandate and 50-pending is the single biggest architectural debt after the JS mirror.

Draft issue:
> **Title:** Ratchet the handler-migration ledger: pending count may only fall, and the JS worker must consume handler-precedence.lino
> **Body:** data/meta/handler-migration-ledger.lino records 50 of 54 handlers pending migration to memory+meta-methods, with ceilings (specialized_handler_files_max 38 / try_dispatch_entries_max 49) that mirror the worker line-budget pattern. Add a CI assertion that `pending` count and both ceilings only decrease. Separately, the browser worker downloads handler-precedence.lino and discards it (seed_loader.js:48; zero consumers), so seed reordering changes Rust routing but not browser routing — a silent cross-runtime divergence. Make the worker's syncHandlers order derive from the fetched seed, or document per-handler why not in routing-parity.lino.
> **Acceptance criteria:** (1) CI fails if ledger pending count rises; (2) at least the 11 contextual + 5 prelude `match` arms become registry rows with data-declared gating cues; (3) worker sync-handler order is read from the seed at startup with a parity test that reorders two rows and observes both runtimes change; (4) meta_method_dispatch.rs contains zero `if name == "..."` special cases.

---

## Dimension 5 — Structure / health (partial; see agent section below)

### 5.1 (MEDIUM) The 1000-line file cap is being satisfied by mechanical splitting, not modularization

Evidence:
- `scripts/check-file-size.rs:18-24` — max 1,000 / warn 900 for Rust.
- 13 files sit in the 950-1000 band; four at exactly 999-1000 (`src/world_model.rs`:1000, `src/solver_handlers/web_search_intent.rs`:1000, `src/agentic_coding/planner.rs`:999, `src/solver_handlers/installation_conversion.rs`:996).
- Files openly state they exist to dodge the cap: `src/solver_handlers_policy.rs:1-3` ("extracted from solver_handlers.rs to keep that module under the 1000-line cap"), same admission in `src/calculation_word_problem.rs:2`, `src/solver_handler_units.rs:2`, `src/solver_handler_how.rs:2`, `src/solver_helpers/code.rs:2`, `src/solver_helpers/mod.rs:665`, `src/solver.rs:294`.
- Result: a flat god-directory — **192 top-level entries in src/**, 426 .rs files, 116 `mod` declarations in `src/lib.rs`, with sibling families (`solver.rs`, `solver_dispatch.rs`, `solver_formalization.rs`, `solver_handler_how.rs`, `solver_handler_oracle.rs`, `solver_handler_units.rs`, `solver_handlers_policy.rs`, `solver_helpers/`, `solver_search.rs`, `solver_synthesis.rs`, `solver_terminal.rs`, `solver_diagnostics.rs`, `solver_unknown_reasoning.rs`) that are one logical module scattered to satisfy the lint.

Why: the lint's goal (reviewable units) is being gamed; cohesion is decided by line count, not boundaries. ARCHITECTURE.md's module story (§2 table lists ~10 modules) describes a fraction of the 116.

Draft issue:
> **Title:** Reorganize src/ into directory modules; stop counting lines as the module boundary
> **Body:** src/ has 192 top-level entries and 116 lib.rs modules; at least 7 files carry a header admitting they were split solely for scripts/check-file-size.rs. Group the solver family (solver*, meta_method_dispatch, method_registry), world_model family, dreaming family, and translation family into directory modules with mod.rs re-exports, so the split follows responsibility instead of the 1000-line cap. Update ARCHITECTURE.md's module map to enumerate the real tree.
> **Acceptance criteria:** (1) top-level src/ entries below 60; (2) no file header cites the line cap as its reason to exist; (3) ARCHITECTURE.md module map matches `ls src/` in a doc-pin test; (4) public API unchanged (cargo public-api diff empty or reviewed).

*(Further structure findings — TODO/unwrap inventory, dead code, feature gates, module-map drift — in the agent-verified section below.)*

---

### 5.2 (LOW) Debt markers and panic hygiene are genuinely excellent

Evidence: TODO/FIXME/HACK/unimplemented!/todo!() = **0** across src/ (only 3 false-positive `XXX` hits: `src/translation/wikidata.rs:232` doc text, `src/agentic_coding/report_script.rs:29,32` mktemp templates). `.unwrap()` = 12 (10 are infallible `writeln!` into String at `src/coding/blueprint_programs.rs:697-711`; 2 are length-guarded at `src/arithmetic.rs:130,476`). `panic!` = 3, all documented invariant guards (`src/change_request.rs:76`, `src/self_explanation.rs:86` — an anti-fabrication guard, `src/solver_dispatch.rs:382`). `src/server.rs`, `src/engine.rs`, `src/solver.rs`, `src/lib.rs` contain **zero** unwrap/expect/panic. This is a strength worth recording, not a finding.

Residual nits: `src/coding/catalog/templates_core.rs:344,752` `.expect("failed to read directory")` — fallible FS I/O with a pathless message; `src/draft_portfolio.rs:327` `.join().expect("draft evaluation panicked")` propagates worker panics; ~8 seed-shape `.expect(...)` sites panic the library on a malformed seed file (e.g. `src/seed/proof_programs.rs:43`).

### 5.3 (LOW) Stale dead-code allows and no-op lint suppressions

Evidence: `#[allow(dead_code)]` on heavily-used items — `src/translation/cache.rs:98` `with_online` (37 refs), `:106` `cache_dir` (82 refs) — the allows outlived their reason. Three file-level `#![allow(clippy::module_name_repetitions)]` (`src/arithmetic.rs:19`, `src/web_engine_core.rs:17`, `src/web_search_core.rs:14`) are no-ops: `Cargo.toml:83` already allows it workspace-wide. `src/web_search_core.rs:14-17` also blanket-allows three numeric-cast lints file-wide in a 900-line module.

Draft issue:
> **Title:** Remove stale dead_code allows and redundant file-level clippy suppressions
> **Body:** src/translation/ has accumulated six #[allow(dead_code)] attributes, at least two of which (with_online, cache_dir) sit on heavily used items. Three file-level module_name_repetitions allows duplicate the workspace-level setting. web_search_core.rs suppresses cast_possible_truncation/cast_sign_loss/cast_precision_loss file-wide. Delete the stale/no-op allows and narrow the cast allows to the specific expressions.
> **Acceptance criteria:** cargo clippy clean; no #[allow(dead_code)] without an adjacent comment justifying it (pattern already used in src/proof_engine/decision/sat.rs:101).

### 5.4 (MEDIUM) ARCHITECTURE.md documents ~32% of the module surface

Evidence: `src/lib.rs` declares 155 modules (+16 binary-only `cli_*` in `src/main.rs:9-24`); **105 of 155 (68%) never appear in ARCHITECTURE.md**, including whole subsystems: `mcp`, `proxy`, `client_integrations`, the `self_*` self-hosting family (`self_ast_census`, `self_explanation`, `self_healing`, `self_improvement`, `self_source_links`), `memory_query_language`, `draft_portfolio`, `external_benchmarks`. ARCHITECTURE.md has no module-map section at all (30 pipeline-concern headings). The repo already has the enforcement mechanism (doc-pin tests, e.g. `tests/unit/docs_requirements_issue_890.rs:35`) — it just isn't applied to a module map.

Draft issue:
> **Title:** Add a generated module map to ARCHITECTURE.md pinned by a doc test
> **Body:** 105 of 155 lib modules are undocumented in ARCHITECTURE.md, including the MCP surface, the proxy, and the entire self-hosting family. Add a §"Module map" section generated from src/lib.rs (one line per module: name, one-sentence responsibility, owning doc section), and pin it with the existing docs_requirements test pattern so drift fails CI.
> **Acceptance criteria:** (1) every lib.rs module appears in ARCHITECTURE.md; (2) a test asserts the doc list equals the mod list; (3) new modules fail CI until documented.

---

## Dimension 3 — Precached / seed data quality

### 3.0 Baseline correction

`src/web/seed/` **does not exist** in the tree or in git — `.gitignore:82` ignores it; `scripts/sync-seed.sh:5-6` declares it "a deploy artefact"; CI regenerates it at deploy (`.github/workflows/release.yml:1344,1912`). So there is no mirror-staleness risk by construction. Minor: `sync-seed.sh --check` mode is documented (`sync-seed.sh:12`, case-study docs) but invoked by no workflow — dead flag.

### 3.1 (HIGH) The browser loads 88 of 117 seed files — web and native answer from different lexicons

Evidence: `src/seed/embedded.rs:433-444` embeds ~109 files; `src/web/seed_loader.js` fetches 88. 29 files the browser never loads include all four `meanings-lexicon-import-0*.lino` (208 lexemes × 4 languages dropped from the browser), `multilingual-responses-summarization.lino`, `question-generation-lexicon.lino`, `sources-registry.lino`, `model-aliases.lino`, `computer-use-tasks.lino`. Nothing enforces the JS list against the Rust list.

Why: violates "Data Is The Interface" ("data/seed/ is the canonical knowledge surface for **every** interface", VISION.md:213) and cross-runtime parity claims (ROADMAP pillar 18/E34).

Draft issue:
> **Title:** Enforce seed-manifest parity between src/seed/embedded.rs and src/web/seed_loader.js
> **Body:** The browser worker loads 88 of the 117 seed files the Rust core embeds; the four meanings-lexicon-import files alone remove 832 lexemes from the web runtime. Generate both lists from one manifest (data/seed/manifest.lino) or add a CI check that diffs the embedded.rs include list against seed_loader.js and fails on divergence, with an explicit per-file opt-out annotation for genuinely native-only seeds.
> **Acceptance criteria:** (1) a single manifest drives both loaders or a test diffs them; (2) each intentionally web-excluded file carries a reason string; (3) count of silently-excluded files is 0.

### 3.2 (HIGH) Multilingual coverage is two-tier: meanings layer near-perfect, response/pattern layer broken

Evidence:
- Excellent by construction: `meanings-*.lino` totals en=844/hi=834/zh=834/ru=832; `operation-vocabulary.lino` 57/57/57/57; `shell-intents.lino` 35/35/35/35.
- Broken: **55 of 261 intents across `multilingual-responses*.lino` are English-only** — whole families: `external_benchmark_*` (25 intents, `multilingual-responses-agentic.lino:930-1050`), `statement_audit_*` (12, `multilingual-responses.lino:1400-1445`), `algorithm_*` (5, `multilingual-responses-pattern.lino`).
- `prompt-patterns.lino` totals en=61/ru=69/**hi=36**/zh=43; `how_it_works` has 6 ru vs 2 hi/zh patterns; `pattern_concept_prefix` has 24 en, 23 ru, 7 zh, **0 hi**.
- `greetings.lino`: greeting/farewell/courtesy_response have **zero hi and zero zh** surfaces (`greetings.lino:53-83`).
- `data/cache/wiktionary/` and `data/cache/wordnet/` contain **only `en/`** subdirectories (2,002 files, 8.9 MB, monolingual grounding cache).

Why: "multilingual en/ru/hi/zh by construction" is doctrine; here Hindi is systematically second-class in exactly the layer users touch first (greetings, prompt patterns, responses). A Hindi "नमस्ते" has no greeting surface at all.

Draft issue:
> **Title:** Close the en/ru/hi/zh parity gap in responses, prompt patterns, and greetings; add a parity lint
> **Body:** The meanings layer is 4-way balanced, but 55 response intents are English-only, prompt-patterns give Hindi half of Russian's coverage, and greetings.lino has zero hi/zh greetings/farewells. Because language is declared per-record in these files, a lint can enforce parity by construction: for every intent, require all four language variants or an explicit waiver record. Backfill the 55 intents and the hi/zh greeting surfaces first (the external_benchmark_* and statement_audit_* families are bounded).
> **Acceptance criteria:** (1) scripts lint fails on any intent lacking one of en/ru/hi/zh without a waiver; (2) greetings/farewell/courtesy answer in all four languages (e2e: "नमस्ते" gets a Hindi greeting); (3) prompt-pattern counts per language within ±20% per intent; (4) waiver count published and ratcheted downward.

### 3.3 (HIGH) `identity.lino` and `greetings.lino` are literal prompt→answer tables with duplicated payloads

Evidence: `data/seed/identity.lino:1-15` — three prompts ("Who are you?", "What are you?", "Tell me about yourself") each carrying the same byte-identical `answer "I am formal-ai, a deterministic symbolic AI..."`. `data/seed/greetings.lino` — 16 `answer "..."` literals, with byte-identical duplicates at `:5,10,15` ("Hi, how may I help you?" ×3), `:28,33`, `:55,60`, `:65,70`, `:75,80`. Loaded into the binary via `src/seed/embedded.rs:159-160`.

Why: NON-GOALS ("memoized answer cache is not a substitute"); VISION says responses should resolve through meanings/`response_link` indirection — the `response_link "response:identity"` field exists but the inline `answer` bypasses it, and duplication proves the data is denormalized memoization rather than linked knowledge.

Draft issue:
> **Title:** Normalize identity/greeting seeds: one response record per meaning, surfaces link to it
> **Body:** identity.lino and greetings.lino pin 19 prompt→answer pairs with byte-identical answer strings repeated up to 3×. Replace the inline `answer` fields with links to a single multilingual response record (the response_link mechanism already present), so surfaces are recognition data and answers are meanings — and hi/zh variants (finding 3.2) only need to be added once.
> **Acceptance criteria:** (1) no duplicated answer literals in seed (lint: byte-identical `answer` values > 1 fails); (2) identity/greeting answers resolve through response records; (3) existing greeting tests pass unchanged.

### 3.4 (MEDIUM) `closure-generated-*.lino`: 281 KB of English-only data whose only consumer is the metric it satisfies

Evidence: `data/seed/closure-generated-01..08.lino` — 10,228 lines, 2,044 lexemes, en=100%/ru=hi=zh=0. Loaded by nothing in `src/` or `tests/`; referenced only by its generator `scripts/close-total.py:19-21` ("Compute the unresolved value tokens over the base seed — every data/seed/*.lino file except this script's own generated output") and swept up by `scripts/audit-total-closure.py:84`'s glob.

Why: this is a self-satisfying metric — generated data that exists solely so the closure audit reports zero unresolved tokens; honest-metrics doctrine says the audit should report the real number instead. It is also the largest monolingual block in the seed.

Draft issue:
> **Title:** Stop counting closure-generated seed shards as closure; report the honest unresolved-token number
> **Body:** scripts/close-total.py generates 8 seed files (2,044 English glosses, 281 KB) that no runtime loads and whose only purpose is to make scripts/audit-total-closure.py report zero unresolved tokens. Either promote them into real, 4-language, runtime-loaded lexicon records, or exclude generated shards from the audit and let it report the true closure gap (0% acceptable per doctrine).
> **Acceptance criteria:** (1) the closure audit's denominator excludes generator output, or the shards become runtime-loaded 4-language records; (2) the audit number quoted in docs matches the recomputed honest value; (3) dead files removed if not promoted.

### 3.5 (LOW) data/ organization notes

- `data/README.md` is 24 lines and documents only `data/benchmarks/`; `data/seed/` (117 files), `cache/`, `overrides/`, `parity/`, `meta/`, `view/`, `training/` are undescribed.
- `data/overrides/` is exemplary (reason-required, CI fails on redundant overrides, `tests/unit/overrides.rs`) — the model the rest should follow; its single real override exists to add a missing Hindi label (`overrides/wikidata/entity/Q131560.lino`).
- `src/promotion.rs:47` names `data/seed/learned-program-rules.lino` which does not exist on disk (runtime write target) — `data/seed/` is simultaneously a curated read-only tree and a runtime write target; worth a doc note.
- Staleness correlates with quality: the three worst-coverage files (`identity.lino` 2026-06-09, `greetings.lino`/`prompt-patterns.lino` 2026-07-13) are the oldest.
- Benchmark seeds (`industry-suite.lino:83-95` etc.) carry `expected_contains` oracles — acceptable as test data (descriptive `expected_answer`, held-out paraphrase policy), not servable answers.

---

## Dimension 6 — Terminology doctrine (associative-only)

### 6.1 (HIGH) The terminology lint enforces ~2% of the doctrine; core public types are named GraphNode/GraphEdge/KnowledgeGraph

Evidence:
- `scripts/check-associative-terminology.rs` checks **only** the word `graph`, and only in (a) `/v1/`-prefixed route strings and (b) module names/file stems (`:38,82,211-255`). It never looks for edge/vertex/node/table/embedding; its own doc comment (`:20-22`) concedes internal graph identifiers "are neither public routes nor module names, so a route/module scan never reaches them". Wired into CI (`release.yml:393`) but omitted from CONTRIBUTING.md's local-check list (`CONTRIBUTING.md:231`).
- Public API types: `src/engine.rs:294-299` `pub struct KnowledgeGraph { pub nodes: Vec<GraphNode>, pub edges: Vec<GraphEdge> }`; 86 references to `GraphNode|GraphEdge|KnowledgeGraph|SubstitutionGraph` across 12 files (engine.rs 32, links_query.rs 15, associative_package.rs 11, world_model.rs 6, program_plan.rs 5, …).
- `src/links_query.rs` is a Cypher-style query surface that **emits the literal token `edge` into Links Notation output** (`:131,161` `out.push_str("  edge\n    from \"")`) and uses edge/node vocabulary in user-facing errors (`:274` "edge pattern must end with ->").
- The deprecated `/v1/graph` alias is allowlisted, but its name leaked into the `graphUrl` field on every client: `desktop/main.cjs:157,319`, `desktop/lib/local-server.cjs:179`, `vscode/src/lib/config.cjs:82,112`, `vscode/src/extension.node.cjs:356-357`, `src/web/app/main.jsx:4973,5249,9202` (UI label says "Network", field is still `graphUrl`).
- Seed data ships graph vocabulary as user-facing answers: `data/seed/concepts.lino:62,68,80` — `summary "A graph is a mathematical structure made of vertices and edges..."` (also duplicated, and the Hindi variant keeps the English terms untranslated); `data/view/en/graph.lino:16,20`, `data/view/en/node.lino:36`.
- Vector-`embeddings` in user-facing answer text: `src/solver_handlers/research_table.rs:454` and its JS mirror `src/web/worker/formal_ai_worker_16.js:571` ("Pattern recognition, embeddings, sequence modeling, ...").
- Self-contradicting files: `src/world_model_dialog.rs:29` and `src/world_model_atoms.rs:16` state "no graph/edge/vertex" in modules that use `edge` in doc comments (`src/world_model.rs:69-73,106,119,151,700-704`); `src/associative_persistence.rs:12-13` declares the doctrine and holds 4 graph-type references.
- Legitimate/excluded: "Vertex AI" product name (~40 of 43 vertex hits), Node.js references, parse-tree "node" (borderline), SQL surface `src/memory_query_language/sql.rs`, third-party `devalue` tables in `src/shared_dialog.rs`, "edge case" idioms.

Why: doctrine says associative-only vocabulary in code, identifiers, UI, and docs. The doctrine is *stated* in REQUIREMENTS.md:1226,1410 and enforced nowhere it matters; the recipe `data/meta/links-network-terminology-recipe.lino` (#664) claims "keeping every public surface a links network, not a graph" while the flagship serialization struct is named `KnowledgeGraph`.

Draft issue:
> **Title:** Rename Graph* public types to link-network vocabulary and widen the terminology lint to identifiers
> **Body:** check-associative-terminology.rs guards only routes and module names for "graph"; the actual violations are the public types KnowledgeGraph/GraphNode/GraphEdge (86 refs), the graphUrl client field (9 files across desktop/vscode/web), links_query.rs emitting the token `edge` into Links Notation, and seed answers defining "graph ... vertices and edges" as user-visible content. Rename types (LinkNetwork/NetworkLink or the existing links_network vocabulary), rename graphUrl → networkUrl with a deprecation shim, re-gloss the seed entries through the meaning layer (a "graph" concept may legitimately *describe* graph theory — but the assistant's own structures must not be described with it), and extend the lint to scan identifiers and struct/field names for graph|edge|vertex|embedding with a burn-down allowlist like R379's.
> **Acceptance criteria:** (1) zero Graph*-named public types; (2) lint covers identifiers with a shrinking allowlist; (3) links_query output uses link vocabulary; (4) graphUrl gone from desktop/vscode/web configs; (5) /v1/graph alias retains its documented deprecation path only.

---

## Dimension 1 (cont.) — Hardcoded-NL lint: current state of the burn-down

### 1.7 (HIGH) The R379 allowlist is flat (net +31 in 17 days), and the two biggest NL surfaces are guarded by no lint at all

Evidence:
- `scripts/check-hardcoded-language.rs` (820 lines) scans **src/**/*.rs only** (`SCAN_DIR`, `:45`); bidirectional gate (new literal fails, stale row fails) wired at `release.yml:402`. Good design.
- Allowlist: **1,353 entries across 173 files** (1,367 lines). Trend on main: 1322 (2026-07-18, gate introduced) → peak 1371 (07-30) → 1353 (08-04). **Net +31**; growth events were feature merges (#816 +48, #817). The burn-down loop works when exercised (CHANGELOG.md:951, :309, :447) but is outpaced by new features.
- Worst categories: pinned encyclopedia prose (`src/proof_engine/library.rs` 70 rows — Pythagoras/√2/Gödel proofs as verbatim strings), pinned third-party API docs in 4 languages (`src/solver_handler_docs.rs` — 772-1176-char pandas `DataFrame.join` texts), per-prompt canned cells (`research_table.rs:414-460`, allowlist:1071-1084), fabricated-looking pinned self-analysis (`src/coding/blueprint.rs` allowlist:172 — "Response self-analysis: ... functions=0, loops=0, ... complexity_score=1" with literal zeros baked in), 1.9k-char refusal blobs (`document_request.rs`).
- Coverage holes: the web lint `tests/e2e/scripts/check-web-hardcoded-ui-strings.mjs` is **not in any workflow** (only `tests/e2e/package.json:11`) and parses only `main.jsx` h() children — so the 27.7k-line worker mirror (which duplicates the pinned answers, e.g. `formal_ai_worker_16.js:571`) is guarded by **neither** lint.
- The lint catches strings, not structure: **254 `*.contains("` prompt-match sites** across src/ (`shell_command_transform.rs` 29, `capability_router.rs` 22, `solver_helpers/mod.rs` 21, `research_table.rs` 18, ...) are the unlinted special-case skeleton the strings hang on.

Draft issue:
> **Title:** Ratchet the hardcoded-language allowlist downward and extend coverage to the JS worker
> **Body:** The R379 allowlist has been net-flat since the gate landed (1322 → 1353 entries, peak 1371); features add strings faster than migrations remove them. Add a max-entries ratchet (CI fails if the entry count exceeds the last released count), wire check-web-hardcoded-ui-strings.mjs into the release workflow, and extend it (or a sibling) to src/web/worker/*.js, which currently mirrors pinned English answers with no guard. Track a companion metric for prompt-`contains` sites so the structure burns down with the strings.
> **Acceptance criteria:** (1) CI enforces entry count ≤ previous release; (2) web + worker lints run in release.yml; (3) allowlist below 1,300 within one milestone with per-family migration issues (proof_engine library → seed, solver_handler_docs → cached sources, research_table → derived); (4) `contains("` site count recorded in the ledger.

---

## Dimension 7 — The two untracked root `.lino` files

### 7.1 (MEDIUM) They are misdirected report-flow exports; gitignore lacks the rule that exists for .log siblings

Evidence:
- `formal-ai-harness-latest.lino` (311 lines, 10 KB) and `formal-ai-server-latest.lino` (157 lines, 235 KB — base64-inlined message bodies including a full OpenCode system prompt), both mtime Jul 25 22:12, untracked.
- Generated by the "Report issue" flow: `src/agentic_coding/report_issue.rs:349-363` builds `formal-ai context export --session latest --source harness|server --output formal-ai-{harness,server}-{dialog_id}.lino`; `latest` comes from `LATEST_SESSION` (`:33`) via the `:448` fallback and the CLI default `src/cli_report.rs:55`. The captured session itself contains these exact commands (base64-decoded), so the files are self-documenting: a human ran the report flow with CWD = repo root and an unresolved session id.
- Placement rule in the repo: 2,714 tracked `.lino` files, **zero at repo root**; captures live under `docs/case-studies/issue-N/...` (closest sibling: `docs/case-studies/issue-822/agent-cli-e2e/formal-ai-context.lino`).
- `.gitignore` has the two-part policy for logs (`:66` `*.log` ignored, `:70-77` `!docs/case-studies/**/*.log` un-ignored "Real logs in the case study, not synthesized" per CONTRIBUTING rule 8) — **no equivalent for `.lino`**.
- Root cause bug: `report_issue.rs:367-379` passes a bare relative filename to `--output` (no directory), while the GitHub-issue target allocates a mktemp scratch dir (`report_script.rs:29-32`). Two of four report destinations pollute the CWD; one doesn't.

Conclusion: **not meant to be committed at root.** Either file them as case-study evidence (after `scripts/check-secrets.sh` — the server capture embeds a full third-party system prompt) or delete them and close the gitignore gap.

Draft issue:
> **Title:** Report-flow harness/server exports must write to a scratch or session directory, not the CWD
> **Body:** ReportTarget::HarnessLog/ServerLog build `formal-ai context export --output formal-ai-{harness,server}-<id>.lino` with a bare relative path (report_issue.rs:367-379), so runs from a repo checkout drop session dumps at the repo root — two such files (session "latest", Jul 25) are sitting untracked there now. The GithubIssue target already uses a mktemp scratch dir (report_script.rs:29-32); apply the same to the other targets, and add root-anchored gitignore rules (`/formal-ai-harness-*.lino`, `/formal-ai-server-*.lino`) mirroring the `.log` policy at .gitignore:66-77. Decide the fate of the two stray files: file under docs/case-studies/<issue>/ with secrets check, or delete.
> **Acceptance criteria:** (1) all four report targets write into the same scratch/session directory; (2) a test asserts no report path is CWD-relative; (3) gitignore rules added with the case-study negation pattern documented; (4) the two stray files are removed from the root by either path.

---

## Dimension 8 — JS-logic doctrine inventory (owner directive: JS = glue + JSX UI only; logic = Rust/WASM)

Classification method: file-level line counts verified; glue/UI/logic classification is a sampled judgment call for the large files, spot-verified for every claim below.

### Summary table

| Surface | Total JS/JSX LOC (source) | Violating (logic-in-JS) | Notes |
|---|---|---|---|
| src/web/worker/*.js (24 shards) | 27,705 | ~27,000 | Near-total: mirrors the Rust core (Dim 2.1) |
| src/web/app/main.jsx | 9,269 | ~8,000 | 238 of 253 top-level functions are lowercase logic; only 15 React components (first at :1987); ~91 lines contain JSX |
| src/web/seed_loader.js | 1,151 | ~1,100 | Duplicates `src/seed/parser.rs`, which is **already compiled into the shipped WASM** (`src/web/wasm-worker/src/lib.rs:28-30`) |
| desktop/ | 5,828 | ~1,600 | tool-router.cjs (893) + parts of agent-chat-adapter.cjs; rest is legitimate Electron glue |
| vscode/ | 1,260 | <15 | Thin webview host — proves the doctrine is achievable |
| Committed bundles (app.js, *.bundle.js) | (artifacts) | — | Excluded from source counts; see 8.4 |
| **Total** | **~48,700** | **~38,200 (≈78%)** | |

### 8.1 (HIGH) The WASM engine is shared by all surfaces — but any failure silently reverts to the 27.7k-line JS mirror

Evidence:
- The same 281 KB `formal_ai_worker.wasm` ships to web, desktop (via `desktop/scripts/prepare-resources.mjs:55-56` wholesale copy), and vscode (webview renders committed `src/web/`, `vscode/src/lib/webview-html.cjs:7`). Genuine single-engine reuse — the doctrine's transport story is intact.
- But `src/web/worker/formal_ai_worker_20.js:1332-1339`: `try { ... WebAssembly.instantiate(bytes, {}) } catch (_error) { wasm = null; mode = "js fallback"; }` — any fetch/instantiation failure silently switches every surface to the full JS mirror, with no user-visible warning and no test distinguishing the modes' answers beyond the 5-case parity fixture.
- Consequence: the doctrine is unenforceable while the fallback exists — the JS mirror can never be deleted, and a corrupted/missing .wasm demotes all three surfaces to the unverified path without telling anyone.

Draft issue:
> **Title:** Make WASM the sole engine: hard-fail (or explicit diagnostic opt-in) instead of silent JS fallback
> **Body:** formal_ai_worker_20.js:1332-1339 silently falls back to the 27.7k-line JS mirror when WASM instantiation fails, on every surface (web/desktop/vscode share the file). This makes the JS mirror load-bearing forever and hides engine degradation from users and tests. Change the failure mode: surface a visible "engine unavailable" state (honest-metrics doctrine: a failure is a failure, not a silent downgrade), keep a diagnostic-only override for development, and emit a `policy:` event recording which engine answered. This is the enforcement precondition for every worker-absorption slice (issue #658/E39).
> **Acceptance criteria:** (1) WASM failure produces a user-visible error state, not fallback; (2) every answer records engine=wasm in its trace; (3) e2e test asserts the demo fails loudly with a stubbed 404 .wasm; (4) after this lands, migrated shard functions can actually be deleted (verified by shard line-count drop).

### 8.2 (HIGH) main.jsx is a 9,269-line logic module wearing a UI extension

Evidence: `src/web/app/main.jsx` — 238 lowercase top-level `function`s vs 15 components; first component at :1987 (the preceding ~2,000 lines are pure logic); ~91 of 9,269 lines contain JSX. Logic living here includes issue-report body construction (`createIssueReportBody`, cited from REQUIREMENTS.md R115), URL fitting (`fitIssueUrl`), desktop status normalization, memory-bundle handling, evidence-slug construction (`main.jsx:5249`). Some has Rust twins already (`src/issue_report.rs` mirrors the report body per REQUIREMENTS.md R112 — a *documented* dual implementation).

Draft issue:
> **Title:** Split main.jsx: components stay, the 200+ logic functions move to WASM calls or die as duplicates
> **Body:** src/web/app/main.jsx is 9,269 lines with 15 components and 238 logic functions; under the JS-glue/JSX-only doctrine roughly 8k lines are misplaced. Inventory the functions into (a) already-in-Rust duplicates (e.g. issue-report rendering vs src/issue_report.rs) — replace with WASM/worker calls; (b) UI-adjacent formatting — keep only what touches the DOM; (c) new logic that never had a Rust home — port. Enforce with a jsx-file budget (components + wiring only) in check-file-size.rs or the web lint.
> **Acceptance criteria:** (1) main.jsx < 2,500 lines, all top-level functions either components or event wiring; (2) issue-report bodies produced by one implementation (Rust), byte-pinned test retained; (3) no new lowercase logic functions land in app/ (lint).

### 8.3 (HIGH) seed_loader.js re-implements a parser the shipped WASM already contains

Evidence: `src/web/seed_loader.js` (1,151 lines) parses `.lino` seed files in JS; `src/web/wasm-worker/src/lib.rs:28-30` `#[path = "../../../seed/parser.rs"] mod seed_parser;` — the same parser is compiled into the .wasm the page already downloads, and lib.rs:420 exposes writes "without duplicating the parser". This is the cheapest, lowest-risk absorption in the repo: pure deletion of a duplicate with an existing Rust owner. It is also the file whose 88-of-117 manifest divergence causes finding 3.1.

Draft issue:
> **Title:** Route browser seed loading through the WASM seed parser; delete the JS parser
> **Body:** seed_loader.js maintains a 1,151-line JS .lino parser while seed/parser.rs is already compiled into the shipped formal_ai_worker.wasm. Replace the JS parsing with fetch → WASM parse calls, and drive the fetched-file list from the shared manifest (see finding 3.1) so the parity and manifest problems close together.
> **Acceptance criteria:** (1) seed_loader.js < 150 lines (fetch + version glue only); (2) seed parsing has exactly one implementation; (3) browser seed-category counts equal Rust `seed_files()` counts in an e2e assertion.

### 8.4 (MEDIUM) Security-relevant logic in unverified JS: desktop tool-router; plus committed build artifacts

Evidence:
- `desktop/lib/tool-router.cjs` (893 lines) — "Permission-gated tool dispatch for the desktop main process" (`:3`), mutable grant state (`:165`), hand-rolled path confinement (`:197-198` `path.relative(...)` + `startsWith("..")` checks, `:215-218`). This duplicates the confinement/permission concern that `src/computer_use/` and the Rust permission model own, in the only language the project's own test/lint doctrine does not cover. A path-traversal bug here is a sandbox escape on the desktop surface.
- Committed build artifacts: `package.json:6` `build:web` emits `src/web/app.js` (minified main.jsx), `vendor.bundle.js`, `web-search-component.bundle.js`, `ocr.bundle.js` — all committed to git. Reviewers cannot diff minified IIFEs; the artifact can drift from its source between builds (supply-chain and review risk), and it inflates every JS metric that forgets to exclude it.

Draft issues:
> **Title:** Move desktop tool permission/path-confinement decisions into the Rust core
> **Body:** tool-router.cjs implements grant state and path confinement in main-process JS while src/computer_use/ and the Rust permission model implement the same concepts natively. The desktop should ask the local formal-ai binary (which it already supervises) to authorize/execute tool effects, keeping JS as IPC glue. At minimum, the confinement predicate should be one shared implementation with adversarial tests (symlinks, `..` normalization, UNC paths on Windows).
> **Acceptance criteria:** (1) authorization decisions produced by Rust (unit-tested, incl. adversarial paths); (2) tool-router.cjs reduced to dispatch plumbing; (3) a red-team test suite for confinement runs in CI.

> **Title:** Stop committing minified bundles; build them in CI at deploy time
> **Body:** src/web/app.js and three *.bundle.js files are committed minified build outputs of package.json's build:web. The seed mirror already solved this correctly (src/web/seed/ is gitignored and produced by scripts/sync-seed.sh in the release workflow); apply the same policy to JS bundles so git holds only reviewable sources.
> **Acceptance criteria:** (1) bundles gitignored; (2) release/pages workflow runs build:web before artifact upload; (3) a CI check fails if a *.bundle.js or app.js is tracked.

### 8.5 (LOW) The compliant counter-examples — desktop supervision and vscode

Evidence: `desktop/dreaming.cjs` supervises the **native binary's** dreaming rather than reimplementing `src/dreaming.rs` (legitimate glue); `vscode/` (1,260 lines) hosts the committed web app with <15 lines of borderline logic (`vscode/src/lib/config.cjs:12` mirrors `normalizeDesktopStatus`). These prove the doctrine is achievable in this repo today; the violations are concentrated, not diffuse.

---

## Ranked Top 10

1. **JS mirror + silent WASM fallback (2.1 + 8.1, HIGH)** — 27.7k lines of Rust-duplicating JS that has *grown* ~1k lines since absorption was chartered; the shrink ratchet only ever re-baselined upward; the silent `js fallback` at `formal_ai_worker_20.js:1332-1339` makes the mirror permanently load-bearing. Single largest violation of both the owner's JS doctrine and pillar 18.
2. **"Only memory + meta algorithm" is ~7% migrated (4.1, HIGH)** — 50 of 54 handlers pending in the project's own ledger; 19.6k lines in solver_handlers; 16 hardcoded match-arm dispatch entries + 5 `if name ==` special cases inside the "uniform" executor; the browser worker fetches `handler-precedence.lino` and never uses it.
3. **main.jsx as a logic module (8.2, HIGH)** — 9,269 lines, 238 logic functions, 15 components; ≈8k lines of doctrine-violating JS in the app shell alone (78% of all JS LOC violates the glue/UI-only rule).
4. **Terminology doctrine unenforced at its core (6.1, HIGH)** — public types `KnowledgeGraph`/`GraphNode`/`GraphEdge` (86 refs), `graphUrl` across all three clients, `links_query.rs` emitting the token `edge` into Links Notation, seed answers teaching "vertices and edges" — while the lint checks only route/module names for the single word "graph".
5. **Memoized answer surfaces (1.1 + 1.2 + 3.3, HIGH)** — seeded canned summaries/brainstorms (`summary-topics.lino` + `benchmark_prompts.rs`), three ML-topic canned comparison tables (`research_table.rs:414-460`), and 19 duplicated prompt→answer pairs in `identity.lino`/`greetings.lino` — the exact "memoized answer table" NON-GOALS forbids.
6. **Multilingual two-tier gap (3.2, HIGH)** — 55 English-only response intents, zero hi/zh greetings/farewells, Hindi at half of Russian's prompt-pattern coverage, en-only grounding caches — against "en/ru/hi/zh by construction".
7. **R379 hardcoded-NL burn-down stalled and half-scoped (1.7, HIGH)** — allowlist net +31 since the gate landed (1,353 entries); web lint wired into no workflow; the 27.7k-line worker guarded by neither lint; 254 prompt-`contains` sites unlinted.
8. **Web/native seed divergence (3.1, HIGH)** — browser loads 88 of 117 seed files (832 lexemes missing), no manifest parity check; compounded by seed_loader.js duplicating the WASM-resident parser (8.3 — the cheapest fix in this list).
9. **Self-satisfying closure metric (3.4, MEDIUM)** — 281 KB of generated English-only seed loaded by nothing, existing solely to zero the closure audit; plus the internal 10/10 benchmark headline vs honest external 2/120 in the docs (1.4).
10. **Structural hygiene cluster (5.1 + 5.4 + 7.1, MEDIUM)** — 1000-line cap gamed by mechanical splits (18 files parked in the 900-999 warning band, distribution stops dead at 1000), 192-entry flat src/, 68% of modules undocumented in ARCHITECTURE.md, and report-flow exports dumping session captures into the CWD with no gitignore rule.

### Positive observations (for balance)
- Zero TODO/FIXME/unimplemented! debt; zero unwrap/expect/panic in engine/server/solver/lib; documented invariant panics only.
- External benchmark harness policy is exemplary honesty ("0 passed is recorded as 0 passed") and records 0s without flinching.
- `data/overrides/` (reason-required, self-pruning) and `data/meta/` recipe grounding (CI fails on drift) are strong patterns.
- Cross-surface reuse (desktop/vscode render the committed web app; one WASM binary everywhere) is the right architecture — the gap is what the shared JS payload contains, not how it is shared.
- The precedence-permutation assertion (`solver_dispatch.rs:360-396`) and the honest handler-migration ledger are real, verifiable data-driven mechanics.

*Finding counts: 13 high (1.1, 1.2, 1.3, 1.7, 2.1, 3.1, 3.2, 3.3, 4.1, 6.1, 8.1, 8.2, 8.3), 7 medium (1.4, 1.6, 3.4, 5.1, 5.4, 7.1, 8.4; 1.5 merged into 1.7), 5 low (2.2, 3.5, 5.2, 5.3, 8.5 — several recorded as strengths).*
