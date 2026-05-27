//! Intent formalization tests.
//!
//! Issue #299 requires routing to consume a Links-Notation intent structure
//! instead of treating the legacy prompt matcher or handler table as the
//! primary router.

use formal_ai::translation::formalize_prompt;
use formal_ai::{
    formalize_intent, IntentFormalizationCache, IntentKind, MemoryStore, UniversalSolver,
};

#[test]
fn prompt_formalizes_to_intent_links_with_kind_knowns_and_relevants() {
    let candidate = formalize_prompt("translate apple to Russian", "en");
    let intent = formalize_intent("translate apple to Russian", "en", Some(&candidate));

    assert_eq!(intent.kind, IntentKind::Task);
    assert_eq!(intent.route.as_deref(), Some("translation"));
    assert!(
        intent
            .knowns
            .iter()
            .any(|known| known == "formalization:predicate_p:wikidata:P5972"),
        "{intent:?}",
    );
    assert!(
        intent
            .knowns
            .iter()
            .any(|known| known == "formalization:object_q:wikidata:Q89"),
        "{intent:?}",
    );
    assert!(
        intent
            .relevants
            .iter()
            .any(|relevant| relevant == "handler:translation"),
        "{intent:?}",
    );

    let lino = intent.to_links_notation();
    assert!(lino.contains("intent_formalization"), "{lino}");
    assert!(lino.contains("kind \"task\""), "{lino}");
    assert!(
        lino.contains("known \"formalization:predicate_p:wikidata:P5972\""),
        "{lino}",
    );
    assert!(lino.contains("relevant \"handler:translation\""), "{lino}");
}

#[test]
fn write_program_formalization_records_language_and_task_parameters() {
    let intent = formalize_intent("Write a Python program that counts to three", "en", None);

    assert_eq!(intent.kind, IntentKind::Task);
    assert_eq!(intent.route.as_deref(), Some("write_program"));
    assert_eq!(
        intent.parameters.get("language").map(String::as_str),
        Some("python")
    );
    assert_eq!(
        intent.parameters.get("task").map(String::as_str),
        Some("count_to_three")
    );
    assert!(
        intent
            .knowns
            .iter()
            .any(|known| known == "parameter:language:python"),
        "{intent:?}",
    );
    assert!(
        intent
            .knowns
            .iter()
            .any(|known| known == "parameter:task:count_to_three"),
        "{intent:?}",
    );

    let lino = intent.to_links_notation();
    assert!(lino.contains("parameter \"language=python\""), "{lino}");
    assert!(lino.contains("parameter \"task=count_to_three\""), "{lino}");

    let response = UniversalSolver::default().solve("Write a Python program that counts to three");
    assert_eq!(response.intent, "write_program");
    assert!(response
        .evidence_links
        .iter()
        .any(|link| link == "program_parameter:language:python"));
    assert!(response
        .evidence_links
        .iter()
        .any(|link| link == "program_parameter:task:count_to_three"));
}

#[test]
fn repeated_prompt_hits_intent_formalization_cache() {
    let solver = UniversalSolver::default();
    let mut cache = IntentFormalizationCache::new();

    let first = solver.solve_with_intent_cache("translate apple to Russian", &mut cache);
    let second = solver.solve_with_intent_cache("translate apple to Russian", &mut cache);

    assert_eq!(first.intent, second.intent);
    assert_eq!(first.answer, second.answer);
    assert!(
        !first
            .evidence_links
            .iter()
            .any(|link| link.starts_with("cache_hit:intent_formalization:")),
        "{:?}",
        first.evidence_links,
    );
    assert!(
        second
            .evidence_links
            .iter()
            .any(|link| link.starts_with("cache_hit:intent_formalization:")),
        "{:?}",
        second.evidence_links,
    );
    assert!(
        second
            .links_notation
            .contains("intent_formalization_cache hit"),
        "{}",
        second.links_notation,
    );
}

#[test]
fn legacy_greeting_route_is_derived_from_formalized_intent() {
    let response = UniversalSolver::default().solve("Hi");

    assert_eq!(response.intent, "greeting");
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link == "intent_formalization:route:greeting"),
        "{:?}",
        response.evidence_links,
    );
    assert!(
        response
            .links_notation
            .contains("intent_formalization:route greeting"),
        "{}",
        response.links_notation,
    );
}

#[test]
fn supported_language_greetings_route_through_intent_formalization() {
    struct Case {
        language: &'static str,
        prompt: &'static str,
    }

    let cases = [
        Case {
            language: "en",
            prompt: "Hi",
        },
        Case {
            language: "ru",
            prompt: "Привет",
        },
        Case {
            language: "hi",
            prompt: "नमस्ते",
        },
        Case {
            language: "zh",
            prompt: "你好",
        },
    ];

    for case in cases {
        let intent = formalize_intent(case.prompt, case.language, None);
        assert_eq!(intent.language, case.language);
        assert_eq!(intent.kind, IntentKind::Courtesy);
        assert_eq!(intent.route.as_deref(), Some("greeting"), "{intent:?}");

        let response = UniversalSolver::default().solve(case.prompt);
        assert_eq!(response.intent, "greeting");
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link == "intent_formalization:route:greeting"),
            "{:?}",
            response.evidence_links,
        );
    }
}

#[test]
fn intent_formalization_cache_exports_to_durable_links_store() {
    let mut cache = IntentFormalizationCache::new();
    let candidate = formalize_prompt("translate apple to Russian", "en");
    let entry = cache.formalize_or_insert("translate apple to Russian", "en", Some(&candidate));
    assert!(!entry.cache_hit);

    let mut memory = MemoryStore::new();
    let inserted = cache
        .append_to_link_store(&mut memory)
        .expect("intent cache should export to memory store");
    assert_eq!(inserted, 1);

    let exported = memory.export_links_notation();
    assert!(
        exported.contains("kind \"intent_formalization\""),
        "{exported}"
    );
    assert!(exported.contains("intent_formalization"), "{exported}");
    assert!(exported.contains("kind \\\"task\\\""), "{exported}");
}
