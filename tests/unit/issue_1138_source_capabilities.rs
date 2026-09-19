//! Executable source-backed capability regressions for issue #1138 plan 10.
//!
//! Routing a request to a named capability is only the first half. These tests
//! require the solver to execute the shared source-discovery path and project
//! either attributable evidence or an explicit observed boundary. No essay or
//! measurement is memorized for the examples.

use formal_ai::concept_lookup::ConceptSense;
use formal_ai::relative_meta_logic::SourceTier;
use formal_ai::solver::{SolverConfig, UniversalSolver};
use formal_ai::source_capability::{compose_source_evidence, measurement_source_evidence};

fn sense(gloss: &str, source: &str, url: &str) -> ConceptSense {
    ConceptSense {
        surface: String::from("birch"),
        lemma: String::from("birch"),
        language: String::from("en"),
        gloss: gloss.to_owned(),
        part_of_speech: String::from("noun"),
        synonyms: Vec::new(),
        source_id: source.to_owned(),
        source_url: url.to_owned(),
        sha256: format!("sha256-{source}"),
        fetched_at: String::from("2026-09-16T00:00:00Z"),
        cached: true,
        tier: SourceTier::IndependentCorroboration,
        license_name: String::from("fixture-license"),
        license_url: String::from("https://license.invalid/fixture"),
        depth: 0,
    }
}

#[test]
fn composition_assembles_distinct_attributable_source_statements() {
    let evidence = compose_source_evidence(
        "Explain birch growth.",
        &[
            sense(
                "A mature birch commonly reaches 20 metres.",
                "source-a",
                "https://a.invalid/birch",
            ),
            sense(
                "A mature birch commonly reaches 20 metres.",
                "source-a",
                "https://a.invalid/birch",
            ),
            sense(
                "Growth depends on species and habitat.",
                "source-b",
                "https://b.invalid/birch",
            ),
        ],
    );
    assert_eq!(
        evidence,
        "source_capability\n  capability \"compose_from_sources\"\n  request \"Explain birch growth.\"\n  status \"evidence_assembled\"\n  source_count \"2\"\n  statement\n    text \"A mature birch commonly reaches 20 metres.\"\n    source \"source-a\"\n    url \"https://a.invalid/birch\"\n    sha256 \"sha256-source-a\"\n    license \"fixture-license\"\n  statement\n    text \"Growth depends on species and habitat.\"\n    source \"source-b\"\n    url \"https://b.invalid/birch\"\n    sha256 \"sha256-source-b\"\n    license \"fixture-license\"\n"
    );
}

#[test]
fn measurement_extracts_only_quantities_with_grounded_units() {
    let evidence = measurement_source_evidence(
        "How tall is a mature birch?",
        &[
            sense(
                "A mature birch commonly reaches 20 metres and may live 80 years.",
                "source-a",
                "https://a.invalid/birch",
            ),
            sense(
                "Growth depends on species and habitat.",
                "source-b",
                "https://b.invalid/birch",
            ),
        ],
    );
    assert_eq!(
        evidence,
        "source_capability\n  capability \"concept_measurement_lookup\"\n  request \"How tall is a mature birch?\"\n  status \"measurements_extracted\"\n  source_count \"2\"\n  measurement\n    value \"20\"\n    unit \"metres\"\n    source \"source-a\"\n    url \"https://a.invalid/birch\"\n    sha256 \"sha256-source-a\"\n"
    );
}

#[test]
fn offline_composition_executes_and_reports_the_observed_boundary() {
    let solver = UniversalSolver::new(SolverConfig {
        offline: true,
        ..SolverConfig::default()
    });
    let answer = solver.solve("Write an extended piece about how the uncertainty principle arose.");
    assert_eq!(answer.intent, "compose_from_sources");
    assert_eq!(
        answer.answer,
        "source_capability\n  capability \"compose_from_sources\"\n  request \"Write an extended piece about how the uncertainty principle arose.\"\n  status \"unavailable\"\n  reason \"offline\"\n  source_count \"0\"\n"
    );
}

#[test]
fn offline_measurement_executes_and_reports_the_observed_boundary() {
    let solver = UniversalSolver::new(SolverConfig {
        offline: true,
        ..SolverConfig::default()
    });
    let answer = solver.solve("How tall does a mature birch normally grow?");
    assert_eq!(answer.intent, "concept_measurement_lookup");
    assert_eq!(
        answer.answer,
        "source_capability\n  capability \"concept_measurement_lookup\"\n  request \"How tall does a mature birch normally grow?\"\n  status \"unavailable\"\n  reason \"offline\"\n  source_count \"0\"\n"
    );
}
