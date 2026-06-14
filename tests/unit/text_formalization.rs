//! Regression tests for the deterministic text-to-knowledge formalization
//! module (issue #468).
//!
//! These pin the behaviours the case study claims so none can silently regress:
//! the JSON wire format round-trips the article's **own** example, the curated
//! Tale exercises all nine primitives, the whole base reduces to a fixed number
//! of links/doublets, the declarative query selects the right assertions, and
//! the constrained extractor reproduces the worked example while refusing to
//! guess off-template.

use formal_ai::{
    formalize_sentence, formalize_tale, tale_knowledge_base, Extractor, KbFormat, KnowledgeBase,
    PredicateRef, ProtocolDocument, Query, Term,
};

/// The article's authoritative JSON example (a single annotated assertion),
/// quoted verbatim from `docs/case-studies/issue-468/raw-data/article-summary.md`.
const ARTICLE_EXAMPLE_JSON: &str = r#"{
  "doc_id": "doc-0001",
  "annotations": [
    {
      "id": "a1",
      "type": "Assertion",
      "subject": { "type": "Entity", "id": "ent:petrov_petr", "label": "Пётр Петров" },
      "predicate": { "type": "Predicate", "id": "pred:open", "name": "открыл" },
      "object": [ { "type": "Entity", "id": "ent:shop_001", "label": "магазин" } ],
      "time": { "type": "Instant", "value": "2019-00-00", "granularity": "year" },
      "context": { "id": "ctx:loc", "properties": { "location": "Москва" } },
      "modal": { "type": "assertion", "confidence": 0.95 },
      "provenance": { "source_doc": "doc-0001", "offsets": [0, 37], "extractor": "nlp_v1" }
    }
  ]
}"#;

/// R307/R311: the canonical JSON wire format round-trips the article's own
/// example losslessly — the serializer is conformant, not merely inspired.
#[test]
fn article_json_round_trips_losslessly() {
    let document = ProtocolDocument::from_json(ARTICLE_EXAMPLE_JSON)
        .expect("the article example should parse into a ProtocolDocument");

    // Parse the original text and the re-serialized text into structural JSON
    // values and compare those, so field ordering and whitespace do not matter.
    let original: serde_json::Value =
        serde_json::from_str(ARTICLE_EXAMPLE_JSON).expect("the example is valid JSON");
    let reserialized: serde_json::Value =
        serde_json::from_str(&document.to_json()).expect("our output is valid JSON");

    assert_eq!(
        original, reserialized,
        "the article example must survive a parse/serialize round-trip unchanged"
    );

    // A pure-assertion document carries no directory: the operational format is
    // exactly {doc_id, annotations}, with no schema overhead (R309).
    assert!(
        document.directory.is_empty(),
        "the article example declares no directory"
    );
    assert!(
        !reserialized
            .as_object()
            .expect("document is a JSON object")
            .contains_key("directory"),
        "an empty directory must be omitted from the JSON wire format"
    );

    // The one decoded assertion matches the article field-for-field.
    assert_eq!(document.annotations.len(), 1);
    let assertion = &document.annotations[0];
    assert_eq!(assertion.id, "a1");
    assert_eq!(assertion.subject_id(), Some("ent:petrov_petr"));
    assert_eq!(assertion.predicate_id(), "pred:open");
    assert_eq!(assertion.object.len(), 1);
    assert_eq!(assertion.object[0].node_id(), "ent:shop_001");
    assert!((assertion.confidence() - 0.95).abs() < f64::EPSILON);
    assert_eq!(
        assertion
            .time
            .as_ref()
            .and_then(formal_ai::Temporal::calendar_year),
        Some(2019)
    );
    assert_eq!(
        assertion
            .context
            .as_ref()
            .and_then(formal_ai::Context::location),
        Some("Москва")
    );
}

/// R312: the curated Tale knowledge base exercises **all nine** primitives, with
/// the exact counts pinned so it cannot quietly lose one.
#[test]
fn tale_covers_all_nine_primitives() {
    let kb = tale_knowledge_base();
    let coverage = kb.coverage();

    assert!(
        coverage.covers_all_nine(),
        "the curated tale must cover all nine primitives: {coverage:?}"
    );

    // Exact counts (also asserted in docs/case-studies/issue-468/README.md §4.3).
    assert_eq!(coverage.concepts, 3, "concepts");
    assert_eq!(coverage.entities, 5, "entities");
    assert_eq!(coverage.predicates, 7, "predicates");
    assert_eq!(coverage.assertions, 7, "assertions");
    assert_eq!(coverage.procedures, 1, "procedures");
    assert_eq!(coverage.contexts, 4, "contexts");
    assert_eq!(coverage.temporals, 2, "temporals");
    assert_eq!(coverage.modals, 7, "modals");
    assert_eq!(coverage.annotations, 1, "annotations");
}

