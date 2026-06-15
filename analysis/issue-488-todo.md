# Issue 488 Todo: Deep Thinking

Source issue: https://github.com/link-assistant/formal-ai/issues/488
Follow-up PR comment: https://github.com/link-assistant/formal-ai/pull/489#issuecomment-4711750533

## Subtask 1: Thinking Data Model And Solver Logic

- [x] Read issue #488 and PR #489 comments.
- [x] Merge the latest `main` into `issue-488-54ea7213044c` before follow-up work.
- [x] Make thinking steps a first-class solver output, not only a browser UI projection.
- [x] Ensure every solver answer can expose ordered thinking steps derived from the same append-only `EventLog` that powers diagnostics.
- [x] Preserve the raw diagnostics trace for maintainers while adding separate user-facing thinking metadata.
- [x] Carry thinking metadata through OpenAI-compatible chat/responses outputs so desktop/API/browser surfaces can reuse it.
- [x] Keep the model recursive/composite by preserving step kind, detail, id/order, level, source event, and optional parent metadata instead of flattening everything into prose.

## Subtask 2: User-Facing Presentation, Granularity, And Evidence

- [x] Add a collapsed thinking preview that shows the latest step by default.
- [x] Show part of the previous step with a fading/clipped treatment.
- [x] Provide an expand/collapse control for all human-readable steps.
- [x] Naturalize meta-language step identifiers into localized user-facing language.
- [x] Add a user-configurable thinking detail/granularity setting.
- [x] Compile issue evidence and research in `docs/case-studies/issue-488`.
- [x] Update the case study and tests to cover the solver/API thinking metadata, not only the browser preview.
- [x] Run focused local checks and keep large logs under the case-study raw-data directory.
