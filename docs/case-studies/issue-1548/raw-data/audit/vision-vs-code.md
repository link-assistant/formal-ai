# Vision vs code audit (issue #1548)

Repository audited: the working copy of this repository (branch `issue-1548-592fb16cb273`, tip `a50cc3c43`, release v0.355.34).
Method: read-only grep and file reads. No cargo build, no test suite, no GitHub calls.

Evidence labels used on every claim:

- **[CODE]** verified by reading source or committed data at the quoted `path:line`.
- **[DOCS]** stated in documentation only; not confirmed in code, or contradicted by code where noted.
- **[NOT FOUND]** searched for and not found; the search used is stated.

Paths are relative to the repository root. Issue numbers are those the repository's own docs and source comments name near the topic; their open/closed state on GitHub was not checked (no GitHub access was used).

---

## Q1. Recursion: is the meta algorithm truly recursive?

**Short answer: partly, in three disconnected places, and none of them is the recursion the README picture describes.** The solver does call itself on sub-impulses, but decomposition is a textual split on four English literals, per-leaf tests and drafts are stubs, composition is three special-case composers, and most answers leave the loop through early returns before decomposition matters. The one genuine attempt/fail/split/recompose recursion in the code base is reachable only when delegating to external agent CLIs.

### What the docs claim

- [DOCS] `README.md:76-110`: "Every prompt ... walks the same loop"; "This is not a separate subsystem bolted onto a catalogue of intents — it is the main solver path. `rust/src/solver.rs::UniversalSolver` runs the same 11-step loop for every impulse". The table maps decomposition to `record_decomposition`, tests to `record_validation`, drafts to `record_candidates`, and step 8 to "Combination of the smallest sufficient candidate".
- [DOCS] `VISION.md:342`: step 5 "split it into sub-impulses and recursively formalize each one until every sub-requirement is small enough to be solved directly"; `:343` step 6 "derive at least one executable check"; `:344` step 7 "ranking candidates by likelihood and simplicity".
- [DOCS] `ROADMAP.md:426`: "Universal 11-step solver runs for every prompt | Done (pillar 2); the recipe is data (`data/meta/recursive-core-recipe.lino`) executed by `rust/src/recipe_interpreter.rs`".
- [DOCS] `docs/requirements/doctrine-standing-doctrine-recursive-meta-algorithm-2026-10-06.md:15-26`: R1001 to R1012 all "Delivered".

### What the code does

**(a) Same-solver self-call exists, bounded depth 4, fan-out 6.**
[CODE] `rust/src/solver_synthesis.rs:67-95` `solve_sub_impulses`: `sub_config.max_decomposition_depth -= 1; let sub_solver = Self::new(sub_config);` over `sub_impulses.iter().take(6)`. Default depth is 4 (`rust/src/solver.rs:160-187`, `max_decomposition_depth 4`). This is real self-recursion.

**(b) Decomposition is a textual split, not a semantic one.**
[CODE] `rust/src/solver_helpers/mod.rs:135` `record_decomposition`: `let triggers = [" and ", " with tests", " with benchmarks", "; "];`. `max_depth` is used only as an `== 0` guard. A request with no " and " or ";" has no sub-impulses, so the recursion never starts. The triggers are English only.

**(c) "Test per leaf" (step 6) is a stub.**
[CODE] `rust/src/solver_helpers/mod.rs:372` `record_validation`: the only branch is `if lower.contains("prime")`.

**(d) "Drafts" (step 7) is a stub.**
[CODE] `rust/src/solver_helpers/mod.rs:351` `record_candidates`: `["LinkLight", "Doublet", "FormalLeaf"]` for "suggest a name", `"primary"`/`"secondary"` for "pick a", otherwise one candidate equal to the intent. `rust/src/solver.rs:414` `try_portfolio_rule(..., draft_count)` carries the comment "At the default `draft_count` of 1 this is a no-op".

**(e) "Simplification" (step 10) is a constant string.**
[CODE] `rust/src/solver.rs:815`: `log.append("trace:simplification", "smallest_sufficient".to_owned());`. `rust/src/least_action.rs` has no production caller: outside its own file the name appears only at `rust/src/lib.rs:119` (`pub mod least_action;`), as string labels at `rust/src/draft_portfolio.rs:492` and `rust/src/selection_heuristics.rs:267`, and in tests (`rust/tests/unit/issue_491_least_action.rs`, `rust/tests/unit/source_module_contracts.rs:239-284`). The architect note's own finding ("`least_action.rs` was called only from its own tests", `docs/architect-notes/2026-10-06-recursive-meta-algorithm.md:42-43`) still holds today.

**(f) Per-level composition is three hardcoded composers.**
[CODE] `rust/src/solver_synthesis.rs:97-158` `try_synthesize_from_sub_results`: `compose_algebra_substitution`, `compose_remainder_sale` (gated by `lower.contains("remainder") && lower.contains("sell")`), and `compose_compound_response` which is `answers.join("\n\n")`. They are ranked by `rank_probability_candidates` (`:139`) with fixed priors 1.15 / 1.05 / 0.9. There is no general "compose child results into the parent result" step.

**(g) The "recursive core" is a trace recorder, not an executor.**
[CODE] `rust/src/meta_core.rs:50-52`: "The recording stages are append-only: each stage appends Links Notation artifacts to the event log." `rust/src/meta_core.rs:134-150`: `record_meta_core_execution` is `#[allow(dead_code)]` with the comment "Not yet called from `src/solver.rs`: plan 05 leaf 10 asks for the call "after `meta_method_dispatch`", and `try_dispatch` *returns the answer* (`src/solver.rs:551-560`), so there is no seam after it that every path reaches."
[CODE] `rust/src/recipe_interpreter.rs:1-29`: "The interpreter changes no routing and no answer; it is a second, data-driven way to produce the same trace-only artifacts (R13)." Its only source reference outside itself is `rust/src/method_registry.rs:199-229`.
So ROADMAP.md:426 ("recipe is data ... executed by `recipe_interpreter.rs`") is true only in the sense that the recipe replays trace events.

**(h) The meta reasoner recurses one fixed level.**
[CODE] `js/worker/formal_ai_worker_meta_reasoner.js:361` `metaGround`: cycle guard `stack.includes(word)`, then `if (depth > 0) return { word, status: "open", ... }` — a gloss is grounded exactly one level deeper. `META_BOUNDS.groundDepth: 2` is declared at `:20-32` and never referenced in JS.
[CODE] `js/worker/formal_ai_worker_meta_composite.js:18` `metaSubRequests` is a sentence split; `:40` `metaReasonComposite` runs `const sub = await metaReason(request, language, settings);` once per sentence (it calls `metaReason`, not itself); `:78` `metaCompositeAnswer` concatenates `### subrequest` sections with `program: null`. One fixed level, composition by concatenation.
[CODE] Rust twin `rust/src/meta_reasoner/` (13 files, about 6,543 lines) is "ported by hand from JavaScript"; BOUNDS `ground_depth 2`, `program_length 4`, `candidate_budget 20_000`.

**(i) Genuine failure-driven recursion exists, but only for external CLIs.**
[CODE] `rust/src/recursive_execution.rs:242` `pub fn solve_recursively<E: TaskExecutor>(root: &RecursiveTask, executor: &mut E) -> RecursiveRun` with `attempt` (`:99`), `split` (`:117`), `DEFAULT_SPLIT_DEPTH_BOUND: u8 = 4` (`:228`). Its only caller is `rust/src/orchestration/incremental.rs:145` (`let run = solve_recursively(&root, &mut executor);`), "Incremental, failure-driven dispatch to external agentic CLIs". `UniversalSolver::solve` never calls it.