/// R311: the entire knowledge base reduces to a fixed doublet stream. The count
/// is pinned so any drift in a primitive's reduction trips this test.
#[test]
fn tale_reduces_to_exactly_one_hundred_fifteen_links() {
    let kb = tale_knowledge_base();
    let links = kb.to_links();

    assert_eq!(
        kb.link_count(),
        115,
        "the curated tale reduces to exactly 115 links/doublets"
    );
    assert_eq!(
        links.len(),
        kb.link_count(),
        "to_links() length and link_count() must agree"
    );

    // Every reduced link is a real source -> target edge with a non-empty id.
    for link in &links {
        assert!(!link.id.is_empty(), "every link carries an id");
        assert!(!link.source.is_empty(), "every link has a source");
        assert!(!link.target.is_empty(), "every link has a target");
    }

    // The reduced doublet stream is deterministic: identical bases produce an
    // identical link set in identical order.
    let again = tale_knowledge_base().to_links();
    assert_eq!(links, again, "the doublet reduction must be deterministic");
}

/// R311: the structured `.lino` view carries the header record (with the
/// no-neural-inference policy), while the reduced doublet view does not — the
/// two renderings are distinct but derived from the same base.
#[test]
fn lino_and_links_views_have_distinct_shapes() {
    let kb = tale_knowledge_base();

    let lino = kb.to_lino();
    assert!(
        lino.contains("formal_ai_text_formalization"),
        "the structured lino view starts with the header record"
    );
    assert!(
        lino.contains("deterministic reduction; no neural network inference"),
        "the header records the no-neural-inference policy"
    );

    let links = kb.to_links_lino();
    assert!(
        !links.contains("formal_ai_text_formalization"),
        "the reduced doublet view carries no header record"
    );
    assert!(
        links.starts_with("lnk:concept:greed:type"),
        "the doublet stream starts with the first concept's type edge"
    );
    assert!(
        links.contains("target \"Concept\""),
        "doublet targets are quoted Links-Notation values"
    );
}

/// R306/R311: the three renderings are all derived from one base; the JSON one
/// round-trips losslessly through `ProtocolDocument`.
#[test]
fn three_renderings_are_consistent() {
    let kb = tale_knowledge_base();

    let lino = formalize_tale(KbFormat::Lino);
    let json = formalize_tale(KbFormat::Json);
    let links = formalize_tale(KbFormat::Links);

    assert_eq!(lino, kb.to_lino());
    assert_eq!(json, kb.to_json_pretty());
    assert_eq!(links, kb.to_links_lino());

    // The JSON wire format round-trips the curated base unchanged.
    let reparsed = KnowledgeBase::from_json(&json).expect("tale JSON parses back");
    assert_eq!(
        reparsed.to_json_pretty(),
        json,
        "the curated tale must survive a JSON round-trip"
    );
}

/// R308/R312: a nested (higher-order) assertion — the old woman demands *that the
/// fish make her a ruler* — is encoded as an assertion-reference object (§12).
#[test]
fn nested_assertion_is_an_assertion_reference() {
    let kb = tale_knowledge_base();
    let demand = kb
        .assertion("a:demand_sea")
        .expect("the tale contains the sea-demand assertion");

    assert_eq!(demand.object.len(), 1, "the demand has a single object");
    match &demand.object[0] {
        Term::AssertionRef { id } => {
            assert_eq!(
                id, "b:make_ruler",
                "the object references the nested make-ruler assertion"
            );
        }
        other => panic!("expected an assertion reference, got {other:?}"),
    }

    // The nested assertion itself is a *possibility* (the article's modality
    // spectrum), with a confidence below certainty.
    let nested = kb
        .assertion("b:make_ruler")
        .expect("the nested assertion is declared");
    assert_eq!(nested.modal.kind, "possibility");
    assert!(
        nested.confidence() < 1.0,
        "a possibility is not asserted with full confidence"
    );
}

