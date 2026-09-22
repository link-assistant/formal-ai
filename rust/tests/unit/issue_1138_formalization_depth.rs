//! Issue #1138, plan 04 L4–L14 — formalization that says what it lacks.
//!
//! Today the formalizer stores sentences: an unfamiliar requirement yields zero
//! grounded concepts in all five languages, and it cannot report that it
//! understood nothing. After plan 04 every unresolved surface becomes a `Need`,
//! a satisfied need becomes a concept grounded in the exact retrieved bytes, a
//! gloss may raise its own needs at the next depth, and a document with an
//! unresolved need can never be reported as covered.
//!
//! The held-out requirements come from
//! `data/benchmarks/formalization-depth-requirements.lino`; none of them says
//! what `isogram` or `lipogram` means.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use formal_ai::agentic_coding::formalize_text_to_links;
use formal_ai::coding_research_learning as research_learning;
use formal_ai::concept_lookup::{ConceptSense, LookupOutcome, RegistrySourceLookup};
use formal_ai::formalization::concept_links::{ConceptGraph, formalize_deeply};
use formal_ai::formalization::concepts::concept_from_sense;
use formal_ai::formalization::procedures::ExtractedProcedure;
use formal_ai::how_to_guide::{GuideStep, ServicePreferences};
use formal_ai::needs::{Need, NeedKind, NeedState};
use formal_ai::procedure_text::ProcedureStepRecord;
use formal_ai::relative_meta_logic::SourceTier;
use formal_ai::service_accessibility::ServiceAccessibilityCache;
use formal_ai::source_fetch::{CachedSourceClient, CurlSourceTransport};
use formal_ai::source_walk::{LookupBounds, SourceLookup, WalkSourceOutcome};

const CORPUS: &str = "data/benchmarks/formalization-depth-requirements.lino";
// Deep formalization consumes the exact concept captures recorded by plan 01.
// Reusing that content-addressed fixture keeps one canonical copy of source
// bytes instead of duplicating it under a plan-specific directory.
const SOURCE_FIXTURE_DIR: &str = "rust/tests/fixtures/issue-1138-b1";
const PARITY_FIXTURE_DIR: &str = "rust/tests/fixtures/issue-1138-b4";
const PARITY_FILE: &str = "expected-graphs.json";
const RECORDED_GRAPH_IDENTITY: &str = "concept_graph_3a37a219dc8e55c7";

const NEED_REPORT_INTENTS: &[&str] = &[
    "formalization_unresolved_need",
    "formalization_grounded_concept",
    "formalization_extracted_procedure",
    "formalization_depth_exhausted",
    "formalization_offline_need",
];
const NEED_REPORT_LANGUAGES: &[&str] = &["en", "ru", "hi", "zh", "es"];
const NEED_REPORT_EXAMPLES: &[(&str, &str, &str)] = &[
    (
        "formalization_unresolved_need",
        "en",
        "3 of 7 needs grounded; “unfamiliar term” unresolved at depth 2.",
    ),
    (
        "formalization_unresolved_need",
        "ru",
        "Обоснованы 3 из 7 потребностей; «незнакомый термин» не разрешён на глубине 2.",
    ),
    (
        "formalization_unresolved_need",
        "hi",
        "7 में से 3 आवश्यकताएँ प्रमाणित हैं; “अपरिचित पद” गहराई 2 पर अनसुलझा है।",
    ),
    (
        "formalization_unresolved_need",
        "zh",
        "7 个需求中有 3 个已获依据；“陌生术语”在深度 2 仍未解决。",
    ),
    (
        "formalization_unresolved_need",
        "es",
        "3 de 7 necesidades fundamentadas; «término desconocido» sigue sin resolverse en la profundidad 2.",
    ),
];

struct Requirement {
    family: String,
    language: String,
    prompt: String,
}

fn requirements() -> Vec<Requirement> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the repository root sits one level above the crate");
    let text = fs::read_to_string(root.join(CORPUS)).expect("formalization-depth corpus");
    let mut out = Vec::new();
    let mut current: Option<Requirement> = None;
    for line in text.lines() {
        let trimmed = line.trim();
        if line.starts_with("  paraphrase ") {
            if let Some(record) = current.take() {
                out.push(record);
            }
            current = Some(Requirement {
                family: String::new(),
                language: String::new(),
                prompt: String::new(),
            });
        } else if let Some(record) = &mut current {
            if let Some(value) = trimmed.strip_prefix("family ") {
                value.trim().clone_into(&mut record.family);
            } else if let Some(value) = trimmed.strip_prefix("language ") {
                value.trim().clone_into(&mut record.language);
            } else if let Some(value) = trimmed.strip_prefix("prompt ") {
                record.prompt = value.trim().trim_matches('"').replace("\"\"", "\"");
            }
        }
    }
    out.extend(current);
    out
}

