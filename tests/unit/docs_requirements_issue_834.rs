use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

const POLICY: &str = "LEGAL-COMPLIANCE.md";
const CONTRIBUTING: &str = "CONTRIBUTING.md";
const REGISTRY: &str = "data/training/source-registry.json";
const TRAINING_README: &str = "data/training/README.md";
const SOURCE_REVIEW: &str = "docs/legal/source-review.md";
const CASE_STUDY: &str = "docs/case-studies/issue-834/README.md";
const REQUIREMENTS: &str = "docs/case-studies/issue-834/requirements.md";
const RESEARCH: &str = "docs/case-studies/issue-834/raw-data/online-research.md";

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(relative: &str) -> String {
    fs::read_to_string(root().join(relative))
        .unwrap_or_else(|error| panic!("read {relative}: {error}"))
}

fn assert_contains_all(relative: &str, needles: &[&str]) {
    let content = read(relative).to_lowercase();
    for needle in needles {
        assert!(
            content.contains(&needle.to_lowercase()),
            "{relative} must document {needle:?}"
        );
    }
}

fn registry() -> Value {
    serde_json::from_str(&read(REGISTRY)).expect("training source registry must be valid JSON")
}

fn value_is_populated(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::String(value) => !value.trim().is_empty(),
        Value::Array(values) => !values.is_empty(),
        Value::Object(values) => !values.is_empty(),
        Value::Bool(_) | Value::Number(_) => true,
    }
}

fn artifact_files(path: &Path) -> Vec<String> {
    if !path.exists() {
        return Vec::new();
    }

    let mut files = Vec::new();
    for entry in fs::read_dir(path).expect("read training artifact directory") {
        let path = entry.expect("read training artifact entry").path();
        if path.is_dir() {
            files.extend(artifact_files(&path));
        } else {
            files.push(
                path.strip_prefix(root())
                    .expect("artifact must be inside repository")
                    .to_string_lossy()
                    .replace('\\', "/"),
            );
        }
    }
    files.sort();
    files
}

#[test]
fn public_domain_dedication_covers_human_authored_contributions() {
    assert_contains_all(
        POLICY,
        &[
            "human-authored",
            "unlicense",
            "public domain",
            "fallback",
            "third-party",
        ],
    );
    assert_contains_all(
        CONTRIBUTING,
        &[
            "intentionally submitting",
            "authority",
            "unlicense",
            "third-party material",
        ],
    );
}

#[test]
fn every_training_or_distillation_artifact_requires_registered_provenance() {
    let registry = registry();
    assert_eq!(registry["schema_version"], 1);
    assert_eq!(registry["artifacts_root"], "data/training/artifacts");
    assert_eq!(registry["current_state"], "no-approved-training-sources");

    let required_fields = [
        "id",
        "artifact_paths",
        "source_kind",
        "upstream_creator",
        "upstream_name",
        "upstream_version",
        "provider",
        "provider_route",
        "acquired_at",
        "acquisition_method",
        "intended_use",
        "intended_output_name",
        "intended_output_license",
        "intended_distribution",
        "input_provenance",
        "upstream_license",
        "license_url",
        "terms_url",
        "terms_checked_at",
        "terms_snapshot",
        "training_permission",
        "distribution_permission",
        "attribution_requirements",
        "naming_requirements",
        "acceptable_use_requirements",
        "scale_revenue_thresholds",
        "patent_trademark_terms",
        "downstream_license_requirements",
        "territory_restrictions",
        "rights_reservation_status",
        "personal_data_status",
        "reidentification_assessment",
        "retention_deletion_duties",
        "privacy_review",
        "safety_review",
        "security_review",
        "regional_review",
        "reviewer",
        "approval_status",
        "approved_at",
        "re_review_triggers",
        "evidence_paths",
        "sha256",
    ];

    let declared_fields = registry["required_source_fields"]
        .as_array()
        .expect("registry required_source_fields must be an array");
    assert_eq!(
        declared_fields.len(),
        required_fields.len(),
        "the registry schema and executable required-field contract must stay equal"
    );
    for field in required_fields {
        assert!(
            declared_fields.iter().any(|declared| declared == field),
            "registry schema must declare required source field {field}"
        );
    }

    let sources = registry["sources"]
        .as_array()
        .expect("registry sources must be an array");
    let mut registered_artifacts = Vec::new();
    for source in sources {
        for field in required_fields {
            assert!(
                value_is_populated(&source[field]),
                "registry entry {:?} has a missing or empty {field}",
                source["id"]
            );
        }
        assert_eq!(
            source["approval_status"], "approved",
            "only approved sources may enter the canonical registry"
        );
        for artifact in source["artifact_paths"]
            .as_array()
            .expect("artifact_paths must be an array")
        {
            let artifact = artifact
                .as_str()
                .expect("artifact path must be a string")
                .to_owned();
            assert!(
                artifact.starts_with("data/training/artifacts/"),
                "training artifact must use the gated directory: {artifact}"
            );
            assert!(root().join(&artifact).is_file(), "missing {artifact}");
            registered_artifacts.push(artifact);
        }
    }
    registered_artifacts.sort();
    assert_eq!(
        artifact_files(&root().join("data/training/artifacts")),
        registered_artifacts,
        "every canonical training artifact must have one approved registry entry"
    );

    assert_contains_all(
        TRAINING_README,
        &[
            "not a training source",
            "nemotron",
            "source-registry.json",
            "fail closed",
        ],
    );
}

