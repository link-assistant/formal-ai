//! Issue #1138: repository completion is a world-state delta, not narration.
//!
//! The three public canaries are fixtures for the generic comparison. Their
//! repository names and Hello World requirements never enter production code.

use formal_ai::repository_workspace::world_model::{
    AttributionVerdict, RepositoryGapKind, RepositoryWorldModel,
};

const CANARIES: &str = include_str!("../../data/benchmarks/repository-world-model-canaries.lino");

fn model(case_id: &str) -> RepositoryWorldModel {
    RepositoryWorldModel::from_lino(CANARIES, case_id)
        .unwrap_or_else(|error| panic!("{case_id}: {error}"))
}

fn gap_ids(model: &RepositoryWorldModel) -> Vec<&str> {
    model
        .gaps()
        .iter()
        .map(|gap| gap.requirement_id.as_str())
        .collect()
}

#[test]
fn kotlin_wrong_path_generated_artifact_and_no_ci_remain_open() {
    let model = model("kotlin_claude_latest");

    assert_eq!(model.head(), "302f35a24da3327398302f365793ef4f8ed563c2");
    assert_eq!(
        gap_ids(&model),
        vec![
            "kotlin_source_extension",
            "kotlin_exact_output",
            "kotlin_comments",
            "kotlin_idioms",
            "kotlin_run_instructions",
            "kotlin_ci_workflow",
            "kotlin_ci_push_main",
            "kotlin_ci_pull_request",
            "kotlin_ci_compiler",
            "kotlin_ci_runs_program",
            "kotlin_ci_exact_output",
            "kotlin_ci_green",
            "kotlin_standard_library",
            "kotlin_no_build_artifact",
        ]
    );
    assert_eq!(model.obligation_ledger().satisfied_count(), 0);
    assert_eq!(model.obligation_ledger().refuted_count(), 4);
    assert_eq!(model.obligation_ledger().unattempted_count(), 10);
    assert!(!model.completion_ready());

    let extension = model
        .gaps()
        .iter()
        .find(|gap| gap.requirement_id == "kotlin_source_extension")
        .expect("the wrong source extension remains visible");
    assert_eq!(extension.kind, RepositoryGapKind::Mismatched);
    assert_eq!(extension.expected, vec![String::from(".kt")]);
    assert_eq!(extension.observed, vec![String::from(".java")]);
}

#[test]
fn rust_placeholder_is_not_a_repository_solution_or_a_formal_ai_failure() {
    let model = model("rust_codex_latest");

    assert_eq!(model.head(), "9d74fb354ad3ee68ea64167543be59b6604d0357");
    assert_eq!(model.obligation_ledger().satisfied_count(), 1);
    assert_eq!(model.obligation_ledger().refuted_count(), 3);
    assert_eq!(model.obligation_ledger().unattempted_count(), 10);
    assert!(!model.completion_ready());
    assert_eq!(
        model
            .attributions()
            .iter()
            .map(|attribution| (
                attribution.component.as_str(),
                attribution.verdict,
                attribution.external_issue.as_deref(),
            ))
            .collect::<Vec<_>>(),
        vec![
            (
                "hive_mind",
                AttributionVerdict::Defect,
                Some("https://github.com/link-assistant/hive-mind/issues/2259"),
            ),
            ("formal_ai", AttributionVerdict::NotReached, None),
            ("codex", AttributionVerdict::NotReached, None),
        ]
    );
}

#[test]
fn scala_complete_source_workflow_and_green_checks_discharge_every_requirement() {
    let model = model("scala_agent_latest");

    assert_eq!(model.head(), "724fb2be3c4e10808256368da910c0c3e9eb3d09");
    assert_eq!(
        model.satisfied_requirement_ids(),
        vec![
            "scala_source_extension",
            "scala_exact_output",
            "scala_comments",
            "scala_idioms",
            "scala_run_instructions",
            "scala_ci_workflow",
            "scala_ci_push_main",
            "scala_ci_pull_request",
            "scala_ci_compiler",
            "scala_ci_runs_program",
            "scala_ci_exact_output",
            "scala_ci_green",
            "scala_standard_library",
            "scala_no_build_artifact",
        ]
    );
    assert_eq!(model.gaps(), &[]);
    assert_eq!(model.obligation_ledger().satisfied_count(), 14);
    assert!(model.completion_ready());
    assert_eq!(
        model
            .attributions()
            .iter()
            .map(|attribution| (attribution.component.as_str(), attribution.verdict))
            .collect::<Vec<_>>(),
        vec![
            ("agent_cli", AttributionVerdict::PositiveEvidence),
            ("formal_ai", AttributionVerdict::PositiveEvidence),
            ("hive_mind", AttributionVerdict::PositiveEvidence),
        ]
    );
}