fn english(family: &str) -> String {
    requirements()
        .into_iter()
        .find(|case| case.family == family && case.language == "en")
        .map(|case| case.prompt)
        .expect("an English requirement per family")
}

/// A lookup that answers from an explicit list of senses, each carrying the
/// provenance of the bytes it stands for, and records every consulted source.
struct FixtureLookup {
    senses: Vec<ConceptSense>,
    consulted: Vec<String>,
}

impl FixtureLookup {
    const fn empty() -> Self {
        Self {
            senses: Vec::new(),
            consulted: Vec::new(),
        }
    }

    const fn with(senses: Vec<ConceptSense>) -> Self {
        Self {
            senses,
            consulted: Vec::new(),
        }
    }
}

impl SourceLookup for FixtureLookup {
    fn lookup(&mut self, need: &Need, _bounds: &LookupBounds) -> LookupOutcome {
        self.consulted.push(need.subject.clone());
        let matched: Vec<ConceptSense> = self
            .senses
            .iter()
            .filter(|sense| sense.surface.eq_ignore_ascii_case(&need.subject))
            .cloned()
            .collect();
        if matched.is_empty() {
            return LookupOutcome::NotFound {
                consulted: vec![WalkSourceOutcome {
                    source_id: "wiktionary".to_owned(),
                    status: "no_items".to_owned(),
                    detail: need.subject.clone(),
                    pages: 1,
                    items: 0,
                }],
            };
        }
        LookupOutcome::Found(matched)
    }
}

fn sense(surface: &str, gloss: &str) -> ConceptSense {
    ConceptSense {
        surface: surface.to_owned(),
        lemma: surface.to_owned(),
        language: "en".to_owned(),
        gloss: gloss.to_owned(),
        part_of_speech: "noun".to_owned(),
        synonyms: Vec::new(),
        source_id: "wiktionary".to_owned(),
        source_url: format!("https://en.wiktionary.org/wiki/{surface}"),
        sha256: "e".repeat(64),
        fetched_at: "2026-09-16T00:00:00Z".to_owned(),
        cached: true,
        tier: SourceTier::IndependentCorroboration,
        license_name: "CC BY-SA 4.0".to_owned(),
        license_url: "https://creativecommons.org/licenses/by-sa/4.0/".to_owned(),
        depth: 0,
    }
}

fn graph_with(lookup: &mut FixtureLookup, text: &str, depth: usize) -> ConceptGraph {
    formalize_deeply(
        text,
        "doc:requirement",
        lookup,
        &LookupBounds::default(),
        depth,
    )
}

#[test]
fn every_formalization_need_outcome_has_a_five_language_meaning() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the repository root sits one level above the crate");
    let path = "data/seed/meanings-formalization-needs.lino";
    let meanings = fs::read_to_string(root.join(path)).expect("formalization need meanings");

    assert!(
        formal_ai::seed::seed_files()
            .iter()
            .any(|(registered, _)| *registered == path),
        "the native offline bundle must include the need-reporting meanings"
    );
    for intent in NEED_REPORT_INTENTS {
        assert_eq!(
            meanings.matches(&format!("\n  {intent}\n")).count(),
            1,
            "one language-independent meaning for {intent}"
        );
        for language in NEED_REPORT_LANGUAGES {
            assert_eq!(
                meanings
                    .matches(&format!("\n  response_{intent}_{language}\n"))
                    .count(),
                1,
                "one {language} reporting meaning for {intent}"
            );
        }
    }
    assert_eq!(NEED_REPORT_INTENTS.len() * NEED_REPORT_LANGUAGES.len(), 25);
    for (intent, language, example) in NEED_REPORT_EXAMPLES {
        let heading = format!("  response_{intent}_{language}");
        let mut record = meanings
            .lines()
            .skip_while(|line| *line != heading)
            .skip(1)
            .take_while(|line| line.starts_with("    "));
        let actual = record
            .find_map(|line| line.strip_prefix("    example "))
            .expect("localized response meaning has an example");
        assert_eq!(
            actual,
            format!("\"{example}\""),
            "{intent}/{language} must retain its exact public example"
        );
    }
}

