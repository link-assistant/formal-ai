TASK (tag CIFIX2): fix the Rust CI failures on 6e1c539fc. The panic excerpts are in `experiments/formal_ai_subagent/sandboxes/ci-6e1c/panics.txt`; the full job logs are next to it (`all.txt`, `*.log`).

Already handled, skip these: the self-AST census tests (issue_673/538/558; LEAD has a census commit), `issue_699` 51 vs 48 (MIGRATE4 updated it), p133 (ROUTE3), the clippy errors, and `planner_route_arms` (TEACH-F is folding arms).

To fix:
1. **Write requests now plan `read_file`.** Four tests show it:
   - `issue_745::explicit_content_and_file_object_route_write_variations`: "add file note.txt containing hello"
   - `issue_680::write_intent_routes_to_write_tool_in_any_phrasing`: "add a new file src/lib.rs with content pub fn ready() {}"
   - `issue_712::declarative_new_file_routes_to_write_and_never_read`
   - `issue_712::all_reported_capability_classes_route_in_one_matrix`: "new file: notes.txt, contents: hello"

   Suspects from round 14:
   - TEACH-E's `contents_source` (G50/G51, "the contents of a.txt" reads the source file);
   - LEAD's G68 `grounded_setting` change (`file_contents_source_cue`);
   - TEACH-F's G69 `unquoted_addition_path` arm.

   First run the same prompts through the JS planner (`node experiments/formal_ai_subagent/probe.mjs plan "<prompt>" --tools read,write,edit,bash`). If JS already writes, make the Rust twin match. If JS is wrong too, fix JS first. A file being created *with* contents is a write; only "the contents of <existing file>" is a source read.
2. **Explicit and search grep commands are rewritten.**
   - `issue_749::explicit_passthrough_covers_the_full_command_taxonomy`: "execute grep TODO note.txt" must run `grep TODO note.txt` as written.
   - `issue_749::local_search_is_shell_routed_instead_of_web_searched` and `issue_745::code_search_prefers_an_advertised_grep_capability_over_shell_lowering`: these expect the pattern `RouteIntent`, not `\bRouteIntent\b`, and the earlier grep form.

   The cause is TEACH-C's `workspace_search` arm (round 14). An explicit command runs verbatim. Decide whether the whole-word form is a deliberate improvement. If it is, update the tests with a written reason; if not, restore the earlier lowering. Keep JS and Rust the same either way.
3. **`issue_745::directory_listing_routes_shell_variations_in_every_supported_language`:** "qué hay en la carpeta actual" gives `ls 'la'` where `ls` is expected. Spanish articles must not be read as a path; the seed vocabulary for articles likely needs to cover this.
4. **`issue_1138_self_use_concept_lookup::every_seeded_response_intent_serves_all_five_languages_or_has_exact_debt`:** response debt is measured 94 against a ceiling of 93. Find the seeded response intent that lacks a language and add the missing translations. Do not raise the ceiling. Measure it on the current tree, since agents have added responses since.
5. **`ci_cd::issue_1012::audited_warning_band_sources_stay_below_their_limits`:** `src/solver.rs` has 903 lines against a 900-line warning band. Move a cohesive piece into a submodule. Check the current line count first.
6. **`repository_workspace::tests::module_files_are_where_the_convention_says`:** the module tree has more files than the plan declares. REPO-PROTO added `rust/src/repository_workspace/trace.rs`; update the declared plan list (find where the test reads it).
7. **`specification::debug_session` tests:**
   - `every_stage_carries_the_recipe_diagram_its_own_emitters_and_the_routed_method` panics at `tests/unit/specification/debug_session.rs:109` because the expected location names `solve_with_history_probability_store_and_intent_cache`.
   - `a_served_chat_turn_answers_only_after_its_last_stage` shows stage 0 advanced twice.

   Compare them with `data/meta/debug-stage-sources.lino` as it is now (LEAD regenerated it, adding `operand_program.rs`) and with the JS twin `rust/tests/web/server-debug-session.test.mjs`, which passes. Fix whichever side is wrong.

Rules:
- **No cargo and no rust-script locally.** Check Rust with `rustfmt --edition 2024 --check` only; files must stay ≤1000 lines. Use `node experiments/formal_ai_subagent/local-gates.mjs` (it runs JS twins where they exist).
- JS first, with a test in both roots whenever behaviour changes.
- Claim files in `claims.md`.
- TEACH-F is folding planner arms in `rust/src/agentic_coding/planner.rs` and `js/agentic/planner.mjs`, and editing `js/agentic/*` and `write_request`. Coordinate through claims and do not edit `planner.rs` / `planner.mjs` yourself. If a fix needs them, describe it in your report.
- SCRIPTS-A and SCRIPTS-B own `scripts/*.mjs`.
- Use Formal AI for the small edits (ledger rows T310–T319). Do not commit.
- Report each item: its cause, the fix, and the evidence.
