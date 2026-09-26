//! Executable source-backed capability regressions for issue #1138 plan 10.
//!
//! Routing a request to a named capability is only the first half. These tests
//! require the solver to execute the shared source-discovery path and project
//! either attributable evidence or an explicit observed boundary. No essay or
//! measurement is memorized for the examples.

use formal_ai::concept_lookup::ConceptSense;
use formal_ai::relative_meta_logic::SourceTier;
use formal_ai::solver::{SolverConfig, UniversalSolver};
use formal_ai::source_capability::{
    compose_document, compose_source_evidence, measurement_source_evidence,
};

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
    // Plan 10 leaf 17 (issue #1063): the question asks for a property ("tall")
    // and no source sentence names it, so the body says what it looked for —
    // the concept, the asked property and its surface, the source kinds —
    // instead of silently returning an unattributed number.
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
        "source_capability\n  capability \"concept_measurement_lookup\"\n  request \"How tall is a mature birch?\"\n  status \"no_grounded_property\"\n  source_count \"2\"\n  asked_property\n    role \"measurement_property_height\"\n    surface \"tall\"\n    matched \"0\"\n  concept \"birch\"\n  source_kinds \"source-a source-b\"\n  measurement\n    value \"20\"\n    unit \"metres\"\n    source \"source-a\"\n    url \"https://a.invalid/birch\"\n    sha256 \"sha256-source-a\"\n"
    );
}

#[test]
fn measurement_matches_the_quantity_whose_sentence_names_the_asked_property() {
    let evidence = measurement_source_evidence(
        "How tall does a mature birch normally grow?",
        &[
            sense(
                "A mature birch commonly grows 25 metres tall in good habitat.",
                "source-a",
                "https://a.invalid/birch",
            ),
            sense(
                "The trunk may reach a diameter of 40 centimetres.",
                "source-b",
                "https://b.invalid/birch",
            ),
        ],
    );
    assert_eq!(
        evidence,
        "source_capability\n  capability \"concept_measurement_lookup\"\n  request \"How tall does a mature birch normally grow?\"\n  status \"measurements_extracted\"\n  source_count \"2\"\n  asked_property\n    role \"measurement_property_height\"\n    surface \"tall\"\n    matched \"1\"\n  measurement\n    value \"25\"\n    unit \"metres\"\n    property \"measurement_property_height\"\n    source \"source-a\"\n    url \"https://a.invalid/birch\"\n    sha256 \"sha256-source-a\"\n  measurement\n    value \"40\"\n    unit \"centimetres\"\n    source \"source-b\"\n    url \"https://b.invalid/birch\"\n    sha256 \"sha256-source-b\"\n"
    );
}

#[test]
fn measurement_without_an_asked_property_keeps_every_grounded_quantity() {
    // A measurement request that names no seeded property keeps the
    // property-agnostic behaviour: every grounded quantity is evidence.
    let evidence = measurement_source_evidence(
        "Measure a mature birch for me.",
        &[sense(
            "A mature birch commonly reaches 20 metres.",
            "source-a",
            "https://a.invalid/birch",
        )],
    );
    assert_eq!(
        evidence,
        "source_capability\n  capability \"concept_measurement_lookup\"\n  request \"Measure a mature birch for me.\"\n  status \"measurements_extracted\"\n  source_count \"1\"\n  measurement\n    value \"20\"\n    unit \"metres\"\n    source \"source-a\"\n    url \"https://a.invalid/birch\"\n    sha256 \"sha256-source-a\"\n"
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

#[test]
fn composition_procedure_builds_an_attributable_document() {
    let document = compose_document(
        "Write me an extended piece on how the uncertainty principle came about.",
        &[
            sense(
                "Heisenberg stated the uncertainty principle in 1927.",
                "source-a",
                "https://a.invalid/uncertainty",
            ),
            sense(
                "The principle bounds how precisely paired values can be known.",
                "source-b",
                "https://b.invalid/uncertainty",
            ),
        ],
    );
    assert!(document.starts_with("# "), "{document}");
    assert!(
        document.contains("Heisenberg stated the uncertainty principle in 1927."),
        "{document}"
    );
    assert!(
        document.contains("[1]") && document.contains("[2]"),
        "every statement carries its citation marker: {document}"
    );
    assert!(
        document.contains("https://a.invalid/uncertainty")
            && document.contains("https://b.invalid/uncertainty"),
        "the sources footer names every url: {document}"
    );
    assert!(document.contains("fixture-license"), "{document}");
}

#[test]
fn composition_procedure_adds_nothing_beyond_the_cited_statements() {
    // A "few pages" request with one captured statement composes one
    // statement: the document's prose is the boundary note and the glosses,
    // nothing else. Padding to the requested length would be invention.
    let document = compose_document(
        "Write me a few pages on birch growth.",
        &[sense(
            "A mature birch commonly reaches 20 metres.",
            "source-a",
            "https://a.invalid/birch",
        )],
    );
    let prose = document
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            !trimmed.is_empty()
                && !trimmed.starts_with('#')
                && !trimmed.starts_with('[')
                && !trimmed.starts_with("http")
        })
        .collect::<Vec<_>>();
    assert!(
        prose.iter().any(|line| line.contains("20 metres")),
        "{document}"
    );
    assert_eq!(
        prose.len(),
        2,
        "the prose must be the boundary note and the captured statement only: {document}"
    );
}

#[test]
fn composition_procedure_composes_nothing_without_verified_capture() {
    let document = compose_document("Write me a few pages on birch growth.", &[]);
    assert!(
        document.contains("verified") && !document.starts_with("# "),
        "an empty graph must report the observed boundary, not compose: {document}"
    );
}
