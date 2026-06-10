use super::*;

#[test]
fn seed_files_are_present_and_non_empty() {
    let files = seed_files();
    assert!(files.len() >= 10);
    for (name, contents) in files {
        assert!(!contents.trim().is_empty(), "{name} should not be empty");
    }
}

#[test]
fn multilingual_responses_contain_four_languages() {
    let records = multilingual_responses();
    let languages: std::collections::BTreeSet<String> =
        records.iter().map(|r| r.language.clone()).collect();
    for expected in ["en", "ru", "hi", "zh"] {
        assert!(
            languages.contains(expected),
            "expected language {expected} in seed",
        );
    }
    let intents: std::collections::BTreeSet<String> =
        records.iter().map(|r| r.intent.clone()).collect();
    for expected in [
        "greeting",
        "courtesy_response",
        "test_status",
        "identity",
        "assistant_name",
        "capabilities",
        "capabilities_more",
        "unknown",
        "unknown_reasoning_question",
        "unknown_reasoning_trace",
    ] {
        assert!(
            intents.contains(expected),
            "expected intent {expected} in seed",
        );
    }
}

#[test]
fn response_for_returns_known_text() {
    let greeting = response_for("greeting", "en").expect("english greeting");
    assert!(greeting.contains("Hi"), "got {greeting}");
    let identity = response_for("identity", "ru").expect("russian identity");
    assert!(identity.contains("formal-ai"));
}

#[test]
fn issue_394_responses_are_seed_backed() {
    for intent in [
        "capabilities",
        "capabilities_more",
        "unknown_reasoning_question",
        "unknown_reasoning_trace",
    ] {
        for language in ["en", "ru", "hi", "zh"] {
            assert!(
                response_for(intent, language).is_some(),
                "missing {intent}/{language} response",
            );
        }
    }

    let russian_capabilities =
        response_for("capabilities", "ru").expect("russian capabilities response");
    assert!(russian_capabilities.contains("Покажи правила поведения"));
    assert!(russian_capabilities.contains("Покажи правило unknown"));
    assert!(!russian_capabilities.contains("List behavior rules"));

    let russian_reasoning =
        response_for("unknown_reasoning_trace", "ru").expect("russian reasoning response");
    assert!(russian_reasoning.contains("{focus}"));
    assert!(russian_reasoning.contains("правило связей в формате Links Notation"));
    assert!(russian_reasoning.contains("Покажи правила поведения"));
    assert!(russian_reasoning.contains("Покажи правило unknown"));

    let english_unknown = response_for("unknown", "en").expect("english unknown response");
    assert!(english_unknown.contains("local links rules"));
    assert!(!english_unknown.contains("local Links Notation rules"));

    let unsupported_reasoning = response_for("unknown_reasoning_trace", "unknown")
        .expect("unsupported-language reasoning response");
    assert!(unsupported_reasoning.contains("unsupported language"));
}

#[test]
fn agent_info_exposes_expected_keys() {
    let info = agent_info();
    for key in ["name", "supported_languages", "default_language"] {
        assert!(info.contains_key(key), "missing key {key} in agent_info");
    }
    assert_eq!(info.get("name").map(String::as_str), Some("formal-ai"));
}

#[test]
fn language_rules_cover_ru_hi_zh() {
    let rules = language_rules();
    let languages: std::collections::BTreeSet<String> =
        rules.iter().map(|r| r.language.clone()).collect();
    for expected in ["ru", "hi", "zh"] {
        assert!(
            languages.contains(expected),
            "expected language rule for {expected}",
        );
    }
    for rule in rules.iter().filter(|r| r.language != "en") {
        assert!(rule.start > 0 && rule.end >= rule.start);
    }
}

#[test]
fn prompt_patterns_have_intents() {
    let patterns = prompt_patterns();
    let intents: std::collections::BTreeSet<String> =
        patterns.iter().map(|p| p.intent.clone()).collect();
    assert!(intents.contains("concept_lookup"));
    assert!(intents.contains("greeting"));
}

#[test]
fn merged_bundle_includes_every_file_name() {
    let bundle = merged_bundle();
    assert!(bundle.starts_with("formal_ai_seed_bundle"));
    for (name, _) in seed_files() {
        assert!(bundle.contains(name), "bundle missing entry for {name}");
    }
}

#[test]
fn intent_routing_loads_greeting_identity_unknown() {
    let routing = intent_routing();
    let ids: std::collections::BTreeSet<String> =
        routing.intents.iter().map(|r| r.id.clone()).collect();
    for expected in [
        "intent_greeting",
        "intent_farewell",
        "intent_test_status",
        "intent_courtesy_response",
        "intent_assistant_name",
        "intent_identity",
        "intent_unknown",
        "intent_write_program",
        "intent_concept_lookup",
    ] {
        assert!(ids.contains(expected), "missing intent {expected}");
    }
}

