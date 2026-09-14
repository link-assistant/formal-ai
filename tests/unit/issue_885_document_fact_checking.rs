//! Document-relative statement-audit coverage for issue #885.

use formal_ai::ChatMessage;
use formal_ai::agentic_coding::{AgenticPlan, plan_chat_step};
use formal_ai::relative_meta_logic::{SourceTier, Stance};
use formal_ai::statement_audit::{
    AuditConfig, EvidenceCapture, RepositoryCorpus, RepositoryDocument, audit_corpus,
};

fn audit(
    markdown: &str,
    evidence: &[EvidenceCapture],
) -> formal_ai::statement_audit::RepositoryAudit {
    let corpus =
        RepositoryCorpus::from_documents(vec![RepositoryDocument::new("README.md", markdown)]);
    audit_corpus(&corpus, evidence, AuditConfig::default())
}

#[test]
fn a_pronoun_resolves_to_the_closest_preceding_subject() {
    let result = audit(
        "The hosted API is optional.\nThe local runtime is deterministic.\nIt has no neural weights.\n",
        &[],
    );
    let antecedent = result
        .statements
        .iter()
        .find(|statement| statement.text == "The local runtime is deterministic.")
        .expect("closest referentially complete statement");
    let dependent = result
        .statements
        .iter()
        .find(|statement| statement.text == "It has no neural weights.")
        .expect("dependent statement");

    assert_eq!(
        dependent.resolved_text,
        "The local runtime has no neural weights."
    );
    assert_eq!(dependent.references.len(), 1, "{dependent:#?}");
    assert_eq!(dependent.references[0].surface, "It");
    assert_eq!(
        dependent.references[0].antecedent_statement_id,
        antecedent.id
    );
}

#[test]
fn evidence_can_target_the_context_resolved_statement() {
    let evidence = [EvidenceCapture::for_statement(
        "Formal AI does not require neural inference.",
        "runtime architecture inspection",
        "repo:src",
        SourceTier::OriginalFirstParty,
        Stance::Supports,
        1.0,
    )
    .with_capture("repository_snapshot", "sha256:runtime")];

    let result = audit(
        "Formal AI is a symbolic assistant.\nIt does not require neural inference.\n",
        &evidence,
    );
    let dependent = result
        .statements
        .iter()
        .find(|statement| statement.text.starts_with("It does"))
        .expect("dependent statement");

    assert_eq!(
        dependent.resolved_text,
        "Formal AI does not require neural inference."
    );
    assert!(dependent.assessment.posterior.get() > 0.6, "{dependent:#?}");
    assert_eq!(dependent.evidence.len(), 1);
}

#[test]
fn a_dependent_statement_cannot_outrank_a_disproved_antecedent() {
    let evidence = [EvidenceCapture::for_statement(
        "Formal AI is a hosted model.",
        "runtime architecture inspection",
        "repo:src",
        SourceTier::OriginalFirstParty,
        Stance::Contradicts,
        1.0,
    )
    .with_capture("repository_snapshot", "sha256:runtime")];

    let result = audit(
        "Formal AI is a hosted model.\nIt is always available.\n",
        &evidence,
    );
    let antecedent = &result.statements[0];
    let dependent = &result.statements[1];

    assert!(antecedent.contextual_posterior < 0.5, "{antecedent:#?}");
    assert!(
        (dependent.contextual_posterior - antecedent.contextual_posterior).abs() < f64::EPSILON,
        "a claim about a disproved referent must inherit that upper bound: {dependent:#?}"
    );
    assert!(
        result
            .learning
            .degree(&dependent.id)
            .saturating_sub(dependent.evidence.len() as u64)
            >= 1,
        "the learned network must retain the statement-to-antecedent link"
    );
    assert!(
        result
            .to_links_notation()
            .contains("antecedent_statement_id"),
        "the dependency must be inspectable in the replay artifact"
    );
}

#[test]
fn references_never_cross_document_boundaries() {
    let corpus = RepositoryCorpus::from_documents(vec![
        RepositoryDocument::new("first.md", "Formal AI is symbolic.\n"),
        RepositoryDocument::new("second.md", "It is inspectable.\n"),
    ]);

    let result = audit_corpus(&corpus, &[], AuditConfig::default());
    let statement = result
        .statements
        .iter()
        .find(|statement| statement.location.path == "second.md")
        .expect("second document statement");

    assert_eq!(statement.resolved_text, statement.text);
    assert!(statement.references.is_empty(), "{statement:#?}");
}

#[test]
fn a_demonstrative_determiner_is_not_mistaken_for_a_pronoun() {
    let result = audit(
        "Formal AI is symbolic.\nThis document explains the architecture.\n",
        &[],
    );
    let statement = &result.statements[1];

    assert_eq!(statement.resolved_text, statement.text);
    assert!(statement.references.is_empty(), "{statement:#?}");
}

#[test]
fn a_soft_wrapped_continuation_is_not_mistaken_for_a_new_reference() {
    let result = audit(
        "The dataset is synthetic.\nAvailability is not permission, and synthetic text inherits questions about\nits seeds and generator terms.\nHuman review is required for truth claims\nthat cannot be established from captured evidence.\n",
        &[],
    );

    for text in [
        "its seeds and generator terms.",
        "that cannot be established from captured evidence.",
    ] {
        let statement = result
            .statements
            .iter()
            .find(|statement| statement.text == text)
            .expect("soft-wrapped continuation");
        assert_eq!(statement.resolved_text, statement.text);
        assert!(statement.references.is_empty(), "{statement:#?}");
    }
}

#[test]
fn an_agent_audit_task_uses_the_named_external_evidence_file() {
    let messages = vec![ChatMessage::user(
        "Fact-check every repository statement. Use evidence.json as the external evidence capture file and preserve statement-audit.lino.",
    )];
    let Some(AgenticPlan::ToolCalls(calls)) = plan_chat_step(&messages, &["bash"]) else {
        panic!("statement audit must emit a client-owned command");
    };
    let arguments: serde_json::Value =
        serde_json::from_str(&calls[0].arguments).expect("valid shell arguments");

    assert_eq!(
        arguments["command"],
        "formal-ai statement-audit --root . --evidence evidence.json --output statement-audit.lino"
    );
}