fn step(ordinal: usize, text: &str, license: &str) -> ProcedureStepRecord {
    ProcedureStepRecord {
        ordinal,
        text: text.to_owned(),
        source_id: "wikihow".to_owned(),
        source_url: "https://www.wikihow.com/Check-A-Lipogram".to_owned(),
        sha256: "f".repeat(64),
        fetched_at: "2026-09-16T00:00:00Z".to_owned(),
        license_name: license.to_owned(),
        license_url: "https://creativecommons.org/licenses/by-nc-sa/3.0/".to_owned(),
        depth: 0,
    }
}

#[test]
fn an_unfamiliar_requirement_raises_a_need_for_every_unresolved_surface() {
    let text = english("isogram_requirement");
    let mut lookup = FixtureLookup::empty();
    let graph = graph_with(&mut lookup, &text, 1);

    assert!(
        graph
            .needs
            .iter()
            .any(|need| need.subject.eq_ignore_ascii_case("isogram")),
        "the unknown word must become a need: {:?}",
        graph.needs
    );
    for need in &graph.needs {
        assert_eq!(need.kind, NeedKind::Concept);
        assert!(
            need.source_span.starts_with("doc:requirement@"),
            "a need names the exact span it was raised from: {}",
            need.source_span
        );
    }
    assert!(
        lookup.consulted.iter().any(|subject| subject == "isogram"),
        "a raised need is actually asked about"
    );
}

#[test]
fn a_need_is_satisfied_by_the_registry_lookup_and_becomes_a_grounded_concept() {
    let text = english("isogram_requirement");
    let mut lookup = FixtureLookup::with(vec![sense(
        "isogram",
        "a word in which no letter is repeated",
    )]);
    let graph = graph_with(&mut lookup, &text, 1);

    let concept = graph
        .concepts
        .iter()
        .find(|concept| concept.label.eq_ignore_ascii_case("isogram"))
        .expect("the retrieved sense becomes a concept");
    assert_eq!(concept.id, "concept:isogram");
    assert_eq!(concept.genus.as_deref(), Some("word"));
    assert_eq!(concept.source_id, "wiktionary");
    assert_eq!(concept.sha256.len(), 64);
    assert_eq!(concept.license_name, "CC BY-SA 4.0");
    assert!(
        graph
            .needs
            .iter()
            .filter(|need| need.subject.eq_ignore_ascii_case("isogram"))
            .all(|need| need.state == NeedState::Satisfied),
        "a grounded need is satisfied, and only by evidence"
    );
    assert_eq!(
        concept_from_sense(&sense("isogram", "a word in which no letter is repeated"))
            .map(|extracted| extracted.id),
        Some("concept:isogram".to_owned())
    );
}

#[test]
fn a_grounded_gloss_raises_its_own_needs_at_the_next_depth_and_stops_at_the_bound() {
    let text = english("isogram_requirement");
    let mut lookup = FixtureLookup::with(vec![
        sense("isogram", "a word that repeats no grapheme"),
        sense("grapheme", "the smallest unit of a writing system"),
    ]);
    let graph = graph_with(&mut lookup, &text, 1);

    assert!(
        graph.needs.iter().any(|need| need.depth == 1),
        "a retrieved gloss raises its own needs at depth 1: {:?}",
        graph.needs
    );
    assert!(
        graph.needs.iter().all(|need| need.depth <= 1),
        "recursion stops at the declared bound, and is never a time budget"
    );
}

#[test]
fn an_ungrounded_need_is_reported_with_its_origin_span_and_consulted_sources() {
    let text = english("isogram_requirement");
    let mut lookup = FixtureLookup::empty();
    let graph = graph_with(&mut lookup, &text, 1);

    let unresolved = graph.unresolved();
    assert!(
        !unresolved.is_empty(),
        "an unmet need is reported, not dropped"
    );
    for need in unresolved {
        assert_eq!(need.state, NeedState::Unsatisfiable);
        assert_eq!(need.satisfied_by, None, "nothing satisfies an unmet need");
        let (_, span) = need
            .source_span
            .split_once('@')
            .expect("a span of the form doc@start:end");
        let (start, end) = span.split_once(':').expect("start:end");
        let start: usize = start.parse().expect("start offset");
        let end: usize = end.parse().expect("end offset");
        assert_eq!(
            &text[start..end],
            need.subject,
            "the span selects the surface"
        );
    }
}

