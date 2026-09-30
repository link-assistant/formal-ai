# Historical duplicate groups under rust/src (issue #1182, E147)

This is the interrupted agent’s historical static inventory, not the output
of the corrected gate. The lexical scanner now preserves string whitespace,
handles literal braces and actually recognizes `fn NAME`; these changes can
change grouping. No fresh count or baseline has been claimed. Generate the
current machine inventory in CI with `rust-script scripts/check-duplicate-functions.rs --json`
and initialize the reviewed baseline with `--write-baseline`. The following
older inventory was measured on the `qa-reasoning-coding-bulk-fixes` branch,
2026-09-30: **156 groups**, 480 sites. The audit at
`main` d209aac64 found 75 groups; the wave-2/3 lanes grew the surface, so the
baseline the maintainer cuts from here starts at the honest current number
and may only shrink from there.

The consolidation plans below are the R1–R6 edit plans; the edit sites live in
files owned by other forks this wave, so each plan is applied by the main
session as the `docs/integration-manifest.md` rows record.

| # | Sites | ~Lines | Names | Plan |
|---:|---:|---:|---|---|
| 1 | 18 | 6 | `seed_text` | New wave-2/3 growth — one `seed_text` helper in `rust/src/seed/text.rs`; the eighteen sites each re-derive a seed sentence |
| 2 | 15 | 3 | `field, nested, push_quoted_field, push_quoted_nested_fiel…` | R2 — `field`/`nested`/`push_quoted_field` move beside `quote_value` into `rust/src/links_format.rs` |
| 3 | 13 | 7 | `template` | Companion of `seed_text` — one template-selector helper beside it |
| 4 | 11 | 3 | `render_document` | R6 — one `LearningReport` trait in `rust/src/agentic_coding/learning_report/mod.rs` with default methods; the eleven modules implement it |
| 5 | 11 | 3 | `render_document_from` | R6 — one `LearningReport` trait in `rust/src/agentic_coding/learning_report/mod.rs` with default methods; the eleven modules implement it |
| 6 | 10 | 3 | `collapse_thinking_whitespace, collapse_whitespace, compac…` | Design — one `collapse_whitespace` in a shared text module (`rust/src/text_normalization.rs`) |
| 7 | 10 | 3 | `is_associative_learning_task, is_code_rewrite_learning_ta…` | Companion of R6 — the task predicates join the `LearningReport` consolidation |
| 8 | 10 | 3 | `final_answer` | R6 — one `LearningReport` trait in `rust/src/agentic_coding/learning_report/mod.rs` with default methods; the eleven modules implement it |
| 9 | 8 | 8 | `quote` | R2 — one `quote_value` in `rust/src/links_format.rs` (then link-foundation/lino-objects-codec#59); every site calls it |
| 10 | 7 | 6 | `granted` | R5 — one `ReviewApproval { reviewer, granted }` in `rust/src/learning_ledger.rs`; the learning modules re-export it |
| 11 | 7 | 6 | `declined` | R5 — one `ReviewApproval { reviewer, granted }` in `rust/src/learning_ledger.rs`; the learning modules re-export it |
| 12 | 7 | 3 | `new` | Deliberate constructors — identical trivial bodies, different types; keep |
| 13 | 6 | 3 | `events` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 14 | 6 | 3 | `shell_quote, shell_string` | R3 — replace with `command_stream::quote` (already exported by the pinned 0.16.0) |
| 15 | 5 | 3 | `fmt` | Deliberate per-type `Display` impls — same body, different type; keep, or a macro once a third wave appears |
| 16 | 5 | 7 | `passed` | R5 — one `ReviewApproval { reviewer, granted }` in `rust/src/learning_ledger.rs`; the learning modules re-export it |
| 17 | 5 | 7 | `failed` | R5 — one `ReviewApproval { reviewer, granted }` in `rust/src/learning_ledger.rs`; the learning modules re-export it |
| 18 | 5 | 7 | `push_doublet` | R2 family — the doublet/field writers join `links_format.rs` |
| 19 | 5 | 3 | `is_human_gated, syntax_is_valid` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 20 | 5 | 3 | `localized, localized_label, pattern_template, template` | Companion of `seed_text` — one template-selector helper beside it |
| 21 | 5 | 6 | `triggered` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 22 | 4 | 3 | `digest, sha256` | One digest helper (the sites wrap the same hex call) |
| 23 | 4 | 3 | `path` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 24 | 4 | 3 | `fmt` | Deliberate per-type `Display` impls — same body, different type; keep, or a macro once a third wave appears |
| 25 | 4 | 3 | `escape, lino_escape` | R2 — the escaping helpers join `links_format.rs` |
| 26 | 4 | 7 | `evidence, payload, render_fields, trace_payload` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 27 | 4 | 6 | `epoch_seconds, now_seconds, unix_now` | One `unix_now` in a shared time module (the four sites re-derive epoch seconds) |
| 28 | 4 | 7 | `child_values, values` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 29 | 4 | 21 | `word_entries` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 30 | 3 | 7 | `normalize` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 31 | 3 | 5 | `new` | Deliberate constructors — identical trivial bodies, different types; keep |
| 32 | 3 | 8 | `prefix_literals` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 33 | 3 | 8 | `suffix_frames, suffix_literals` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 34 | 3 | 6 | `push_trimmed, push_trimmed_segment` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 35 | 3 | 8 | `escape_value, quote` | R2 — one `quote_value` in `rust/src/links_format.rs` (then link-foundation/lino-objects-codec#59); every site calls it |
| 36 | 3 | 3 | `is_ideographic, is_unspaced_script` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 37 | 3 | 9 | `push_field` | R2 family — the doublet/field writers join `links_format.rs` |
| 38 | 3 | 5 | `unix_now` | One `unix_now` in a shared time module (the four sites re-derive epoch seconds) |
| 39 | 3 | 3 | `collapse_whitespace, collapse_ws, normalize_single_line` | Design — one `collapse_whitespace` in a shared text module (`rust/src/text_normalization.rs`) |
| 40 | 3 | 3 | `root` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 41 | 3 | 6 | `localized_for` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 42 | 3 | 5 | `mapping_rows` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 43 | 3 | 9 | `number_value` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 44 | 3 | 8 | `hex, hex_nibble` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 45 | 3 | 6 | `field` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 46 | 3 | 3 | `render_or_id, runtime_message` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 47 | 3 | 3 | `is_green` | R5 — one `ReviewApproval { reviewer, granted }` in `rust/src/learning_ledger.rs`; the learning modules re-export it |
| 48 | 3 | 3 | `read_arguments` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 49 | 3 | 6 | `one_call, plan_one` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 50 | 2 | 5 | `new` | Deliberate constructors — identical trivial bodies, different types; keep |
| 51 | 2 | 3 | `observation_count` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 52 | 2 | 3 | `is_green` | R5 — one `ReviewApproval { reviewer, granted }` in `rust/src/learning_ledger.rs`; the learning modules re-export it |
| 53 | 2 | 5 | `new` | Deliberate constructors — identical trivial bodies, different types; keep |
| 54 | 2 | 5 | `plan_for` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 55 | 2 | 6 | `escape` | R2 — the escaping helpers join `links_format.rs` |
| 56 | 2 | 8 | `consume` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 57 | 2 | 3 | `peek` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 58 | 2 | 3 | `json_line, to_send_message_body` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 59 | 2 | 3 | `len` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 60 | 2 | 3 | `is_empty` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 61 | 2 | 9 | `contains_token` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 62 | 2 | 5 | `push_unique` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 63 | 2 | 18 | `record_construction` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 64 | 2 | 3 | `is_sentence_terminal, sentence_terminator` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 65 | 2 | 7 | `capitalize_first` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 66 | 2 | 16 | `days_from_iso_date, parse_iso_date` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 67 | 2 | 3 | `lower_operator` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 68 | 2 | 3 | `upper_operator` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 69 | 2 | 3 | `drop` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 70 | 2 | 3 | `fmt` | Deliberate per-type `Display` impls — same body, different type; keep, or a macro once a third wave appears |
| 71 | 2 | 3 | `flush, sync_parent` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 72 | 2 | 3 | `step_count` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 73 | 2 | 3 | `name` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 74 | 2 | 6 | `is_sha256, is_sha256_hex` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 75 | 2 | 6 | `flatten_lino_value, sanitize_payload` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 76 | 2 | 3 | `user` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 77 | 2 | 3 | `assistant` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 78 | 2 | 9 | `is_ideographic, is_unspaced_script` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 79 | 2 | 3 | `catalog, google_trends_catalog` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 80 | 2 | 3 | `escape_lino_value, escape_value` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 81 | 2 | 8 | `push_response_stream_event, push_sse_event` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 82 | 2 | 5 | `learned_program_rule_lino` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 83 | 2 | 3 | `default` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 84 | 2 | 3 | `len` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 85 | 2 | 3 | `is_empty` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 86 | 2 | 3 | `export_links_notation, to_links_notation` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 87 | 2 | 3 | `peek` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 88 | 2 | 3 | `get, slug` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 89 | 2 | 5 | `config` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 90 | 2 | 3 | `is_identifier_character, is_word_character` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 91 | 2 | 3 | `adopted` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 92 | 2 | 3 | `unadopted` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 93 | 2 | 3 | `live_api_enabled` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 94 | 2 | 3 | `module_count` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 95 | 2 | 4 | `concepts, public_concepts` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 96 | 2 | 8 | `render_source_link` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 97 | 2 | 6 | `unquote, unquote_seed_value` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 98 | 2 | 6 | `language_code_of, meaning_defined_language_code` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 99 | 2 | 5 | `language_code` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 100 | 2 | 12 | `field_value` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 101 | 2 | 3 | `records` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 102 | 2 | 3 | `validated` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 103 | 2 | 6 | `validated_candidates` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 104 | 2 | 3 | `named_child, optional_child` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 105 | 2 | 7 | `navigate_mut` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 106 | 2 | 6 | `seeded` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 107 | 2 | 7 | `next_u64` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 108 | 2 | 10 | `below` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 109 | 2 | 3 | `from` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 110 | 2 | 5 | `config` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 111 | 2 | 7 | `split_tuple` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 112 | 2 | 3 | `source_url` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 113 | 2 | 3 | `fetched_at` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 114 | 2 | 3 | `sha256` | One digest helper (the sites wrap the same hex call) |
| 115 | 2 | 3 | `cached` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 116 | 2 | 4 | `with_online` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 117 | 2 | 7 | `finite_clamped` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 118 | 2 | 4 | `usize_to_f32` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 119 | 2 | 8 | `circumfix_frames, circumfix_literals` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 120 | 2 | 8 | `bare_literals` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 121 | 2 | 3 | `new` | Deliberate constructors — identical trivial bodies, different types; keep |
| 122 | 2 | 5 | `new` | Deliberate constructors — identical trivial bodies, different types; keep |
| 123 | 2 | 8 | `pick_topic` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 124 | 2 | 5 | `matches_trigger` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 125 | 2 | 13 | `collect_language_values` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 126 | 2 | 10 | `draft_strategies_from, planner_precedence_from` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 127 | 2 | 3 | `has_char_in_range, meta_has_char_in_range` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 128 | 2 | 17 | `function_name` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 129 | 2 | 6 | `identifier_at` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 130 | 2 | 5 | `render_line, response` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 131 | 2 | 6 | `is_url_wrapper_punctuation` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 132 | 2 | 3 | `is_url_trailing_punctuation` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 133 | 2 | 8 | `localized_text` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 134 | 2 | 6 | `echo` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 135 | 2 | 7 | `rate_source_step` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 136 | 2 | 3 | `iso, iso_date` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 137 | 2 | 3 | `is_numeral_token_character, is_word_character` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 138 | 2 | 3 | `new` | Deliberate constructors — identical trivial bodies, different types; keep |
| 139 | 2 | 7 | `required` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 140 | 2 | 3 | `is_identifier_start, is_variable_start` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 141 | 2 | 6 | `new` | Deliberate constructors — identical trivial bodies, different types; keep |
| 142 | 2 | 3 | `record_type` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 143 | 2 | 3 | `live_fetch_enabled` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 144 | 2 | 6 | `reuse_slug` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 145 | 2 | 11 | `annotation_type, type_from_annotation` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 146 | 2 | 12 | `neutral_literal` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 147 | 2 | 6 | `normalized_words, words` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 148 | 2 | 19 | `python_identifier, rust_identifier` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 149 | 2 | 3 | `render` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 150 | 2 | 3 | `steps` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 151 | 2 | 6 | `arg_or_empty, arg_str` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 152 | 2 | 7 | `required` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 153 | 2 | 5 | `config, seed_text` | New wave-2/3 growth — one `seed_text` helper in `rust/src/seed/text.rs`; the eighteen sites each re-derive a seed sentence |
| 154 | 2 | 3 | `round_confidence` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 155 | 2 | 8 | `read_arguments` | Shared definition where the types agree; otherwise deliberate divergence to note here |
| 156 | 2 | 7 | `argument_path, tool_argument_path` | Shared definition where the types agree; otherwise deliberate divergence to note here |

## The sites (per group)

Digests are the gate's group keys (first 16 hex of the sha256 of the
normalized body) — the baseline carries them.

### Group 1 — seed_text (18 sites, ~6 lines, `4237c8b8d14e66f1`)

- `rust/src/agentic_coding/repair_loop.rs:88` `fn seed_text`
- `rust/src/legality_warning.rs:75` `fn seed_text`
- `rust/src/si_units.rs:167` `fn seed_text`
- `rust/src/small_model_fallback.rs:107` `fn seed_text`
- `rust/src/solver_handlers/code_debugging.rs:27` `fn seed_text`
- `rust/src/solver_handlers/code_explanation.rs:33` `fn seed_text`
- `rust/src/solver_handlers/code_refactoring.rs:26` `fn seed_text`
- `rust/src/solver_handlers/code_review.rs:29` `fn seed_text`
- `rust/src/solver_handlers/creative_composition.rs:40` `fn seed_text`
- `rust/src/solver_handlers/creative_writing.rs:38` `fn seed_text`
- `rust/src/solver_handlers/format_conversion.rs:25` `fn seed_text`
- `rust/src/solver_handlers/product_search.rs:53` `fn seed_text`
- `rust/src/solver_handlers/regex_synthesis.rs:26` `fn seed_text`
- `rust/src/solver_handlers/shell_command_compose.rs:45` `fn seed_text`
- `rust/src/solver_handlers/sql_synthesis.rs:24` `fn seed_text`
- `rust/src/solver_handlers/test_generation.rs:26` `fn seed_text`
- `rust/src/solver_handlers/text_rewrite.rs:24` `fn seed_text`
- `rust/src/triz_solver.rs:61` `fn seed_text`

### Group 2 — field, nested, push_quoted_field, push_quoted_nested_field, sub (15 sites, ~3 lines, `c34427a11a9fc980`)

- `rust/src/change_request.rs:330` `fn field`
- `rust/src/change_request.rs:334` `fn nested`
- `rust/src/google_trends_learning.rs:226` `fn field`
- `rust/src/google_trends_learning.rs:230` `fn nested`
- `rust/src/learning_adoption_ledger.rs:270` `fn field`
- `rust/src/learning_adoption_ledger.rs:278` `fn nested`
- `rust/src/learning_ledger.rs:476` `fn field`
- `rust/src/learning_ledger.rs:480` `fn nested`
- `rust/src/rebuild_plan.rs:281` `fn field`
- `rust/src/rebuild_plan.rs:285` `fn sub`
- `rust/src/repair_strategy.rs:344` `fn field`
- `rust/src/self_healing.rs:293` `fn field`
- `rust/src/self_healing.rs:297` `fn nested`
- `rust/src/self_improvement.rs:733` `fn push_quoted_field`
- `rust/src/self_improvement.rs:737` `fn push_quoted_nested_field`

### Group 3 — template (13 sites, ~7 lines, `10b9f39ffa5d423b`)

- `rust/src/legality_warning.rs:228` `fn template`
- `rust/src/solver_handlers/code_debugging.rs:64` `fn template`
- `rust/src/solver_handlers/code_explanation.rs:70` `fn template`
- `rust/src/solver_handlers/code_refactoring.rs:63` `fn template`
- `rust/src/solver_handlers/code_review.rs:66` `fn template`
- `rust/src/solver_handlers/creative_composition.rs:48` `fn template`
- `rust/src/solver_handlers/creative_writing.rs:46` `fn template`
- `rust/src/solver_handlers/format_conversion.rs:62` `fn template`
- `rust/src/solver_handlers/product_search.rs:224` `fn template`
- `rust/src/solver_handlers/regex_synthesis.rs:86` `fn template`
- `rust/src/solver_handlers/shell_command_compose.rs:130` `fn template`
- `rust/src/solver_handlers/sql_synthesis.rs:84` `fn template`
- `rust/src/solver_handlers/test_generation.rs:86` `fn template`

### Group 4 — render_document (11 sites, ~3 lines, `ca6bf79697a59dad`)

- `rust/src/agentic_coding/associative_learning.rs:30` `fn render_document`
- `rust/src/agentic_coding/code_rewrite_learning.rs:29` `fn render_document`
- `rust/src/agentic_coding/execution_learning.rs:29` `fn render_document`
- `rust/src/agentic_coding/external_benchmark_learning.rs:30` `fn render_document`
- `rust/src/agentic_coding/learning_report/context_hierarchy_learning.rs:31` `fn render_document`
- `rust/src/agentic_coding/learning_report/handler_precedence_learning.rs:31` `fn render_document`
- `rust/src/agentic_coding/learning_report/hardcoded_language_learning.rs:24` `fn render_document`
- `rust/src/agentic_coding/learning_report/lexeme_import_learning.rs:24` `fn render_document`
- `rust/src/agentic_coding/learning_report/search_fusion_learning.rs:24` `fn render_document`
- `rust/src/agentic_coding/learning_report/self_hosting_learning.rs:33` `fn render_document`
- `rust/src/agentic_coding/routing_learning.rs:28` `fn render_document`

### Group 5 — render_document_from (11 sites, ~3 lines, `e283abdcc87d6c3a`)

- `rust/src/agentic_coding/associative_learning.rs:36` `fn render_document_from`
- `rust/src/agentic_coding/code_rewrite_learning.rs:35` `fn render_document_from`
- `rust/src/agentic_coding/execution_learning.rs:35` `fn render_document_from`
- `rust/src/agentic_coding/external_benchmark_learning.rs:36` `fn render_document_from`
- `rust/src/agentic_coding/learning_report/context_hierarchy_learning.rs:36` `fn render_document_from`
- `rust/src/agentic_coding/learning_report/handler_precedence_learning.rs:36` `fn render_document_from`
- `rust/src/agentic_coding/learning_report/hardcoded_language_learning.rs:29` `fn render_document_from`
- `rust/src/agentic_coding/learning_report/lexeme_import_learning.rs:29` `fn render_document_from`
- `rust/src/agentic_coding/learning_report/search_fusion_learning.rs:29` `fn render_document_from`
- `rust/src/agentic_coding/learning_report/self_hosting_learning.rs:39` `fn render_document_from`
- `rust/src/agentic_coding/routing_learning.rs:34` `fn render_document_from`

### Group 6 — collapse_thinking_whitespace, collapse_whitespace, compact, compact_log_value, normalize (10 sites, ~3 lines, `1f2bb0e6510cc3bd`)

- `rust/src/agentic_coding/question_catalog.rs:272` `fn collapse_whitespace`
- `rust/src/change_request.rs:319` `fn collapse_whitespace`
- `rust/src/coding/function_catalog/python_docs.rs:199` `fn collapse_whitespace`
- `rust/src/concept_lookup.rs:536` `fn compact`
- `rust/src/dreaming/learning.rs:619` `fn normalize`
- `rust/src/event_log.rs:357` `fn collapse_thinking_whitespace`
- `rust/src/google_trends_catalog.rs:465` `fn collapse_whitespace`
- `rust/src/google_trends_learning.rs:240` `fn collapse_whitespace`
- `rust/src/how_to_guide/extract.rs:272` `fn collapse_whitespace`
- `rust/src/solver_handlers/research_table.rs:538` `fn compact_log_value`

### Group 7 — is_associative_learning_task, is_code_rewrite_learning_task, is_context_hierarchy_learning_task, is_execution_learning_task, is_handler_precedence_learning_task, is_hardcoded_language_learning_task, is_lexeme_import_learning_task, is_routing_learning_task, is_search_fusion_learning_task, is_self_hosting_learning_task (10 sites, ~3 lines, `9035a683850add50`)

- `rust/src/agentic_coding/associative_learning.rs:25` `fn is_associative_learning_task`
- `rust/src/agentic_coding/code_rewrite_learning.rs:24` `fn is_code_rewrite_learning_task`
- `rust/src/agentic_coding/execution_learning.rs:24` `fn is_execution_learning_task`
- `rust/src/agentic_coding/learning_report/context_hierarchy_learning.rs:26` `fn is_context_hierarchy_learning_task`
- `rust/src/agentic_coding/learning_report/handler_precedence_learning.rs:26` `fn is_handler_precedence_learning_task`
- `rust/src/agentic_coding/learning_report/hardcoded_language_learning.rs:19` `fn is_hardcoded_language_learning_task`
- `rust/src/agentic_coding/learning_report/lexeme_import_learning.rs:19` `fn is_lexeme_import_learning_task`
- `rust/src/agentic_coding/learning_report/search_fusion_learning.rs:19` `fn is_search_fusion_learning_task`
- `rust/src/agentic_coding/learning_report/self_hosting_learning.rs:28` `fn is_self_hosting_learning_task`
- `rust/src/agentic_coding/routing_learning.rs:23` `fn is_routing_learning_task`

### Group 8 — final_answer (10 sites, ~3 lines, `51f320d270166f99`)

- `rust/src/agentic_coding/associative_learning.rs:41` `fn final_answer`
- `rust/src/agentic_coding/code_rewrite_learning.rs:40` `fn final_answer`
- `rust/src/agentic_coding/execution_learning.rs:40` `fn final_answer`
- `rust/src/agentic_coding/learning_report/context_hierarchy_learning.rs:41` `fn final_answer`
- `rust/src/agentic_coding/learning_report/handler_precedence_learning.rs:41` `fn final_answer`
- `rust/src/agentic_coding/learning_report/hardcoded_language_learning.rs:34` `fn final_answer`
- `rust/src/agentic_coding/learning_report/lexeme_import_learning.rs:34` `fn final_answer`
- `rust/src/agentic_coding/learning_report/search_fusion_learning.rs:34` `fn final_answer`
- `rust/src/agentic_coding/learning_report/self_hosting_learning.rs:44` `fn final_answer`
- `rust/src/agentic_coding/routing_learning.rs:39` `fn final_answer`

### Group 9 — quote (8 sites, ~8 lines, `22be0c6ae2e95a5c`)

- `rust/src/change_request.rs:338` `fn quote`
- `rust/src/learning_ledger.rs:484` `fn quote`
- `rust/src/rebuild_plan.rs:289` `fn quote`
- `rust/src/repair_strategy.rs:348` `fn quote`
- `rust/src/self_explanation.rs:319` `fn quote`
- `rust/src/self_healing.rs:316` `fn quote`
- `rust/src/self_improvement.rs:741` `fn quote`
- `rust/src/self_source_links.rs:337` `fn quote`

### Group 10 — granted (7 sites, ~6 lines, `01701da125a2d4de`)

- `rust/src/algorithm_discovery/execution.rs:48` `fn granted`
- `rust/src/coding_research_learning.rs:209` `fn granted`
- `rust/src/learning_ledger.rs:51` `fn granted`
- `rust/src/search_fusion_learning.rs:289` `fn granted`
- `rust/src/skill_procedure/learning.rs:284` `fn granted`
- `rust/src/task_decomposition/learning.rs:101` `fn granted`
- `rust/src/workspace_change_learning.rs:438` `fn granted`

### Group 11 — declined (7 sites, ~6 lines, `02b0c63701995291`)

- `rust/src/algorithm_discovery/execution.rs:56` `fn declined`
- `rust/src/coding_research_learning.rs:217` `fn declined`
- `rust/src/learning_ledger.rs:60` `fn declined`
- `rust/src/search_fusion_learning.rs:297` `fn declined`
- `rust/src/skill_procedure/learning.rs:292` `fn declined`
- `rust/src/task_decomposition/learning.rs:109` `fn declined`
- `rust/src/workspace_change_learning.rs:446` `fn declined`

### Group 12 — new (7 sites, ~3 lines, `2947685106538857`)

- `rust/src/associative_persistence.rs:142` `fn new`
- `rust/src/bounded_autonomy.rs:103` `fn new`
- `rust/src/sequences/converter.rs:133` `fn new`
- `rust/src/sequences/store.rs:81` `fn new`
- `rust/src/sequences/symbols.rs:32` `fn new`
- `rust/src/summarization/gathering.rs:209` `fn new`
- `rust/src/web_formalize.rs:981` `fn new`

### Group 13 — events (6 sites, ~3 lines, `194c7650371271ac`)

- `rust/src/event_log.rs:59` `fn events`
- `rust/src/link_store.rs:425` `fn events`
- `rust/src/memory.rs:217` `fn events`
- `rust/src/memory_sync.rs:267` `fn events`
- `rust/src/research_learning.rs:274` `fn events`
- `rust/src/world_model_dialog.rs:397` `fn events`

### Group 14 — shell_quote, shell_string (6 sites, ~3 lines, `c8d49efffbde1593`)

- `rust/src/agentic_coding/capability_router.rs:819` `fn shell_quote`
- `rust/src/agentic_coding/file_read.rs:834` `fn shell_string`
- `rust/src/agentic_coding/general_planner.rs:428` `fn shell_quote`
- `rust/src/agentic_coding/local_search.rs:707` `fn shell_quote`
- `rust/src/agentic_coding/report_script.rs:109` `fn shell_quote`
- `rust/src/client_integrations/global_verify.rs:331` `fn shell_quote`

### Group 15 — fmt (5 sites, ~3 lines, `7e7765d55542a7c8`)

- `rust/src/coding_research_learning.rs:47` `fn fmt`
- `rust/src/reasoning_standard/mod.rs:63` `fn fmt`
- `rust/src/search_fusion_learning.rs:31` `fn fmt`
- `rust/src/skill_procedure.rs:412` `fn fmt`
- `rust/src/workspace_change_learning.rs:31` `fn fmt`

### Group 16 — passed (5 sites, ~7 lines, `8fc7fac2ecd835a4`)

- `rust/src/algorithm_discovery/execution.rs:16` `fn passed`
- `rust/src/search_fusion_learning.rs:259` `fn passed`
- `rust/src/skill_procedure/learning.rs:253` `fn passed`
- `rust/src/task_decomposition/learning.rs:71` `fn passed`
- `rust/src/workspace_change_learning.rs:408` `fn passed`

### Group 17 — failed (5 sites, ~7 lines, `ba5218593deed039`)

- `rust/src/algorithm_discovery/execution.rs:25` `fn failed`
- `rust/src/search_fusion_learning.rs:268` `fn failed`
- `rust/src/skill_procedure/learning.rs:262` `fn failed`
- `rust/src/task_decomposition/learning.rs:80` `fn failed`
- `rust/src/workspace_change_learning.rs:417` `fn failed`

### Group 18 — push_doublet (5 sites, ~7 lines, `9951b888acf656bf`)

- `rust/src/associative_package.rs:883` `fn push_doublet`
- `rust/src/link_store.rs:971` `fn push_doublet`
- `rust/src/skill_compiler.rs:809` `fn push_doublet`
- `rust/src/skill_procedure.rs:804` `fn push_doublet`
- `rust/src/substitution.rs:801` `fn push_doublet`

### Group 19 — is_human_gated, syntax_is_valid (5 sites, ~3 lines, `b5bea41b6c623f7c`)

- `rust/src/change_request.rs:102` `fn is_human_gated`
- `rust/src/coding/composition.rs:409` `fn syntax_is_valid`
- `rust/src/rebuild_plan.rs:120` `fn is_human_gated`
- `rust/src/repair_strategy.rs:131` `fn is_human_gated`
- `rust/src/self_healing.rs:184` `fn is_human_gated`

### Group 20 — localized, localized_label, pattern_template, template (5 sites, ~3 lines, `9b9d2d9121c15203`)

- `rust/src/agentic_coding/local_search.rs:703` `fn localized`
- `rust/src/agentic_coding/report_issue.rs:188` `fn localized`
- `rust/src/definition_merge.rs:327` `fn localized_label`
- `rust/src/sequences/inference.rs:213` `fn pattern_template`
- `rust/src/solver_handlers/procedure_rules.rs:81` `fn template`

### Group 21 — triggered (5 sites, ~6 lines, `05afc5404a17fff7`)

- `rust/src/solver_handlers/code_debugging.rs:263` `fn triggered`
- `rust/src/solver_handlers/code_explanation.rs:390` `fn triggered`
- `rust/src/solver_handlers/code_refactoring.rs:253` `fn triggered`
- `rust/src/solver_handlers/code_review.rs:251` `fn triggered`
- `rust/src/solver_handlers/regex_synthesis.rs:299` `fn triggered`

### Group 22 — digest, sha256 (4 sites, ~3 lines, `6126e0941ea4ba7d`)

- `rust/src/computer_use/executor.rs:763` `fn digest`
- `rust/src/memory/upgrade.rs:866` `fn sha256`
- `rust/src/memory_revision.rs:52` `fn digest`
- `rust/src/orchestration/workspace.rs:274` `fn sha256`

### Group 23 — path (4 sites, ~3 lines, `345cdff5ddb9613d`)

- `rust/src/coding/discovered_procedures.rs:146` `fn path`
- `rust/src/concept_sense_ledger.rs:55` `fn path`
- `rust/src/service_accessibility.rs:215` `fn path`
- `rust/src/verifiable_task/ledger.rs:101` `fn path`

### Group 24 — fmt (4 sites, ~3 lines, `faf5a65d5fae7027`)

- `rust/src/memory/upgrade.rs:186` `fn fmt`
- `rust/src/memory_program.rs:176` `fn fmt`
- `rust/src/memory_query_language/mod.rs:586` `fn fmt`
- `rust/src/statement_audit/evidence.rs:48` `fn fmt`

### Group 25 — escape, lino_escape (4 sites, ~3 lines, `d2c1e588ebbfd2f2`)

- `rust/src/agentic_coding/repair_loop.rs:448` `fn lino_escape`
- `rust/src/external_benchmarks/ledger.rs:288` `fn escape`
- `rust/src/links_query.rs:173` `fn escape`
- `rust/src/needs.rs:238` `fn escape`

### Group 26 — evidence, payload, render_fields, trace_payload (4 sites, ~7 lines, `c14a36411e4e81fb`)

- `rust/src/event_log.rs:211` `fn render_fields`
- `rust/src/question_necessity.rs:388` `fn trace_payload`
- `rust/src/summarization/validation/criteria.rs:238` `fn evidence`
- `rust/src/trace_record.rs:20` `fn payload`

### Group 27 — epoch_seconds, now_seconds, unix_now (4 sites, ~6 lines, `bd498dbc744749d2`)

- `rust/src/anticipation.rs:748` `fn epoch_seconds`
- `rust/src/coding/discovered_procedures.rs:215` `fn unix_now`
- `rust/src/dreaming_runtime.rs:168` `fn now_seconds`
- `rust/src/verifiable_task/ledger.rs:186` `fn unix_now`

### Group 28 — child_values, values (4 sites, ~7 lines, `103319dd08a1d040`)

- `rust/src/coding/discovered_procedures.rs:262` `fn values`
- `rust/src/method_registry.rs:811` `fn child_values`
- `rust/src/skill_procedure/artifact.rs:344` `fn child_values`
- `rust/src/verifiable_task/ledger.rs:201` `fn values`

### Group 29 — word_entries (4 sites, ~21 lines, `36c0ca599ab730ed`)

- `rust/src/solver_handlers/regex_synthesis.rs:63` `fn word_entries`
- `rust/src/solver_handlers/shell_command_compose.rs:107` `fn word_entries`
- `rust/src/solver_handlers/sql_synthesis.rs:61` `fn word_entries`
- `rust/src/solver_handlers/test_generation.rs:63` `fn word_entries`

### Group 30 — normalize (3 sites, ~7 lines, `f4f42ea3699e739f`)

- `rust/src/client_contract_learning.rs:521` `fn normalize`
- `rust/src/computer_use/seed.rs:331` `fn normalize`
- `rust/src/option_network.rs:849` `fn normalize`

### Group 31 — new (3 sites, ~5 lines, `1a7db71f1f4a47f8`)

- `rust/src/coding/discovered_procedures.rs:139` `fn new`
- `rust/src/concept_sense_ledger.rs:47` `fn new`
- `rust/src/verifiable_task/ledger.rs:93` `fn new`

### Group 32 — prefix_literals (3 sites, ~8 lines, `576bcf3c6fd3a93f`)

- `rust/src/entity_resolution.rs:151` `fn prefix_literals`
- `rust/src/solver_handlers/user_intent.rs:17` `fn prefix_literals`
- `rust/src/web_search_markers.rs:146` `fn prefix_literals`

### Group 33 — suffix_frames, suffix_literals (3 sites, ~8 lines, `1d29b33db9920a76`)

- `rust/src/entity_resolution.rs:162` `fn suffix_literals`
- `rust/src/translation/prompt.rs:239` `fn suffix_frames`
- `rust/src/web_search_markers.rs:157` `fn suffix_literals`

### Group 34 — push_trimmed, push_trimmed_segment (3 sites, ~6 lines, `cd12e85004ab0f46`)

- `rust/src/meta_frame.rs:587` `fn push_trimmed`
- `rust/src/solver_helpers/mod.rs:266` `fn push_trimmed_segment`
- `rust/src/task_decomposition.rs:648` `fn push_trimmed`

### Group 35 — escape_value, quote (3 sites, ~8 lines, `3cb7723f1459ded2`)

- `rust/src/memory.rs:584` `fn escape_value`
- `rust/src/promotion.rs:861` `fn quote`
- `rust/src/research_learning.rs:600` `fn quote`

### Group 36 — is_ideographic, is_unspaced_script (3 sites, ~3 lines, `c20f5069d4f015fe`)

- `rust/src/agentic_coding/directory_listing.rs:77` `fn is_unspaced_script`
- `rust/src/seed/caller_context.rs:193` `fn is_unspaced_script`
- `rust/src/skill_procedure.rs:758` `fn is_ideographic`

### Group 37 — push_field (3 sites, ~9 lines, `b83978cd3af71554`)

- `rust/src/associative_package.rs:873` `fn push_field`
- `rust/src/skill_compiler.rs:799` `fn push_field`
- `rust/src/skill_procedure.rs:794` `fn push_field`

### Group 38 — unix_now (3 sites, ~5 lines, `a3ed690b08a30fc2`)

- `rust/src/github_logs.rs:423` `fn unix_now`
- `rust/src/service_accessibility.rs:399` `fn unix_now`
- `rust/src/source_fetch.rs:541` `fn unix_now`

### Group 39 — collapse_whitespace, collapse_ws, normalize_single_line (3 sites, ~3 lines, `2157f6921ed301cf`)

- `rust/src/issue_report.rs:421` `fn normalize_single_line`
- `rust/src/proof_engine/decision.rs:104` `fn collapse_whitespace`
- `rust/src/solver_handlers/conversation_memory/memory_write.rs:275` `fn collapse_ws`

### Group 40 — root (3 sites, ~3 lines, `b5ffd466cf35300b`)

- `rust/src/agent.rs:210` `fn root`
- `rust/src/computer_use/executor.rs:121` `fn root`
- `rust/src/repository_workspace/mod.rs:181` `fn root`

### Group 41 — localized_for (3 sites, ~6 lines, `430ae1d5a19fbf67`)

- `rust/src/seed.rs:557` `fn localized_for`
- `rust/src/seed/facts.rs:78` `fn localized_for`
- `rust/src/seed/projects.rs:90` `fn localized_for`

### Group 42 — mapping_rows (3 sites, ~5 lines, `01cf7c8403deeb1e`)

- `rust/src/solver_handlers/shell_command_compose.rs:140` `fn mapping_rows`
- `rust/src/solver_handlers/sql_synthesis.rs:94` `fn mapping_rows`
- `rust/src/solver_handlers/test_generation.rs:96` `fn mapping_rows`

### Group 43 — number_value (3 sites, ~9 lines, `d4e504f77cf26a27`)

- `rust/src/solver_handlers/regex_synthesis.rs:112` `fn number_value`
- `rust/src/solver_handlers/shell_command_compose.rs:253` `fn number_value`
- `rust/src/solver_handlers/sql_synthesis.rs:110` `fn number_value`

### Group 44 — hex, hex_nibble (3 sites, ~8 lines, `253c762b88d79785`)

- `rust/src/coding/function_catalog/rosetta_code.rs:159` `fn hex`
- `rust/src/solver_helpers/mod.rs:546` `fn hex_nibble`
- `rust/src/translation/cache.rs:304` `fn hex_nibble`

### Group 45 — field (3 sites, ~6 lines, `fd60ff401780e72b`)

- `rust/src/agentic_coding/dreaming_audit.rs:34` `fn field`
- `rust/src/agentic_coding/lexicon.rs:396` `fn field`
- `rust/src/external_benchmarks/ledger.rs:21` `fn field`

### Group 46 — render_or_id, runtime_message (3 sites, ~3 lines, `277d8eafbcf8a3ab`)

- `rust/src/coding/ir_lowering/python.rs:459` `fn render_or_id`
- `rust/src/coding/ir_lowering/rust.rs:232` `fn render_or_id`
- `rust/src/coding/program_ir/infer.rs:399` `fn runtime_message`

### Group 47 — is_green (3 sites, ~3 lines, `8f6bcb8fab07b757`)

- `rust/src/algorithm_discovery/execution.rs:34` `fn is_green`
- `rust/src/skill_procedure/learning.rs:270` `fn is_green`
- `rust/src/task_decomposition/learning.rs:88` `fn is_green`

### Group 48 — read_arguments (3 sites, ~3 lines, `79d931adf6f9f795`)

- `rust/src/agentic_coding/code_artifact.rs:334` `fn read_arguments`
- `rust/src/agentic_coding/structured_edit.rs:712` `fn read_arguments`
- `rust/src/agentic_coding/workspace_change.rs:656` `fn read_arguments`

### Group 49 — one_call, plan_one (3 sites, ~6 lines, `d2c3a1d09cd3536a`)

- `rust/src/agentic_coding/command_reroute.rs:111` `fn one_call`
- `rust/src/agentic_coding/file_read.rs:877` `fn plan_one`
- `rust/src/agentic_coding/planner.rs:953` `fn plan_one`

### Group 50 — new (2 sites, ~5 lines, `91ce4ad3c338cd98`)

- `rust/src/search_fusion_learning.rs:171` `fn new`
- `rust/src/workspace_change_learning.rs:321` `fn new`

### Group 51 — observation_count (2 sites, ~3 lines, `d03b2569b6479c69`)

- `rust/src/search_fusion_learning.rs:197` `fn observation_count`
- `rust/src/workspace_change_learning.rs:347` `fn observation_count`

### Group 52 — is_green (2 sites, ~3 lines, `6a8f036c9d863ddb`)

- `rust/src/search_fusion_learning.rs:276` `fn is_green`
- `rust/src/workspace_change_learning.rs:425` `fn is_green`

### Group 53 — new (2 sites, ~5 lines, `ef80a4c841c8e83a`)

- `rust/src/search_fusion_learning.rs:321` `fn new`
- `rust/src/workspace_change_learning.rs:469` `fn new`

### Group 54 — plan_for (2 sites, ~5 lines, `8f50e3b18c3159e1`)

- `rust/src/search_fusion_learning.rs:369` `fn plan_for`
- `rust/src/workspace_change_learning.rs:517` `fn plan_for`

### Group 55 — escape (2 sites, ~6 lines, `3a33116cf31ae752`)

- `rust/src/agentic_coding/general_planner.rs:810` `fn escape`
- `rust/src/fact_checking.rs:659` `fn escape`

### Group 56 — consume (2 sites, ~8 lines, `01345dbd16a172f2`)

- `rust/src/calculation.rs:332` `fn consume`
- `rust/src/proof_engine/decision/linear.rs:460` `fn consume`

### Group 57 — peek (2 sites, ~3 lines, `72b5d951bb23fbd7`)

- `rust/src/calculation.rs:341` `fn peek`
- `rust/src/proof_engine/decision/linear.rs:469` `fn peek`

### Group 58 — json_line, to_send_message_body (2 sites, ~3 lines, `205a26c5f51d0dae`)

- `rust/src/client_contract_learning.rs:81` `fn json_line`
- `rust/src/telegram.rs:729` `fn to_send_message_body`

### Group 59 — len (2 sites, ~3 lines, `2af2fdb9d7ccdbe4`)

- `rust/src/learning_ledger.rs:257` `fn len`
- `rust/src/memory_revision.rs:93` `fn len`

### Group 60 — is_empty (2 sites, ~3 lines, `30132c5fd2e5a558`)

- `rust/src/learning_ledger.rs:263` `fn is_empty`
- `rust/src/memory_revision.rs:99` `fn is_empty`

### Group 61 — contains_token (2 sites, ~9 lines, `9dbb1d444d3b95dd`)

- `rust/src/cue_lexicon.rs:190` `fn contains_token`
- `rust/src/intent_formalization.rs:402` `fn contains_token`

### Group 62 — push_unique (2 sites, ~5 lines, `93ac12c3af498b7c`)

- `rust/src/intent_formalization.rs:751` `fn push_unique`
- `rust/src/method_registry.rs:819` `fn push_unique`

### Group 63 — record_construction (2 sites, ~18 lines, `9a6278ef02a5f8d3`)

- `rust/src/rule_synthesis.rs:123` `fn record_construction`
- `rust/src/rule_synthesis_portfolio.rs:156` `fn record_construction`

### Group 64 — is_sentence_terminal, sentence_terminator (2 sites, ~3 lines, `55bbafa1528e86d5`)

- `rust/src/agentic_coding/web_research.rs:701` `fn is_sentence_terminal`
- `rust/src/search_fusion.rs:711` `fn sentence_terminator`

### Group 65 — capitalize_first (2 sites, ~7 lines, `b5621159dd3ab2cb`)

- `rust/src/search_fusion.rs:715` `fn capitalize_first`
- `rust/src/web_search_fusion_core.rs:617` `fn capitalize_first`

### Group 66 — days_from_iso_date, parse_iso_date (2 sites, ~16 lines, `ba12390742184da9`)

- `rust/src/memory_program/execution.rs:666` `fn parse_iso_date`
- `rust/src/release_timeline.rs:160` `fn days_from_iso_date`

### Group 67 — lower_operator (2 sites, ~3 lines, `cbfe357ed937537b`)

- `rust/src/number_constraints.rs:27` `fn lower_operator`
- `rust/src/proof_program/core.rs:24` `fn lower_operator`

### Group 68 — upper_operator (2 sites, ~3 lines, `72748194da20a8a7`)

- `rust/src/number_constraints.rs:31` `fn upper_operator`
- `rust/src/proof_program/core.rs:28` `fn upper_operator`

### Group 69 — drop (2 sites, ~3 lines, `cd79e8ef6eb73a52`)

- `rust/src/authoring_loop.rs:104` `fn drop`
- `rust/src/cli_solve.rs:497` `fn drop`

### Group 70 — fmt (2 sites, ~3 lines, `7e34ff5cb5facff7`)

- `rust/src/links_query.rs:91` `fn fmt`
- `rust/src/links_substitution_query/mod.rs:78` `fn fmt`

### Group 71 — flush, sync_parent (2 sites, ~3 lines, `b32551e9b7f4fc6f`)

- `rust/src/file_legality.rs:747` `fn flush`
- `rust/src/memory/upgrade.rs:862` `fn sync_parent`

### Group 72 — step_count (2 sites, ~3 lines, `eaade6c954a86b78`)

- `rust/src/meta_construction.rs:133` `fn step_count`
- `rust/src/recipe_interpreter.rs:95` `fn step_count`

### Group 73 — name (2 sites, ~3 lines, `2c23ff6aa63a08b8`)

- `rust/src/coding_research_learning.rs:92` `fn name`
- `rust/src/rule_interpreter.rs:496` `fn name`

### Group 74 — is_sha256, is_sha256_hex (2 sites, ~6 lines, `711127fdfacd8e31`)

- `rust/src/coding_research_learning.rs:745` `fn is_sha256`
- `rust/src/source_fetch.rs:494` `fn is_sha256_hex`

### Group 75 — flatten_lino_value, sanitize_payload (2 sites, ~6 lines, `722548147d9927ae`)

- `rust/src/event_log.rs:798` `fn sanitize_payload`
- `rust/src/links_format.rs:127` `fn flatten_lino_value`

### Group 76 — user (2 sites, ~3 lines, `1219ac039e00bc1f`)

- `rust/src/protocol.rs:202` `fn user`
- `rust/src/summarization/dialog.rs:35` `fn user`

### Group 77 — assistant (2 sites, ~3 lines, `7922dd14a3d193a5`)

- `rust/src/protocol.rs:208` `fn assistant`
- `rust/src/summarization/dialog.rs:41` `fn assistant`

### Group 78 — is_ideographic, is_unspaced_script (2 sites, ~9 lines, `64c1c5137beaf9f4`)

- `rust/src/obligation_ledger/derivation.rs:89` `fn is_unspaced_script`
- `rust/src/question_necessity.rs:615` `fn is_ideographic`

### Group 79 — catalog, google_trends_catalog (2 sites, ~3 lines, `c30e0ba7809141d9`)

- `rust/src/agentic_coding/question_catalog.rs:234` `fn catalog`
- `rust/src/google_trends_catalog.rs:188` `fn google_trends_catalog`

### Group 80 — escape_lino_value, escape_value (2 sites, ~3 lines, `3b0e43ac253d7922`)

- `rust/src/agentic_coding/question_catalog.rs:268` `fn escape_value`
- `rust/src/google_trends_catalog.rs:461` `fn escape_lino_value`

### Group 81 — push_response_stream_event, push_sse_event (2 sites, ~8 lines, `ab1f9f915eb65f65`)

- `rust/src/anthropic.rs:595` `fn push_sse_event`
- `rust/src/responses_stream.rs:331` `fn push_response_stream_event`

### Group 82 — learned_program_rule_lino (2 sites, ~5 lines, `bada04661c3e5995`)

- `rust/src/promotion.rs:552` `fn learned_program_rule_lino`
- `rust/src/self_improvement.rs:635` `fn learned_program_rule_lino`

### Group 83 — default (2 sites, ~3 lines, `8c8198986aa23614`)

- `rust/src/memory.rs:132` `fn default`
- `rust/src/world_model_dialog.rs:292` `fn default`

### Group 84 — len (2 sites, ~3 lines, `b1234253b257738d`)

- `rust/src/link_store.rs:431` `fn len`
- `rust/src/memory.rs:287` `fn len`

### Group 85 — is_empty (2 sites, ~3 lines, `378dfdd94d7ac80a`)

- `rust/src/link_store.rs:437` `fn is_empty`
- `rust/src/memory.rs:292` `fn is_empty`

### Group 86 — export_links_notation, to_links_notation (2 sites, ~3 lines, `e7317f32f4b12a67`)

- `rust/src/memory.rs:299` `fn export_links_notation`
- `rust/src/memory_sync.rs:273` `fn to_links_notation`

### Group 87 — peek (2 sites, ~3 lines, `cad6feefaab58bc7`)

- `rust/src/arithmetic.rs:600` `fn peek`
- `rust/src/memory_query_language/syntax.rs:143` `fn peek`

### Group 88 — get, slug (2 sites, ~3 lines, `0bc8b3378920e56d`)

- `rust/src/language.rs:52` `fn slug`
- `rust/src/relative_meta_logic.rs:70` `fn get`

### Group 89 — config (2 sites, ~5 lines, `eb8b0f89202bcb6c`)

- `rust/src/cli_context.rs:440` `fn config`
- `rust/src/cli_report.rs:493` `fn config`

### Group 90 — is_identifier_character, is_word_character (2 sites, ~3 lines, `56e2f676fcce712e`)

- `rust/src/solver_handlers/calendar.rs:516` `fn is_word_character`
- `rust/src/workspace_change_learning.rs:156` `fn is_identifier_character`

### Group 91 — adopted (2 sites, ~3 lines, `5d8bb444e3c56268`)

- `rust/src/language_adoption.rs:100` `fn adopted`
- `rust/src/learning_adoption_ledger.rs:107` `fn adopted`

### Group 92 — unadopted (2 sites, ~3 lines, `f073de676a2fffc6`)

- `rust/src/language_adoption.rs:106` `fn unadopted`
- `rust/src/learning_adoption_ledger.rs:113` `fn unadopted`

### Group 93 — live_api_enabled (2 sites, ~3 lines, `066d33139e5afad9`)

- `rust/src/lexeme_import.rs:190` `fn live_api_enabled`
- `rust/src/translation/cache.rs:330` `fn live_api_enabled`

### Group 94 — module_count (2 sites, ~3 lines, `15602965f39146f4`)

- `rust/src/self_ast_census.rs:325` `fn module_count`
- `rust/src/self_source_links.rs:174` `fn module_count`

### Group 95 — concepts, public_concepts (2 sites, ~4 lines, `d04904d36818b0ad`)

- `rust/src/concepts.rs:30` `fn concepts`
- `rust/src/solver_unknown_reasoning.rs:531` `fn public_concepts`

### Group 96 — render_source_link (2 sites, ~8 lines, `61ccce3aded5f080`)

- `rust/src/concepts.rs:690` `fn render_source_link`
- `rust/src/solver_unknown_reasoning.rs:607` `fn render_source_link`

### Group 97 — unquote, unquote_seed_value (2 sites, ~6 lines, `2b665c20f1e74952`)

- `rust/src/language.rs:305` `fn unquote`
- `rust/src/web_engine_core.rs:263` `fn unquote_seed_value`

### Group 98 — language_code_of, meaning_defined_language_code (2 sites, ~6 lines, `8bd0bbbaef27efb8`)

- `rust/src/concepts.rs:118` `fn meaning_defined_language_code`
- `rust/src/translation/language_markers.rs:76` `fn language_code_of`

### Group 99 — language_code (2 sites, ~5 lines, `b80a4fd6dff1f872`)

- `rust/src/concepts.rs:125` `fn language_code`
- `rust/src/translation/language_markers.rs:89` `fn language_code`

### Group 100 — field_value (2 sites, ~12 lines, `d17dfab89f0efb0d`)

- `rust/src/meta_self_improvement.rs:326` `fn field_value`
- `rust/src/recipe_interpreter.rs:480` `fn field_value`

### Group 101 — records (2 sites, ~3 lines, `c0bbe933ebe85d86`)

- `rust/src/probability.rs:324` `fn records`
- `rust/src/service_accessibility.rs:233` `fn records`

### Group 102 — validated (2 sites, ~3 lines, `02c3335370256926`)

- `rust/src/algorithm_discovery.rs:142` `fn validated`
- `rust/src/learning_cycle.rs:211` `fn validated`

### Group 103 — validated_candidates (2 sites, ~6 lines, `ae30c32038323766`)

- `rust/src/algorithm_discovery.rs:422` `fn validated_candidates`
- `rust/src/learning_cycle.rs:260` `fn validated_candidates`

### Group 104 — named_child, optional_child (2 sites, ~3 lines, `74f0ef30aef76c4d`)

- `rust/src/algorithm_discovery.rs:960` `fn optional_child`
- `rust/src/solver_handlers/formalization_task.rs:162` `fn named_child`

### Group 105 — navigate_mut (2 sites, ~7 lines, `ae4ad9d4e5d44d52`)

- `rust/src/json_lino.rs:533` `fn navigate_mut`
- `rust/src/seed/parser.rs:69` `fn navigate_mut`

### Group 106 — seeded (2 sites, ~6 lines, `71384022875fc145`)

- `rust/src/solver_search.rs:242` `fn seeded`
- `rust/src/summarization/validation/sampling.rs:148` `fn seeded`

### Group 107 — next_u64 (2 sites, ~7 lines, `8c5e0316318faaa9`)

- `rust/src/solver_search.rs:249` `fn next_u64`
- `rust/src/summarization/validation/sampling.rs:154` `fn next_u64`

### Group 108 — below (2 sites, ~10 lines, `5f7a43f2f6d54592`)

- `rust/src/solver_search.rs:257` `fn below`
- `rust/src/summarization/validation/sampling.rs:162` `fn below`

### Group 109 — from (2 sites, ~3 lines, `e1a89c5f5837a96b`)

- `rust/src/agent.rs:172` `fn from`
- `rust/src/computer_use/executor.rs:38` `fn from`

### Group 110 — config (2 sites, ~5 lines, `8a6f9c1d8f875311`)

- `rust/src/conversation_context.rs:171` `fn config`
- `rust/src/server/conversation_reports.rs:57` `fn config`

### Group 111 — split_tuple (2 sites, ~7 lines, `76453f607229e650`)

- `rust/src/box_language_projects.rs:306` `fn split_tuple`
- `rust/src/prerequisite/probe.rs:325` `fn split_tuple`

### Group 112 — source_url (2 sites, ~3 lines, `cd7e8fd955cf313d`)

- `rust/src/probability.rs:64` `fn source_url`
- `rust/src/source_fetch.rs:182` `fn source_url`

### Group 113 — fetched_at (2 sites, ~3 lines, `505079a1fbb87d21`)

- `rust/src/probability.rs:69` `fn fetched_at`
- `rust/src/source_fetch.rs:187` `fn fetched_at`

### Group 114 — sha256 (2 sites, ~3 lines, `d436a8ab29e857a7`)

- `rust/src/probability.rs:74` `fn sha256`
- `rust/src/source_fetch.rs:192` `fn sha256`

### Group 115 — cached (2 sites, ~3 lines, `3dce509f4bf8b37e`)

- `rust/src/probability.rs:79` `fn cached`
- `rust/src/source_fetch.rs:197` `fn cached`

### Group 116 — with_online (2 sites, ~4 lines, `2611489e54361551`)

- `rust/src/source_fetch.rs:252` `fn with_online`
- `rust/src/translation/cache.rs:99` `fn with_online`

### Group 117 — finite_clamped (2 sites, ~7 lines, `dd8118bdbac1a7c2`)

- `rust/src/probability.rs:792` `fn finite_clamped`
- `rust/src/translation/selection.rs:289` `fn finite_clamped`

### Group 118 — usize_to_f32 (2 sites, ~4 lines, `42f9891c7703fd0a`)

- `rust/src/probability.rs:800` `fn usize_to_f32`
- `rust/src/translation/selection.rs:376` `fn usize_to_f32`

### Group 119 — circumfix_frames, circumfix_literals (2 sites, ~8 lines, `92d403320c6b98ce`)

- `rust/src/translation/prompt.rs:250` `fn circumfix_frames`
- `rust/src/web_search_markers.rs:168` `fn circumfix_literals`

### Group 120 — bare_literals (2 sites, ~8 lines, `5bd9c22e6ed3594d`)

- `rust/src/solver_handlers/user_intent.rs:29` `fn bare_literals`
- `rust/src/web_search_markers.rs:180` `fn bare_literals`

### Group 121 — new (2 sites, ~3 lines, `97f12c40438088cf`)

- `rust/src/links_substitution_query/links.rs:101` `fn new`
- `rust/src/normal_markov.rs:58` `fn new`

### Group 122 — new (2 sites, ~5 lines, `2e1ae1f6c5ff7387`)

- `rust/src/memory_query_language/mod.rs:578` `fn new`
- `rust/src/statement_audit/evidence.rs:40` `fn new`

### Group 123 — pick_topic (2 sites, ~8 lines, `41d31ed5ad234f50`)

- `rust/src/seed/personas.rs:82` `fn pick_topic`
- `rust/src/seed/summary_topics.rs:79` `fn pick_topic`

### Group 124 — matches_trigger (2 sites, ~5 lines, `5738d167dc597505`)

- `rust/src/seed/brainstorm.rs:38` `fn matches_trigger`
- `rust/src/seed/personas.rs:63` `fn matches_trigger`

### Group 125 — collect_language_values (2 sites, ~13 lines, `a03f85e08f1277db`)

- `rust/src/seed/shell_intents.rs:341` `fn collect_language_values`
- `rust/src/seed/terminal_commands.rs:79` `fn collect_language_values`

### Group 126 — draft_strategies_from, planner_precedence_from (2 sites, ~10 lines, `db53e661f773fd62`)

- `rust/src/seed/draft_strategies.rs:32` `fn draft_strategies_from`
- `rust/src/seed/planner_precedence.rs:53` `fn planner_precedence_from`

### Group 127 — has_char_in_range, meta_has_char_in_range (2 sites, ~3 lines, `b062021f54d65fbe`)

- `rust/src/solver_handlers/meta_explanation.rs:269` `fn meta_has_char_in_range`
- `rust/src/solver_handlers/self_awareness.rs:518` `fn has_char_in_range`

### Group 128 — function_name (2 sites, ~17 lines, `598824c755086706`)

- `rust/src/solver_handlers/code_debugging.rs:147` `fn function_name`
- `rust/src/solver_handlers/code_explanation.rs:176` `fn function_name`

### Group 129 — identifier_at (2 sites, ~6 lines, `825ea9d69432d583`)

- `rust/src/solver_handlers/code_explanation.rs:195` `fn identifier_at`
- `rust/src/solver_handlers/test_generation.rs:162` `fn identifier_at`

### Group 130 — render_line, response (2 sites, ~5 lines, `9d5b031b78ffa69a`)

- `rust/src/coding/rosetta_request.rs:166` `fn response`
- `rust/src/solver_handlers/text_rewrite.rs:230` `fn render_line`

### Group 131 — is_url_wrapper_punctuation (2 sites, ~6 lines, `8708cf72c8f5e491`)

- `rust/src/solver_handlers/web_requests/url_parse.rs:32` `fn is_url_wrapper_punctuation`
- `rust/src/solver_handlers/web_search_intent.rs:218` `fn is_url_wrapper_punctuation`

### Group 132 — is_url_trailing_punctuation (2 sites, ~3 lines, `df5e80d11fa04f1d`)

- `rust/src/solver_handlers/web_requests/url_parse.rs:39` `fn is_url_trailing_punctuation`
- `rust/src/solver_handlers/web_search_intent.rs:225` `fn is_url_trailing_punctuation`

### Group 133 — localized_text (2 sites, ~8 lines, `f561911e4cebf2fc`)

- `rust/src/solver_handlers/behavior_rule_followups.rs:89` `fn localized_text`
- `rust/src/solver_handlers/behavior_rules.rs:297` `fn localized_text`

### Group 134 — echo (2 sites, ~6 lines, `64c1f44a16e4d71b`)

- `rust/src/solver_handlers/shell_command_compose.rs:220` `fn echo`
- `rust/src/solver_handlers/sql_synthesis.rs:101` `fn echo`

### Group 135 — rate_source_step (2 sites, ~7 lines, `8e69dfbeadac7671`)

- `rust/src/solver_handlers/calculator_rate.rs:97` `fn rate_source_step`
- `rust/src/solver_handlers/compound_interest.rs:435` `fn rate_source_step`

### Group 136 — iso, iso_date (2 sites, ~3 lines, `1cd9218a772c89d8`)

- `rust/src/solver_handlers/calendar.rs:224` `fn iso`
- `rust/src/solver_handlers/calendar_ics.rs:28` `fn iso_date`

### Group 137 — is_numeral_token_character, is_word_character (2 sites, ~3 lines, `c99efd5b809e0d7a`)

- `rust/src/solver_handlers/calendar.rs:468` `fn is_numeral_token_character`
- `rust/src/solver_handlers/software_project_phrases.rs:353` `fn is_word_character`

### Group 138 — new (2 sites, ~3 lines, `13737a4c7937db04`)

- `rust/src/translation/pipeline.rs:93` `fn new`
- `rust/src/translation/wikidata.rs:24` `fn new`

### Group 139 — required (2 sites, ~7 lines, `d730b878e435620a`)

- `rust/src/coding/discovered_procedures.rs:254` `fn required`
- `rust/src/verifiable_task/ledger.rs:193` `fn required`

### Group 140 — is_identifier_start, is_variable_start (2 sites, ~3 lines, `c155ae01bd0e9fba`)

- `rust/src/proof_engine/decision/linear.rs:478` `fn is_variable_start`
- `rust/src/summarization/file.rs:762` `fn is_identifier_start`

### Group 141 — new (2 sites, ~6 lines, `232e28bb6c304463`)

- `rust/src/statement_audit/model.rs:15` `fn new`
- `rust/src/summarization/validation/mod.rs:403` `fn new`

### Group 142 — record_type (2 sites, ~3 lines, `b3f629e8a0884902`)

- `rust/src/agentic_coding/dreaming_audit.rs:41` `fn record_type`
- `rust/src/external_benchmarks/ledger.rs:29` `fn record_type`

### Group 143 — live_fetch_enabled (2 sites, ~3 lines, `f54afa05616b6c08`)

- `rust/src/coding/synthesis_runtime.rs:424` `fn live_fetch_enabled`
- `rust/src/external_benchmarks/mod.rs:338` `fn live_fetch_enabled`

### Group 144 — reuse_slug (2 sites, ~6 lines, `b5434a8d886ab8f7`)

- `rust/src/coding/fragment_catalog.rs:720` `fn reuse_slug`
- `rust/src/coding/program_ir.rs:429` `fn reuse_slug`

### Group 145 — annotation_type, type_from_annotation (2 sites, ~11 lines, `1ce313a3293450e8`)

- `rust/src/coding/composition_search/lowering.rs:76` `fn annotation_type`
- `rust/src/coding/program_ir.rs:377` `fn type_from_annotation`

### Group 146 — neutral_literal (2 sites, ~12 lines, `e4d4b30c7a3f4f2a`)

- `rust/src/coding/composition_search/lowering.rs:64` `fn neutral_literal`
- `rust/src/coding/program_ir.rs:402` `fn neutral_literal`

### Group 147 — normalized_words, words (2 sites, ~6 lines, `bbc8a28ddceea3a2`)

- `rust/src/coding/composition_search/lowering.rs:182` `fn words`
- `rust/src/coding/program_ir.rs:422` `fn normalized_words`

### Group 148 — python_identifier, rust_identifier (2 sites, ~19 lines, `99e2917ec0c58a54`)

- `rust/src/coding/ir_lowering/python.rs:427` `fn python_identifier`
- `rust/src/coding/ir_lowering/rust.rs:200` `fn rust_identifier`

### Group 149 — render (2 sites, ~3 lines, `4c4ff9f33cfa4580`)

- `rust/src/coding/ir_lowering/python.rs:455` `fn render`
- `rust/src/coding/ir_lowering/rust.rs:228` `fn render`

### Group 150 — steps (2 sites, ~3 lines, `82d5d62d6bdabdbb`)

- `rust/src/agentic_coding/mutating_action.rs:63` `fn steps`
- `rust/src/repository_workspace/mod.rs:380` `fn steps`

### Group 151 — arg_or_empty, arg_str (2 sites, ~6 lines, `76178c7c919150ae`)

- `rust/src/agentic_coding/driver.rs:502` `fn arg_str`
- `rust/src/computer_use/executor.rs:736` `fn arg_or_empty`

### Group 152 — required (2 sites, ~7 lines, `d3c264a933a5ec5d`)

- `rust/src/task_decomposition/artifact.rs:173` `fn required`
- `rust/src/task_decomposition/learning.rs:389` `fn required`

### Group 153 — config, seed_text (2 sites, ~5 lines, `92b59794e904a0f6`)

- `rust/src/agentic_coding/report_issue.rs:582` `fn config`
- `rust/src/agentic_coding/web_research.rs:756` `fn seed_text`

### Group 154 — round_confidence (2 sites, ~3 lines, `926c21f40002829b`)

- `rust/src/agentic_coding/google_trends_catalog.rs:150` `fn round_confidence`
- `rust/src/agentic_coding/question_catalog.rs:278` `fn round_confidence`

### Group 155 — read_arguments (2 sites, ~8 lines, `f0541f2178cf31b5`)

- `rust/src/agentic_coding/general_execution.rs:529` `fn read_arguments`
- `rust/src/agentic_coding/intent_router.rs:146` `fn read_arguments`

### Group 156 — argument_path, tool_argument_path (2 sites, ~7 lines, `edd14cd8afec88d5`)

- `rust/src/agentic_coding/general_execution.rs:521` `fn tool_argument_path`
- `rust/src/agentic_coding/progress.rs:561` `fn argument_path`

