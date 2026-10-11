# Issue #1156: Single output obligations

Repeated identical quoted stdout requirements are deduplicated in first-mention order; distinct output operands retain their order. The generated output verifier therefore compares against the composed single obligation. Workflows use the recipe artifact stem rather than a shared run.yml path.

## Requirement status

Drafted. Inline regressions cover duplicate and distinct output requirements. Existing output-verifier template already uses byte comparison and propagates process failure. No local execution was performed. Repository/task-specific workflow naming beyond artifact stems remains open.

## Review evidence

Changes were inspected as source; local builds and test execution are prohibited by the user. Integration and CI evidence must be recorded before closure.