#[test]
fn a_document_with_an_unresolved_need_is_never_reported_as_covered() {
    let text = english("isogram_requirement");
    let mut lookup = FixtureLookup::empty();
    let graph = graph_with(&mut lookup, &text, 1);
    let (grounded, total) = graph.grounded_ratio();

    assert!(total > 0, "an unfamiliar requirement raises needs");
    assert!(
        grounded < total,
        "nothing was grounded, so coverage is partial"
    );

    let formalized = formalize_text_to_links(&text, "doc:requirement");
    assert_eq!(formalized.summary.needs_raised, total);
    assert_eq!(formalized.summary.needs_grounded, grounded);
    assert!(
        !formalized.summary.covers_all_nine(),
        "a document whose key concept is unresolved is not fully covered"
    );
}

#[test]
fn preserved_sentences_no_longer_satisfy_the_assertion_primitive() {
    let text = english("isogram_requirement");
    let formalized = formalize_text_to_links(&text, "doc:requirement");

    assert!(
        !formalized.links_notation.contains("pred:states"),
        "a preserved sentence is a span, not a stated assertion: {}",
        formalized.links_notation
    );
    assert!(
        formalized.links_notation.contains("preserved_span"),
        "the preserved text keeps its own honest label"
    );
}

#[test]
fn the_same_requirement_in_five_languages_produces_one_concept_graph_identity() {
    let mut identities: BTreeMap<String, String> = BTreeMap::new();
    for case in requirements() {
        let mut lookup = FixtureLookup::with(vec![
            sense("isogram", "a word in which no letter is repeated"),
            sense("lipogram", "a text that avoids a chosen letter"),
        ]);
        let graph = graph_with(&mut lookup, &case.prompt, 1);
        let identity = graph.identity();
        assert!(!identity.is_empty(), "a graph always has an identity");
        if let Some(first) = identities.get(&case.family) {
            assert_eq!(
                &identity, first,
                "{} {} must share the family identity",
                case.family, case.language
            );
        } else {
            assert_eq!(case.language, "en", "English leads each family");
            identities.insert(case.family.clone(), identity);
        }
    }
    assert_eq!(identities.len(), 2, "two held-out families");
}

#[test]
fn an_imperative_clause_sequence_becomes_an_ordered_extracted_procedure() {
    let steps = [
        step(1, "Read the text.", "CC BY-SA 4.0"),
        step(2, "Drop spacing and punctuation.", "CC BY-SA 4.0"),
        step(
            3,
            "Confirm the forbidden letter never appears.",
            "CC BY-SA 4.0",
        ),
    ];
    let procedure = ExtractedProcedure::from_step_records("check a lipogram", &steps, "en")
        .expect("three ordered instructions are a procedure");

    assert_eq!(procedure.steps.len(), 3);
    assert_eq!(procedure.steps[0].position, 1);
    assert_eq!(procedure.steps[0].imperative, "Read");
    assert_eq!(
        procedure.steps[2].object.as_deref(),
        Some("the forbidden letter")
    );
    assert!(
        procedure.steps.iter().all(|step| !step.verified),
        "extraction never marks a step verified; only an execution record does"
    );
    assert_eq!(procedure.source_id, "wikihow");
    assert_eq!(procedure.sha256.len(), 64);

    assert_eq!(
        ExtractedProcedure::from_step_records("check a lipogram", &steps[..1], "en"),
        None,
        "one instruction is not a procedure"
    );
}

#[test]
fn a_guide_step_and_a_captured_step_produce_the_same_record_shape() {
    let captured = step(
        2,
        "Confirm the forbidden letter never appears.",
        "CC BY-SA 4.0",
    );
    let guide = GuideStep {
        text: captured.text.clone(),
        source_id: captured.source_id.clone(),
        source_name: "wikiHow".to_owned(),
        source_url: captured.source_url.clone(),
        sha256: captured.sha256.clone(),
        fetched_at: captured.fetched_at.clone(),
        cached: true,
        tier: SourceTier::IndependentCorroboration,
        license_name: captured.license_name.clone(),
        license_url: captured.license_url.clone(),
        depth: captured.depth,
        position: captured.ordinal,
    };

    assert_eq!(guide.to_step_record(), captured);
}

