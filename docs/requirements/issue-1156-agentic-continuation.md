## Issue #1156: Single output obligations

Source: https://github.com/link-assistant/formal-ai/issues/1156

| Requirement | Drafted behavior | Status |
|---|---|---|
| Preserve the requested behavior in the agentic harness | Repeated identical quoted stdout requirements are deduplicated in first-mention order; distinct output operands retain their order. The generated output verifier therefore compares against the composed single obligation. Workflows use the recipe artifact stem rather than a shared run.yml path. | Drafted. Inline regressions cover duplicate and distinct output requirements. Existing output-verifier template already uses byte comparison and propagates process failure. No local execution was performed. Repository/task-specific workflow naming beyond artifact stems remains open. |
