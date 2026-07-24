# Issue #834 Requirements

The nine checklist items are kept one-to-one with regression tests. R834-10
collects the issue's community questions, research, operational workflow, and
delivery evidence into a whole-task requirement.

| ID | Requirement | Evidence | Regression test |
| --- | --- | --- | --- |
| R834-01 | Make the public-domain dedication explicit for human-authored contributions, with authority, fallback, and third-party-rights boundaries. | `LEGAL-COMPLIANCE.md`; `CONTRIBUTING.md` | `public_domain_dedication_covers_human_authored_contributions` |
| R834-02 | Record complete license, terms, version, provider, provenance, privacy, obligation, review, and hash evidence for every training/distillation artifact. | `data/training/`; `docs/legal/source-review.md` | `every_training_or_distillation_artifact_requires_registered_provenance` |
| R834-03 | Forbid leaked/proprietary code, unauthorized paid/access-controlled data, and large verbatim copyrighted payloads in every contribution surface. | `CONTRIBUTING.md`; `LEGAL-COMPLIANCE.md` | `contribution_rules_reject_leaks_paid_data_and_large_copyrighted_payloads` |
| R834-04 | Prohibit automated collection from closed APIs when contract/terms forbid competing-model training or extraction. | `LEGAL-COMPLIANCE.md` | `closed_api_scraping_and_competing_model_training_are_prohibited` |
| R834-05 | Track exact-version attribution, notice, naming, use, scale, and downstream obligations, including answers for Llama 3.3, Mistral 7B, and OpenRouter. | `LEGAL-COMPLIANCE.md`; `docs/legal/source-review.md` | `model_specific_attribution_and_naming_obligations_are_reviewed` |
| R834-06 | Exclude real personal data from training and require privacy review for purportedly anonymous/synthetic material. | `LEGAL-COMPLIANCE.md` | `real_personal_data_is_excluded_from_training` |
| R834-07 | Define prohibited-use and dual-use safeguards, human review, and an incident path. | `LEGAL-COMPLIANCE.md` | `prohibited_use_safeguards_cover_high_risk_abuse` |
| R834-08 | Assess current EU AI Act status and the exact limits of a future free and open-source Article 53 exemption. | `LEGAL-COMPLIANCE.md`; `raw-data/online-research.md` | `eu_ai_act_open_source_exemption_is_assessed_without_overclaiming` |
| R834-09 | Preserve the Unlicense **AS IS** / no-warranty terms while explaining the limits imposed by mandatory law and harm. | `LEGAL-COMPLIANCE.md`; `LICENSE` | `as_is_disclaimer_and_non_waivable_limits_are_explicit` |
| R834-10 | Complete workflow: preserve source/feedback and primary research, answer model/filter/regional questions, add a community review gate, provide requirement traceability and release metadata, and test the composition end to end. | Entire `docs/case-studies/issue-834/` tree; PR template; `REQUIREMENTS.md`; changelog fragment | `issue_834_whole_task_has_traceable_research_and_an_operational_gate` |

## Acceptance

All ten tests must pass in one run. Every future training artifact must be
inside the canonical directory and equal the union of approved registry paths.
Documentation alone cannot override that fail-closed check.
