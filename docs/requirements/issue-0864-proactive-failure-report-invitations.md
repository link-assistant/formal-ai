## Issue #864 Proactive Failure-Report Invitations

Issue [#864](https://github.com/link-assistant/formal-ai/issues/864) asks
Formal AI to initiate issue reporting after failures it detects in every UX
surface. PR [#910](https://github.com/link-assistant/formal-ai/pull/910) adds an
opt-in invitation while preserving #839's contextual report and confirmation
boundary. See `docs/case-studies/issue-864/`.

| ID | Requirement | Status |
| --- | --- | --- |
| R864-1 | Proactively ask for issue-report consent after detected reasoning, provider, and tool failures in Rust, browser/desktop, and agentic coding harnesses. | A shared seed-backed Rust invitation, explicit browser `detectedFailure` state, and Agent-plan aggregation cover every surface; focused Rust and Playwright regressions exercise each route. |
| R864-2 | Detect semantic failure signals without treating ordinary error-like prose, refusal, denial, cancellation, abort, pending approval, or missing grants as Formal AI failures. | Rust uses structured result fields plus the seeded tool-failure role; the browser uses intents and structured results only. Positive and negative regressions pin the boundary. |
| R864-3 | Localize the invitation and preserve detected-failure state across UI persistence and nested Agent execution. | Seed invitations cover English, Russian, Hindi, Chinese, and Spanish; four UI-catalog prompts cover the currently published browser locales; IndexedDB hydration and every Agent subanswer retain the failure bit. |
| R864-4 | Reuse the contextual six-section issue report and never file automatically. | The inline action calls #839's existing report builder; browser assertions inspect all six sections and the live Agent CLI E2E rejects any `gh issue create` before consent. |
| R864-5 | Preserve reproducible browser, real Agent CLI, CI, and self-application evidence. | Before/after screenshots, raw two-round failure evidence, a dedicated live-client workflow, and Agent-authored policy leaf from session `ses_03b54b716ffe3E7D9TZMDg6Evs` live in the case study. |
