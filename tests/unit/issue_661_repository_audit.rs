//! Repository-wide executable acceptance coverage for issue #661.
//!
//! The conversational examples in `issue_661` cover the user-facing warning.
//! These tests exercise the generalized substrate requested during review: a
//! location-aware corpus, replayable evidence provenance, temperature-sensitive
//! weights, contradiction links, and usage-weighted associative learning.

use formal_ai::relative_meta_logic::{SourceTier, Stance};
use formal_ai::statement_audit::{
    audit_corpus, parse_evidence_json, AuditConfig, EvidenceCapture, EvidenceSelector,
    RepositoryCorpus, RepositoryDocument, SourceKind,
};
use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn corpus(documents: &[(&str, &str)]) -> RepositoryCorpus {
    RepositoryCorpus::from_documents(
        documents
            .iter()
            .map(|(path, content)| RepositoryDocument::new(*path, *content))
            .collect(),
    )
}

#[test]
fn repository_audit_falls_back_to_the_tree_for_an_untracked_git_fixture() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "formal-ai-issue-661-untracked-{}-{nonce}",
        std::process::id()
    ));
    fs::create_dir_all(&root).expect("fixture directory");
    fs::write(root.join("README.md"), "The implementation is inspectable.\n")
        .expect("fixture prose");
    let status = Command::new("git")
        .arg("init")
        .arg("--quiet")
        .arg(&root)
        .status()
        .expect("git available");
    assert!(status.success());

    let snapshot = RepositoryCorpus::from_repository(&root).expect("repository snapshot");
    fs::remove_dir_all(&root).expect("remove isolated fixture");

    assert_eq!(snapshot.documents.len(), 1, "{snapshot:#?}");
    assert_eq!(snapshot.documents[0].path, "README.md");
}

#[test]
fn repository_audit_extracts_prose_code_comments_and_structured_facts_with_locations() {
    let corpus = corpus(&[
        (
            "README.md",
            "# Runtime\n\nThe implementation lives in src/runtime.rs.\n",
        ),
        (
            "src/lib.rs",
            "pub fn run() {}\n// The retry limit is always three.\n",
        ),
        ("config.toml", "protocol = \"links\"\n"),
    ]);

    let audit = audit_corpus(&corpus, &[], AuditConfig::default());

    assert!(
        audit.statements.iter().any(|statement| {
            statement.location.path == "README.md"
                && statement.location.line == 3
                && statement.location.kind == SourceKind::Prose
                && statement.text.contains("src/runtime.rs")
        }),
        "Markdown prose must retain its source line: {:#?}",
        audit.statements,
    );
    assert!(
        audit.statements.iter().any(|statement| {
            statement.location.path == "src/lib.rs"
                && statement.location.line == 2
                && statement.location.kind == SourceKind::CodeComment
                && statement.text == "The retry limit is always three."
        }),
        "code comments are repository statements too: {:#?}",
        audit.statements,
    );
    assert!(
        audit.statements.iter().any(|statement| {
            statement.location.path == "config.toml"
                && statement.location.line == 1
                && statement.location.kind == SourceKind::Structured
                && statement.claim.as_ref().is_some_and(|claim| {
                    claim.subject == "config.toml"
                        && claim.predicate == "protocol"
                        && claim.value == "links"
                })
        }),
        "functional structured values must become explicit claims: {:#?}",
        audit.statements,
    );
}

#[test]
fn repository_index_is_first_party_evidence_against_a_missing_path_statement() {
    let corpus = corpus(&[(
        "README.md",
        "The implementation lives in src/missing_runtime.rs.\n",
    )]);

    let audit = audit_corpus(&corpus, &[], AuditConfig::default());
    let statement = audit
        .statements
        .iter()
        .find(|statement| statement.text.contains("src/missing_runtime.rs"))
        .expect("the path statement must be extracted");

    assert!(
        statement.assessment.posterior.get() < 0.5,
        "the tracked-file index must lower a false repository claim, got {}",
        statement.assessment.posterior,
    );
    assert!(
        statement.evidence.iter().any(|evidence| {
            evidence.capture.source_url == "git:index"
                && evidence.capture.tier == SourceTier::OriginalFirstParty
                && evidence.capture.stance == Stance::Contradicts
        }),
        "the probability change needs inspectable first-party provenance: {:#?}",
        statement.evidence,
    );
}

