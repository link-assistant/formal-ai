use std::fs;
use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(path: &str) -> String {
    let path = root().join(path);
    fs::read_to_string(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
}

fn assert_contains_all(label: &str, text: &str, needles: &[&str]) {
    for needle in needles {
        assert!(text.contains(needle), "{label} is missing {needle:?}");
    }
}

#[test]
fn legal_guides_distinguish_symbolic_formal_ai_from_language_model_services() {
    let boundary = read("docs/legal/formal-ai-and-language-models.md");
    assert_contains_all(
        "Formal AI/LLM boundary",
        &boundary,
        &[
            "symbolic",
            "does not contain or require neural-network inference",
            "OpenAI-compatible",
            "protocol compatibility",
            "external Agent CLI",
            "not a runtime dependency",
            "LEGAL-COMPLIANCE.md",
        ],
    );

    let readme = read("README.md");
    let compliance = read("LEGAL-COMPLIANCE.md");
    for path in [
        "docs/legal/formal-ai-and-language-models.md",
        "docs/legal/public-domain-output.md",
        "docs/legal/compatible-datasets.md",
        "docs/legal/distillable-models.md",
    ] {
        assert!(readme.contains(path), "README must link {path}");
        assert!(compliance.contains(path), "legal policy must link {path}");
    }
}

#[test]
fn public_domain_guide_separates_ownership_from_permission_and_clearance() {
    let guide = read("docs/legal/public-domain-output.md");
    assert_contains_all(
        "public-domain guide",
        &guide,
        &[
            "As between you and OpenAI",
            "does not override",
            "competing AI model",
            "third-party rights",
            "human authorship",
            "Unlicense",
            "fail closed",
            "not legal advice",
            "https://openai.com/policies/services-agreement/",
            "https://www.copyright.gov/ai/",
        ],
    );
}

#[test]
fn dataset_shortlist_has_ten_versioned_candidates_and_no_blanket_approval() {
    let guide = read("docs/legal/compatible-datasets.md");
    assert_contains_all(
        "dataset shortlist",
        &guide,
        &[
            "Reviewed on 2026-08-01",
            "not an approval registry",
            "data/training/source-registry.json",
            "FineWeb2",
            "FineWeb",
            "Dolma 1.7",
            "Common Pile v0.1",
            "SmolTalk",
            "SYNTH",
            "DCLM Baseline 1.0",
            "RedPajama-Data-v2",
            "The Stack v2",
            "Tulu 3 SFT Mixture",
        ],
    );
    for index in 1..=10 {
        assert!(
            guide.contains(&format!("| {index} |")),
            "dataset shortlist is missing numbered row {index}"
        );
    }
}

#[test]
fn model_shortlist_has_ten_distinct_families_and_route_specific_caveats() {
    let guide = read("docs/legal/distillable-models.md");
    assert_contains_all(
        "model shortlist",
        &guide,
        &[
            "Reviewed on 2026-08-01",
            "not an approval registry",
            "hosted-service terms",
            "Granite 4.1",
            "DeepSeek-V4",
            "Mistral Small 4",
            "Qwen3.5",
            "GLM-5",
            "Apertus",
            "gpt-oss",
            "SmolLM3",
            "OLMo 2",
            "Phi-4-mini",
        ],
    );
    for index in 1..=10 {
        assert!(
            guide.contains(&format!("| {index} |")),
            "model shortlist is missing numbered row {index}"
        );
    }
}

#[test]
fn philosophy_guide_marks_metaphors_and_mathematical_claims_precisely() {
    let philosophy = read("docs/philosophy.md");
    assert_contains_all(
        "philosophy guide",
        &philosophy,
        &[
            "AI = data + algorithm",
            "Everything is a link",
            "design thesis",
            "not a theorem",
            "Markov algorithm",
            "Turing-complete",
            "one doublet link",
            "transformation network",
            "Self-modification",
            "human-gated",
            "relative-meta-logic",
        ],
    );
    assert!(read("README.md").contains("docs/philosophy.md"));
    assert!(read("VISION.md").contains("docs/philosophy.md"));
}

#[test]
fn shared_conversation_research_preserves_only_accessible_metadata() {
    let metadata = read("docs/case-studies/issue-885/raw-data/shared-conversation-metadata.md");
    assert_contains_all(
        "shared conversation metadata",
        &metadata,
        &[
            "ab4PEVZtx9OKDP2Gb",
            "shmd",
            "Is formal ai development",
            "generated answer itself was not present",
        ],
    );
}

#[test]
fn context_aware_audit_is_documented_as_bounded_and_inspectable() {
    let case_study = read("docs/case-studies/issue-885/README.md");
    assert_contains_all(
        "context-aware audit",
        &case_study,
        &[
            "resolved_text",
            "contextual_posterior",
            "closest preceding",
            "same-document",
            "Ambiguous or unsupported cases",
        ],
    );

    let implementation = read("src/statement_audit/model.rs");
    assert_contains_all(
        "statement audit model",
        &implementation,
        &["resolved_text", "references", "contextual_posterior"],
    );
}

#[test]
fn formal_ai_and_real_agent_cli_evidence_covers_authorship_and_audit() {
    let case_study = read("docs/case-studies/issue-885/README.md");
    let agent_leaf =
        read("docs/case-studies/issue-885/agent-cli-evidence/agent-authored-audit-policy.md");
    assert!(
        case_study.contains(agent_leaf.trim()),
        "case study must include the audited leaf authored through Formal AI and Agent CLI"
    );
    assert_contains_all(
        "Agent CLI statement-audit evidence",
        &read(
            "docs/case-studies/issue-885/agent-cli-evidence/statement-audit/statement-audit.lino",
        ),
        &[
            "resolved_text",
            "contextual_posterior",
            "antecedent_statement_id",
            "closest_preceding_subject_same_document",
        ],
    );
}

#[test]
fn issue_case_study_preserves_requirements_research_and_solution_artifacts() {
    let case_study = read("docs/case-studies/issue-885/README.md");
    assert_contains_all(
        "issue 885 case study",
        &case_study,
        &[
            "requirements.md",
            "solution-plan.md",
            "raw-data/shared-conversation-metadata.md",
            "raw-data/online-research.md",
            "agent-cli-evidence",
            "repository-audit-summary.md",
        ],
    );

    let requirements = read("docs/case-studies/issue-885/requirements.md");
    for requirement in 1..=10 {
        assert!(
            requirements.contains(&format!("R885-{requirement}")),
            "case study is missing R885-{requirement}"
        );
    }
}

#[test]
fn whole_solution_is_linked_and_has_a_release_fragment() {
    let changelog = read("changelog.d/20260801_000000_issue_885.md");
    assert_contains_all(
        "issue 885 changelog",
        &changelog,
        &["bump: minor", "relative references", "legal source-review"],
    );

    let case_study = read("docs/case-studies/issue-885/README.md");
    assert!(case_study.contains("repository-audit-summary.md"));
    assert_contains_all(
        "whole repository audit summary",
        &read("docs/case-studies/issue-885/repository-audit-summary.md"),
        &[
            "215,945",
            "4,085,030",
            "bccbcdc8b91676a406786466706a2e17e8a8f6cb898da71dbf0b11b780ab1ff6",
            "soft-wrapped continuation",
            "triage signals",
            "ses_042daf081ffecMWSd71J5l7B6E",
            "7bd45aa123575af2d7a9f548e79f68c57d96018ba3c6e834114b3fbd54adf18a",
        ],
    );
    assert_contains_all(
        "whole repository Agent CLI evidence",
        &read("docs/case-studies/issue-885/agent-cli-evidence/whole-repository.md"),
        &[
            "ses_042daf081ffecMWSd71J5l7B6E",
            "formal-ai statement-audit --root .",
            "215975",
            "7bd45aa123575af2d7a9f548e79f68c57d96018ba3c6e834114b3fbd54adf18a",
        ],
    );
}