#[test]
fn an_unrelated_case_completes_only_with_matching_evidence_for_every_goal() {
    const COMPLETE: &str = r#"
repository_case unrelated
  record_type "repository_case"
  id "unrelated"
  head "0123456789012345678901234567890123456789"
  language "en"
repository_requirement source
  record_type "repository_requirement"
  case "unrelated"
  id "source"
  origin "issue"
  text "Create a source file."
  subject "primary_source"
  predicate "present"
  accepted "true"
repository_requirement checks
  record_type "repository_requirement"
  case "unrelated"
  id "checks"
  origin "issue"
  text "Run the checks."
  subject "hosted_checks"
  predicate "conclusion"
  accepted "success"
repository_observation source_seen
  record_type "repository_observation"
  case "unrelated"
  id "source_seen"
  subject "primary_source"
  predicate "present"
  value "true"
  evidence_command "repository census"
  evidence_url "https://example.invalid/repository/tree/head"
  evidence_exit "0"
  evidence_kind "symbolic_check"
  evidence_source "harness"
repository_observation checks_seen
  record_type "repository_observation"
  case "unrelated"
  id "checks_seen"
  subject "hosted_checks"
  predicate "conclusion"
  value "success"
  evidence_command "hosted check snapshot"
  evidence_url "https://example.invalid/repository/actions/1"
  evidence_exit "0"
  evidence_kind "symbolic_check"
  evidence_source "harness"
"#;
    const UNEVIDENCED: &str = r#"
repository_case unrelated
  record_type "repository_case"
  id "unrelated"
  head "0123456789012345678901234567890123456789"
  language "en"
repository_requirement source
  record_type "repository_requirement"
  case "unrelated"
  id "source"
  origin "issue"
  text "Create a source file."
  subject "primary_source"
  predicate "present"
  accepted "true"
repository_observation source_seen
  record_type "repository_observation"
  case "unrelated"
  id "source_seen"
  subject "primary_source"
  predicate "present"
  value "true"
"#;

    let complete = RepositoryWorldModel::from_lino(COMPLETE, "unrelated").unwrap();
    assert!(complete.completion_ready());
    assert_eq!(complete.gaps(), &[]);
    assert_eq!(complete.obligation_ledger().satisfied_count(), 2);

    let unevidenced = RepositoryWorldModel::from_lino(UNEVIDENCED, "unrelated").unwrap();
    assert!(!unevidenced.completion_ready());
    assert_eq!(unevidenced.gaps().len(), 1);
    assert_eq!(
        unevidenced.gaps()[0].kind,
        RepositoryGapKind::MissingEvidence
    );
    assert_eq!(unevidenced.obligation_ledger().unattempted_count(), 1);
}

#[test]
fn the_projection_retains_goal_current_gap_need_obligation_and_evidence_links() {
    let projection = model("kotlin_claude_latest").to_links_notation();

    for exact_record in [
        "record_type \"repository_world_model\"",
        "record_type \"repository_goal_fact\"",
        "record_type \"repository_current_fact\"",
        "record_type \"repository_gap\"",
        "record_type \"obligation_ledger\"",
        "record_type \"obligation_node\"",
        "record_type \"evidence\"",
        "kind evidence",
        "state open",
    ] {
        assert!(
            projection.contains(exact_record),
            "missing `{exact_record}` in:\n{projection}"
        );
    }
    assert!(projection.contains("head \"302f35a24da3327398302f365793ef4f8ed563c2\""));
    assert!(projection.contains("requirement_id \"kotlin_source_extension\""));
    assert!(projection.contains("expected \".kt\""));
    assert!(projection.contains("observed \".java\""));
}
