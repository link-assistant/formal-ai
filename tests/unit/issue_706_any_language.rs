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
        assert!(slugs.contains(&slug.to_owned()), "{slug} missing from registry");
    }
    assert_eq!(formal_ai::language::fallback_language().slug(), "en");
    assert_eq!(
        formal_ai::language::from_slug("es").map(|language| language.slug()),
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
fn a_language_without_localized_openers_reports_a_gap_not_english() {
    // "¿Qué tal la fotosíntesis submarina?" is Spanish (detected from seed
    // rules alone) and has no memoized answer. Issue #706 requires the honest
    // `language_gap` behavior instead of a silent English fallback.
    let engine = formal_ai::FormalAiEngine::default();
    let answer = engine.answer("¿Qué es la fotosíntesis submarina de xyzzy?");
    assert!(
        answer.answer.contains("I detected an unsupported language"),
        "expected the explicit language gap answer, got: {}",
        answer.answer
    );
}