#[test]
fn captured_primary_evidence_moves_probability_while_reposts_are_preserved_but_ignored() {
    let statement_text = "The protocol is a W3C Recommendation.";
    let corpus = corpus(&[("docs/protocol.md", statement_text)]);
    let evidence = [
        EvidenceCapture::for_statement(
            statement_text,
            "W3C specification",
            "https://www.w3.org/TR/prov-o/",
            SourceTier::OriginalFirstParty,
            Stance::Supports,
            1.0,
        )
        .with_capture("2026-07-19T00:00:00Z", "sha256:primary"),
        EvidenceCapture::for_statement(
            statement_text,
            "article mirror",
            "https://mirror.invalid/prov-o",
            SourceTier::Unoriginal,
            Stance::Contradicts,
            1.0,
        )
        .with_capture("2026-07-19T00:00:00Z", "sha256:mirror"),
    ];

    let audit = audit_corpus(&corpus, &evidence, AuditConfig::default());
    let statement = audit
        .statements
        .iter()
        .find(|statement| statement.text == statement_text)
        .expect("the prose statement must be extracted");

    assert!(statement.assessment.posterior.get() > 0.6);
    assert_eq!(
        statement.evidence.len(),
        2,
        "ignored evidence stays visible"
    );
    assert!(statement.evidence.iter().any(|evidence| {
        evidence.capture.sha256 == "sha256:mirror" && evidence.effective_mass == 0.0
    }));
    assert!(statement.evidence.iter().all(|evidence| {
        !evidence.capture.captured_at.is_empty() && !evidence.capture.sha256.is_empty()
    }));
}

#[test]
fn exclusive_requirement_conflicts_get_evidence_and_temperature_weighted_resolution() {
    let first = "Always write the report as JSON.";
    let second = "Never write the report as JSON.";
    let corpus = corpus(&[("REQUIREMENTS.md", &format!("- {first}\n- {second}\n"))]);
    let evidence = [EvidenceCapture::for_statement(
        first,
        "accepted architecture decision",
        "repo:decisions/0001",
        SourceTier::OriginalFirstParty,
        Stance::Supports,
        0.9,
    )
    .with_capture("2026-07-19T00:00:00Z", "sha256:decision")];

    let warm = audit_corpus(
        &corpus,
        &evidence,
        AuditConfig {
            temperature: 1.0,
            ..AuditConfig::default()
        },
    );
    let cold = audit_corpus(
        &corpus,
        &evidence,
        AuditConfig {
            temperature: 0.2,
            ..AuditConfig::default()
        },
    );

    assert_eq!(warm.contradictions.len(), 1, "{:#?}", warm.contradictions);
    let conflict = &warm.contradictions[0];
    assert_eq!(conflict.statement_ids.len(), 2);
    assert!(conflict.proposed_resolution.contains("retract"));

    let weights = |audit: &formal_ai::statement_audit::RepositoryAudit| {
        let first_weight = audit
            .statements
            .iter()
            .find(|statement| statement.text == first)
            .expect("first requirement")
            .relative_weight;
        let second_weight = audit
            .statements
            .iter()
            .find(|statement| statement.text == second)
            .expect("second requirement")
            .relative_weight;
        (first_weight, second_weight)
    };
    let warm_weights = weights(&warm);
    let cold_weights = weights(&cold);
    assert!((warm_weights.0 + warm_weights.1 - 1.0).abs() < 0.000_001);
    assert!(warm_weights.0 > warm_weights.1, "{warm_weights:?}");
    assert!(
        cold_weights.0 - cold_weights.1 > warm_weights.0 - warm_weights.1,
        "lower temperature must sharpen evidence-backed alternatives: warm={warm_weights:?}, cold={cold_weights:?}",
    );
}

