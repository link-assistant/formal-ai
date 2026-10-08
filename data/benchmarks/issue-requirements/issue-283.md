Parent: #244
Audit source: PR #245 post-implementation audit on 2026-05-26, after issues #246-#259 and #262 were merged.

## Problem

Issue #259 implemented a deterministic natural-language skill compiler for trigger/response rules. `ARCHITECTURE.md` still calls out the next step: broader lowering of natural-language skill definitions into executable Rust/JavaScript/native handlers and package data.

The current compiler is useful, but it does not yet cover typed arguments, multi-step procedures, validations, generated tests, or target-specific handler lowering.

## Scope

- Extend the skill language beyond trigger/response into typed inputs, preconditions, steps, effects, and expected tests.
- Lower compatible skills into package records and, where appropriate, generated Rust/JavaScript/native handler stubs.
- Keep generated behavior deterministic and traceable.
- Refuse or mark unsupported natural-language instructions instead of silently compiling unsafe behavior.
- Integrate with package/permission records from the associative-package follow-up.

## Acceptance criteria

- Tests cover typed skill definitions, multi-step procedures, generated tests, unsupported-instruction refusal, and deterministic replay.
- Generated handlers or package records can be inspected as Links Notation.
- Unsafe or permissioned actions require explicit package/tool permissions.
- Documentation distinguishes the supported skill subset from future natural-language programming goals.
- `ARCHITECTURE.md` no longer lists skill compilation as only trigger/response work.

## Requirement links

- `ARCHITECTURE.md` section 16.2
- `REQUIREMENTS.md` R65 and the issue #244 skill/learning requirements
- Follow-up from #259

