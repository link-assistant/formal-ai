## Issue #1185 Error-Driven Repair Loop

Issue [#1185](https://github.com/link-assistant/formal-ai/issues/1185) (E149)
asks the executor not to stop at the first failed step: the compiler or
runtime diagnostic in the raw failed output is formalized into structured
data, the error text is searched, a fetched source that addresses the exact
error is retained as a fix, and the failed step is retried on a bounded
ladder — with every attempt recorded as evidence and every dead end reported
honestly instead of guessed around. The audit, the seam survey, and the
worked trace live in `docs/case-studies/issue-1185/`.

| ID | Requirement | Status / Evidence |
| --- | --- | --- |
| R1185-1 | The raw output of a failed build or run is formalized into a structured diagnostic — file, line, error code, message, and the raw line — through the meta language's shape table, not per-language regexes in Rust. | Partial: `formalize_diagnostic` in `rust/src/agentic_coding/repair_loop.rs` runs one generic `{slot}` template matcher over the `language` records in `data/seed/diagnostic-code-shapes.lino` (14 languages: rust, typescript, javascript, kotlin, scala, java, go, python, c, cpp, csharp, ruby, php, swift), validating each slot by shape so a template cannot read prose as a path or a line number. Location lines bind to the preceding or following diagnostic per the language's print order (rustc's ` --> file:line:col` after, gcc/python's before), and a python traceback's innermost frame wins. Absent slots stay `None`; nothing is fabricated. The E128/#1163 fetched-page formalizer is the deeper route for prose-shaped output; the shape table covers the compiler formats it does not. |
| R1185-2 | The diagnostic becomes a search query that names the language and carries the exact error code or message; the fetched pages are read, and a fragment is retained only when it addresses that exact code/message. | Partial: `search_query` composes language + error code (or bounded message head, ≤14 tokens) and `repair_step` plans the search, then the fetch of each unseen source URL the search returned (tracked through `Progress::attempted_fetches`, so a source is never refetched). `page_addresses` requires the page to contain the diagnostic's error code or cover ≥ half its message tokens; `fix_fragment` retains a fenced code block only when its preceding window addresses the diagnostic, else the best addressing sentence. |
| R1185-3 | The fix is expressed and applied in the meta language — a Links Notation `repair_edit` record written beside the artifact, never a raw text patch. | Partial: `repair_edit_document` renders the `repair_edit` record (language, file, line, error_code, message, source URL, retained fragment, `rendering meta_language`, `applied false`) and `repair_step` writes it to `<stem>.repair.lino` beside the failing artifact via the executor's write tool. The apply half — rendering that record back into the target language and editing the source — is the #1167/E132 renderer's seam; the loop delivers the record it renders and re-runs the command after the record is written. |
| R1185-4 | Retries are bounded, and the bound is reported honestly when reached. | Implemented 2026-10-07: `MAX_REPAIR_RUNGS = 3` caps the ladder; the caller owns the rung counter (a retry that fails again re-enters one rung higher), `repair_step` returns `Stop(Exhausted)` at the top, and `ladder_note` renders "rung {rung} of at most {max_rungs}" from the seed template rather than truncating silently. The executor now reports it: `plan_symbolic_command_reroute` (`rust/src/agentic_coding/command_reroute.rs`, twin `js/agentic/command_reroute.mjs`) runs `repair_step` and, on a stop, appends `stop_note` to the failure report, so a spent ladder reaches the answer. Pinned by `stop_note_reports_the_spent_ladder_and_is_silent_without_a_rung` and `rust/tests/web/issue-1185-stop-note.test.mjs`. |
| R1185-5 | Every attempt is recorded as an evidence chain reachable from the answer's derivation. | Implemented 2026-10-07: `attempts_from` reconstructs the chain from the transcript (diagnostic, query, candidate fix, applied, resolved) and `evidence_document` renders it as one `repair_attempts` record with `attempt_count` and one `repair_attempt` row per attempt, in the form #1184's derivation record attaches. The executor attaches it: a stopped loop's `stop_note` follows the honest note with the `repair_attempts` record in a fenced `lino` block in the final answer (Rust and JS roots), pinned by `stop_note_names_the_unresolved_need_and_carries_the_attempt_chain` and its JS twin. |
| R1185-6 | When no fetched source matches, the loop reports an unresolved need; it never fabricates a fix. | Implemented 2026-10-07: `Stop(NoMatch)` is returned when every fetched page was read and none addresses the diagnostic, `plan_repair` yields `None` so the caller falls back to today's honest failure report, and `unresolved_note` names the query and says the need is unresolved rather than guessed. The note is no longer only available: the executor appends it (with the attempt chain) to the failure report whenever the loop stops on `NoMatch`, in the request's language. |
| R1185-7 | The loop is uniform across all fourteen emitted languages. | Partial: the shape table carries all fourteen (rust, typescript, javascript, kotlin, scala, java, go, python, c, cpp, csharp, ruby, php, swift), the matcher, search, retention, ladder, and stops are language-agnostic, and the loop's only user-facing prose (ladder-exhausted and unresolved-need notes) renders through `localized_response` from `data/seed/multilingual-responses-repair.lino` — no language name or message shape is hardcoded in Rust. |
| R1185-8 | Three-roots parity: the same loop reachable from the js and ts roots (`formal-ai translate --to js\|ts`). | Partial: the JS root is delivered — `js/agentic/repair_loop.mjs` mirrors every public function of the Rust module (formalize, search query, retention, `repair_edit` record, attempts, evidence, ladder, stops, `stopNote`) and `js/agentic/command_reroute.mjs` drives it exactly as the Rust executor does; pinned by `rust/tests/web/agentic-write.test.mjs` and `rust/tests/web/issue-1185-stop-note.test.mjs`. The TS root does not mirror `js/agentic/` for any agentic module yet, so the third root remains open. |

Traceability: the shape table is `data/seed/diagnostic-code-shapes.lino` with
its byte-identical embedded mirror `rust/embedded/data/seed/diagnostic-code-shapes.lino`;
the response templates are `data/seed/multilingual-responses-repair.lino` with
its byte-identical mirror `rust/embedded/data/seed/multilingual-responses-repair.lino`;
the loop is `rust/src/agentic_coding/repair_loop.rs`; the probes are
`rust/tests/unit/issue_1185_error_repair_loop.rs` (twelve tests: rustc/kotlinc/
python-traceback formalization, no-shape refusal, search-before-final, no-match
stop, bounded ladder, non-diagnostic decline, record-then-retry, Links Notation
record, attempt-chain evidence, bounded query, planner-contract call shapes).

Wiring the main session owes (the loop is delivered inert by design, like an
unregistered cue table): `pub mod repair_loop;` in
`rust/src/agentic_coding/modules.rs`; `mod issue_1185_error_repair_loop;` in
`rust/tests/unit/mod.rs`; the two seeds registered in
`data/meta/seed-registry.lino` (`diagnostic-code-shapes` bundle,
`multilingual-responses-repair` bundle + `lexicon response`) with the embedded
registry refreshed; and the executor hookup — the failure branch of
`plan_symbolic_command_reroute` in `rust/src/agentic_coding/command_reroute.rs`
consults `repair_loop::plan_repair` before returning `AgenticPlan::Final`,
threading a `repair_rung: u8` through `RecipeProgress`. The exact snippet is
in the module's doc comment and the case study.