#[test]
fn findings_are_append_only_links_and_persist_in_associative_memory() {
    let corpus = corpus(&[(
        "README.md",
        "The implementation lives in src/missing_runtime.rs.\n",
    )]);

    let audit = audit_corpus(&corpus, &[], AuditConfig::default());
    let finding = audit
        .findings
        .first()
        .expect("the false path claim must yield a finding");

    assert!(audit.learning.contains(&finding.id));
    assert!(audit.learning.degree(&finding.id) > 0);
    assert!(audit.learning.retention_score(&finding.id) > 1);

    let links = audit.to_links_notation();
    assert!(links.contains("relative_weight"), "{links}");
    assert!(links.contains("source_url"), "{links}");
    assert!(links.contains("audit_finding"), "{links}");
    assert!(links.contains("issue_candidate"), "{links}");
    assert!(links.contains("associations"), "{links}");
    lino_objects_codec::format::parse_indented(&links)
        .expect("the audit artifact must be valid Links Notation");
}

#[test]
fn evidence_json_accepts_statement_and_claim_selectors_with_replay_metadata() {
    let captures = parse_evidence_json(
        r#"{
          "captures": [
            {
              "statement": "The protocol is a W3C Recommendation.",
              "source_label": "W3C specification",
              "source_url": "https://www.w3.org/TR/prov-o/",
              "tier": "original_first_party",
              "stance": "supports",
              "strength": 1.0,
              "captured_at": "2026-07-19T00:00:00Z",
              "sha256": "sha256:primary"
            },
            {
              "subject": "Cargo.toml",
              "predicate": "version",
              "value": "0.298.1",
              "source_label": "release manifest",
              "source_url": "repo:Cargo.toml",
              "tier": "original_first_party",
              "stance": "supports",
              "strength": 0.95,
              "captured_at": "repository_snapshot",
              "sha256": "sha256:manifest"
            }
          ]
        }"#,
    )
    .expect("valid replayable evidence");

    assert_eq!(captures.len(), 2);
    assert!(matches!(
        &captures[0].selector,
        EvidenceSelector::StatementText(text)
            if text == "The protocol is a W3C Recommendation."
    ));
    assert!(matches!(
        &captures[1].selector,
        EvidenceSelector::Claim { subject, predicate, value }
            if subject == "Cargo.toml"
                && predicate == "version"
                && value.as_deref() == Some("0.298.1")
    ));
    assert_eq!(captures[0].tier, SourceTier::OriginalFirstParty);
    assert_eq!(captures[0].stance, Stance::Supports);
    assert!((captures[1].strength - 0.95).abs() < f64::EPSILON);
    assert_eq!(captures[1].sha256, "sha256:manifest");
}

#[test]
fn evidence_json_rejects_ambiguous_selectors_and_unknown_provenance_values() {
    let ambiguous = parse_evidence_json(
        r#"{"captures":[{
          "statement":"one",
          "subject":"two",
          "predicate":"kind",
          "source_label":"source",
          "source_url":"repo:source",
          "tier":"original_first_party",
          "stance":"supports",
          "strength":1.0,
          "captured_at":"snapshot",
          "sha256":"sha256:value"
        }]}"#,
    )
    .expect_err("a capture must select by text or claim, never both");
    assert!(ambiguous.to_string().contains("selector"), "{ambiguous}");

    let unknown_tier = parse_evidence_json(
        r#"{"captures":[{
          "statement":"one",
          "source_label":"source",
          "source_url":"repo:source",
          "tier":"search_result",
          "stance":"supports",
          "strength":1.0,
          "captured_at":"snapshot",
          "sha256":"sha256:value"
        }]}"#,
    )
    .expect_err("unoriginal search output must be classified explicitly");
    assert!(unknown_tier.to_string().contains("tier"), "{unknown_tier}");
}
