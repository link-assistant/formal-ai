## Issue #710 Dropped-Requirements Re-verification

Issue [#710](https://github.com/link-assistant/formal-ai/issues/710) audits 32
requirements that earlier issue closures did not prove. The detailed evidence
matrix was re-verified on 2026-08-10 against v0.337.0 in
`docs/case-studies/issue-710/README.md`; this table is the compact current
requirement-status authority. `still-broken` means the linked open focused
owner remains required and must not be read as implemented.

| ID | Requirement | Status |
| --- | --- | --- |
| R710-01 | Conversation-history recall. | `works-now` — pinned history-search and previous-question specifications. |
| R710-02 | Russian identity and capabilities. | `works-now` — pinned localized identity/capability specifications. |
| R710-03 | Multi-statement and many-question composition. | `works-now` — issue-710 native and browser regressions. |
| R710-04 | Context-qualified questions such as IIR in ML. | `works-now` — four-language contextual concept regressions. |
| R710-05 | Typo tolerance, clarification, and full-path fuzzy matching. | `works-now` — worker and native fuzzy regressions. |
| R710-06 | Antiregime and false-totality definition class. | `works-now` — seeded multilingual concept regressions. |
| R710-07 | Folder-listing prompt variants. | `superseded` — #745, #758, and PR #850 generalized capability routing. |
| R710-08 | Target-less modifications ask exactly one question. | `works-now` — issue-710 four-language specification. |
| R710-09 | Multiple deterministic free-time replies. | `works-now` — issue-710 stable-variant specification. |
| R710-10 | Assistant name set/read and attribution. | `works-now` — issue-710 four-language naming plus issue-157 attribution coverage. |
| R710-11 | Issue #292 rules, answer language, parity, and Markdown. | `works-now` — native/browser localization and generated parity checks. |
| R710-12 | Thinking localization on CLI/API/Telegram. | `works-now` — issue-889 cross-surface regressions for every registered language. |
| R710-13 | Collapsed thinking animation and top placement. | `works-now` — issue-488 browser and issue-676 narrative-order regressions. |
| R710-14 | Translate formal proofs to programming languages. | `works-now` — issue-890 compile-and-execute regressions for every registered target. |
| R710-15 | At least 50 verified equation types. | `works-now` — issue-891 catalog and non-decreasing ratchet cover 72 types. |
| R710-16 | Compose calculations with other instructions. | `works-now` — calculator continuation and issue-710 composition regressions. |
| R710-17 | Word problems beyond train meeting. | `works-now` — Fibonacci and box-relation regressions. |
| R710-18 | Current source-backed film release ordering. | `works-now` — issue-892 timestamped Wikidata timeline regressions. |
| R710-19 | Closest contextual pronoun resolution. | `works-now` — issue-465 follow-up specification. |
| R710-20 | How-to multi-source synthesis and seven-day availability cache. | `still-broken` — [#991](https://github.com/link-assistant/formal-ai/issues/991); #709 delivered statement fusion, not the procedural contract. |
| R710-21 | Iterative two-file summary validation and 80% quality bar. | `works-now` — issue-893 iteration, threshold, and ratchet regressions. |
| R710-22 | Interior/plain-capitalized entity reasoning class. | `works-now` — issue-571 class regression and worker entity coverage. |
| R710-23 | Calendar interchange and Apple/Google/Microsoft flows. | `works-now` — RFC 5545 plus Google insertion regression. |
| R710-24 | Optional gated OCR and attachment transcription. | `works-now` — issue-493 real-worker OCR regressions. |
| R710-25 | E2E against deployed GitHub Pages. | `works-now` — deployment-output and matching-deployment workflow regressions. |
| R710-26 | Four-template CI comparison and upstream filings. | `works-now` — issue-894 four-template comparison and filing-ledger regressions. |
| R710-27 | Published coverage with a non-decreasing ratchet. | `works-now` — issue-895 80% published-coverage floor and ratchet regressions. |
| R710-28 | Gemini headless tools. | `works-now` — #671 / PR #814 real-client matrix evidence. |
| R710-29 | macOS signed/notarized auto-update production path. | `works-now` — desktop workflow and issue-548 regressions. |
| R710-30 | link-foundation/start and command-stream adoption. | `still-broken` — [#990](https://github.com/link-assistant/formal-ai/issues/990); start-command shipped, but production command runners still bypass command-stream. |
| R710-31 | web-search/web-capture as production components. | `works-now` — issue-896 production-component and feature-wiring regressions. |
| R710-32 | Iframe pre-check and external-link actions. | `works-now` — browser navigation/embedding regressions. |
