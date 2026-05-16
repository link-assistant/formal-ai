---
bump: patch
---

### Fixed
- **Report issue** dialog annotation now marks the reported message with `intent: <intent>, reported` for any intent, not only `unknown` (issue #73). Previously, clicking "Report issue" on a TypeScript hello-world response (intent: `hello_world_typescript`) produced a prefilled GitHub issue body with no annotation on the reported message — a maintainer could not tell which turn was considered problematic. The `appendDialogBlock` function now always adds `intent: <intent>` and `reported` to the focused message regardless of its intent value.
- Updated E2E Playwright coverage in `tests/e2e/tests/demo.spec.js`: the known-dialog report test now asserts `A (intent: …, reported):` appears in the body, and a new test verifies that a TypeScript hello-world dialog report includes `A (intent: hello_world_typescript, reported):`.
