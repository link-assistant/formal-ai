## Issue #1582: Client Read Access, Diagnostics and Context

Source: https://github.com/link-assistant/formal-ai/issues/1582, including its
2026-11-17 evidence/case-study edit. No required sub-issues were listed.
The investigation and retained data are in `docs/case-studies/issue-1582/`.

| ID | Requirement | Status / Evidence |
| --- | --- | --- |
| R1582-1 | Enable read-only network/source retrieval by default and honor explicit restrictions. | Implemented: shared JavaScript/native read policy and serving worker HTTP transport; `rust/tests/web/issue-1582-defaults.test.mjs`. |
| R1582-2 | Explain how the harness can lift restrictions and enable internet access. | Implemented: seed-defined permission intent and actionable localized guidance with automatic detection in all five registered languages, including quoted-input exclusions for all 16 catalog programming targets; `rust/tests/web/issue-1582-permissions.test.mjs`. Operator policy still requires the operator's change. |
| R1582-3 | Keep routine wrapper startup and exit diagnostics out of the harness; persist logs by default. | Partial: `rust/tests/web/server-structured-log-cli.test.mjs` passes; native wrapper and file sink changed; runtime regressions remain unexecuted under pre-merge policy. Production type/lint checks do not execute them. |
| R1582-4 | Keep single-line default logs and offer nested/indented presentation. | Implemented: opt-in nested formatting after redaction; `rust/tests/web/issue-1582-defaults.test.mjs`. |
| R1582-5 | Configure actual/resource-derived input context for every supported client, including Claude's unknown model. | Partial: Claude, Codex, Qwen, Aider, Grok and OpenCode-shaped integrations receive limits. Gemini input context remains blocked by its upstream fixed fallback; https://github.com/google-gemini/gemini-cli/issues/30333. Native adapter runtime regression remains unexecuted under pre-merge policy. |
| R1582-6 | Preserve available logs/data, timeline, root causes, primary-source research, solution plans and relevant upstream reports. | Partial: repository case study and retrieved evidence retained; original Mac-only files were never uploaded, review-continuation evidence distinguishes all 12 registered tools and JavaScript, TypeScript and Rust verification; current continuation CI evidence is tracked in the PR case study; the new picomatch advisory also leaves dependency qualification unfinished. |
