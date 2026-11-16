# Issue #1156: Single output obligations

Repeated identical quoted stdout requirements are deduplicated in first-mention order; distinct output operands retain their order. The generated output verifier therefore compares against the composed single obligation. Workflows use the recipe artifact stem rather than a shared run.yml path.

## Requirement status

Drafted. Inline regressions cover duplicate and distinct output requirements. Existing output-verifier template already uses byte comparison and propagates process failure. No local execution was performed. Repository/task-specific workflow naming beyond artifact stems remains open.

## Review evidence

Changes were inspected as source; local builds and test execution are prohibited by the user. Integration and CI evidence must be recorded before closure.

## Issue #1162 follow-up (2026-11-16)

**Root cause.** The expected stdout was built by joining every quoted print literal. An issue body that names `Hello, World!` three times (the title, the requirement and the check) therefore produced a program and a verify script that both expected the line more than once, so the generated test agreed with the wrong program. A paraphrase ("the program outputs …") produced no output obligation at all.

**Fix** (commit `e2cc497df`). One line per distinct literal. A line repeats only when a count ("twice", "three times", "дважды") or "again" asks for it, or when distinct literals are sequenced. "outputs" is an output verb. Workflows are named after the request (commit `718c340e9`), not a shared `run.yml`.

**Tests.** `rust/tests/unit/issue_1156_explicit_stdout_coreference.rs` (7 tests, including Kotlin, Scala and Rust printing once) and `rust/tests/web/issue-1162-delivery-variants.test.mjs`. The independent grader (`rust/tests/web/issue-1162-delivery-obligations.test.mjs`) rejects a doubled line even when the generated test expects it. The variants test also pins the "again" vocabulary in all five registered languages (en, ru, hi, zh, es) and the JavaScript twin of `a_file_name_is_never_the_output`.

**Known limits** (`experiments/issue-1162/repetition-again-languages.mjs`). The "again" word is read after the repeated output, as in "prints `Hi` again" or "打印 `Hi` 再一次". A word placed before the verb, as in the common Chinese "再次打印 `Hi`", does not make a further line, so such a request prints the line once. The Spanish subjunctive "imprima" is not an output verb ("imprime" is), so "que imprima `Hi`" binds no output.

The manual end-to-end check on a released version is still **blocked**: it needs a release that contains this change and Hive Mind credentials, and neither is available to the solver. The issue therefore stays open after pull request https://github.com/link-assistant/formal-ai/pull/1579.
