# Issue #1185 Traceability Rows

Rows for the shard
`docs/requirements/issue-1185-error-driven-repair-loop.md`, kept here for
the maintainer to merge into `docs/requirements-traceability.md` (the shared
table is not edited by this change). Honesty rules follow the table's own
header: `not yet confirmed` means no manual confirmation has been recorded
for this shard, and every automated-test cell below is dormant until the
wiring rows land (module registration, test registration, seed registry),
which is the main session's half of this change.

| ID | Shard | Delivered | Automated test | Manual confirmation |
| --- | --- | --- | --- | --- |
| R1185-1 | docs/requirements/issue-1185-error-driven-repair-loop.md | 2026-09-30 (this pull request) | rust/tests/unit/issue_1185_error_repair_loop.rs: `formalize_diagnostic_extracts_file_line_code_and_message_from_rustc_output`, `formalize_diagnostic_extracts_from_kotlinc_output_with_no_error_code`, `formalize_diagnostic_reads_python_traceback_head_and_location`, `formalize_diagnostic_yields_nothing_for_output_without_a_shape` | not yet confirmed |
| R1185-2 | docs/requirements/issue-1185-error-driven-repair-loop.md | 2026-09-30 (this pull request) | same suite: `repair_loop_plans_the_diagnostic_search_before_any_final`, `search_query_bounds_the_message_fragment`, `repair_loop_records_the_fix_then_retries_the_failed_command` (fragment retention inside `repair_edit` content) | not yet confirmed |
| R1185-3 | docs/requirements/issue-1185-error-driven-repair-loop.md | 2026-09-30 (this pull request) | same suite: `repair_edit_document_is_links_notation_not_a_patch` (asserts `rendering meta_language` and the record fields), `repair_loop_records_the_fix_then_retries_the_failed_command` (writes `<stem>.repair.lino`) | not yet confirmed |
| R1185-4 | docs/requirements/issue-1185-error-driven-repair-loop.md | 2026-09-30 (this pull request) | same suite: `repair_loop_is_bounded_and_reports_reaching_the_top_rung` (`rung 3 of at most 3`) | not yet confirmed |
| R1185-5 | docs/requirements/issue-1185-error-driven-repair-loop.md | 2026-09-30 (this pull request) | same suite: `repair_attempt_chain_is_recorded_for_the_answer_derivation` (`repair_attempts`, `attempt_count 1`, `resolved true`) | not yet confirmed |
| R1185-6 | docs/requirements/issue-1185-error-driven-repair-loop.md | 2026-09-30 (this pull request) | same suite: `repair_loop_stops_when_no_fetched_source_matches_the_diagnostic` (plus the `unresolved_note` assertions) | not yet confirmed |
| R1185-7 | docs/requirements/issue-1185-error-driven-repair-loop.md | 2026-09-30 (this pull request) | reviewed: all 14 `language` records in data/seed/diagnostic-code-shapes.lino are read by the one generic matcher; the only prose surfaces are the two `localized_response` templates in data/seed/multilingual-responses-repair.lino; no language row is a Rust literal (debt-ratchet `hardcoded_language_rows` unchanged) | not yet confirmed |
| R1185-8 | docs/requirements/issue-1185-error-driven-repair-loop.md | open follow-up | none (needs the translator built; recorded as open in the shard) | not yet confirmed |
