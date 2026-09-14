## Issue #835 Multi-Jurisdiction File Legal-Risk Assessment

Issue [#835](https://github.com/link-assistant/formal-ai/issues/835) asks Formal
AI to check files across jurisdictions without pretending that one global
legality database exists. PR
[#900](https://github.com/link-assistant/formal-ai/pull/900) implements an
evidence-oriented library and CLI pipeline. See
`docs/case-studies/issue-835/` for the decomposition, sources, raw captures,
solution, and real Agent CLI evidence.

| ID | Requirement | Status |
| --- | --- | --- |
| R835-1 | Inspect files without returning a blanket global legality decision. | `check_file_legality` always emits `verdict: not_provided` plus no-global-verdict and not-legal-advice limitations. |
| R835-2 | Assess national-security, forbidden-content, and copyright/IP risk independently for every jurisdiction. | The Cartesian jurisdiction/category report preserves `unknown`, negative signal, risk signal, and confirmed prohibited match as distinct statuses. |
| R835-3 | Preserve jurisdiction, versioned policy, evidence, confidence, and provenance. | `JurisdictionPolicy`, `DetectorObservation`, category assessments, and provider-run receipts serialize all review inputs and actions. |
| R835-4 | Run detector integrations independently. | `LegalityEvidenceProvider` executes category-declared adapters separately and records completed/failed/skipped states without one timeout suppressing other evidence. |
| R835-5 | Fail closed for confirmed child-safety hashes without storing or reproducing content. | An authorized-provider receipt suppresses SHA-256 and Exif/GPS derivatives, skips ordinary providers, emits only safe provider references, and requires refusal/escalation. |
| R835-6 | Extract relevant Exif/GPS metadata with field-level provenance. | Generated-TIFF tests cover author, copyright, camera make/model, capture time, latitude, longitude, source, and locator. |
| R835-7 | Generalize to image, document, audio, video, and other files. | Byte-signature and extension classification feed the same report schema; a five-family regression verifies it. |
| R835-8 | Expose a callable function and CLI. | The Rust API and `formal-ai file-legality FILE [--config JSON] [--jurisdiction CODE]` share the serializable configuration/report. |
| R835-9 | Preserve reproducible TDD, research, and self-hosting evidence. | Focused unit and CLI tests, primary-source research, raw GitHub JSON, and session `ses_03d2a3c95ffe1gfVxnh24MtxFi` live in the issue #835 case study. |
