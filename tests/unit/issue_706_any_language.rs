//! Acceptance contract for the issue-706 any-language protocol.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(path: impl AsRef<Path>) -> String {
    fs::read_to_string(root().join(path)).expect("issue-706 fixture should be readable")
}

fn registered_languages(ledger: &str) -> Vec<String> {
    ledger
        .lines()
        .filter_map(|line| line.strip_prefix("  language "))
        .map(str::to_owned)
        .collect()
}

#[test]
fn registry_drives_an_automatically_generated_round_trip_matrix() {
    let ledger = read("data/seed/languages.lino");
    let languages = registered_languages(&ledger);
    assert!(languages.len() >= 5, "a fifth language must be registered");

    let matrix = read("docs/case-studies/issue-706/round-trip-matrix.lino");
    for source in &languages {
        assert!(
            matrix.contains(&format!("same_language {source}")),
            "missing {source}->meta->{source} contract"
        );
        for target in &languages {
            assert!(
                matrix.contains(&format!("pair {source}_{target}")),
                "missing generated {source}->{target} contract"
            );
        }
    }
}

#[test]
fn spanish_passes_at_least_eighty_percent_and_gaps_are_explicit() {
    let report = read("docs/case-studies/issue-706/coverage-es.lino");
    assert!(report.contains("language es"));
    assert!(report.contains("suite_coverage_permille 1000"));
    assert!(report.contains("meaning_coverage"));
    assert!(report.contains("language_gap"));
    assert!(report.contains("fallback_policy explicit_gap"));
}

#[test]
fn fifth_language_data_covers_the_required_surfaces() {
    let meanings = read("data/seed/meanings-translation.lino");
    assert!(meanings.contains("    lexeme es"));
    assert!(meanings.contains("        text manzana"));
    assert!(meanings.contains("        text hola"));

    let responses = read("data/seed/multilingual-responses-language-protocol.lino");
    for intent in ["greeting", "identity", "language_gap"] {
        assert!(
            responses.contains(&format!("    intent {intent}\n    language es")),
            "Spanish response missing for {intent}"
        );
    }

    let operations = read("data/seed/operation-vocabulary.lino");
    assert!(operations.contains("    language es"));

    assert_eq!(
        formal_ai::seed::response_for("greeting", "es").as_deref(),
        Some("¡Hola! ¿Cómo puedo ayudarte?")
    );
    assert!(formal_ai::seed::operation_vocabulary().matches("uppercase", "convertir a mayúsculas"));
}

