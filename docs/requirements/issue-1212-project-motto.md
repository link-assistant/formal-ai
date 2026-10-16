## Issue #1212 Project Motto and Evidence-Driven Discovery

Authority: [issue #1212](https://github.com/link-assistant/formal-ai/issues/1212), included through #1207 in [the single PR #1221 scope](../case-studies/pull-request-1221/scope.md). The exact architect quote is retained in [the architect note](../architect-notes/2026-10-15-know-just-enough.md).

| ID | Requirement and acceptance | Status |
| --- | --- | --- |
| R1212-1 | Put the exact motto at the top of README, VISION and GOALS; preserve the historical architect note. | Source implemented; scripts/check-project-motto.mjs and rust/tests/web/project-motto.test.mjs check exact surfaces and missing-motto mutations; hosted acceptance Pending. |
| R1212-2 | Prefer likely, simple, logical and provable candidates; use failures as evidence; do not prescribe exhaustive/random search. | Source implemented in VISION; mutation controls reject regressions; actual general discovery acceptance Pending. |
| R1212-3 | Preserve the architect words verbatim with their original attribution. | Source implemented; exact quote hash and mutation controls; hosted acceptance Pending. |
| R1212-4 | Apply the motto to every published product package description and the crate description. | Source implemented; released package acceptance Pending. |
| R1212-5 | Show the exact motto on the app, docs, CLI, VS Code and download surfaces. | Source metadata implemented and tested; deployed acceptance Pending. |
| R1212-6 | Set the GitHub repository description to the exact motto. | Manually verified 2026-10-16 17:04 UTC: authenticated GitHub API set and independently read back the exact motto at https://github.com/link-assistant/formal-ai; no release or solver acceptance is implied. |
| R1212-7 | Run a CI docs check that fails if the motto or required evidence-driven vision is removed. | Source gate and 20 mutation controls implemented; fresh hosted acceptance Pending. |
| R1212-8 | Use just-enough trusted discovery, coding first and permissioned recursive self-improvement; retain failures as data. | Recorded and linked to R1207-3/-6/-7; actual cold discovery and self-coding acceptance Pending. |