**(j) Summarization recurses over a directory tree.**
[CODE] `rust/src/summarization/resource.rs:5-21`: "a repository is a recursive tree of folders and files ... subdirectories by recursing into this same ..."; "Recursion depth is bounded by the summarization *mode ladder*". This is structural recursion over input data, a real decompose/summarize/compose loop, but for one task class.

### Pipelines that bypass the loop

[CODE] `rust/src/solver.rs` order inside `solve_with_history_probability_store_and_intent_cache` (`:264`). Every row except the decomposition row is an early return:

| Line | Exit |
| --- | --- |
| `:327-338` | Clarify |
| `:355-362` | `meta_reasoner::try_meta_answer` (only when `status == "solved"`) |
| `:395` | `coding::program_contract::answer` |
| `:399-402` | `record_decomposition` + `solve_sub_impulses` (the recursion) |
| `:461` | `try_program_blueprint` |
| `:491` | `operand_program::try_write_operand_program` |
| `:500` | `try_unsupported_write_program` |
| `:528` | `try_synthesize_from_sub_results` |
| `:549-561` | `meta_method_dispatch::try_dispatch` (the handler table; `rust/src/meta_method_dispatch.rs:38`, first answer wins) |
| `:572` | `rosetta_request::try_rosetta_code_request` |
| `:581-680` | Unknown branch (`try_budget_search`, `try_meta_discovery`, `record_external_search`, `answer_unknown_prompt`) |
| `:685-809` | WriteProgram via `spec.template.code` |

[CODE] JavaScript: `js/worker/formal_ai_worker_solve.js:108` `let meta = await metaReason(prompt, language, {});` followed by a first-match cascade naming 108 distinct `try*` functions; registry at `:891-1000`.
[CODE] The agentic planner (`rust/src/agentic_coding/*.rs`, 50,128 lines top level) and summarization are separate pipelines with their own control flow.

### Handler table versus loop

- [CODE] `data/seed/handler-precedence.lino`: 93 `handler` rows. `data/seed/browser-handler-precedence.lino`: 75 rows.
- [CODE] `data/meta/handler-migration-ledger.lino`: 98 rows = 59 `migrated`, 26 `justified-native`, 13 `pending`. "Migrated" means the handler's rules moved to `data/seed/handler-rules.lino` and run through `rule_interpreter`; it is still a per-task-class rule set, not the general loop.
- [CODE] `data/meta/core-boundary-ledger.lino`: `outside_core_file_count_max 50`, `outside_core_lines_max 18603` of `source_file_count_max 88`, `source_lines_max 35662` — 52% of the classified lines are outside the minimal core.
- [NOT FOUND] No ledger records the fraction of *prompts* answered by the handler table versus the loop. The nearest measurements are `data/meta/debt-ratchet.lino` (`store-read-share 3`, `try-dispatch-entries 33`, `literal-predicates 437`) and the coding ladder (`data/meta/ladder-ratchet.lino`: `coding-ladder-passing 23` of 130; `coding-ladder-l1-passing 0` of 16).

---

## Q2. Formalization and deformalization (naturalization)

**Short answer: the pipeline exists as named modules in both directions, but its measured fidelity is low and the source-code leg is about 11% translated, with the output going to test fixtures rather than replacing the hand ports.**

### Natural language to meaning and back

- [CODE] Modules: `rust/src/formalization/` (12 files, about 4,260 lines), `rust/src/translation/` (16 files), `rust/src/intent_formalization.rs` (769 lines) plus `intent_formalization/{obligations,prompt_relevants,requirements,write_program_request}.rs`.
- [CODE] `rust/src/translation/formalization.rs:306` `formalize_prompt_candidates` returns a single candidate for concept queries and translation requests; multiple candidates only for `parse_binary_relation` with `ambiguous_relation_alternatives`; otherwise a standalone subject slot. `rust/src/translation/selection.rs:155,174` ranks through `rank_probability_candidates`; `select_from_probabilities` returns `OnlyCandidate` when `candidates.len() == 1`. So "generate formalization hypotheses and select" is real machinery that usually has one hypothesis to select from.
- [CODE] Round-trip fidelity is measured and is poor: `data/meta/round-trip-translation-ratchet.lino`: `sentences 320`, `sentence-survival 0.105`, `term-survival 0.436` (20 ordered pairs over en, ru, hi, zh, es; `en-ru-sentence-survival 0.093`, `ru-en 0.125`). Roughly one sentence in ten survives NL to meaning to NL.
- [DOCS] `ROADMAP.md:131` lists "Translation through link-native meanings | Built".

### Source code to meta-language and back

- [CODE] `scripts/translate-js-rust.mjs` (1,034 lines) runs upstream `selfTranslate(source, 'JavaScript', 'Rust')` and writes to `rust/tests/fixtures/js-rust-translation/<root>/<module>.rs`. Scope: `js/agentic/crate/*.mjs` and `js/worker/formal_ai_worker_meta_*.js`. Output is a test fixture, not `rust/src`.
- [CODE] `data/meta/js-rust-translation.lino`: `modules 156`, `translated_items 316`, `workaround_items 34`, `carried_items 2463`, `translated_modules 107`, `fully_translated_modules 1`, `refused_modules 49`. 316 + 34 of 2,813 items is about 12.4% touched, 11.2% cleanly translated; 1 of 156 modules is fully translated.
- [CODE] `data/meta/translation-blockers.lino`: `carried_items 2463`, `scanned_clear 14`; 791 items have one blocker, 256 have two, up to 24.
- [CODE] `scripts/self-translate.mjs` covers only the `portable-pure-v1` fragment (`data/meta/self-translation/constructs.lino`: number/boolean/string and operators).
- [CODE] Parity between engines is maintained by hand-written twins, not translation: `data/meta/js-parity-ratchet.lino` `native-only-ceiling 0`; `data/meta/planner-twin-ratchet.lino` `no-twin-ceiling 0`; `rust/src/meta_reasoner/mod.rs` "ported by hand from JavaScript".
- [DOCS] Doctrine R1012 admits it: "it translates 1 of the 73 top-level items ... so the hand port stands" (`docs/requirements/doctrine-standing-doctrine-recursive-meta-algorithm-2026-10-06.md:26`).
- [CODE] Direction coverage: only the js to rust leg is scripted. No rust to js script was found (`ls scripts | grep -i translat` shows `translate-js-rust.mjs`, `check-js-rust-translation.mjs`, `self-translate.mjs`, `self-translation/`, `measure-round-trip-translation.mjs`, and `translate-es.mjs`, which is natural-language Spanish).
- [CODE] `data/meta/self-ast/README.md`: "## Superseded source-model authority — The owner's 2026-10-18 doctrine rejects this census/signature layout as the full source representation. These files are legacy partial inputs" (850 files remain under `data/meta/self-ast`).

### Is "naturalization" used?

Yes in docs and in one JS module; the runtime term is "deformalization".

- [DOCS] `VISION.md:120` "**The codebase is the main playground for formalization, naturalization and verified translation.**"; `GOALS.md:121`; `CONTRIBUTING.md:1108-1109` "The principle is **meanings ↔ naturalization**"; `CHANGELOG.md:631`.
- [CODE] `js/meta/comment-meaning.js` (`naturalizeParameterDocumentation`), `scripts/lib/source-cst-documentation-transform.mjs:3,129`.
- [CODE] Runtime uses "deformaliz": `rust/src/solver_handlers/formalization_task_render.rs` (17 hits), `js/worker/formal_ai_worker_formalization_request.js` (10), `rust/src/formalization/page.rs` (6), `rust/src/summarization/mod.rs` (5).
- [NOT FOUND] No Rust source file uses "naturaliz" as an API name (`grep -ri naturaliz rust/src` returns nothing; the only hit under `data/meta/self-ast/src/thinking.lino` is a mirrored JS comment).

