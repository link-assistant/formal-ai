````````````text
# Compiled-skill first production slice, source hold draft

Prepared from committed head d205948915f74f56078a9740c5e834f1daab44ee. This is selected scratch planning only. No production source or native expectation is changed. The authoritative production result remains1173 total,136 carried,136 passing; candidate145 and scratch25/25 are separate observations.

## Scope and source contract

Deliver genuine deterministic trigger/response packages through real WorkerHost teaching, counting, listing and replay using actual user history. Native structured IR is a subsequent phase; the first slice must reject it explicitly rather than invent equivalent behavior. The existing native source stays authoritative and unchanged.

| Owned production touchpoint after release | Smallest change | Native authority |
| --- | --- | --- |
| js/agentic/crate/skill_compiler.mjs | Keep looksLikeSkillDescription. Add seeded pair extraction and pure compile entry returning an explicit unsupported result or a genuine package. Retain source_description, canonical normalized trigger, native response, legacy ID, compiled package/rule/handler IDs; package exposes replay, ordered notation and actual typed link records. | rust/src/skill_compiler.rs compile_natural_language_skill, CompiledSkillPackage::new/replay/links_notation/link_records; current native structured parser runs first |
| js/worker/formal_ai_worker_capabilities_and_runtime_rules.js | Replace legacy runtimeRuleFromText data with the shared compiled package. Collect user turns only, first-seen package-ID dedup in history order, newest matching user for replay. Teaching records actual compile package ID plus legacy update ID. Count/list use the same collection. Replay emits real package notation, matched rule ID and cache hit in native order before response:packageID. Preserve earlier conversation preference/correction/compiled procedure and remaining handler precedence. | rust/src/solver_handlers/behavior_rules.rs try_behavior_rules_with_runtime, collect_runtime_rules, runtime_rule_for_prompt, log_behavior_rule_count |
| js/worker/formal_ai_worker_behavior_rules_and_self_facts.js | Read package.response for the seeded when-then renderer. Teaching links include compiled type, legacy ID, original trigger/response, when-then, actual handler and replay mode. Runtime list prints package ID and legacy ID together. Preserve seeded prose and all built-in catalog rows. Add native count closing newline. | Same native file render_runtime_rule_update/render_behavior_rule_list; behavior_rule_followups.rs render_behavior_rule_count |
| scripts/generate-worker-crate-modules.mjs plus actual worker inventory | Coordinator adds crate/skill_compiler.mjs to real entry closure or the portable package module if split, then faithful factories/TypeScript/worker budget projection. No hand-generated duplicate compiler and no arbitrary ceiling raise. | Existing ENTRY_MODULES mechanism; compiler factory currently absent |

Reuse existing normalizePrompt and stableId helpers. Use existing native Links Notation codec and schema version constant, not new JSON-only evidence or guessed IDs. The shared package constructor should be portable and stateless, with no source-tree filesystem dependency in the worker. A separate small package module is acceptable if splitting keeps each meaningful authored module and generated frame under its real bound. No new language prompt inventory belongs in code.

## Actual integration behavior

The server WorkerHost boots the real browser worker; its solve method accepts history turns. Each typed history turn becomes actual prior_turn:user or prior_turn:assistant evidence. The worker tryBehaviorRules handler receives that history directly. Existing solverEventLog consumes answer.solverEvents and answer.responseLink and the native event projector preserves them. Therefore source-owned package events must be attached at the handler that genuinely compiled and matched them. Event projection and server solve stay unchanged unless a real incompatibility is observed.

Teaching must produce a compiled package confirmation without needing a replay first. Counting/listing compile the actual prior user teaching descriptions and ignore assistant impostors. Canonically equivalent triggers with equal responses deduplicate by package ID even if legacy IDs differ. Different responses remain different packages; newest matching user wins replay. A nonmatch preserves normal solver routing. Count events should carry actual built-in/runtime/total quantities and native reasoning operation/result, not prefilled marker strings.

## Meaningful original assertions and heldouts

1. Preserve all five assertions in rust/tests/unit/specification/natural_language_skill_compilation.rs solver_prefers_compiled_skill_from_history_and_records_cache_hit. Its SKILL/TRIGGER/RESPONSE come from real native source constants; actual production WorkerHost must pass without in-memory handler replacement.
2. Preserve behavior_rules_count_includes_dialog_local_runtime_rules exact native answer including its closing newline. Use real history and assert one genuine package counted.