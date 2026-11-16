## Issue #1157: Idempotent recipe commits

Source: https://github.com/link-assistant/formal-ai/issues/1157

| Requirement | Drafted behavior | Status |
|---|---|---|
| Preserve the requested behavior in the agentic harness | Recipe commits stage only declared artifacts and inspect the cached diff restricted to those paths. No-change runs emit __formal_ai_commit_unchanged, then still push and print the current hash. Continuations consume the planner evidence window, retaining already completed writes and commands. | Partial. Idempotent commit and continuation replay are drafted. Output ignore patterns and verification of artifacts after externally modified continuation state remain open. No local builds or tests were run. |