#[test]
fn intent_routing_greeting_separates_keywords_from_tokens() {
    let routing = intent_routing();
    let greeting = routing
        .intents
        .iter()
        .find(|r| r.id == "intent_greeting")
        .expect("greeting route should exist");
    assert!(greeting.keywords.iter().any(|k| k == "hello"));
    assert!(
        greeting.tokens.iter().any(|t| t == "greet"),
        "the 'greet' fragment must be a token (substring match), not a keyword (exact match), \
             so that prompts like 'Write me hello world program' don't get routed to greeting",
    );
    assert!(
        !greeting.keywords.iter().any(|k| k == "greet"),
        "regression guard: 'greet' must not be a keyword (exact-prompt match)",
    );
}

#[test]
fn intent_routing_identity_combos_are_split_on_plus() {
    let routing = intent_routing();
    let identity = routing
        .intents
        .iter()
        .find(|r| r.id == "intent_identity")
        .expect("identity route should exist");
    let combos: Vec<&Vec<String>> = identity.combos.iter().collect();
    let who_you = combos
        .iter()
        .find(|c| c.len() == 2 && c[0] == "who" && c[1] == "you");
    assert!(who_you.is_some(), "expected 'who+you' combo to be parsed");
}

#[test]
fn intent_routing_carries_article_and_trace_prefixes() {
    let routing = intent_routing();
    assert!(routing.article_prefixes.iter().any(|a| a == "the "));
    assert!(routing.trace_prefixes.iter().any(|p| p == "trace_"));
}

#[test]
fn bundle_round_trips_through_parse_bundle() {
    let bundle = merged_bundle();
    let sections = parse_bundle(&bundle);
    let files = seed_files();
    assert_eq!(
        sections.len(),
        files.len(),
        "parsed bundle should have one section per seed file",
    );
    for ((parsed_name, parsed_body), (orig_name, orig_body)) in sections.iter().zip(files.iter()) {
        assert_eq!(parsed_name, orig_name, "section names should round-trip");
        // The bundle drops blank lines on emit; compare the non-empty
        // content lines instead of byte-for-byte to keep the test
        // resilient to that normalization.
        let parsed_lines: Vec<&str> = parsed_body.lines().filter(|l| !l.is_empty()).collect();
        let orig_lines: Vec<&str> = orig_body.lines().filter(|l| !l.is_empty()).collect();
        assert_eq!(
            parsed_lines, orig_lines,
            "section body for {orig_name} should round-trip",
        );
    }
}

#[test]
fn parse_bundle_accepts_nested_formal_ai_bundle_dialect() {
    // The browser's `Download bundle` button (and `memory::export_bundle`
    // on the Rust side) writes a `formal_ai_bundle` document where the
    // per-file sections are nested under a `seed_files` wrapper. The
    // parser must recover the same `(name, body)` pairs.
    let files = seed_files();
    let bundle = crate::memory::export_bundle(&files, &[]);
    let sections = parse_bundle(&bundle);
    assert!(
        sections.len() >= files.len(),
        "nested bundle parse should recover every seed file, got {} of {}",
        sections.len(),
        files.len(),
    );
    let names: Vec<&str> = sections.iter().map(|(n, _)| n.as_str()).collect();
    for (name, _) in &files {
        assert!(
            names.contains(name),
            "nested bundle parse missed section {name}",
        );
    }
}

#[test]
fn parse_bundle_recovers_intent_routing_via_inner_parser() {
    // End-to-end smoke test: bundle, parse, then feed one of the inner
    // sections back through the per-file parser. This is the contract
    // that makes single-file import meaningful.
    let bundle = merged_bundle();
    let sections = parse_bundle(&bundle);
    let routing_section = sections
        .iter()
        .find(|(name, _)| name == "data/seed/intent-routing.lino")
        .expect("bundle must contain intent-routing section");
    let tree = parse_lino(&routing_section.1);
    let root = tree
        .children
        .iter()
        .find(|c| c.name == "intent_routing")
        .expect("parsed tree should start with intent_routing");
    let intent_count = root.children.iter().filter(|c| c.name == "intent").count();
    assert!(
        intent_count >= 5,
        "expected at least 5 intent routes after round-trip, got {intent_count}",
    );
}

#[test]
fn parse_lino_preserves_hash_inside_single_quoted_colon_values() {
    let tree = parse_lino(
        "root\n  source: 'Entity[\"Language#Tag\", \"English::385w8\"]' # trailing comment\n  next ok",
    );
    let root = tree
        .children
        .iter()
        .find(|child| child.name == "root")
        .expect("root node should parse");
    let source = root
        .children
        .iter()
        .find(|child| child.name == "source")
        .expect("source field should parse");
    assert_eq!(source.id, "'Entity[\"Language#Tag\", \"English::385w8\"]'",);
    assert!(
        root.children.iter().any(|child| child.name == "next"),
        "trailing comment should not swallow the following line",
    );
}
