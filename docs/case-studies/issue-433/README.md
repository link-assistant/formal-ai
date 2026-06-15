# Case study: issue #433

> Source issue: <https://github.com/link-assistant/formal-ai/issues/433>
> Branch: `issue-433-6671aae1da9e` - PR: #434
> Follow-up to: issue #423 / PR #424 (stacked — this branch contains #424's
> `installation_conversion` handler so the recognizer can be generalized here).
> Raw data: [`raw-data/`](./raw-data) (issue JSON, issue comments, PR JSON)

## Summary

Issue #433 is the generalization pass spun out of #423. The reviewer asked that
recognizers across the solver be *truly general* — derived compositionally from
structure — rather than *memoized*: answered by matching against a hardcoded
enumeration that has to be extended one literal at a time.

The issue names three concrete memoization-flavored spots and sets three
acceptance criteria. This case study delivers all three:

| # | Acceptance criterion | Status |
|---|----------------------|--------|
| AC1 | A written audit listing each handler that relies on a fixed enumeration vs. compositional rules. | Done — see [Audit](#ac1-handler-recognizer-audit). |
| AC2 | At least the installation-conversion command recognizer generalized away from the prefix whitelist, with no regression in the 100-repo corpus and added false-positive (prose) coverage. | Done — see [Generalization](#ac2-installation-conversion-command-recognizer). |
| AC3 | A documented plan (or prototype) showing one existing coding handler reconstructed from the meta-algorithm rule primitives. | Done — see [Reconstruction](#ac3-reconstructing-numeric_list-from-the-meta-algorithm-primitives). |

## What "memoization vs reasoning" means here

A **memoized** recognizer answers by matching against a hardcoded enumeration:
to support one more case you extend the list. A **compositional** recognizer
derives the answer from structure — token shape, provenance, grammar, or a
data-driven seed vocabulary — so unseen-but-well-formed inputs are handled
without editing a table.

The distinction is not "uses any literal" — almost every recognizer mentions
some literal. It is *where the closure lives*: does an unseen-but-valid input
fail purely because it is absent from a list (memoized), or is it accepted
because it has the right shape (compositional)?

## AC1: Handler recognizer audit

Each entry in `src/solver_dispatch.rs::SPECIALIZED_HANDLERS` (and the
`try_contextual_override` history-aware variants) was read at its recognizer —
the gating logic that decides "does this prompt belong to me". Classification:

- **FIXED ENUMERATION** — gating is dominated by a hardcoded literal
  whitelist / substring / prefix / keyword table. Unseen-but-valid inputs fail
  for absence from the list.
- **COMPOSITIONAL** — gating reasons about structure or reads a data-driven
  seed vocabulary (`data/seed/*.lino`, semantic `ROLE_*` meanings).
- **HYBRID** — a meaningful mix (compositional core with a small hardcoded
  side table, or vice versa).

| Intent | Handler fn | Recognizer location | Class | Evidence |
|--------|-----------|---------------------|-------|----------|
| http_fetch | `try_http_fetch` | `solver_handlers/web_requests.rs` | COMPOSITIONAL | `role_evidences_web_intent` over seed `ROLE_HTTP_FETCH` |
| url_navigate | `try_url_navigate` | `solver_handlers/web_requests.rs` | COMPOSITIONAL | `is_url_navigate_prompt`: seed `ROLE_URL_NAVIGATE` + bare-URL shape |
| web_search | `try_web_search` | `solver_handlers/web_search_intent.rs` | COMPOSITIONAL | `extract_web_search_request` over `ROLE_WEB_SEARCH_*` |
| research_comparison_table | `try_research_comparison_table` | `solver_handlers/research_table.rs` | COMPOSITIONAL | seed `ROLE_COMPARISON_TABLE_TRIGGER` + difference cue |
| docs_method_explanation | `try_docs_method_explanation` | `solver_handler_docs.rs` | HYBRID | data-driven `ROLE_CODE_METHOD_NOUN` + hardcoded `pandas.DataFrame.join` identifiers |
| procedural_how_to | `try_how_to_procedure` | `solver_handler_how.rs` | COMPOSITIONAL | seed `ROLE_PROCEDURAL_REQUEST` prefix forms |
| conversation_memory | `try_conversation_memory` | `solver_handlers/mod.rs` | HYBRID | seed conversation-summary roles + hardcoded "what is my name" substrings |
| software_project_followup | `try_software_project_followup` | `solver_handlers/software_project_followup.rs` | COMPOSITIONAL | seed `ROLE_SOFTWARE_FOLLOWUP_*` |
| summarization | `try_summarization_request` | `solver_handlers/benchmark_prompts.rs` | COMPOSITIONAL | data-driven `SummaryTopicSeeds` `matches_trigger` |
| text_manipulation | `try_text_manipulation` | `solver_handlers/text_manipulation.rs` | COMPOSITIONAL | seed `operation_vocabulary()` semantic match |
| brainstorming | `try_brainstorming_request` | `solver_handlers/benchmark_prompts.rs` | COMPOSITIONAL | data-driven `BrainstormSeeds` |
| conversation_topic | `try_conversation_topic_request` | `solver_handlers/benchmark_prompts.rs` | COMPOSITIONAL | seed `ROLE_CONVERSATION_TOPIC_OPENER` |
| fact_lookup | `try_fact_lookup` | `solver_handlers/benchmark_prompts.rs` | COMPOSITIONAL | seed-based fact trigger detection |
| coreference | `try_coreference_request` | `solver_handlers/benchmark_prompts.rs` | COMPOSITIONAL | seed-based coreference recognition |
| roleplay | `try_roleplay_request` | `solver_handlers/benchmark_prompts.rs` | COMPOSITIONAL | seed-based roleplay trigger |
| translation | `try_translation` | `solver_handlers/mod.rs` | COMPOSITIONAL | `is_translation_request`: seed `ROLE_TRANSLATION_ACTION` (typology-aware) |
| capabilities | `try_capabilities` | `solver_handlers/user_intent.rs` | COMPOSITIONAL | seed `ROLE_CAPABILITY_QUERY` |
| calendar_reasoning | `try_calendar_reasoning` | `solver_handlers/calendar.rs` | COMPOSITIONAL | seed `ROLE_CALENDAR_*` roles |
| compound_interest | `try_compound_interest` | `solver_handlers/compound_interest.rs` | COMPOSITIONAL | three independent seed cues (investment/interest/compounding) |
| numeric_list | `try_numeric_list` | `solver_handlers/numeric_list/mod.rs` | COMPOSITIONAL | `detect_operation` over seed `operation_vocabulary()` |
| number_constraint_reasoning | `try_number_riddle` | `solver_handlers/number_riddle.rs` | **FIXED ENUMERATION** | `looks_like_number_riddle`: hardcoded multilingual arrays (`["число","number","integer"]`, identity/bounds phrases) |
| arithmetic | `handle_arithmetic` | `solver_handlers/mod.rs` | COMPOSITIONAL | `calculation_expression_candidates` data-driven parse |
| javascript_execution | `handle_javascript_execution` | `solver_handlers/mod.rs` | COMPOSITIONAL | `extract_javascript_program` structural code detection |
| definition_merge | `try_definition_merge` | `solver_handlers/definition_merge.rs` | COMPOSITIONAL | seed `ROLE_DEFINITION_MERGE_ACTION` |
| concept_lookup | `handle_concept_lookup` | `solver_handlers/mod.rs` | COMPOSITIONAL | `extract_concept_query` data-driven |
| who_is | `try_who_is_question` | `solver_handlers/user_intent.rs` | HYBRID | data-driven `WHO_QUESTION_LEAD/TAIL` + hardcoded celebrity typo table |
| how_it_works | `try_how_it_works` | `solver_handler_how.rs` | COMPOSITIONAL | concept lookup + structural topic extraction |
| meta_explanation | `try_meta_explanation` | `solver_handlers/meta_explanation.rs` | COMPOSITIONAL | data-driven `ROLE_*` (why/how-you-work/architecture) |
| network_query | `try_network_query` | `solver_handlers/mod.rs` | **FIXED ENUMERATION** | hardcoded substrings: "show me the current network", "export the network", "list my facts", … |
| execution_failure | `try_execution_failure` | `solver_handlers/mod.rs` | **FIXED ENUMERATION** | `normalized.contains("undefined_function")` |
| installation_conversion (command recognizer) | `looks_like_command` | `solver_handlers/installation_conversion.rs` | COMPOSITIONAL *(after AC2)* | structural `looks_like_command` + `is_executable_head` + provenance; was a `PREFIXES` whitelist |
| installation_conversion (intent gate) | `is_install_conversion_request` | `solver_handlers/installation_conversion.rs` | HYBRID | data-flow IR core; gate still keys on conversion/surface keyword arrays |
| write_script | `try_write_script` | `solver_handlers/mod.rs` | COMPOSITIONAL | `is_write_script_request` data-driven language/task detection |
| program_synthesis | `try_program_synthesis` | `solver_handlers/program_synthesis.rs` | COMPOSITIONAL | three seed roles (subject/domain/action) compose |
| software_project | `try_software_project_request` | `solver_handlers/software_project.rs` | COMPOSITIONAL | seed `ROLE_SOFTWARE_AUTHORING_ACTION` + artifact kind |
| algorithm | `try_algorithm` | `solver_handlers/mod.rs` | **FIXED ENUMERATION** | `contains("algorithm") || contains("sort")` |
| source_refresh | `try_source_refresh` | `solver_handlers/mod.rs` | **FIXED ENUMERATION** | `contains("refresh") && (contains("cache") || contains("page"))` |
| source_conflict | `try_source_conflict` | `solver_handlers/mod.rs` | **FIXED ENUMERATION** | `contains("conflict") || (contains("born in") && contains(" or "))` |
| clarification | `try_clarification` | `solver_handlers/user_intent.rs` | COMPOSITIONAL | seed `ROLE_CLARIFICATION_REQUEST` |
| punctuation_only_prompt | `try_punctuation_only_prompt` | `solver_handlers/user_intent.rs` | FIXED ENUMERATION | hardcoded punctuation set (intentionally closed — punctuation is finite) |
| ill_formed | `try_ill_formed` | `solver_handlers/user_intent.rs` | FIXED ENUMERATION | `contains("teach this fact")` + paren-balance check |
| physical_action_question | `try_physical_action_question` | `solver_handlers_policy.rs` | COMPOSITIONAL | seed `ROLE_PHYSICAL_ACTION_TRIGGER` |
| kupi_slona | `try_kupi_slona` | `solver_handlers_policy.rs` | COMPOSITIONAL | seed `ROLE_CIRCULAR_JOKE_PHRASE` |
| shell_refusal | `try_shell_refusal` | `solver_handlers/user_intent.rs` | FIXED ENUMERATION | hardcoded risk patterns (`run \`` + `rm `/`sudo`/"on my behalf") |
| proof_request | `try_proof_request` | `solver_handlers/user_intent.rs` | COMPOSITIONAL | seed `ROLE_PROOF_DIRECTIVE`/`_LEAD`/`_MARKER` |
| opinion_question | `try_opinion_question` | `solver_handlers/user_intent.rs` | **FIXED ENUMERATION** | 15 hardcoded `starts_with` openers ("do you think", "in your opinion", …) |
| incompatible_units | `try_incompatible_units` | `solver_handler_units.rs` | COMPOSITIONAL | seed `ROLE_MEASUREMENT_UNIT` + `defined_by` dimension graph |

### Tally and reading

- **COMPOSITIONAL: ~29** — the majority of the solver already recognizes by
  semantic role / seed vocabulary, the pattern issue #433 endorses.
- **HYBRID: 4** — `docs_method_explanation`, `conversation_memory`, `who_is`,
  and the `installation_conversion` intent gate.
- **FIXED ENUMERATION: ~10** — `number_riddle`, `network_query`,
  `execution_failure`, `algorithm`, `source_refresh`, `source_conflict`,
  `ill_formed`, `shell_refusal`, `opinion_question`, `punctuation_only_prompt`.

`punctuation_only_prompt` is a *legitimately* closed enumeration (the set of
sentence-final punctuation is finite), so it is enumerated by nature, not by
shortcut. The rest are genuine generalization candidates.

### Worst offenders (future generalization candidates)

Ranked by breadth of the hardcoded list and how often a valid input would fall
outside it:

1. **`opinion_question`** — 15 literal `starts_with` openers; any unseen
   phrasing ("are you of the view that…") escapes. Should become a seed
   `ROLE_OPINION_QUERY` surface set.
2. **`number_riddle`** — large multilingual keyword tables (RU/EN + math
   symbols). Should read number/bound/identity cues from seed vocabulary.
3. **`algorithm`** — `contains("algorithm") || contains("sort")` is the most
   brittle gate in the table; should defer to the seed operation vocabulary
   that `numeric_list` already uses.
4. **`source_refresh` / `source_conflict` / `network_query`** — small fixed
   substring sets that would generalize to seed roles.
5. The **`installation_conversion` intent gate** (`is_install_conversion_request`)
   — AC2 generalized the *command* recognizer; the *intent* gate still keys on
   conversion/surface keyword arrays and is the natural next step.

### Compositional exemplars worth imitating

- **`web_search`**, **`program_synthesis`**, **`compound_interest`** — verify
  several independent semantic signals from seed and only fire when they
  coincide.
- **`numeric_list`** — recognition, classification, computation, and code
  generation are all seed data; adding an operation is data plus one arithmetic
  clause, not a new handler. (Reconstructed in AC3 below.)
- **`incompatible_units`** — resolves physical dimensions through a `defined_by`
  graph instead of a unit table.
- **`installation_conversion::looks_like_command`** — the AC2 result: a
  structural recognizer that any well-formed command passes regardless of tool.

## AC2: Installation-conversion command recognizer

### Before — a fixed prefix whitelist

`looks_like_command` accepted a line only if it began with one of an enumerated
`PREFIXES` set (`apt `, `brew `, `cargo `, `npm `, `pip `, …) or contained `|` /
`&&`. Any tool absent from the list — `bun`, `deno`, `uv`, `just`, `zig`,
`pdm`, `poetry`, `nix`, … — was silently dropped from the install-step IR. The
companion `describe_command` mapped fixed substrings (`git clone` → "Clone the
repository") to prose, so a recognized-but-unlisted tool produced the generic
fallback.

### After — a structural, provenance-aware recognizer

`looks_like_command(command, provenance)` now reasons about command *shape*:

1. **Executable head.** `is_executable_head` requires the first token to look
   like an executable name or path — lowercase/digit/`./` lead, body limited to
   `[a-z0-9._/+-]`. A capitalized or non-ASCII lead ("Clone", "Выполните",
   "运行") reads as prose and is rejected. This is what lets *any* tool through:
   `bun`, `zig`, `pdm` all pass on shape alone.
2. **Shell composition.** A pipeline or `&&`/`||`/`;` sequence is unambiguous
   command shape and is accepted regardless of provenance.
3. **Prose function words.** An executable-looking head can still front a
   wrapped note ("make sure you run …"); a small English function-word set
   ("the", "your", "manually", …) betrays it.
4. **Provenance.** `Provenance::CodeSpan` (inline `` `…` `` spans and fenced
   shell/PowerShell lines — author-marked code) trusts the shape. A
   `Provenance::BareLine` (a raw document line) must prove itself: it needs an
   argument or a path, and a bare line that *embeds* a code span ("Run
   `npm install` to set up") is treated as prose, because the inline collector
   already lifted the real command out of the span.

`describe_command` was likewise generalized: `ParsedCommand` strips
privilege/wrapper tokens (`sudo`, `env`, `command`), resolves `python -m <mod>`
to its module, separates flags from objects, and `classify_verb` keys the
action off the *verb* (`install`/`build`/`test`/`clone`/…) rather than the
tool. So `bun install`, `pdm install`, `just test`, and `zig build` all get
accurate descriptions with no table entry, and an unknown verb is synthesized
("Run the `frobnicate widgets` step") rather than flattened to a constant.

### No regression + added false-positive coverage

- **100-repo corpus stays green.** `tests/unit/installation_conversion.rs`'s
  `popular_github_projects_route_through_install_conversion` (the captured
  top-100 GitHub snapshot from #423) passes unchanged.
- **Unlisted tools now route.**
  `unlisted_tools_still_route_through_install_conversion` exercises `bun`,
  `deno`, `uv`, `zig` — all absent from the retired whitelist.
- **Prose is rejected.** `prose_bullets_do_not_leak_into_generated_scripts`
  (engine level) and the source-mirror private-function suite
  (`tests/source/source_tests/solver_handlers/installation_conversion/tests.rs`:
  `prose_lines_are_rejected`, `bare_line_embedding_a_code_span_is_prose`,
  `bare_lines_need_more_than_a_lone_word`, `executable_head_separates_tools_from_words`)
  pin the false-positive behavior down at the recognizer level.
- **Cross-runtime parity.** The same structural recognizer and verb/object
  describer were mirrored into `src/web/formal_ai_worker.js`; the
  `experiments/issue-423-js-installation-conversion.mjs` harness asserts the
  unlisted-tool and prose-rejection behavior in the browser worker too.

## AC3: Reconstructing `numeric_list` from the meta-algorithm primitives

Issue #423 introduced an explicit **algorithm-construction layer**: the
installation-conversion response records how the solver *constructs* a
conversion algorithm from a fixed set of stages
(`ALGORITHM_CONSTRUCTION_STAGES` in
`src/solver_handlers/installation_conversion.rs`) and how that recipe projects
onto the existing coding surfaces (`CODING_SURFACE_PROJECTIONS`). The seven
stages are the **meta-algorithm rule primitives**:

`collect_corpus → derive_surfaces → extract_ir → synthesize_operations →
project_targets → mirror_runtimes → promote_capability`

The claim under test for AC3 is that these primitives are not specific to
installation conversion — they are a general recipe for building a coding
handler. To prove it, we reconstruct an *existing, independently written*
coding handler — `numeric_list` (issue #395) — purely as an instantiation of
the seven stages. `numeric_list` is the right subject: it is already in the
`CODING_SURFACE_PROJECTIONS` table
(`operation/data/language IR -> generated code plus evaluated result`) and it
was implemented before this meta-algorithm framing existed, so a clean mapping
is evidence the recipe is general rather than retrofitted.

| Meta-algorithm stage | `numeric_list` instantiation | Where it lives today |
|----------------------|------------------------------|----------------------|
| **collect_corpus** — representative problem-class examples | The numeric-list task family (sort / reverse_sort / reverse / sum / product / min / max over a given list), seeded from issues #395 and #412. | `solver_handlers/numeric_list/mod.rs` module docs; integration fixtures `issue_395_*`, `issue_412_*`. |
| **derive_surfaces** — source/target surface ontology | Source surface = a natural-language imperative naming an operation, a programming language, and ≥2 numbers. Target surfaces = the supported `ProgramLanguage` set. | `detect_operation`, language detection via `program_language_<slug>` aliases. |
| **extract_ir** — shared intermediate representation | The `(operation, values, language)` triple, plus the seed ontology mapping each operation to a *family* (`list_transformation`/`list_reduction`) and *result kind* (`list`/`scalar`). | `data/seed/numeric-list-operations.lino`; parsed in `mod.rs`. |
| **synthesize_operations** — recognizers, extractors, renderers, validators | Recognizer = seed `operation_vocabulary()`; extractor = number/`language` parse; the validator = the deterministic reducer/transformer that folds the values, so the *result* is computed, not rendered as a guess. | `operation_vocabulary()`, the generic fold in `mod.rs`. |
| **project_targets** — target-specific renderers | Per-language code is composed at execution time by walking the language's inheritance chain in `data/seed/coding-idioms.lino` and recursively expanding idiom slots — there are no per-language renderer functions. | `numeric_list/codegen.rs`, `data/seed/coding-idioms.lino`. |
| **mirror_runtimes** — Rust + browser-worker projections | The same numeric-list algorithm exists in the browser worker so the deployed surface answers identically. | `src/web/formal_ai_worker.js` numeric-list path; parity experiments. |
| **promote_capability** — reusable construction pattern | The follow-up recovery (issue #412: a bare imperative over a *new* list continues an established coding context) is exactly the "promote to a reusable pattern" stage — the handler's capability is reused across turns, not re-recognized from scratch. | `InheritedCoding` history recovery in `mod.rs`. |

### What the reconstruction shows

Every stage of the installation-conversion meta-algorithm has a direct,
already-existing counterpart in `numeric_list` — recognition, IR, computation,
and code generation are each seed data, and the runtime mirror and capability
promotion are present too. The two handlers were written for unrelated problem
classes (README↔script conversion vs. numeric coding tasks), yet both decompose
into the same seven primitives. That is the evidence AC3 asks for: the
meta-algorithm is a *general* coding-handler construction recipe, and a
fixed-enumeration handler can be migrated onto it stage by stage by replacing
each memoized table with the corresponding seed-data primitive.

### Prototype direction for a fixed-enumeration handler

The same mapping gives a concrete migration recipe for the worst offenders from
AC1. Taking `algorithm` (`contains("algorithm") || contains("sort")`) as the
worked example:

1. **extract_ir** — reuse `numeric_list`'s `(operation, …)` IR; "sort" is
   already an entry in `operation_vocabulary()`.
2. **synthesize_operations** — replace the two-substring gate with a seed-role
   recognizer that fires when an operation verb is present, exactly as
   `numeric_list::detect_operation` does.
3. **project_targets / validators** — defer to the existing numeric-list
   renderer/reducer instead of returning a result-less stub.

That collapses `algorithm` into the `numeric_list` construction and deletes a
fixed-enumeration recognizer — the generalization direction issue #433 asks the
codebase to move in.

## Files changed

- `src/solver_handlers/installation_conversion.rs` — structural `looks_like_command`
  (provenance + `is_executable_head` + `has_shell_operator` + `reads_as_prose`)
  and verb/object `describe_command` (`ParsedCommand` + `classify_verb`).
- `tests/source/solver_handlers/installation_conversion.rs` — source-mirror copy.
- `tests/source/source_tests/solver_handlers/installation_conversion/tests.rs` —
  8 private-function tests for the recognizer and describer.
- `tests/unit/installation_conversion.rs` — engine-level unlisted-tool and
  prose-rejection tests.
- `src/web/formal_ai_worker.js` — browser-worker mirror of the recognizer and
  describer.
- `experiments/issue-423-js-installation-conversion.mjs` — unlisted-tool and
  prose adversarial checks for the worker.
- `docs/case-studies/issue-433/` — this case study and archived raw data.

## Tests

```
cargo test --test unit            # 840 passed (incl. installation_conversion ×10)
cargo test --test source          # 397 passed (incl. recognizer private-fn ×8)
cargo test --test integration     # 35 passed
node experiments/issue-423-js-installation-conversion.mjs   # all checks pass
cargo fmt -- --check && cargo clippy --all-targets --all-features
node --check src/web/formal_ai_worker.js
```