### Missing steps

1. A general NL to meaning formalizer that produces several ranked hypotheses for arbitrary sentences (today: one S/P/O candidate in most cases).
2. A meaning to NL deformalizer whose round trip preserves sentences (today 10.5%).
3. A rust to meta-language to js leg, and a js to rust leg that covers more than 11% of items.
4. Translation output that replaces the hand ports instead of living in test fixtures.
5. One vocabulary: docs say "naturalization", runtime says "deformalization".

---

## Q3. Discovery, cache, trusted sources

**Short answer: the trusted-source registry, tiering, and cache-on-fetch are real and well built. The weak points are that the network is off by default, the Rust meta reasoner is cache-only, the coding research path that the #1165 module header says is wired into production is called only from tests, and stopping is by fixed bounds rather than a "just enough evidence" rule.**

- **Trusted sources list.** [CODE] `data/seed/sources-registry.lino` (330 lines, 20 `source` rows): wikidata, wiktionary, wordnet and others, each with `service_group external_trusted`, `settings_key`, `primacy`, `license_name` / `license_url` (for example "CC0 1.0", "CC BY-SA 3.0", "CC BY 4.0"), `cache_path data/cache/<name>/`, and a `source_tier` (`independent_corroboration`, `original_first_party` at `:212`, `original_journalism` at `:257`). The web-search row (about `:299`) "holds the weakest grade".
- **Trust ranking.** [CODE] `rust/src/relative_meta_logic.rs:186` `SourceTier`; `data/meta/search-fusion-source-policy.lino` stages `capture → formalize_each_statement → semantic_merge → source_tier_rank → smallest_conflict_complete_select → deformalize_preserving_quote`, `unoriginal_contribution zero`.
- **Automatic caching.** [CODE] `rust/src/source_fetch.rs:302-340` `fetch`: a cache hit is returned `if !self.online || age <= self.ttl_seconds`; on a live fetch `write_capture(&capture, &cache_root, &metadata_path)?;` stores the capture with sha256 and `fetched_at`. TTL default is 60 days (`rust/src/solver.rs:160-187`).
- **Network is off by default.** [CODE] `rust/src/source_fetch.rs:262`: "Cache-backed source client. Network fallback is disabled unless `online` is explicitly enabled."; `:278` `online: false`; `if !self.online { return Err(FetchError::OfflineCacheMiss(url.to_owned())); }`. `rust/src/coding/synthesis_runtime.rs:434`: `pub fn live_fetch_enabled() -> bool { crate::cli_env::flag_enabled("FORMAL_AI_LIVE_FETCH") }`. `rust/src/solver_search.rs:49` `record_external_search`: `.with_online(!offline && crate::coding::synthesis_runtime::live_fetch_enabled())`.
- **Meta reasoner is cache-only in Rust.** [CODE] `rust/src/meta_reasoner/integration.rs:41`: `CachedSourceClient::new(&cache_root, CurlSourceTransport).with_online(false)`. Also `rust/src/agentic_coding/formalization_recipe.rs:107` is `false`.
- **Where live fetch is actually on.** [CODE] `rust/src/solver_handlers/web_requests.rs:97` (true; `:81`, `:337` gated), `word_definition.rs:344`, `fact_live_answer.rs:324`, `solver_handler_how_synthesis.rs:117`, `coding/rosetta_request.rs:28` (all `!offline`), `coding/synthesis_runtime.rs:43,179,237,316` (`live`), `cli_solve.rs:329` (true).
- **`try_meta_discovery`.** [CODE] `rust/src/meta_reasoner/integration.rs:140`: runs only for goals `"synthesize_from_examples" | "synthesize_from_meaning" | "decompose"`; called at `rust/src/solver.rs:650` in the Unknown branch.
- **`research_coding_skill_gap` is test-only. Docs and code disagree.** [CODE] `rust/src/coding_research_learning.rs:492` `research_coding_skill_gap` is called only from `rust/src/discovery_production.rs:512`, inside `cached_or_research` (`:494`). `cached_or_research` is called only from `rust/tests/unit/issue_1165_discovery_production.rs:202,239`. The header of `rust/src/discovery_production.rs` says the old state was "`research_coding_skill_gap` was called only from test modules" and that the wiring points now "call [`cached_or_research`]". They do not: production uses `cached_write_program` (`:443`) and `rust/src/discovery_production_documentation.rs:363` `answer_cache_miss` → `rediscover_from_documentation`, which binds the task's expected output into a documented example from committed captures (`data/seed/coding-documentation-captures.lino`, 901 lines); on failure "`None` keeps the template".
- **`unknown_intent_research`.** [NOT FOUND] in Rust as code; it appears only as a comment at `rust/src/meta_reasoner/integration.rs:138`. It is a JS-side step.
- **The procedure cache ships empty.** [CODE] `data/cache/coding-procedure-cache.lino` (12 lines): "This file ships with no procedure rows on purpose." `data/seed/learned-program-rules.lino` is 2 lines (empty).
- **Committed captures.** [CODE] `data/cache` holds 2,989 files, 14 MB (iana, wikidata, wiktionary, wordnet); `js/source-cache` has 1 file. Budget in `data/meta/source-cache-budget.json`: `maxBytes 67108864`, `maxFiles 2048`, `maxEntries 512`.
- **"Just enough" stopping rule.** [CODE] `rust/src/source_walk.rs:20-52` `LookupBounds` defaults: `max_depth: 2, max_pages_per_service: 4, max_services: 4, max_items: 12, max_capture_age_seconds` 60 days. These are fixed ceilings. The nearest thing to a sufficiency rule is `rust/src/option_network.rs:1-40` ("research continues while the difference is non-empty and the tier ladder is not exhausted"), which applies to constraint/option questions (#781), and the meta reasoner's lazy grounding ("a lookup is spent only on a word that blocks", architect note `:58`). [NOT FOUND] a general "stop once the requirement's evidence need is met" rule on the main solve path.

---

## Q4. Requirements to tests to algorithm

**Short answer: "write a program" is answered from stored programs. There are 138 compiled Rust template literals and 151 seed template rows covering 13 tasks in 14 languages, plus curated blueprints. The only genuinely derived programs come from the meta reasoner, which emits JavaScript only, over 32 primitives, up to length 4. Tests are not generated in the requested language and not executed.**

- **Typed obligations exist, narrowly.** [CODE] `rust/src/intent_formalization/obligations.rs:56-69` `pub enum ObligationKind { OutputLiteral, CiWorkflow, ProgramFile, CodeStyle, FileNaming, CiBadge }` (issue #1166). Six kinds, all shaped by the "write a hello-world program with CI" issue class named in the module header (`:1-36`, the Kotlin Hello World case #1156). There is no obligation kind for behaviour, input/output relation, complexity, or error handling.
- **Tests in the requested language.** [CODE] `rust/src/solver_handlers/test_generation.rs:1-18` (issue #1177): "matches it to a problem shape from the seed `test_cases` records (palindrome / average / max ...) ... and renders a pytest suite. The suite is NOT executed: executing generated tests is issue #1185's scope". Output is pytest regardless of requested language, for three seeded shapes or for examples the request states. The ledger marks `test_generation` as `justified-native`.
- **Step 6 on the main path.** [CODE] `record_validation` handles only `contains("prime")` (see Q1c).
- **Reproduce a known algorithm from the internet.** [CODE] `rust/src/coding/rosetta_request.rs` (`solver.rs:572`, `with_online(!offline)` at `:28`) and `rust/src/coding/concept_discovery.rs` (`discover_with_lookup`, licences at `:598-828`) fetch from registered sources; `rust/src/coding/function_catalog/{wikifunctions,oeis,python_docs}.rs`. [DOCS] `ROADMAP.md:151`: "HumanEval 21/164 and MBPP 68/500 on the full slices (`--online`, sharded, 2026-10-07) ... GSM8K 2/20, MATH 0/20, BIG-bench object counting 0/20, CoEdIT 0/20, and SWE-bench Lite 0/1", and the derivation is Python only ("derive Python programs").
- **Generate by recursive decomposition with reuse.** [CODE] Meta reasoner: `js/worker/formal_ai_worker_meta_synthesis.js:430` `metaSynthesizeFromExamples` (`length <= META_BOUNDS.programLength`, which is 4), `:512` `metaSynthesizeFromMeaning`; primitives are the 32 `primitive` records in `data/seed/meta-reasoning.lino`, each with one JavaScript `code` string; `rust/src/meta_reasoner/answer.rs:379` hardcodes `String::from("```javascript"),`. Chunk learning is per word, with no compression pass (architect note `:72`, `:79-81`).
- **The real "write a program" path.** [CODE]
  1. `rust/src/solver.rs:395` `coding::program_contract::answer`.
  2. `:461` `try_program_blueprint` — `rust/src/coding/blueprint.rs` (574) + `blueprint_data.rs` (469) + `blueprint_programs.rs` (728): `select_blueprint` finds a recipe "that has a curated, review-quality program for the requested language"; answers are marked "not run — needs external libraries / network" (issue #340).
  3. `:491` `operand_program::try_write_operand_program` — `print_text` composed from the hello-world procedure with the operand bound (issue #1173).
  4. `:500` `try_unsupported_write_program`.
  5. `:685-809` the catalog: `spec.template.code`, optionally replaced by a `ProcedureCache` row (the cache ships empty) or `answer_cache_miss` documentation rediscovery (hello world only: `rust/src/coding/catalog/templates_core.rs` header "The hello world programs are no longer compiled here: the documentation route rediscovers them from captured pages (issue #1165 R1165-4)").
- **Catalog contents.** [CODE] `rust/src/coding/catalog/` (3,197 lines): 138 `CompiledTemplate { task_slug, language_slug, code: r#"..."# }` literals. `data/seed/hello-world-programs.lino` (978 lines): 13 tasks (`hello_world count_to_three list_files list_files_arg list_files_reverse_sort list_files_arg_reverse_sort fizzbuzz factorial reverse_string sum_to_ten fibonacci copy_stdin_to_stdout print_text`), 14 languages (`rust python javascript typescript go c cpp java csharp ruby scala kotlin php laravel`), `source "local public-domain seed templates"`, 13 `task_` rows with fixed expected `output`, 151 `template_` rows.
- **"Write an algorithm" is a canned sort.** [CODE] `data/seed/handler-rules.lino:375-428` `handler algorithm`: rule `algorithm-unnamed` is `when substring algorithm` → `value code response "algorithm_sort_code_{snippet}"`, `log "algorithm:refusal" "no operation named"`, `respond algorithm_sort`; tests are the canned `algorithm_sort_tests_{snippet}`; `log "execution_status" "unavailable"`. Snippets: `data/seed/meanings-response-intents-handlers.lino:112` (python), `:128` (rust), `:144` (javascript), `:160` (typescript); default "python when none does" (`data/seed/meanings-agent-actions.lino:549-561`). A prompt containing the word "algorithm" with no named operation gets a sort implementation.
- **Supported languages, by path.** Catalog: 14. Blueprints: those with a curated program per recipe. Algorithm handler: 4. Meta reasoner: JavaScript only. Online synthesis: Python only. Test generation: pytest only.

**Missing versus the vision:** typed behavioural obligations; test derivation from each obligation in the requested language; execution of those tests; language-independent program IR lowered to all 14 languages (`rust/src/coding/program_ir.rs`, `ir_lowering` exist but the catalog path does not go through them); reuse of learned procedures (the cache and learned-rule files are empty in the release).

---

## Q5. GitHub source reuse and licences

**Short answer: licence metadata is carried per registered source and per researched procedure, and two small gates exist. There is no code that finds source on GitHub, reads a repository's licence, or decides compatibility. The legal docs cover model/dataset intake, not source-code reuse.**

- [DOCS] `LEGAL-COMPLIANCE.md` (367 lines) is about training data, model distillation and dataset intake. Relevant lines: `:42-45` "Runtime retrieval or cache ... Its sources are governed by `data/seed/sources-registry.lino`, the source's license and access terms"; `:46-51` "Public visibility does not make it freely reusable ... A benchmark, runtime cache, or GitHub example cannot silently become training data."; `:59-61` the project is under the Unlicense.
- [DOCS] `docs/legal/` has five files (`compatible-datasets.md`, `distillable-models.md`, `formal-ai-and-language-models.md`, `public-domain-output.md`, `source-review.md`). `docs/legal/source-review.md` (67 lines) is a manual checklist ("Upstream license URL:", "Does the license affirmatively permit this training use?", "Copyleft or downstream licensing:"). None describes an automated licence check for reused code.
- [CODE] Registry-level licence: every row in `data/seed/sources-registry.lino` has `license_name` / `license_url`; `rust/src/coding/concept_discovery.rs:827-828` falls back to `String::from("license_unstated")` when the URL matches no registry row.
- [CODE] SPDX gate: `rust/src/coding_research_learning.rs:257-259`: `let license = value("SPDX-License-Identifier")?; if !valid_spdx_expression(license) { return Err("coding_research_source_license_invalid"); }`. `valid_spdx_expression` (`:293-300`) is a character-class check that rejects `NONE` / `NOASSERTION`; it does not validate against the SPDX list and has no compatibility table.
- [CODE] Commercial-reuse gate: `:937-941` `license_forbids_commercial_reuse` is true when any alphanumeric component equals "nc". `adopt_extracted_procedure` (`:859`) and `adopt_decomposed_procedure` (`:911`) are called only from tests (`rust/tests/unit/issue_1138_formalization_depth.rs:515,543`; `rust/tests/unit/issue_1164_code_example_knowledge.rs:630-652`). This whole gate sits behind `research_coding_skill_gap`, which is itself test-only (Q3).
- [CODE] Fragment reuse mode: `rust/src/coding/fragment_catalog.rs:450-457`: `let reuse = if normalized_license.contains("by-sa") || normalized_license.contains("gfdl") || normalized_license.contains("reference-only") { ReuseMode::ShapeOnly } else { ReuseMode::Verbatim };`. Any licence string not containing those three substrings, including GPL, AGPL, proprietary or an unrecognised one, defaults to `Verbatim`.
- [CODE] GitHub API use is unrelated to source reuse: `rust/src/version_resolution.rs:213,222` (releases, commits), `rust/src/cli_solve.rs:383` (issue fetch), `js/worker/formal_ai_worker_web_search_and_projects.js:629` (repository metadata).
- [NOT FOUND] Any code that searches GitHub for implementations, fetches a repository's `LICENSE` file or the `/license` API, detects an SPDX identifier from a repository, restricts reuse to public-domain / Unlicense / permissive sources, or records attribution for copied source. Searches: `grep -ril spdx rust/src js scripts data/seed data/meta` (4 files, all the coding-research contract); `grep -rIl "code search|search/code|search/repositories|repo_license|/license" rust/src js/worker js/agentic` (unrelated hits only).

**Missing:** GitHub code discovery; licence detection from the repository; an allow-list (public domain, Unlicense, CC0, MIT/Apache with attribution) with a deny-by-default rule; attribution records; a production caller for the existing gates.

---

## Q6. Requirements as axioms and relative-meta-logic

**Short answer: relative-meta-logic is not a dependency. It is a 434-line in-repo arithmetic module modelled on the upstream project, used for statement and source-trust probability. It is not used to infer missing requirement details or to score solution candidates. Candidate selection elsewhere is first-match or fixed priors.**

- **Dependency and version.** [NOT FOUND] `rust/Cargo.toml` `[dependencies]` has `links-notation = "0.23.0"`, `link-calculator = "0.22.0"`, `meta-language = { version = "1.0.0", optional = true }`, `web-capture = "0.3.37"`, `web-search = "0.6.0"` and others, and no relative-meta-logic entry. `package.json` has none. `rust/Cargo.lock` and `bun.lock` contain zero matches.
- [DOCS] `docs/associative-tech-stack.md:219-229`: "Formal AI has an in-repository implementation in `rust/src/relative_meta_logic.rs` ... The upstream repository is the architectural reference, not a linked crate." `docs/requirements-traceability.md:867`: "R1024 ... partial — relative-meta-logic named, no practice copied yet".
- **What the module is.** [CODE] `rust/src/relative_meta_logic.rs` (434 lines): "Relative statement probability, modelled on [`relative-meta-logic`]"; "Everything here is pure arithmetic over caller-supplied evidence"; `pub const ASSUMED_TRUE_PRIOR: f64 = 0.6;`; `Aggregator` (`:106`), `SourceTier` (`:186`), `Stance` (`:246`), `RelativeEvidence` (`:269`), `StatementAssessment::assess` (`:362`). Issue #535. JS twin `js/agentic/crate/relative_meta_logic.mjs`.
- **Production call sites.** [CODE] `fact_checking.rs:13`, `statement_verification.rs:18`, `world_model.rs:45`, `world_model_dialog.rs:45`, `world_model_cycles.rs:5`, `summarization/{importance,recheck,dedup,gathering}.rs`, `search_fusion.rs:15`, `concept_lookup.rs:18`, `seed/sources.rs:29`, `fact_check/{git_source,code_source,web_source}.rs`, `solver_handlers/document_originality.rs:9`, `statement_audit/*`, `reasoning_standard/trust.rs:19`, `selection_heuristics.rs:20`, `how_to_guide.rs:26`, `orchestration/analysis.rs:4`. All are statement, fact or source-trust uses.
- **(a) Inferring missing details from requirements-as-axioms.** [NOT FOUND] No call site treats requirements as axioms and derives unstated details. The Clarify early return (`rust/src/solver.rs:327-338`) asks the user instead; `guess_probability 0.8` controls ask-versus-guess.
- **(b) Scoring candidates and auto-selecting.** [CODE] `rust/src/probability.rs:646` `rank_probability_candidates` has three production callers: `rust/src/translation/selection.rs:174` (formalization candidates, usually one), `rust/src/solver_synthesis.rs:139` (three composers with fixed priors 1.15/1.05/0.9), `rust/src/statement_audit/audit.rs:190`. Program and handler selection does not go through it: the handler table is first-match-wins (`rust/src/meta_method_dispatch.rs:38`), the meta reasoner takes the shortest verified program, and `rust/src/solver.rs:815` logs the constant `"smallest_sufficient"`.
- [DOCS vs CODE] `VISION.md:120` paragraph: "Its relative meta logic evaluates conditional and dependent evidence ... to identify contradictions and determine which requirements are delivered, contradicted, partial or unknown." `GOALS.md:82`: "Treat statements as dependent under relative-meta-logic: any change recalculates the probabilities of dependent statements, visibly." No requirement-status computation over the RML module was found; requirement status lives in hand-edited tables (`docs/requirements/*.md`, `docs/requirements-traceability.md`).

---

## Q7. Multiple options and config switching

**Short answer: there are many environment switches, but almost none switch between alternative solution strategies. The knobs named "recursion mode" and "selection mode" change trace verbosity only. Option enumeration exists for one problem family (constraint/product questions) and for rule synthesis when `draft_count > 1`, which is off by default.**

- [DOCS] `docs/configuration/modes.md` (32 lines) describes only the out-of-box engine versus passthrough to an external agent CLI, and Chat / Agent / Full Auto permission levels. It lists no solver knobs. `VISION.md:352-366` lists the configurable knobs.
- [CODE] `rust/src/solver_helpers/mod.rs:811-878` `config_from_env` reads: `FORMAL_AI_OFFLINE`, `FORMAL_AI_AGENT_MODE`, `FORMAL_AI_DIAGNOSTIC_MODE`, `FORMAL_AI_DEFINITION_FUSION` (via `env_definition_fusion_by_default`), `FORMAL_AI_ASSOCIATIVE_PROJECT_PROMOTION` / `FORMAL_AI_PROJECT_PROMOTION`, `FORMAL_AI_EXECUTION_SURFACE` / `FORMAL_AI_SURFACE`, `FORMAL_AI_TEMPERATURE`, `FORMAL_AI_GUESS_PROBABILITY`, `FORMAL_AI_FOLLOW_UP_PROBABILITY`, `FORMAL_AI_CACHE_TTL_SECONDS`, `FORMAL_AI_COMPUTE_BUDGET`, `FORMAL_AI_DRAFT_COUNT`, `FORMAL_AI_BLUEPRINT_COMPOSITION` / `FORMAL_AI_PROGRAM_COMPOSITION`, `FORMAL_AI_WORLD_MODEL_MODE`, then `crate::meta_core::apply_env_modes` for `FORMAL_AI_RECURSION_MODE`, `FORMAL_AI_SELECTION_MODE`, `FORMAL_AI_SKILL_MODE` (`rust/src/meta_core.rs:160`).
- [CODE] Other switches seen: `FORMAL_AI_LIVE_FETCH`, `FORMAL_AI_CACHE_DIR`, `FORMAL_AI_SOURCE_CACHE_DIR`, `FORMAL_AI_PROCEDURE_CACHE`, `FORMAL_AI_META_LANGUAGE`, `FORMAL_AI_DREAMING`, `FORMAL_AI_FULL_TRUST`. Cargo features: `default = ["doublets-native", "equality-saturation", "meta-language"]`, plus `provenance-gpu`.
- [CODE] `SolverConfig` defaults (`rust/src/solver.rs:160-187`): `guess_probability 0.8`, `follow_up_probability 0.75`, `context_sensitivity 0.6`, `questioning_rigor 0.4`, `temperature 0.7`, `max_decomposition_depth 4`, `agent_mode false`, `offline false`, `compute_budget 512`, `draft_count 1`.
- **Mode switches that do not switch behaviour.** [CODE] `rust/src/selection.rs:31-43`: "changes trace verbosity, never routing or any answer (R13)"; `SelectionMode { Off, Record }`. `rust/src/meta_construction.rs:44-51`: `RecursionMode { Down /* Emit the downward decomposition reasoning only */, Up, Both }` — it selects which reasoning is *emitted*.
- **Where options are enumerated and compared.** [CODE] `rust/src/option_network.rs:1-40` (issue #781): "`OptionNetwork::ranked_plans` enumerates every *minimal* satisfying set"; used by `option_evidence.rs`, `source_research.rs`, `summarization/gathering.rs`. `rust/src/draft_portfolio.rs` `run_portfolio`, used by `rust/src/rule_synthesis_portfolio.rs:24` and `rust/src/solver_search/portfolio.rs:21`, active only when `draft_count > 1`. Formalization candidates (Q2). The three composers (Q1f).
- [NOT FOUND] A general mechanism that, for a solution, enumerates alternative approaches (algorithms, libraries, architectures), presents their trade-offs, and lets configuration pick one. `FORMAL_AI_BLUEPRINT_COMPOSITION` is the closest: it toggles the blueprint composition mode for one path.

---

## Q8. Plan to execution

**Short answer: the planner can execute three narrow shapes in a workspace — write a literal file, capture a quoted command's output, or write a catalog `write_program` recipe. Any other repository work item is recorded as a plan and ends `planned_not_executed`. This is the designed behaviour, quoted below.**

- [CODE] `rust/src/agentic_coding/general_planner.rs:44-51` (doc: "What the bounded general planner can truthfully execute."): `pub enum GeneralPlanMode { LiteralFile, CommandOutput, RepositoryWorkItem }` with comments "Write literal request content to a workspace-relative file", "Capture the output of an explicitly quoted command in a file", "Persist a referenced repository work item without fabricating a patch".
- [CODE] `:72-77` `PlanTerminalState { Executed, PlannedNotExecuted }`.
- [CODE] `:378-417` `compose_repository_work_plan` builds `[work_item_step(Capability::Fetch, "read", ...), work_item_step(Capability::Write, "action", ..., PLAN_PATH)]`, `verification_command: String::new()`, `terminal_state: PlanTerminalState::PlannedNotExecuted`. The "action" step writes the plan file, not the change.
- [CODE] Deciding code, `rust/src/agentic_coding/general_execution.rs:786-797` `finish_general_change`: `if plan.mode == GeneralPlanMode::RepositoryWorkItem { return record(AgenticPlan::Final(plan.planned_not_executed_answer()), FinalDisposition::Gap, "repository_work_not_executed", result); }`.
- [CODE] `:294-303`: `return record(AgenticPlan::Final(unresolved.planned_not_executed_answer()), FinalDisposition::Gap, "work_item_execution_unavailable", result);`.
- [CODE] `:505-541` `plan_work_item_execution`: `let mut answer = crate::solver::UniversalSolver::default().solve(objective);` (comment: "the recipe wanted here is the same `write_program` answer a user typing this request directly would get, and the client — not this solver — owns execution"), then `let Some(executable) = compose_general_change_plan(objective).filter(|executable| executable.mode != GeneralPlanMode::RepositoryWorkItem) else { return plan_work_item_evidence(...) };`.
- [CODE] JS twin: `js/agentic/general_execution.mjs:518` `if (plan.mode === GeneralPlanMode.RepositoryWorkItem) return resolvedFinalAnswer(plannedNotExecutedAnswer(plan), FinalDisposition.Gap, 'planned_not_executed');` and `:524` `if (progress.successfulRunCountFor(plan.verification_command) === 0) return resolvedFinalAnswer(generalPlanUnverified(plan), FinalDisposition.Gap, 'planned_not_executed');`. `js/worker/formal_ai_worker_obligations.js:25`: `["general_plan_mode", "repository_work_item", "underivable", "", "planned_not_executed_has_no_verification_command"]`.
- [CODE] Data contract: `data/meta/general-change-plan.lino`: `terminal_state "executed|planned_not_executed"`; invariant `planned_not_executed "a plan whose steps touch no requested artifact ends planned, not successful"`. `data/meta/workspace-change-execution-policy.lino`: `candidate_effect inert_until_reviewed`.
- **Conditions for execution.** The fetched issue text must reduce to (1) a literal-file write, (2) a quoted-command capture, or (3) a `write_program` request the catalog answers. Everything else is a plan record.
- **Edit, run, commit primitives exist.** [CODE] `rust/src/agentic_coding/` has per-shape edit modules (`line_removal.rs`, `literal_addition.rs`, `markdown_section.rs`, `link_edit_rules.rs`), `git_commit.rs`, `git_push_recovery.rs`; 50,128 lines at top level. The capability is there per shape; the general bridge from a plan step to an edit is not.
- **Measured effect.** [CODE] `data/meta/ladder-ratchet.lino`: `leaf_nodes_selected 32`, `leaf_nodes_passing 16`, `deepest_passing_level none`; `coding-ladder-tasks 130`, `coding-ladder-passing 23`, `coding-ladder-l1-tasks 16`, `coding-ladder-l1-passing 0`. `data/meta/self-hosting-ledger.lino` release defaults: `self_authored_lines "0"`, `percentage_basis_points "0"`, `target_percentage_basis_points "389"` across 30 release rows (v0.355.5 to v0.355.34).
- Issues named in this code: #904, #1154, #1155, #1165, #1133, #1085.

---

## Q9. Hardcoded solutions inventory

### Counts

| Measure | Value | Source |
| --- | --- | --- |
| Handler rows, Rust precedence | 93 | `data/seed/handler-precedence.lino` |
| Handler rows, browser precedence | 75 | `data/seed/browser-handler-precedence.lino` |
| Ledger rows | 98 = 59 migrated + 26 justified-native + 13 pending | `data/meta/handler-migration-ledger.lino` |
| Distinct `try*` functions in JS cascade | 108 | `js/worker/formal_ai_worker_solve.js` |
| `rust/src/solver_handlers` | about 75 files, 34,406 lines (30,332 top level) | `wc -l` |
| `js/worker` | 167 files, 61,999 lines | `wc -l` |
| `rust/src/agentic_coding` top level | 50,128 lines | `wc -l` |
| Compiled program templates | 138 literals | `rust/src/coding/catalog/` |
| Seed program templates | 151 rows, 13 tasks x 14 languages | `data/seed/hello-world-programs.lino` |

### Committed ratchet numbers

`data/meta/debt-ratchet.lino`: `handler-files 47`, `handler-migration-pending 13`, `literal-predicates 437`, `hardcoded-language-rows 677`, `authored-ladder-rules 32`, `try-dispatch-entries 33`, `language-parity-gaps 42`, `store-read-share 3`.
`data/meta/js-literal-ratchet.lino`: `worker-ceiling 431`, `meta-ceiling 0`.
`data/meta/core-boundary-ledger.lino`: `outside_core_file_count_max 50`, `outside_core_lines_max 18603`; 50 `migrate`, 38 `promote`.

### Task classes answered by hardcoded handlers or canned templates

"Ledger" is the status in `data/meta/handler-migration-ledger.lino`. Sizes are line counts of the main file. Two kinds of hardcoding are distinguished in the "What" column: **canned** means the output text itself is stored; **procedure** means a hand-written Rust procedure for that one task class (cue phrases and word maps usually in seed data, the recognition-extraction-composition logic in Rust). Rows marked procedure were characterised from the module header, not a line-by-line read.

| Task class | Handler / path | File | Size | What is hardcoded | Ledger |
| --- | --- | --- | --- | --- | --- |
| Write a program (13 tasks) | catalog | `rust/src/coding/catalog/*` + `data/seed/hello-world-programs.lino` | 3,197 + 978 | Canned: complete programs per task and language, fixed expected output | `write_program_concrete`, `write_script`, `write_program_coreference`: pending |
| Larger program requests | `try_program_blueprint` | `rust/src/coding/blueprint*.rs` | 1,771 | Canned: curated whole programs per capability set | `program_blueprint_from_prompt`: pending |
| "Algorithm" | `algorithm` rule set | `data/seed/handler-rules.lino:375-428` + `meanings-response-intents-handlers.lino:112-160` | about 55 rule lines | Canned: sort snippet and its tests in 4 languages; default for any unnamed algorithm | migrated |
| Software project | `software_project` | `rust/src/solver_handlers/software_project.rs` | 850 | Procedure: "build/write/create an extension/app/bot/tool" prompts; plan first, "Code is returned only after the user approves the plan" | pending |
| Test generation | `test_generation` | `rust/src/solver_handlers/test_generation.rs` | 617 | Procedure + canned shapes: pytest for palindrome / average / max, not executed (#1177) | justified-native |
| SQL | `sql_synthesis` | `.../sql_synthesis.rs` | 660 | Procedure: "composes a single SELECT" from table, columns, filters, ordering, limit, aggregates (#1177) | justified-native |
| Regex | `regex_synthesis` | `.../regex_synthesis.rs` | 497 | Procedure: character-class constraints to "an anchored pattern" (#1177) | justified-native |
| Shell command | `shell_command_compose` | `.../shell_command_compose.rs` | 818 | Procedure: composes a command from the flag table in `data/seed/manual-pages.lino` (#1177) | justified-native |
| Code explanation | `code_explanation` | `.../code_explanation.rs` | 600 | Procedure: each line matched against the construct table in `data/seed/meanings-code-structure-explanations.lino` (#1177) | justified-native |
| Code debugging | `code_debugging` | `.../code_debugging.rs` | 571 | Procedure: derives the promised property from the function NAME via seeded `function_intent` records (#1177) | justified-native |
| Code review / refactoring | `code_review`, `code_refactoring` | `.../code_review.rs`, `code_refactoring.rs` | 285 + 279 | Procedure (header not read) | justified-native |
| Statistics | `statistics` | `.../statistics.rs` | 975 | Procedure: mean/median and similar over number lists stated in the prompt (#1176) | justified-native |
| Unit conversion | `unit_conversion` | `.../unit_conversion.rs` | 593 | Procedure (header not read) | justified-native |
| Compound interest | `compound_interest` | `.../compound_interest.rs` | 488 | Procedure for one formula family (header not read) | migrated |
| Calendar | `calendar_*` | `.../calendar.rs` | 662 | Procedure (header not read) | migrated |
| Text manipulation | `text_manipulation` | `.../text_manipulation.rs` | 949 | Procedure (header not read) | migrated |
| Text rewrite | `text_rewrite` | `.../text_rewrite.rs` | 850 | Procedure (header not read) | justified-native |
| Format conversion | `format_conversion` | `.../format_conversion.rs` | 840 | Procedure: JSON to YAML and back for a YAML subset (#1177) | justified-native |
| Installation conversion | `installation_conversion` | `.../installation_conversion.rs` | 829 | Procedure (header not read) | migrated |
| Creative writing | `creative_writing`, `creative_composition`, `creative_writing_planning` | `.../creative_*.rs` | 637 + 815 + 482 | Procedure: brainstorming composes name candidates from topic words; advice (#1178) | justified-native (creative_writing, brainstorm_composition, advice_request) |
| Web search / requests | `web_search`, `http_fetch`, `url_navigate` | `.../web_search_intent.rs`, `web_requests.rs` | 910 + 858 | Procedure (header not read) | migrated |
| Verifiable task | `verifiable_task` | `.../verifiable_task.rs` | 795 | Procedure (header not read) | migrated |
| Name suggestions | `record_candidates` | `rust/src/solver_helpers/mod.rs:351` | few lines | Canned: `["LinkLight", "Doublet", "FormalLeaf"]` | not tracked |
| Prime check validation | `record_validation` | `rust/src/solver_helpers/mod.rs:372` | few lines | `contains("prime")` | not tracked |
| Remainder-sale word problem | `compose_remainder_sale` | `rust/src/solver_synthesis.rs:97-158` | part of file | `contains("remainder") && contains("sell")` | not tracked |
| "kupi slona" | `kupi_slona` | `data/seed/handler-rules.lino:187-193` | about 7 rule lines | Canned: one idiom (`log "kupi_slona:idiom"`, `respond kupi_slona`) | migrated |
| Repository edits (agentic) | per-shape modules | `rust/src/agentic_coding/{line_removal,literal_addition,markdown_section,link_edit_rules}.rs` | part of 50,128 | One module per edit shape; `authored-ladder-rules 32` | debt-ratchet |

All 26 `justified-native` names: advice_request, arithmetic, brainstorm_composition, code_debugging, code_explanation, code_refactoring, code_review, creative_writing, formalization_request, format_conversion, javascript_execution, legality_warning, page_query_text, planning_request, product_search, regex_synthesis, repository_lineage, shell_command_compose, sql_synthesis, statistics, summarization_text, test_generation, text_rewrite, triz_resolution, unit_conversion, word_problem.

All 13 `pending` names: conversation_memory, current_dialogue_fact_checking, exact_memory_query, historical, learn_from_source, link_native_synthesis, program_blueprint_from_prompt, proof_request, software_project, software_project_request, write_program_concrete, write_program_coreference, write_script.

Two structural observations:

1. Every coding-related task class is either `pending` or `justified-native`. The ledger's "justified-native" category gives the classes most central to the vision (tests, review, debugging, refactoring, SQL, regex, shell) a permanent exemption from migration.
2. [DOCS] `docs/meta-algorithm.md:1-7` is titled "Meta-Algorithm: Reproducing Topic Handlers On Demand" and defines the meta-algorithm as "a fixed, ordered procedure that reproduces a deterministic chat intent handler for any language topic"; `:938` "Every handler above reaches an answer by *recognising* a request and *reusing* a catalogued method." The project's own definition of "meta-algorithm" in that document is a recipe for writing more handlers.

---

## Q10. Contradictions

### Docs versus code

| # | Doc claim | Code / data |
| --- | --- | --- |
| 1 | `README.md:76-110` "runs the same 11-step loop for every impulse" | Eleven early-return exits in `rust/src/solver.rs:327-809`; steps 6, 7, 10 are stubs (`solver_helpers/mod.rs:351,372`; `solver.rs:815`) |
| 2 | `ROADMAP.md:426` "Universal 11-step solver runs for every prompt — Done ... executed by `recipe_interpreter.rs`" | `rust/src/recipe_interpreter.rs:1-29` "changes no routing and no answer ... trace-only artifacts"; `rust/src/meta_core.rs:134-150` execution pass is dead code |
| 3 | `rust/src/discovery_production.rs` header: wiring points "call [`cached_or_research`]" | `cached_or_research` called only from `rust/tests/unit/issue_1165_discovery_production.rs:202,239` |
| 4 | Doctrine R1002 "with depth and cycle bounds" | `META_BOUNDS.groundDepth` never referenced in `js/worker/formal_ai_worker_meta_reasoner.js`; depth is the literal `if (depth > 0)` |
| 5 | Doctrine R1005 / R1010 `worker-ceiling 1106` | `data/meta/js-literal-ratchet.lino` `worker-ceiling 431` (doc is stale; the number improved) |
| 6 | Architect note `:38` "the 90 handler rows" | 93 rows today |
| 7 | `ROADMAP.md:431,444` "the #848 ladder measured 65 of 130 tasks at v0.320.0 (`data/meta/ladder-ratchet.lino`)" | Same file: `coding-ladder-passing 23`, with `coding-ladder-correction "corrected overcount"`; `coding-ladder-l1-passing 0` of 16 |
| 8 | `ROADMAP.md:427` "49 outside-core sources and 19,741 outside-core lines" | `data/meta/core-boundary-ledger.lino`: 50 files, 18,603 lines |
| 9 | `VISION.md:120` paragraph: relative meta logic determines "which requirements are delivered, contradicted, partial or unknown" | RML call sites are statement/fact/source-trust only (Q6); `docs/requirements-traceability.md:867` "no practice copied yet" |
| 10 | `VISION.md:438-449` translation through meaning; `ROADMAP.md:131,132` "Built" | `sentence-survival 0.105`; js to rust 11.2% of items, 1 of 156 modules |
| 11 | `ROADMAP.md:151` item 26 "General synthesis ... Built" | Same row: HumanEval 21/164, MBPP 68/500, MATH 0/20, SWE-bench Lite 0/1 |
| 12 | Self-hosting target 389 basis points | `data/meta/self-hosting-ledger.lino`: `self_authored_lines "0"`, `percentage_basis_points "0"` for the release defaults |
| 13 | Doctrine R1001 "The general loop runs on every turn ahead of the handler table, not as a fallback" | It runs first, but returns only on `status == "solved"` (`rust/src/meta_reasoner/integration.rs:100-126`); otherwise the handler table answers. Running first and answering first are different things |
| 14 | `LEGAL-COMPLIANCE.md:42-51` implies source licences govern runtime reuse | `rust/src/coding/fragment_catalog.rs:450-457` defaults unknown licences to `ReuseMode::Verbatim`; adoption gates have no production caller |

### Doc versus doc

| # | Doc A | Doc B |
| --- | --- | --- |
| 1 | `README.md` "not a separate subsystem bolted onto a catalogue of intents" | `docs/meta-algorithm.md:1-7` defines the meta-algorithm as a procedure that "reproduces a deterministic chat intent handler" |
| 2 | Doctrine R1001-R1012 all "Delivered" | The architect note the doctrine cites lists what is still weaker: "No compression pass", "No SMT solver", "Coverage of everyday programming tasks is much narrower" (`:72-81`) |
| 3 | `VISION.md:120` relative meta logic as a working capability | `docs/associative-tech-stack.md:219-229` "architectural reference, not a linked crate"; traceability R1024 "partial" |
| 4 | `VISION.md` / `GOALS.md` "naturalization" | Runtime docs and code say "deformalization" |
| 5 | `ROADMAP.md:426` "Done" for the universal solver | `ROADMAP.md:427` "Partial ... remaining handler logic must still migrate into generic interpreters" — adjacent rows |
| 6 | Owner quote "no solution is hardcoded" (architect note `:13`) | `data/seed/hello-world-programs.lino` `source "local public-domain seed templates"`; ledger category `justified-native` (26 handlers) |
| 7 | `data/meta/self-ast/README.md` "Superseded source-model authority" | 850 files remain committed under `data/meta/self-ast` and are still read |

---

## Top 10 things most wrong that block the vision

Ranked by how much each one blocks "a recursive meta algorithm that discovers, composes and verifies solutions instead of storing them".

**1. The 11-step loop is a trace, not the control flow.**
Steps 5 to 10 are recorded as events while the answer comes from an early return. Decomposition is a split on `[" and ", " with tests", " with benchmarks", "; "]` (`rust/src/solver_helpers/mod.rs:135`); validation is `contains("prime")` (`:372`); candidates are three literal names (`:351`); simplification is a constant string (`rust/src/solver.rs:815`); `least_action` has no caller; `record_meta_core_execution` is dead code (`rust/src/meta_core.rs:134-150`); the recipe interpreter "changes no routing and no answer". README and ROADMAP:426 say "Done".
Issue: #559 (meta-core mandate), #491 (least action), #1073 (reasoning depth). No issue found for "make the loop the executor".

**2. Programs are stored, not derived.**
138 compiled templates plus 151 seed rows for 13 tasks x 14 languages; curated blueprints; a canned sort for any "algorithm" request (`data/seed/handler-rules.lino:375-428`). Every coding handler is `pending` or `justified-native`. The owner's instruction was "no solution is hardcoded".
Issue: #1165 (discovery in production), #914 (coding via formal reasoning), #340 (blueprints), #1173.

**3. The discovery path the docs call production is test-only, and the learned caches ship empty.**
`cached_or_research` and `research_coding_skill_gap` are reached only from `rust/tests/unit/issue_1165_discovery_production.rs`; `data/cache/coding-procedure-cache.lino` "ships with no procedure rows on purpose"; `data/seed/learned-program-rules.lino` is empty. The network is off unless `FORMAL_AI_LIVE_FETCH` is set and the Rust meta reasoner is `with_online(false)`. So "discover, cache, reuse" does not run for a normal user.
Issue: #1165, #1138, #710.

**4. Plans do not become changes.**
A repository work item ends `planned_not_executed` unless it reduces to a literal file, a quoted command, or a catalog program (`rust/src/agentic_coding/general_execution.rs:786-797`). Result: coding ladder 23/130, L1 0/16, agent ladder `deepest_passing_level none`, self-authored share 0 against a 389 bp target.
Issue: #904, #1154, #1155, #1085, #848.

**5. Recursion is real only where Formal AI delegates to someone else.**
`solve_recursively` (attempt, fail, split, recompose) has one caller, external-CLI orchestration (`rust/src/orchestration/incremental.rs:145`). The solver's own recursion fans out on a textual split and composes with three special cases; the meta reasoner goes one level and concatenates.
Issue: #1188 (PR; doctrine R1001-R1012), #703 (orchestration). No issue found for using `solve_recursively` inside `UniversalSolver`.

**6. Tests are not derived from requirements, not written in the requested language, and not run.**
Six obligation kinds, all about hello-world-plus-CI (`rust/src/intent_formalization/obligations.rs:56-69`); test generation emits pytest for three shapes and "The suite is NOT executed" (`rust/src/solver_handlers/test_generation.rs:1-18`). Without this, step 6 cannot gate step 7, and no candidate can be verified against the request.
Issue: #1166 (obligations), #1177 (test generation), #1185 (executing generated tests).

**7. Selection is first-match-wins, and relative-meta-logic is not doing the job the vision gives it.**
The handler table returns the first answer (`rust/src/meta_method_dispatch.rs:38`); composers use fixed priors; RML is an in-repo emulation used for fact and source trust, never for inferring missing requirement details or scoring solution candidates; `SelectionMode` "changes trace verbosity, never routing or any answer".
Issue: #535 (RML), R1024 "partial". No issue found for requirement-as-axiom inference.

**8. Formalization and translation fidelity is too low to carry the vision.**
NL round trip keeps 10.5% of sentences; js to rust translates 11.2% of items and 1 of 156 modules, into test fixtures; both engines are kept in step by hand twins (about 62k lines of JS worker against the Rust port). Every feature is therefore written twice by hand, which is the opposite of "code itself".
Issue: #526, #917 (NL translation), R1000 / meta-language#196 (code translation), #890.

**9. No licence-aware source reuse.**
No GitHub discovery, no repository licence detection, no allow-list; the one reuse-mode gate defaults unknown licences to `Verbatim` (`rust/src/coding/fragment_catalog.rs:450-457`) and the adoption gates have no production caller. The vision's "reuse public-domain / permissive source from GitHub" has no implementation to audit.
Issue: #1138 (plan 04 L10 adoption gate), #1164 (code example knowledge). No issue found for GitHub licence checking.

**10. Status reporting overstates delivery, which hides items 1 to 9.**
"Delivered" on all of R1001-R1012; "Done" at ROADMAP:426; "Built" for general synthesis with 0/20 and 0/1 rows in the same cell; a module header describing wiring that does not exist; stale numbers (65/130 versus 23/130; 1106 versus 431; 90 versus 93 rows). The ratchets themselves are honest and precise — the prose built on top of them is not, and the `justified-native` ledger category converts 26 handlers from debt into accepted state.
Issue: #918 (core-boundary ledger), #657 / #924 (self-hosting metric). No issue found for a docs-versus-ledger consistency gate.

### What is in good shape (for balance)

- The ratchet/ledger discipline: debts are counted, committed, and shrink-only, and several ledgers record honest corrections (`coding-ladder-correction "corrected overcount"`).
- The trusted-source registry with tiers, licences, sha256 captures and cache-on-fetch.
- The meta reasoner's design (grounding, shortest-first typed enumeration, example verification, chunk learning, zero natural-language literals in its modules) is the right seed; it is small (32 JS primitives, length 4), not wrong.
- `recursive_execution.rs` is a correct general attempt/split/recompose engine waiting for an internal executor.
- The planner refuses to fabricate patches and says `planned_not_executed` instead of claiming success.