#[test]
fn sixth_language_dry_run_is_data_only_and_reports_coverage() {
    let output = Command::new("node")
        .current_dir(root())
        .args([
            "scripts/language-protocol.mjs",
            "--language",
            "ar",
            "--candidate",
            "data/language-additions/ar.lino",
            "--dry-run",
        ])
        .output()
        .expect("language protocol should run");
    assert!(
        output.status.success(),
        "dry run failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("language ar"));
    assert!(stdout.contains("code_changes 0"));
    assert!(stdout.contains("coverage_report"));
}

#[test]
fn ci_contract_discovers_every_registered_language_from_the_ledger() {
    let coverage_guard = read("tests/e2e/scripts/check-language-test-coverage.mjs");
    let parity_guard = read("tests/e2e/scripts/check-language-change-parity.mjs");
    for guard in [&coverage_guard, &parity_guard] {
        assert!(guard.contains("parseRegisteredLanguages"));
        assert!(guard.contains("data/seed/languages.lino"));
    }
    assert!(
        coverage_guard.contains("es: ['spanish', 'español', 'espanol']"),
        "the coverage guard must recognize Spanish test evidence"
    );
}

#[test]
fn detection_registry_is_seed_data_not_rust_constants() {
    // Every language the detector knows must come from the seed registry, and
    // the detector's own source must not enumerate them. Issue #706 requires a
    // new language to be addable with zero Rust edits.
    let registry = read("data/seed/language-detection.lino");
    for slug in ["en", "ru", "hi", "zh", "es"] {
        assert!(
            registry.contains(&format!("    language {slug}")),
            "{slug} must be a seed record"
        );
    }

    let detector = read("src/language.rs");
    assert!(
        detector.contains("include_str!(\"../data/seed/language-detection.lino\")"),
        "the detector must read its rules from seed data"
    );

    let slugs: Vec<String> = formal_ai::language::registered_languages()
        .iter()
        .map(|language| language.slug().to_owned())
        .collect();
    for slug in ["en", "ru", "hi", "zh", "es"] {
        assert!(
            slugs.contains(&slug.to_owned()),
            "{slug} missing from registry"
        );
    }
    assert_eq!(formal_ai::language::fallback_language().slug(), "en");
    assert_eq!(
        formal_ai::language::from_slug("es").map(formal_ai::Language::slug),
        Some("es")
    );
}

#[test]
fn fifth_language_is_detected_without_any_rust_change() {
    use formal_ai::language::detect;

    for prompt in ["¿Cómo estás?", "hola, ¿quién eres?", "gracias por favor"] {
        assert_eq!(
            detect(prompt).slug(),
            "es",
            "Spanish prompt {prompt:?} must detect as es"
        );
    }

    // The registry must not regress the four pre-existing languages.
    assert_eq!(detect("hello there").slug(), "en");
    assert_eq!(detect("что это такое").slug(), "ru");
    assert_eq!(detect("क्या हाल है").slug(), "hi");
    assert_eq!(detect("你是谁").slug(), "zh");
}

#[test]
fn markers_never_outrank_another_registered_script() {
    // A Latin proper name carries Spanish markers ("julián andrés quiñones"),
    // but the surrounding script is the real evidence: the marker rules only
    // vote when no other registered script is present.
    use formal_ai::language::detect;

    assert_eq!(detect("Расскажи о julián andrés quiñones?").slug(), "ru");
    assert_eq!(detect("介绍一下 julián andrés quiñones?").slug(), "zh");
    assert_eq!(detect("julián andrés quiñones ¿quién es?").slug(), "es");
}

#[test]
fn unknown_openers_are_seed_data_on_every_surface() {
    // The unknown-intent opener pools used to be four Rust constants and a
    // JavaScript object literal keyed by language. Issue #706 requires a new
    // language's openers to be a data edit, so all three surfaces must read
    // `data/seed/unknown-openers.lino`.
    let pools = read("data/seed/unknown-openers.lino");
    for slug in ["en", "ru", "hi", "zh"] {
        assert!(
            pools.contains(&format!("    language {slug}")),
            "{slug} openers must be seed records"
        );
    }
    assert!(pools.contains("fallback_language en"));

    let core = read("src/web_engine_core.rs");
    assert!(
        core.contains("include_str!(\"../data/seed/unknown-openers.lino\")"),
        "the Rust core must read the opener pools from seed data"
    );
    assert!(
        !core.contains("UNKNOWN_OPENERS_RU"),
        "per-language opener constants must not come back"
    );

    let worker = read("src/web/worker/formal_ai_worker_00.js");
    assert!(
        worker.contains("unknown-openers.lino"),
        "the JS worker must hydrate its pools from the seed file"
    );
    assert!(
        !worker.contains("UNKNOWN_OPENERS_BY_LANGUAGE"),
        "the JS worker must not keep a per-language opener literal"
    );

    // The browser's seed inventory moved to `src/web/seed-files.js` in issue
    // #991, generated from `data/meta/seed-registry.lino` so the Rust engine and
    // the worker cannot disagree about which files exist.
    let inventory = read("src/web/seed-files.js");
    assert!(inventory.contains("seed/unknown-openers.lino"));

    // A language with no pool of its own borrows the fallback language's pool
    // rather than resolving to an empty one.
    assert_eq!(
        formal_ai::web_engine_core::unknown_openers_for("es"),
        formal_ai::web_engine_core::unknown_openers_for("en")
    );
    assert!(!formal_ai::web_engine_core::unknown_opener_sentence_separators().is_empty());
}

#[test]
fn language_metadata_comes_from_the_ledger_not_from_rust_branches() {
    // The thinking-log label table, the concept-slug map and the script check
    // used to be three separate Rust `match` arms over the four original
    // languages. All three must now answer for the fifth language without a
    // Rust edit.
    assert_eq!(formal_ai::language::language_name("es"), Some("Spanish"));
    assert_eq!(formal_ai::language::language_name("qq"), None);
    assert_eq!(
        formal_ai::language::language_for_concept_slug("language_spanish")
            .map(formal_ai::Language::slug),
        Some("es")
    );
    assert!(formal_ai::language::surface_matches_language(
        "manzana", "es"
    ));
    assert!(formal_ai::language::surface_matches_language(
        "яблоко",
        "ru"
    ));
    assert!(!formal_ai::language::surface_matches_language(
        "яблоко",
        "zh"
    ));

    let thinking = read("src/thinking.rs");
    assert!(
        !thinking.contains("\"Russian\""),
        "language display names must come from the ledger"
    );
}

#[test]
fn the_learn_cli_replays_the_language_frontier_through_the_shared_cycle() {
    // Issue #706 asked for auto-learning over languages. The issue-#701 cycle
    // is frontier-agnostic, so this must need a *recorded frontier*, not new
    // learning logic: the same cycle, a second registered frontier.
    let slugs: Vec<&str> = formal_ai::learning_cycle::recorded_frontiers()
        .iter()
        .map(|frontier| frontier.slug)
        .collect();
    assert!(slugs.contains(&"google-trends"));
    assert!(slugs.contains(&"language-gap"));
    assert!(formal_ai::learning_cycle::recorded_frontier("nope").is_none());

    let run = formal_ai::learning_cycle::language_gap_learning_cycle();
    assert_eq!(run.frontier, "language-gap");
    assert_eq!(
        run.frontier_items, 7,
        "the frozen frontier keeps all 7 prompts"
    );
    assert!(
        !run.proposals.is_empty(),
        "the cycle must propose something"
    );
    for candidate in &run.candidates {
        assert_eq!(candidate.language, "es");
        assert!(
            candidate.validated(),
            "a proposed frame must pass every held-out test"
        );
    }
    // Both Spanish request frames were derived from the corpus, never written
    // by hand in Rust.
    let rendered = run.links_notation();
    assert!(rendered.contains("qué es …"), "{rendered}");
    assert!(rendered.contains("cuéntame sobre …"), "{rendered}");
}

#[test]
fn adopting_the_proposals_changed_what_the_engine_answers() {
    // A learning loop that only emits proposals proves nothing (issue #701's
    // rule). The ledger replays the frozen "before" record through the live
    // engine and must show a real capability delta for every prompt.
    let ledger = formal_ai::language_adoption::language_adoption_ledger();
    assert_eq!(ledger.pairs.len(), 7);
    assert_eq!(ledger.unadopted().len(), 0, "every recorded prompt adopted");
    for pair in ledger.adopted() {
        assert_eq!(pair.before_intent, "unknown");
        assert_ne!(pair.after_intent, "unknown");
        assert!(pair.term_recovered(), "{pair:?}");
    }

    // The committed artifact is the byte-for-byte rendering of that ledger.
    assert_eq!(
        read("data/meta/language-adoption-ledger.lino"),
        ledger.links_notation(),
        "run `cargo run --example issue_706_language_adoption > data/meta/language-adoption-ledger.lino`"
    );

    // The adopted surfaces live in seed data, not in Rust.
    let seed = read("data/seed/learned-request-openers.lino");
    assert!(seed.contains("    lexeme es"));
    assert!(seed.contains("qué es …"));
    assert!(seed.contains("cuéntame sobre …"));
}

#[test]
fn re_recording_the_language_frontier_now_finds_nothing_to_learn() {
    // The closing half of the loop: recording the frontier again from the same
    // candidate corpus, through the live engine, must come back empty — and the
    // languages that produced nothing must be preserved as explicit gaps rather
    // than silently dropped.
    let directory = root().join("data/language-additions");
    let record = formal_ai::language_frontier::record_language_gap_frontier(&directory)
        .expect("the candidate directory is readable");
    assert!(record.contains("total_prompts \"7\""), "{record}");
    assert!(record.contains("learning_frontier \"0\""), "{record}");
    assert!(
        record.contains("reason \"every_recorded_prompt_already_routes\""),
        "{record}"
    );
    assert!(
        record.contains("reason \"no_prompt_corpus_in_language_addition_file\""),
        "a candidate language without a corpus is an explicit gap, not a silent skip: {record}"
    );
}

#[test]
fn a_language_without_localized_openers_reports_a_gap_not_english() {
    // "¿Cómo funciona …?" is Spanish (detected from seed rules alone), carries
    // no learned request frame and has no memoized answer. Issue #706 requires
    // the honest `language_gap` behavior instead of a silent English fallback.
    //
    // Its sibling "¿Qué es …?" deliberately does *not* land here any more: the
    // learning cycle adopted that frame, so the prompt now routes exactly like
    // its English counterpart. Adoption narrows the gap; it never hides it.
    let engine = formal_ai::FormalAiEngine;
    assert_eq!(
        engine
            .answer("¿Qué es la fotosíntesis submarina de xyzzy?")
            .intent,
        "web_search",
        "an adopted Spanish frame must route like English"
    );
    let answer = engine.answer("¿Cómo funciona la fotosíntesis submarina de xyzzy?");
    assert!(
        answer.answer.contains("I detected an unsupported language"),
        "expected the explicit language gap answer, got: {}",
        answer.answer
    );
}