#[test]
fn an_extracted_procedure_enters_the_ledger_only_through_execution_and_review() {
    let steps = [
        step(1, "Read the text.", "CC BY-SA 4.0"),
        step(
            2,
            "Confirm the forbidden letter never appears.",
            "CC BY-SA 4.0",
        ),
    ];
    let procedure = ExtractedProcedure::from_step_records("check a lipogram", &steps, "en")
        .expect("a two-step procedure");
    let source = procedure.to_coding_procedure_source();

    assert!(
        source.contains("https://www.wikihow.com/Check-A-Lipogram"),
        "the ledger shape carries the procedure's provenance: {source}"
    );
    let refusal = research_learning::adopt_extracted_procedure(
        &procedure,
        None,
        &research_learning::CodingResearchApproval::granted("maintainer"),
    )
    .expect_err("an unexecuted procedure may not enter the ledger");
    assert_eq!(refusal.reason, "coding_research_execution_missing");
}

#[test]
fn a_non_commercial_licensed_procedure_is_shown_but_refused_for_promotion() {
    let steps = [
        step(1, "Read the text.", "CC BY-NC-SA 3.0"),
        step(
            2,
            "Confirm the forbidden letter never appears.",
            "CC BY-NC-SA 3.0",
        ),
    ];
    let procedure = ExtractedProcedure::from_step_records("check a lipogram", &steps, "en")
        .expect("a two-step procedure");
    assert!(
        procedure
            .to_coding_procedure_source()
            .contains("CC BY-NC-SA 3.0"),
        "the license travels with the procedure that is shown to the user"
    );

    let refusal = research_learning::adopt_extracted_procedure(
        &procedure,
        None,
        &research_learning::CodingResearchApproval::granted("maintainer"),
    )
    .expect_err("a non-commercial license blocks promotion");
    assert!(
        refusal.reason.contains("CC BY-NC-SA 3.0"),
        "the refusal names the license it refused on: {}",
        refusal.reason
    );
}

#[test]
fn an_offline_run_replays_the_committed_captures_and_reproduces_the_graph_identity() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the repository root sits one level above the crate");
    let fixture: PathBuf = root.join(SOURCE_FIXTURE_DIR);
    let client = CachedSourceClient::new(&fixture, CurlSourceTransport).with_online(false);
    let preferences = ServicePreferences::default();
    let mut availability =
        ServiceAccessibilityCache::new(std::env::temp_dir().join("formal-ai-issue-1138-b4"));
    let mut lookup = RegistrySourceLookup::new(
        &client,
        &preferences,
        &mut availability,
        LookupBounds::default(),
        "en",
        u64::MAX / 2,
    );
    let text = english("isogram_requirement");
    let graph = formalize_deeply(
        &text,
        "doc:requirement",
        &mut lookup,
        &LookupBounds::default(),
        1,
    );

    assert!(
        !graph.concepts.is_empty(),
        "the committed captures ground the requirement offline"
    );
    let expected = fs::read_to_string(root.join(PARITY_FIXTURE_DIR).join(PARITY_FILE))
        .expect("parity expectation");
    assert_eq!(
        expected,
        "[\n  {\n    \"identity\": \"concept_graph_3a37a219dc8e55c7\"\n  }\n]\n"
    );
    assert_eq!(graph.identity(), RECORDED_GRAPH_IDENTITY);
}

#[test]
fn the_native_and_browser_runtimes_produce_the_same_graph() {
    let fixture: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the repository root sits one level above the crate")
        .join(PARITY_FIXTURE_DIR);
    let expected = fs::read_to_string(fixture.join(PARITY_FILE)).expect("parity expectation");
    let mut lookup = FixtureLookup::with(vec![sense(
        "isogram",
        "a word in which no letter is repeated",
    )]);
    let graph = graph_with(&mut lookup, &english("isogram_requirement"), 1);

    assert_eq!(graph.identity(), RECORDED_GRAPH_IDENTITY);
    assert_eq!(
        expected,
        "[\n  {\n    \"identity\": \"concept_graph_3a37a219dc8e55c7\"\n  }\n]\n"
    );
    assert!(
        graph
            .procedures
            .iter()
            .flat_map(|procedure| procedure.steps.iter())
            .all(|step| !step.verified),
        "the browser has no runtime, so every step it shows is unverified"
    );
}