/// R308: the declarative conjunctive query selects assertions; querying by the
/// old man's subject returns exactly his two acts, in document order.
#[test]
fn declarative_query_selects_subject_assertions() {
    let kb = tale_knowledge_base();

    // Programmatic builder form.
    let query = Query::new().with_subject("ent:old_man");
    let matched: Vec<&str> = kb.query(&query).iter().map(|a| a.id.as_str()).collect();
    assert_eq!(
        matched,
        vec!["a:catch", "a:release"],
        "the old man is the subject of exactly the catch and release"
    );

    // The textual form (article §9) parses to the same result.
    let textual = kb
        .query_text("SELECT ?act WHERE subject = ent:old_man")
        .expect("the textual query parses");
    let textual_ids: Vec<&str> = textual.iter().map(|a| a.id.as_str()).collect();
    assert_eq!(textual_ids, vec!["a:catch", "a:release"]);

    // An empty query matches every assertion.
    assert_eq!(kb.query(&Query::new()).len(), kb.coverage().assertions);
}

/// R313: the constrained extractor reproduces the article's worked example
/// «Пётр открыл магазин в Москве в 2019 году.» exactly.
#[test]
fn extractor_reproduces_worked_example() {
    let sentence = "Пётр открыл магазин в Москве в 2019 году.";
    let extraction = Extractor::new()
        .extract("doc-0001", sentence)
        .expect("the worked-example sentence matches the template");

    let assertion = &extraction.assertion;
    assert_eq!(assertion.subject_id(), Some("ent:petrov_petr"));
    assert_eq!(assertion.predicate_id(), "pred:open");
    assert_eq!(assertion.object.len(), 1);
    assert_eq!(assertion.object[0].node_id(), "ent:shop_001");
    assert_eq!(
        assertion
            .time
            .as_ref()
            .and_then(formal_ai::Temporal::calendar_year),
        Some(2019)
    );
    assert_eq!(
        assertion
            .context
            .as_ref()
            .and_then(formal_ai::Context::location),
        Some("Москва")
    );
    let provenance = assertion
        .provenance
        .as_ref()
        .expect("the extractor records provenance");
    assert_eq!(provenance.source_doc, "doc-0001");
    assert_eq!(provenance.extractor, "deterministic_v1");

    // The annotation grounds the assertion and tokenizes the source.
    assert_eq!(extraction.annotation.language, "ru");
    assert!(
        !extraction.annotation.tokenization.is_empty(),
        "the extracted annotation tokenizes the source span"
    );

    // The predicate of the extraction matches the supporting declaration.
    let predicate = PredicateRef::new(extraction.predicate.id.clone(), "");
    assert_eq!(predicate.id(), "pred:open");

    // Rendering through the public helper produces non-empty JSON.
    let rendered = formalize_sentence("doc-0001", sentence, KbFormat::Json)
        .expect("the worked example renders");
    assert!(rendered.contains("ent:petrov_petr"));
}

/// R313: the extractor **never guesses** — anything outside its template or
/// lexicon yields `None`.
#[test]
fn extractor_never_guesses_off_template() {
    let extractor = Extractor::new();

    // Wrong template shape (too few tokens / no locative scaffolding).
    assert!(extractor.extract("d", "Пётр открыл магазин.").is_none());
    // A subject outside the lexicon.
    assert!(extractor
        .extract("d", "Иван открыл магазин в Москве в 2019 году.")
        .is_none());
    // A predicate outside the lexicon.
    assert!(extractor
        .extract("d", "Пётр закрыл магазин в Москве в 2019 году.")
        .is_none());
    // A non-numeric year.
    assert!(extractor
        .extract("d", "Пётр открыл магазин в Москве в прошлом году.")
        .is_none());
    // Empty input.
    assert!(extractor.extract("d", "").is_none());

    // The public helper propagates the None.
    assert!(formalize_sentence("d", "Иван пошёл домой.", KbFormat::Json).is_none());
}

/// R309: the operational format separates the fact-free directory (catalogue)
/// from the assertions (facts); a document built from a knowledge base preserves
/// both, and a round-trip through the directory form is lossless.
#[test]
fn document_separates_directory_from_assertions() {
    let kb = tale_knowledge_base();
    let document = kb.to_document();

    // The curated tale has a non-empty directory (it declares concepts,
    // entities, predicates, procedures, contexts and an annotation).
    assert!(
        !document.directory.is_empty(),
        "the curated tale declares a reference directory"
    );
    // The assertions are the facts.
    assert_eq!(document.annotations.len(), kb.coverage().assertions);

    // A document round-trips back into an equivalent knowledge base.
    let rebuilt = KnowledgeBase::from_document(document);
    assert_eq!(rebuilt.to_json_pretty(), kb.to_json_pretty());
}
