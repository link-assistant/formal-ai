# Claims for round 13 (round 12 claims archived in claims-round12.txt; those agents finished and their work is committed at 3ca01c45a).
LEAD js/agentic/write_request.mjs
LEAD rust/src/agentic_coding/write_request.rs
LEAD docs/case-studies/pull-request-1188/formal-ai-dogfood.md rows T21-T38 (TEACH-A takes T39-T49, TEACH-B T50-T59, MIGRATE2 T60-T69, DEBUG2 T70-T79)
LEAD data/meta/self-ast/ (census artifact commits)
TEACH-B js/agentic/positional_edit.mjs (released by LEAD)
TEACH-B rust/src/agentic_coding/positional_edit.rs (released by LEAD)
TEACH-B rust/tests/web/pull-request-1188-replace-semantics.test.mjs (new)
TEACH-B rust/tests/unit/pull_request_1188_replace_semantics.rs (new)
TEACH-B js/agentic/workspace_change.mjs functions: groundedSetting, assignment (small hunks; shared with TEACH-A)
TEACH-B rust/src/agentic_coding/workspace_computed_change.rs functions: grounded_setting, assignment (small hunks; shared with TEACH-A)
LEAD js/agentic/general_planner.mjs (G13 literal-write guard)
LEAD rust/src/agentic_coding/general_planner.rs (G13)
MIGRATE2 rust/src/solver_handlers/calendar.rs
MIGRATE2 rust/src/solver_handlers/calendar_create.rs
MIGRATE2 rust/src/solver_handlers/calendar/ (month.rs, date_weekday.rs)
MIGRATE2 rust/src/retrieval_procedures.rs
MIGRATE2 js/worker/formal_ai_worker_08.js
MIGRATE2 js/worker/formal_ai_worker_calendar_offset.js
MIGRATE2 data/seed/handler-rules.lino (+ rust/embedded mirror)
MIGRATE2 data/seed/multilingual-responses-quantities.lino (+ rust/embedded mirror)
MIGRATE2 rust/tests/unit/issue_918_handler_rules_batch.rs
MIGRATE2 rust/tests/web/issue-0918-handler-rules-batch.test.mjs
MIGRATE2 data/meta/handler-migration-ledger.lino
MIGRATE2 data/meta/debt-ratchet.lino
MIGRATE2 data/meta/core-boundary-ledger.lino
MIGRATE2 data/meta/worker-line-budget/formal_ai_worker_08.lino, formal_ai_worker_calendar_offset.lino
MIGRATE2 scripts/hardcoded-language-allowlist.txt
MIGRATE2 docs/requirements rows R918-2 R914-6 R1085-2 R344 (+ regenerated status files)
LEAD ledger rows T80-T89
DEBUG2 js/server/debug-session.mjs
DEBUG2 js/server/debug-stage.mjs (new)
DEBUG2 js/server/solve.mjs (gate placement only)
DEBUG2 rust/src/server/debug_session.rs
DEBUG2 rust/src/server/debug_stage.rs (new)
DEBUG2 rust/src/server.rs (mod/re-export lines for debug_stage only)
DEBUG2 rust/src/derivation.rs (gate placement in finalize_answer only)
DEBUG2 rust/tests/web/server-debug-session.test.mjs
DEBUG2 rust/tests/unit/specification/debug_session.rs
DEBUG2 js/app/debugger-view.jsx
DEBUG2 js/mermaid-entry.js (new)
DEBUG2 js/styles/07-interactions.css (debugger rules only)
DEBUG2 package.json (build:web + mermaid dep) and bun.lock
DEBUG2 .gitignore (mermaid bundle line)
DEBUG2 docs/vscode/debugger.md
DEBUG2 docs/requirements/issue-0538-detailed-meanings-and-words.md row R383 only
DEBUG2 vscode/scripts/debugger-*.test.mjs (new)
DEBUG2 coverage/browser-unmeasured.txt (debugger-view line only)
TEACH-A js/agentic/workspace_line_operation.mjs (new)
TEACH-A rust/src/agentic_coding/workspace_line_operation.rs (new)
TEACH-A js/agentic/workspace_change.mjs: planWorkspaceChangeStep (one hook line), quotedPayloadAndPath, planComputedChangeStep (fragment guard), groundedLineChange (new), sentenceWords export, isVerificationFailureAnswer
TEACH-A rust/src/agentic_coding/workspace_computed_change.rs: Computation::Line variant, grounded_line_change (new), quoted_payload_and_path, plan_computed_change_step (fragment guard), sentence_words pub(super), compute/edit/reported Line arms
TEACH-A rust/src/agentic_coding/workspace_change.rs: plan_workspace_change_step (one hook), is_verification_failure_answer
TEACH-A rust/src/agentic_coding/mod.rs (register workspace_line_operation)
TEACH-A js/agentic/capability_router.mjs + rust/src/agentic_coding/capability_router.rs: routedArguments Write case (removal never a routed write)
TEACH-A data/seed/meanings-repository-workflow.lino (append new line-operation meanings at end) + data/seed/roles.lino (new roles) + data/seed/multilingual-responses-agentic-tools.lino (append new line-operation responses at end)
TEACH-A rust/tests/web/pull-request-1188-line-operations.test.mjs, rust/tests/unit/pull_request_1188_line_operations.rs, rust/tests/unit/mod.rs
TEACH-A docs/case-studies/pull-request-1188/formal-ai-dogfood.md rows T39-T49 (+ update T29,T30,T33,T35 After cells)
TEACH-B js/agentic/workspace_change.mjs: planWorkspaceChangeStep (one hook line, after TEACH-A's), groundedRewrite (one namesLine guard line), planInsertSequenceStep + insertSequenceEdit (new, appended near groundedPositionalInsert), groundedLineReplacement + replacedLines (new), groundedSetting/assignment/assignedSetting
TEACH-B rust/src/agentic_coding/workspace_change.rs: plan_workspace_change_step (one hook), grounded_rewrite (one guard), plan_insert_sequence_step (new), result_for_command -> pub(super) if needed
TEACH-B rust/src/agentic_coding/workspace_computed_change.rs: Computation::LineReplacement + Setting{old} fields, grounded_line_replacement (new), grounded_setting, assigned_setting, assignment, compute/edit arms for LineReplacement
TEACH-B data/seed/meanings-file-edit.lino (append 3 meanings at end: file_edit_old_lead, file_edit_joiner, file_edit_anchor_context) + rust/embedded mirror; data/seed/roles.lino (3 new file_edit_* roles only, small hunk)
MIGRATE2 rust/tests/unit/issue_699_handler_migration.rs (migrated count assertion)
MIGRATE2 data/meta/js-literal-ratchet.lino (worker_ceiling)
TEACH-B js/agentic/workspace_setting.mjs (new: stated old value + assignment, moved out of workspace_change.mjs)
TEACH-B rust/src/agentic_coding/workspace_setting.rs (new twin; moved assigned_setting/assignment/is_bare_literal out of workspace_computed_change.rs for the 1000-line limit)
TEACH-B rust/src/agentic_coding/modules.rs (one line: mod workspace_setting;)
MIGRATE2 data/seed/meanings-agent-actions.lino (append calendar_* table meanings at end) + data/seed/meanings-response-intents-handlers.lino (append calendar response-intent meanings at end) (+ rust/embedded mirrors)
MIGRATE2 docs/requirements/issue-0559-general-meta-algorithm.md row R344; issue-0914 row R914-6; issue-0918 row R918-2; issue-1085 row R1085-2
LEAD experiments/formal_ai_subagent/README.md and probe.mjs
MIGRATE2 docs/case-studies/pull-request-1188/formal-ai-dogfood.md rows T60-T62
TEACH-A data/seed/meanings-response-intents.lino (append nine line-operation response intents at end)
TEACH-B data/seed/meanings-coding-tasks.lino (setting meaning: bump surfaces)
TEACH-B js/agentic/write_request.mjs + rust/src/agentic_coding/write_request.rs: composeEditRequest/compose_edit_request file-clause-before-new-lead hunk only (G16, minimal)
DEBUG2 js/debugger-client.js (new), ts/debugger-client.ts, ts/mermaid-entry.ts (generated)
DEBUG2 js/agentic/diagram.mjs + rust/src/agentic_coding/diagram.rs (is_diagram_task: cues outside quoted literals, T70)
DEBUG2 rust/tests/web/agentic-recipes-a.test.mjs, rust/tests/unit/issue_538_agentic.rs (one assertion each)
DEBUG2 rust/tests/web/debugger-client.test.mjs (new), vscode/scripts/debugger-panes.test.mjs (new), vscode/package.json (test list)
TEACH-B js/agentic/planner.mjs + rust/src/agentic_coding/planner.rs: skill-description gate reads an edit request's head, not its block payload (one hunk)
