Part of the E127 umbrella (#1183). Architect (2026-09-29): "We also should check all our dependencies, and see which code is duplicated when they are used across all our organizations and repositories, so the most common logic, that usually used in our maintained dependencies will be reported as issues to these dependencies, so we are able to deduplicate as much general logic, make less code in Formal AI and more code in our dependencies".

## Audit result (verified on `main` d209aac64 and the dependency repositories, 2026-09-29)

| Formal AI code | Lines | Belongs in | Dependency status | Upstream issue |
|---|---:|---|---|---|
| **Links Notation parser**: `rust/src/seed/parser.rs` ("Tiny Links Notation parser shared by every `seed` loader", used from 93 files) and `js/seed_loader.js:180-260` (`parseLino`, `parseLinoLine`, `parseQuotedList`, `parseCodepoint`); the `links-notation` crate is declared in `rust/Cargo.toml:47` but not imported anywhere in `rust/src` | 404 + ~150 | link-foundation/links-notation | **has the parser** (`Parser`, `Link`, `formatLinks`, `stripComments` in 0.22.0) | none yet: file in links-notation any syntax `seed/parser.rs` accepts that the crate rejects (R11) |
| `shell_quote` defined **7 times**: `github_logs.rs:413`, `client_integrations/global_verify.rs:331`, `agentic_coding/{capability_router.rs:819, general_planner.rs:428, git_commit.rs:105, local_search.rs:707, report_script.rs:109}` | ~75 | `command-stream` | **already has** `command_stream::quote` / `quote_all` (0.16.0 and 1.1.1) | none needed; replace here |
| `rust/src/relative_meta_logic.rs` | 434 | link-foundation/relative-meta-logic | crate exists in repo (0.19.0, lib `rml`) but is **not published** | https://github.com/link-foundation/relative-meta-logic/issues/185 |
| `rust/src/arithmetic.rs` + `arithmetic_word_tables.rs` + linear-equation solver in `calculation.rs` | 962+ | link-assistant/calculator | lacks big-integer exactness, linear equations, `no_std` | https://github.com/link-assistant/calculator/issues/222 |
| `rust/src/links_format.rs`, `rust/src/json_lino.rs` | 878 | link-foundation/lino-objects-codec | escaping helpers private; no `serde_json::Value` bridge | https://github.com/link-foundation/lino-objects-codec/issues/59 |
| `rust/src/web_search_core.rs` | 753 | link-assistant/web-search | RRF and provider registry are `std`/server-only | https://github.com/link-assistant/web-search/issues/25 |
| `rust/src/source_fetch.rs` | 550 | link-assistant/web-capture | no capture cache | https://github.com/link-assistant/web-capture/issues/156 |
| `rust/src/language.rs` | 607 | link-assistant/human-language | Rust crate unpublished, no detection | https://github.com/link-assistant/human-language/issues/44 |
| `rust/src/es_tokenizer.rs` + `es_meta.rs` | 1,599 | link-foundation/meta-language | no `no_std` ES tokenizer/pivot | https://github.com/link-foundation/meta-language/issues/198 |
| `js/i18n.js` | 220 | link-foundation/lino-i18n | no browser fetch loader | https://github.com/link-foundation/lino-i18n/issues/21 |

Duplication inside this repository as well: `shell_quote` 7 copies (table), `command_argument` 7 copies (`agentic_coding/{progress.rs:466, file_read.rs:718, tool_result.rs:850, narration.rs:82, command_reroute.rs:401, evidence_record.rs:486, workspace_change.rs:608}`, two of them missing the `cmd` key, which caused #1154). Each becomes one shared function.

About 6,000 lines of general logic live in this repository instead of in the dependencies.

## Duplication inside this repository (verified on `main` d209aac64, 2026-09-29)

A scan of every function of at most 22 lines under `rust/src` (5,840 functions) found **75 groups** of bodies that are identical after whitespace normalization and appear in two or more files (full list in the collapsed section below). The largest groups:

| Function | Copies | Where |
|---|---:|---|
| `render_document`, `render_document_from`, `final_answer` | 11, 11, 10 | `agentic_coding/{associative,routing,code_rewrite,execution,external_benchmark}_learning.rs` and `agentic_coding/learning_report/*_learning.rs`: one report shape copied per learning module |
| `quote` (Links Notation value escaping: `\\`, `"`→`'`, `\n`, `\r`, `\t`) | 8 | `self_improvement.rs:741`, `change_request.rs:338`, `repair_strategy.rs:348`, `self_healing.rs:316`, `self_source_links.rs:337`, `learning_ledger.rs:484`, `rebuild_plan.rs:289`, `self_explanation.rs:319` (belongs in lino-objects-codec, link-foundation/lino-objects-codec#59) |
| `shell_quote` | 7 | the table above (`command_stream::quote` already exists) |
| `command_argument` | 7 | `agentic_coding/{progress.rs:466, file_read.rs:718, tool_result.rs:850, narration.rs:82, command_reroute.rs:401, evidence_record.rs:486, workspace_change.rs:608}`; two of them miss the `cmd` key, which caused #1154 |
| `granted`, `declined` (an approval value) | 7, 7 | `search_fusion_learning.rs:289`, `coding_research_learning.rs:209`, `workspace_change_learning.rs:438`, `learning_ledger.rs:51`, `skill_procedure/learning.rs:284`, `task_decomposition/learning.rs:101`, `algorithm_discovery/execution.rs:48`: each module declares its own approval struct |
| `field` | 7 | `google_trends_learning.rs`, `change_request.rs`, `repair_strategy.rs`, `self_healing.rs`, `learning_ledger.rs`, `rebuild_plan.rs`, `learning_adoption_ledger.rs` |
| `collapse_whitespace` | 6 | `google_trends_learning.rs`, `google_trends_catalog.rs`, `change_request.rs`, `coding/function_catalog/python_docs.rs`, `how_to_guide/extract.rs`, `agentic_coding/question_catalog.rs` |
| `push_doublet`, `passed`/`failed`, `nested` | 5 each | see the list |

This is the same observation the vision builds on ("algorithms are inferred by deduplication"): repeated code is a missing shared definition.

## Requirements

- **R1** Every function group in the table above is replaced by one shared definition (in this repository or in the owning dependency), and `git grep -c "fn shell_quote\|fn command_argument\|fn collapse_whitespace" -- rust/src` reports at most one definition each.
- **R2** Links Notation value quoting (`quote`, `field`, `push_field`, `escape` groups) moves to link-foundation/lino-objects-codec (#59 asks for the public escaping helpers); until its release, one shared module `rust/src/links_format.rs` holds it and the copies call it. No new copy may appear (R8).
- **R3** Shell quoting uses `command_stream::quote` / `quote_all` (already exported by the pinned `command-stream` 0.16.0, `src/lib.rs:70,96`).
- **R4** One `command_argument(schema, arguments)` in `agentic_coding` reads the shell-command argument by the client's declared tool schema (`command`, `cmd`, `script`, array forms); the seven copies are deleted (shared with #1154).
- **R5** One approval type (`ReviewApproval { reviewer, granted }` with `granted()`/`declined()`) in `learning_ledger.rs` is used by the seven learning modules.
- **R6** One `LearningReport` trait (with default `render_document`, `render_document_from`, `final_answer`) is implemented by the eleven learning-report modules instead of eleven copies.
- **R7** Each module listed in the audit table at the top is deleted once its dependency issue is released, in the same pull request as the version bump of #1169 (E134); while that issue is open the local module carries a comment with the dependency issue URL beside it (CONTRIBUTING, "Self-maintained dependencies and patches").
- **R8** A duplication gate `scripts/check-duplicate-functions.rs` (rust-script; the same normalization as the scan below: strip comments and whitespace, compare bodies of functions up to 22 lines) runs in the CI rust stage and fails when a new group of two or more identical bodies appears across files; existing groups are listed in `data/meta/duplicate-functions-baseline.lino` and the list may only shrink (no new entries).
- **R9** A weekly cross-organization job finds duplicated code across all repositories of link-foundation and link-assistant and opens one issue per candidate in the repository that should own it (design below).
- **R11** The seed loaders in both roots parse with `links-notation` (Rust crate 0.22 via #1169, JS package): `rust/src/seed/parser.rs` and the parser half of `js/seed_loader.js` are deleted. Before deleting, a conformance test runs every file under `data/` through both parsers and compares the trees; every construct the local parser accepts and the crate does not (e.g. the codepoint metadata of issue #398, quoted lists) is filed in link-foundation/links-notation in general wording and supported there, not kept locally.
- **R10** Three-roots parity: the JS/TS roots use the same shared definitions (translated with `formal-ai translate --from rust --to js|ts --input <file> --write`), and the duplication gate also scans `js/` and `ts/`.

## Design

### In-repository changes (R1–R6)

- `rust/src/links_format.rs` (existing, 132 lines): gains `quote_value`, `field`, `push_field`; the eight `quote` copies, seven `field` copies and three `push_field` copies call it.
- `rust/src/agentic_coding/tool_arguments.rs` (new): `pub(crate) fn command_argument(arguments: &serde_json::Value, schema: Option<&serde_json::Value>) -> Option<String>`.
- `rust/src/learning_ledger.rs`: `pub struct ReviewApproval` with `granted`/`declined` constructors; the seven modules re-export it.
- `rust/src/agentic_coding/learning_report/mod.rs`: `pub trait LearningReport { fn document(&self) -> String; fn render_document(&self) -> String { … } fn render_document_from(…) -> String { … } fn final_answer(&self) -> AgenticPlan { … } }`.
- `rust/src/text_normalization.rs` (new, or an existing text utility module): `collapse_whitespace`.

### Continuous cross-organization job (R9)

- `scripts/cross-org-duplication.rs` (new, rust-script) and `.github/workflows/cross-org-duplication.yml` (new, `schedule: weekly` + `workflow_dispatch`):
  1. Enumerate repositories: `gh repo list link-foundation --limit 1000 --json name,isArchived,isFork,primaryLanguage` and the same for `link-assistant`; skip archived and forks.
  2. Shallow-clone each (`git clone --depth 1 --filter=blob:limit=1m`), cap the total at a configured size, delete after scanning.
  3. Parse every source file whose language has a grammar in `meta-language` (`LinkNetwork::parse`, the same engine `rust/src/coding/cst.rs` uses) and collect function-level subtrees of at least 5 lines.
  4. Normalize (drop comments and whitespace, rename local identifiers to positional names) and hash each subtree (FNV-1a, as the self-AST census does).
  5. Group equal hashes across repositories; rank groups by `lines × number of repositories`.
  6. Choose the owning repository: the dependency that at least one copy's repository already depends on (from `Cargo.toml` / `package.json`) and whose own tree contains a copy; otherwise the most-depended-on repository among the copies' dependencies; otherwise report "no owner" for a human decision.
  7. Open one issue per group in the owner (general wording per CONTRIBUTING, listing every copy with path and lines), after searching for an existing issue with the marker `<!-- cross-org-duplication:<hash> -->`; never open the same group twice.
  8. Write `data/meta/cross-org-duplication.lino` (groups, owners, issue URLs) and a "Cross-organization duplication" table in `docs/status.md` through `scripts/render-status.rs`.

## Tests

- `rust/tests/unit/issue_1182_links_notation_conformance.rs`: every `data/**/*.lino` parses to the same tree with `links_notation::Parser` as with `seed::parser` (run before the switch; after the switch it pins the crate against the committed data).

- `rust/tests/unit/issue_1182_shared_helpers.rs`: the shared `quote_value`, `command_argument` (table over `command`, `cmd`, `script`, array), `ReviewApproval`, `collapse_whitespace` behave exactly as the copies did (the copies' existing tests move here).
- `rust/tests/unit/issue_1182_duplicate_gate.rs`: the gate fails on a fixture with two identical helper bodies in two files and passes on the baseline.
- `rust/tests/unit/issue_1182_cross_org_duplication.rs`: on a fixture of three tiny repositories (one dependency, two consumers with a copy of the same function), the job selects the dependency as owner and renders one issue body; the marker search prevents a second issue.
- Commands: `RUSTUP_TOOLCHAIN=1.98.1 cargo test --test unit issue_1182_`; `rust-script scripts/check-duplicate-functions.rs`.

## Definition of done

- [ ] Requirement shard `docs/requirements/issue-1182-deduplicate-general-logic.md` (R1–R10); `rust-script scripts/assemble-requirements.rs --write`.
- [ ] Traceability rows in `docs/requirements-traceability.md` with automated test and manual confirmation.
- [ ] Case study `docs/case-studies/issue-1182/` with the audit table, the 75-group scan output, and the first weekly cross-organization report.
- [ ] Changelog fragment in `changelog.d/` (repository root).
- [ ] `RUSTUP_TOOLCHAIN=1.98.1 rust-script scripts/run-ci-gates.rs --stage rust` green, including the new duplication gate.
- [ ] `data/meta/duplicate-functions-baseline.lino` is shorter than the 75 groups found today.

## Depends on / blocks

- Uses the dependency issues listed in the audit table (link-foundation/relative-meta-logic#185, link-assistant/calculator#222, link-foundation/lino-objects-codec#59, link-assistant/web-search#25, link-assistant/web-capture#156, link-assistant/human-language#44, link-foundation/meta-language#198, link-foundation/lino-i18n#21) and the cross-organization findings already filed from the first manual run: link-assistant/hive-mind#2325 (Links Notation formatting duplicated from `links-notation`), link-assistant/agent#320 (process lifecycle duplicated from `command-stream`).
- Shares R4 with #1154 (E119).
- Coordinates with #1169 (E134): each dependency release is adopted by the same version-bump pull request.

<details><summary>Scan output: 75 groups of identical small function bodies in rust/src (d209aac64)</summary>

```text
=== group: names=['render_document'] sites=11 nlines~3 ===
  rust/src/agentic_coding/associative_learning.rs:30-32  fn render_document
  rust/src/agentic_coding/routing_learning.rs:28-30  fn render_document
  rust/src/agentic_coding/code_rewrite_learning.rs:29-31  fn render_document
  rust/src/agentic_coding/execution_learning.rs:29-31  fn render_document
  rust/src/agentic_coding/external_benchmark_learning.rs:30-32  fn render_document
  rust/src/agentic_coding/learning_report/search_fusion_learning.rs:24-26  fn render_document
  rust/src/agentic_coding/learning_report/hardcoded_language_learning.rs:24-26  fn render_document
  rust/src/agentic_coding/learning_report/lexeme_import_learning.rs:24-26  fn render_document
  rust/src/agentic_coding/learning_report/handler_precedence_learning.rs:31-33  fn render_document
  rust/src/agentic_coding/learning_report/self_hosting_learning.rs:33-35  fn render_document
  rust/src/agentic_coding/learning_report/context_hierarchy_learning.rs:31-33  fn render_document
=== group: names=['render_document_from'] sites=11 nlines~3 ===
  rust/src/agentic_coding/associative_learning.rs:36-38  fn render_document_from
  rust/src/agentic_coding/routing_learning.rs:34-36  fn render_document_from
  rust/src/agentic_coding/code_rewrite_learning.rs:35-37  fn render_document_from
  rust/src/agentic_coding/execution_learning.rs:35-37  fn render_document_from
  rust/src/agentic_coding/external_benchmark_learning.rs:36-38  fn render_document_from
  rust/src/agentic_coding/learning_report/search_fusion_learning.rs:29-31  fn render_document_from
  rust/src/agentic_coding/learning_report/hardcoded_language_learning.rs:29-31  fn render_document_from
  rust/src/agentic_coding/learning_report/lexeme_import_learning.rs:29-31  fn render_document_from
  rust/src/agentic_coding/learning_report/handler_precedence_learning.rs:36-38  fn render_document_from
  rust/src/agentic_coding/learning_report/self_hosting_learning.rs:39-41  fn render_document_from
  rust/src/agentic_coding/learning_report/context_hierarchy_learning.rs:36-38  fn render_document_from
=== group: names=['final_answer'] sites=10 nlines~3 ===
  rust/src/agentic_coding/associative_learning.rs:41-43  fn final_answer
  rust/src/agentic_coding/routing_learning.rs:39-41  fn final_answer
  rust/src/agentic_coding/code_rewrite_learning.rs:40-42  fn final_answer
  rust/src/agentic_coding/execution_learning.rs:40-42  fn final_answer
  rust/src/agentic_coding/learning_report/search_fusion_learning.rs:34-36  fn final_answer
  rust/src/agentic_coding/learning_report/hardcoded_language_learning.rs:34-36  fn final_answer
  rust/src/agentic_coding/learning_report/lexeme_import_learning.rs:34-36  fn final_answer
  rust/src/agentic_coding/learning_report/handler_precedence_learning.rs:41-43  fn final_answer
  rust/src/agentic_coding/learning_report/self_hosting_learning.rs:44-46  fn final_answer
  rust/src/agentic_coding/learning_report/context_hierarchy_learning.rs:41-43  fn final_answer
=== group: names=['quote'] sites=8 nlines~8 ===
  rust/src/self_improvement.rs:741-748  fn quote
  rust/src/change_request.rs:338-345  fn quote
  rust/src/repair_strategy.rs:348-355  fn quote
  rust/src/self_healing.rs:316-323  fn quote
  rust/src/self_source_links.rs:337-344  fn quote
  rust/src/learning_ledger.rs:484-491  fn quote
  rust/src/rebuild_plan.rs:289-296  fn quote
  rust/src/self_explanation.rs:319-326  fn quote
=== group: names=['granted'] sites=7 nlines~6 ===
  rust/src/search_fusion_learning.rs:289-294  fn granted
  rust/src/coding_research_learning.rs:209-214  fn granted
  rust/src/workspace_change_learning.rs:438-443  fn granted
  rust/src/learning_ledger.rs:51-56  fn granted
  rust/src/skill_procedure/learning.rs:284-289  fn granted
  rust/src/task_decomposition/learning.rs:101-106  fn granted
  rust/src/algorithm_discovery/execution.rs:48-53  fn granted
=== group: names=['declined'] sites=7 nlines~6 ===
  rust/src/search_fusion_learning.rs:297-302  fn declined
  rust/src/coding_research_learning.rs:217-222  fn declined
  rust/src/workspace_change_learning.rs:446-451  fn declined
  rust/src/learning_ledger.rs:60-65  fn declined
  rust/src/skill_procedure/learning.rs:292-297  fn declined
  rust/src/task_decomposition/learning.rs:109-114  fn declined
  rust/src/algorithm_discovery/execution.rs:56-61  fn declined
=== group: names=['field'] sites=7 nlines~3 ===
  rust/src/google_trends_learning.rs:226-228  fn field
  rust/src/change_request.rs:330-332  fn field
  rust/src/repair_strategy.rs:344-346  fn field
  rust/src/self_healing.rs:293-295  fn field
  rust/src/learning_ledger.rs:476-478  fn field
  rust/src/rebuild_plan.rs:281-283  fn field
  rust/src/learning_adoption_ledger.rs:270-272  fn field
=== group: names=['collapse_whitespace'] sites=6 nlines~3 ===
  rust/src/google_trends_learning.rs:240-242  fn collapse_whitespace
  rust/src/google_trends_catalog.rs:465-467  fn collapse_whitespace
  rust/src/change_request.rs:319-321  fn collapse_whitespace
  rust/src/coding/function_catalog/python_docs.rs:199-201  fn collapse_whitespace
  rust/src/how_to_guide/extract.rs:272-274  fn collapse_whitespace
  rust/src/agentic_coding/question_catalog.rs:272-274  fn collapse_whitespace
=== group: names=['new'] sites=6 nlines~3 ===
  rust/src/bounded_autonomy.rs:103-105  fn new
  rust/src/associative_persistence.rs:142-144  fn new
  rust/src/summarization/gathering.rs:209-211  fn new
  rust/src/sequences/converter.rs:133-135  fn new
  rust/src/sequences/store.rs:81-83  fn new
  rust/src/sequences/symbols.rs:32-34  fn new
=== group: names=['fmt'] sites=5 nlines~3 ===
  rust/src/search_fusion_learning.rs:31-33  fn fmt
  rust/src/coding_research_learning.rs:47-49  fn fmt
  rust/src/skill_procedure.rs:412-414  fn fmt
  rust/src/workspace_change_learning.rs:31-33  fn fmt
  rust/src/reasoning_standard/mod.rs:63-65  fn fmt
=== group: names=['passed'] sites=5 nlines~7 ===
  rust/src/search_fusion_learning.rs:259-265  fn passed
  rust/src/workspace_change_learning.rs:408-414  fn passed
  rust/src/skill_procedure/learning.rs:253-259  fn passed
  rust/src/task_decomposition/learning.rs:71-77  fn passed
  rust/src/algorithm_discovery/execution.rs:16-22  fn passed
=== group: names=['failed'] sites=5 nlines~7 ===
  rust/src/search_fusion_learning.rs:268-274  fn failed
  rust/src/workspace_change_learning.rs:417-423  fn failed
  rust/src/skill_procedure/learning.rs:262-268  fn failed
  rust/src/task_decomposition/learning.rs:80-86  fn failed
  rust/src/algorithm_discovery/execution.rs:25-31  fn failed
=== group: names=['nested'] sites=5 nlines~3 ===
  rust/src/google_trends_learning.rs:230-232  fn nested
  rust/src/change_request.rs:334-336  fn nested
  rust/src/self_healing.rs:297-299  fn nested
  rust/src/learning_ledger.rs:480-482  fn nested
  rust/src/learning_adoption_ledger.rs:278-280  fn nested
=== group: names=['push_doublet'] sites=5 nlines~7 ===
  rust/src/skill_procedure.rs:804-810  fn push_doublet
  rust/src/link_store.rs:971-977  fn push_doublet
  rust/src/associative_package.rs:883-889  fn push_doublet
  rust/src/skill_compiler.rs:809-815  fn push_doublet
  rust/src/substitution.rs:801-807  fn push_doublet
=== group: names=['path'] sites=4 nlines~3 ===
  rust/src/concept_sense_ledger.rs:55-57  fn path
  rust/src/service_accessibility.rs:215-217  fn path
  rust/src/verifiable_task/ledger.rs:101-103  fn path
  rust/src/coding/discovered_procedures.rs:146-148  fn path
=== group: names=['fmt'] sites=4 nlines~3 ===
  rust/src/memory_program.rs:176-178  fn fmt
  rust/src/memory_query_language/mod.rs:586-588  fn fmt
  rust/src/memory/upgrade.rs:186-188  fn fmt
  rust/src/statement_audit/evidence.rs:48-50  fn fmt
=== group: names=['normalize'] sites=3 nlines~7 ===
  rust/src/client_contract_learning.rs:521-527  fn normalize
  rust/src/option_network.rs:849-855  fn normalize
  rust/src/computer_use/seed.rs:331-337  fn normalize
=== group: names=['new'] sites=3 nlines~5 ===
  rust/src/concept_sense_ledger.rs:47-51  fn new
  rust/src/verifiable_task/ledger.rs:93-97  fn new
  rust/src/coding/discovered_procedures.rs:139-143  fn new
=== group: names=['escape'] sites=3 nlines~3 ===
  rust/src/links_query.rs:173-175  fn escape
  rust/src/needs.rs:238-240  fn escape
  rust/src/external_benchmarks/ledger.rs:288-290  fn escape
=== group: names=['prefix_literals'] sites=3 nlines~8 ===
  rust/src/entity_resolution.rs:151-158  fn prefix_literals
  rust/src/web_search_markers.rs:146-153  fn prefix_literals
  rust/src/solver_handlers/user_intent.rs:17-24  fn prefix_literals
=== group: names=['events'] sites=3 nlines~3 ===
  rust/src/memory.rs:217-219  fn events
  rust/src/link_store.rs:425-427  fn events
  rust/src/memory_sync.rs:270-272  fn events
=== group: names=['push_field'] sites=3 nlines~9 ===
  rust/src/skill_procedure.rs:794-802  fn push_field
  rust/src/associative_package.rs:873-881  fn push_field
  rust/src/skill_compiler.rs:799-807  fn push_field
=== group: names=['root'] sites=3 nlines~3 ===
  rust/src/agent.rs:210-212  fn root
  rust/src/repository_workspace/mod.rs:181-183  fn root
  rust/src/computer_use/executor.rs:121-123  fn root
=== group: names=['shell_quote'] sites=3 nlines~3 ===
  rust/src/client_integrations/global_verify.rs:331-333  fn shell_quote
  rust/src/agentic_coding/capability_router.rs:819-821  fn shell_quote
  rust/src/agentic_coding/local_search.rs:707-709  fn shell_quote
=== group: names=['read_arguments'] sites=3 nlines~3 ===
  rust/src/agentic_coding/structured_edit.rs:712-714  fn read_arguments
  rust/src/agentic_coding/code_artifact.rs:334-336  fn read_arguments
  rust/src/agentic_coding/workspace_change.rs:667-669  fn read_arguments
=== group: names=['observation_count'] sites=2 nlines~3 ===
  rust/src/search_fusion_learning.rs:197-199  fn observation_count
  rust/src/workspace_change_learning.rs:347-349  fn observation_count
=== group: names=['is_green'] sites=2 nlines~3 ===
  rust/src/search_fusion_learning.rs:276-278  fn is_green
  rust/src/workspace_change_learning.rs:425-427  fn is_green
=== group: names=['plan_for'] sites=2 nlines~5 ===
  rust/src/search_fusion_learning.rs:369-373  fn plan_for
  rust/src/workspace_change_learning.rs:517-521  fn plan_for
=== group: names=['escape'] sites=2 nlines~6 ===
  rust/src/fact_checking.rs:659-664  fn escape
  rust/src/agentic_coding/general_planner.rs:810-815  fn escape
=== group: names=['consume'] sites=2 nlines~8 ===
  rust/src/calculation.rs:332-339  fn consume
  rust/src/proof_engine/decision/linear.rs:460-467  fn consume
=== group: names=['peek'] sites=2 nlines~3 ===
  rust/src/calculation.rs:341-343  fn peek
  rust/src/proof_engine/decision/linear.rs:469-471  fn peek
=== group: names=['digest'] sites=2 nlines~3 ===
  rust/src/memory_revision.rs:52-54  fn digest
  rust/src/computer_use/executor.rs:763-765  fn digest
=== group: names=['contains_token'] sites=2 nlines~9 ===
  rust/src/intent_formalization.rs:397-405  fn contains_token
  rust/src/cue_lexicon.rs:190-195  fn contains_token
=== group: names=['push_unique'] sites=2 nlines~5 ===
  rust/src/intent_formalization.rs:746-750  fn push_unique
  rust/src/method_registry.rs:819-823  fn push_unique
=== group: names=['record_construction'] sites=2 nlines~18 ===
  rust/src/rule_synthesis.rs:123-140  fn record_construction
  rust/src/rule_synthesis_portfolio.rs:156-173  fn record_construction
=== group: names=['capitalize_first'] sites=2 nlines~7 ===
  rust/src/search_fusion.rs:715-721  fn capitalize_first
  rust/src/web_search_fusion_core.rs:617-623  fn capitalize_first
=== group: names=['drop'] sites=2 nlines~3 ===
  rust/src/cli_solve.rs:497-499  fn drop
  rust/src/authoring_loop.rs:104-106  fn drop
=== group: names=['fmt'] sites=2 nlines~3 ===
  rust/src/links_query.rs:91-93  fn fmt
  rust/src/links_substitution_query/mod.rs:78-80  fn fmt
=== group: names=['suffix_literals'] sites=2 nlines~8 ===
  rust/src/entity_resolution.rs:162-169  fn suffix_literals
  rust/src/web_search_markers.rs:157-164  fn suffix_literals
=== group: names=['name'] sites=2 nlines~3 ===
  rust/src/coding_research_learning.rs:92-94  fn name
  rust/src/rule_interpreter.rs:496-498  fn name
=== group: names=['user'] sites=2 nlines~3 ===
  rust/src/protocol.rs:202-204  fn user
  rust/src/summarization/dialog.rs:35-37  fn user
=== group: names=['assistant'] sites=2 nlines~3 ===
  rust/src/protocol.rs:208-210  fn assistant
  rust/src/summarization/dialog.rs:41-43  fn assistant
=== group: names=['push_trimmed'] sites=2 nlines~6 ===
  rust/src/task_decomposition.rs:648-653  fn push_trimmed
  rust/src/meta_frame.rs:587-592  fn push_trimmed
=== group: names=['learned_program_rule_lino'] sites=2 nlines~10 ===
  rust/src/promotion.rs:552-561  fn learned_program_rule_lino
  rust/src/self_improvement.rs:635-644  fn learned_program_rule_lino
=== group: names=['quote'] sites=2 nlines~8 ===
  rust/src/promotion.rs:861-868  fn quote
  rust/src/research_learning.rs:600-607  fn quote
=== group: names=['default'] sites=2 nlines~3 ===
  rust/src/memory.rs:132-134  fn default
  rust/src/world_model_dialog.rs:292-294  fn default
=== group: names=['config'] sites=2 nlines~5 ===
  rust/src/cli_context.rs:440-444  fn config
  rust/src/cli_report.rs:493-497  fn config
=== group: names=['language_code'] sites=2 nlines~5 ===
  rust/src/concepts.rs:125-129  fn language_code
  rust/src/translation/language_markers.rs:89-93  fn language_code
=== group: names=['field_value'] sites=2 nlines~12 ===
  rust/src/meta_self_improvement.rs:326-337  fn field_value
  rust/src/recipe_interpreter.rs:480-491  fn field_value
=== group: names=['validated'] sites=2 nlines~3 ===
  rust/src/algorithm_discovery.rs:142-144  fn validated
  rust/src/learning_cycle.rs:211-213  fn validated
=== group: names=['below'] sites=2 nlines~10 ===
  rust/src/solver_search.rs:257-266  fn below
  rust/src/summarization/validation/sampling.rs:162-168  fn below
=== group: names=['from'] sites=2 nlines~3 ===
  rust/src/agent.rs:172-174  fn from
  rust/src/computer_use/executor.rs:38-40  fn from
=== group: names=['config'] sites=2 nlines~5 ===
  rust/src/conversation_context.rs:171-175  fn config
  rust/src/server/conversation_reports.rs:57-61  fn config
=== group: names=['source_url'] sites=2 nlines~3 ===
  rust/src/source_fetch.rs:182-184  fn source_url
  rust/src/probability.rs:64-66  fn source_url
=== group: names=['fetched_at'] sites=2 nlines~3 ===
  rust/src/source_fetch.rs:187-189  fn fetched_at
  rust/src/probability.rs:69-71  fn fetched_at
=== group: names=['sha256'] sites=2 nlines~3 ===
  rust/src/source_fetch.rs:192-194  fn sha256
  rust/src/probability.rs:74-76  fn sha256
=== group: names=['unix_now'] sites=2 nlines~5 ===
  rust/src/source_fetch.rs:541-545  fn unix_now
  rust/src/github_logs.rs:423-427  fn unix_now
=== group: names=['usize_to_f32'] sites=2 nlines~4 ===
  rust/src/probability.rs:800-803  fn usize_to_f32
  rust/src/translation/selection.rs:376-379  fn usize_to_f32
=== group: names=['bare_literals'] sites=2 nlines~8 ===
  rust/src/web_search_markers.rs:180-187  fn bare_literals
  rust/src/solver_handlers/user_intent.rs:29-36  fn bare_literals
=== group: names=['matches_trigger'] sites=2 nlines~5 ===
  rust/src/seed/brainstorm.rs:38-42  fn matches_trigger
  rust/src/seed/personas.rs:63-67  fn matches_trigger
=== group: names=['collect_language_values'] sites=2 nlines~13 ===
  rust/src/seed/shell_intents.rs:341-353  fn collect_language_values
  rust/src/seed/terminal_commands.rs:79-91  fn collect_language_values
=== group: names=['localized_text'] sites=2 nlines~14 ===
  rust/src/solver_handlers/behavior_rules.rs:297-310  fn localized_text
  rust/src/solver_handlers/behavior_rule_followups.rs:89-102  fn localized_text
=== group: names=['rate_source_step'] sites=2 nlines~7 ===
  rust/src/solver_handlers/calculator_rate.rs:97-103  fn rate_source_step
  rust/src/solver_handlers/compound_interest.rs:435-441  fn rate_source_step
=== group: names=['sha256'] sites=2 nlines~3 ===
  rust/src/memory/upgrade.rs:866-868  fn sha256
  rust/src/orchestration/workspace.rs:274-276  fn sha256
=== group: names=['unix_now'] sites=2 nlines~6 ===
  rust/src/verifiable_task/ledger.rs:186-191  fn unix_now
  rust/src/coding/discovered_procedures.rs:215-220  fn unix_now
=== group: names=['required'] sites=2 nlines~7 ===
  rust/src/verifiable_task/ledger.rs:193-199  fn required
  rust/src/coding/discovered_procedures.rs:254-260  fn required
=== group: names=['values'] sites=2 nlines~7 ===
  rust/src/verifiable_task/ledger.rs:201-207  fn values
  rust/src/coding/discovered_procedures.rs:262-268  fn values
=== group: names=['new'] sites=2 nlines~6 ===
  rust/src/summarization/validation/mod.rs:403-408  fn new
  rust/src/statement_audit/model.rs:15-20  fn new
=== group: names=['render'] sites=2 nlines~3 ===
  rust/src/coding/ir_lowering/rust.rs:228-230  fn render
  rust/src/coding/ir_lowering/python.rs:455-457  fn render
=== group: names=['render_or_id'] sites=2 nlines~3 ===
  rust/src/coding/ir_lowering/rust.rs:232-234  fn render_or_id
  rust/src/coding/ir_lowering/python.rs:459-461  fn render_or_id
=== group: names=['localized'] sites=2 nlines~3 ===
  rust/src/agentic_coding/report_issue.rs:188-190  fn localized
  rust/src/agentic_coding/local_search.rs:703-705  fn localized
=== group: names=['round_confidence'] sites=2 nlines~3 ===
  rust/src/agentic_coding/question_catalog.rs:278-280  fn round_confidence
  rust/src/agentic_coding/google_trends_catalog.rs:150-152  fn round_confidence
=== group: names=['read_arguments'] sites=2 nlines~8 ===
  rust/src/agentic_coding/intent_router.rs:146-153  fn read_arguments
  rust/src/agentic_coding/general_execution.rs:406-413  fn read_arguments
=== group: names=['shell_quote'] sites=2 nlines~3 ===
  rust/src/agentic_coding/general_planner.rs:428-430  fn shell_quote
  rust/src/agentic_coding/report_script.rs:109-111  fn shell_quote
=== group: names=['field'] sites=2 nlines~6 ===
  rust/src/agentic_coding/dreaming_audit.rs:34-39  fn field
  rust/src/agentic_coding/lexicon.rs:396-401  fn field
```
</details>



- Credentials: this workflow takes its GitHub token from the shared resolver of #1187 (E151): GitHub App or one `AUTOMATION_TOKEN` when configured, otherwise the default `GITHUB_TOKEN` with dispatched checks and orphan-branch isolation; it never requires a token.