#[test]
fn contribution_rules_reject_leaks_paid_data_and_large_copyrighted_payloads() {
    assert_contains_all(
        CONTRIBUTING,
        &[
            "leaked",
            "proprietary source code",
            "paid or access-controlled dataset",
            "large verbatim",
            "issues, pull requests",
            "remove",
        ],
    );
}

#[test]
fn closed_api_scraping_and_competing_model_training_are_prohibited() {
    assert_contains_all(
        POLICY,
        &[
            "closed api",
            "automated scraping",
            "competing model",
            "contract",
            "terms of service",
            "openai",
            "anthropic",
            "gemini",
        ],
    );
}

#[test]
fn model_specific_attribution_and_naming_obligations_are_reviewed() {
    assert_contains_all(
        POLICY,
        &[
            "llama 3.3",
            "built with llama",
            "llama",
            "700 million",
            "mistral 7b",
            "apache 2.0",
            "exact model version",
            "openrouter",
            "best-effort",
        ],
    );
    assert_contains_all(
        SOURCE_REVIEW,
        &[
            "attribution",
            "naming",
            "model version",
            "provider route",
            "terms snapshot",
        ],
    );
}

#[test]
fn real_personal_data_is_excluded_from_training() {
    assert_contains_all(
        POLICY,
        &[
            "no real personal data",
            "medical",
            "government identifier",
            "biometric",
            "face",
            "pseudonym",
            "synthetic",
            "gdpr",
            "data minimisation",
        ],
    );
}

#[test]
fn prohibited_use_safeguards_cover_high_risk_abuse() {
    assert_contains_all(
        POLICY,
        &[
            "prohibited uses",
            "csam",
            "malware",
            "bypass security",
            "biological",
            "physical harm",
            "human review",
            "incident",
        ],
    );
}

#[test]
fn eu_ai_act_open_source_exemption_is_assessed_without_overclaiming() {
    assert_contains_all(
        POLICY,
        &[
            "article 53",
            "free and open-source",
            "technical documentation",
            "copyright policy",
            "training-content summary",
            "systemic risk",
            "2 august 2026",
            "not yet a provider",
        ],
    );
}

#[test]
fn as_is_disclaimer_and_non_waivable_limits_are_explicit() {
    assert_contains_all(
        POLICY,
        &[
            "\"as is\"",
            "no warranty",
            "intentional harm",
            "consumer protection",
            "physical harm",
            "mandatory law",
            "does not replace",
        ],
    );
}

#[test]
fn issue_834_whole_task_has_traceable_research_and_an_operational_gate() {
    assert_contains_all(
        CASE_STUDY,
        &[
            "self-audit",
            "gap",
            "training source registry",
            "community review",
            "not legal advice",
        ],
    );
    assert_contains_all(
        REQUIREMENTS,
        &["r834-01", "r834-09", "r834-10", "complete workflow"],
    );
    assert_contains_all(
        RESEARCH,
        &[
            "u.s. copyright office",
            "directive (eu) 2019/790",
            "regulation (eu) 2024/1689",
            "openrouter",
            "llama 3.3",
            "mistral",
        ],
    );
    assert_contains_all(
        ".github/pull_request_template.md",
        &[
            "legal / data-source review",
            "source-registry.json",
            "personal data",
            "third-party",
        ],
    );
    assert_contains_all(
        "REQUIREMENTS.md",
        &["issue #834 legal & compliance self-audit", "r484", "r493"],
    );
}
