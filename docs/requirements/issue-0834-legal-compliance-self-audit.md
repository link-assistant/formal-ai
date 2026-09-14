## Issue #834 Legal & Compliance Self-Audit

Issue [#834](https://github.com/link-assistant/formal-ai/issues/834) asks for a
self-audit before neural training begins: explicit public-domain contribution
terms, complete source provenance, restrictions on unlawful or
contract-forbidden inputs, model-specific obligations, privacy and safety
controls, a precise EU AI Act assessment, disclaimer limits, answers to the
Llama/Mistral/OpenRouter questions, and a community-review anchor. PR
[#837](https://github.com/link-assistant/formal-ai/pull/837) implements a
fail-closed intake boundary while recording the truthful current state: Formal
AI has no approved neural training or distillation sources.

| ID | Requirement | Status |
| --- | --- | --- |
| R484 | Make the public-domain dedication explicit for human-authored submissions without purporting to erase third-party rights. | Implemented by `LEGAL-COMPLIANCE.md` and the inbound submission rule in `CONTRIBUTING.md`; covered by `public_domain_dedication_covers_human_authored_contributions`. |
| R485 | Record complete, machine-readable provenance and approval evidence for every training/distillation artifact and fail closed on omissions. | Implemented by `data/training/source-registry.json`, `data/training/README.md`, `docs/legal/source-review.md`, and the artifact/registry equality test. |
| R486 | Prohibit leaked/proprietary code, unauthorized paid/access-controlled data, large verbatim works, secrets, and real personal data across contribution surfaces. | Implemented by `LEGAL-COMPLIANCE.md`, `CONTRIBUTING.md`, and the PR template; covered by the contribution and privacy tests. |
| R487 | Prohibit automated closed-API extraction for competing-model training when applicable contracts or terms forbid it. | Implemented by the closed-API rule and written-permission gate in `LEGAL-COMPLIANCE.md`; covered by `closed_api_scraping_and_competing_model_training_are_prohibited`. |
| R488 | Track exact-version model/provider attribution, naming, use, scale, route, terms, and downstream obligations. | Implemented by the generic source review plus Llama 3.3, Mistral 7B, and OpenRouter analysis; covered by `model_specific_attribution_and_naming_obligations_are_reviewed`. |
| R489 | Exclude real personal data from training and document prohibited-use, dual-use, human-review, and incident safeguards. | Implemented by the privacy/data-governance and prohibited-use policy sections; covered by dedicated privacy and safety tests. |
| R490 | Assess the EU AI Act free/open-source position without claiming a blanket exemption. | The policy records that Formal AI is not yet a GPAI model provider, explains Article 53's limited future exception, and adds classification/re-review triggers; covered by `eu_ai_act_open_source_exemption_is_assessed_without_overclaiming`. |
| R491 | Preserve the Unlicense **AS IS** / no-warranty language while documenting mandatory-law and harm limits. | Implemented by the warranty/liability policy section; covered by `as_is_disclaimer_and_non_waivable_limits_are_explicit`. |
| R492 | Preserve primary research, source issue/PR feedback, an explicit requirements map, solution plan, honest Agent CLI attempt, and community review hook. | Implemented by `docs/case-studies/issue-834/`, `.github/pull_request_template.md`, and the changelog fragment. |
| R493 | Verify each issue checklist item and the complete workflow with executable regression coverage. | Implemented by ten tests in `tests/unit/docs_requirements_issue_834.rs`, including artifact/registry equality and whole-task traceability. |
