## Issue #283 Generalized Natural-Language Skill Compiler

Issue [#283](https://github.com/link-assistant/formal-ai/issues/283)
extends the issue #259 compiler beyond trigger/response rules while keeping the
supported subset deterministic, reviewable, and permission-gated.

| ID | Requirement | Status |
| --- | --- | --- |
| R243 | Natural-language skills must support typed inputs, preconditions, ordered steps, effects, and generated expected tests without implying arbitrary natural-language programming. | Implemented by the structured `Skill` / `Input` / `Precondition` / `Step` / `Effect` / `Expected test` parser in `src/skill_compiler.rs`; covered by `typed_procedure_skill_compiles_steps_tests_and_handler_stub`. |
| R244 | Generated handlers or package records must remain inspectable as Links Notation and replay deterministically. | Implemented by `CompiledSkillPackage::links_notation`, generated Rust/JavaScript/native handler stubs, and expected-test replay fixtures; associative package lowering turns generated tests into trigger/handler records. Covered by `compiled_package_replays_deterministically_and_exports_links_notation` and `generalized_compiled_skill_package_carries_generated_tests_and_permissions`. |
| R245 | Unsafe or permissioned skill actions must require explicit package/tool permissions and unsupported instructions must be refused. | Implemented by `SkillCompileError::PermissionRequired`, `SkillCompileError::UnsupportedInstruction`, structured `Tool` / `Permission` records, and `AssociativePackage::from_compiled_skill` permission grants. Covered by `permissioned_skill_requires_explicit_package_permission` and `unsupported_instruction_is_refused`. |
