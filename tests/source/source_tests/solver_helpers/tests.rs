//! Unit tests for `src/solver_helpers.rs`. Extracted into a sibling file and
//! mounted with `#[path]` so the implementation file stays under the 1000-line
//! Rust file-size limit enforced by `scripts/check-file-size.rs`.

use super::{extract_fenced_block, extract_javascript_program, humanize_url, is_prime};
use crate::concepts::{extract_concept_query, lookup_concept_query, ConceptQuery};
use crate::solver::{SolverConfig, UniversalSolver};

fn lookup_term(term: &str) -> bool {
    lookup_concept_query(&ConceptQuery {
        term: term.to_owned(),
        context: None,
    })
    .is_some()
}

fn extract_term(prompt: &str) -> Option<String> {
    extract_concept_query(prompt).map(|q| q.term)
}

#[test]
fn defaults_are_bounded_and_offline_capable() {
    let config = SolverConfig::default();
    assert!(!config.agent_mode);
    assert!(!config.diagnostic_mode);
    assert!(!config.offline);
    assert_eq!(config.max_decomposition_depth, 4);
}

#[test]
fn greeting_walks_the_universal_loop() {
    let response = UniversalSolver::default().solve("Hi");
    assert_eq!(response.intent, "greeting");
    assert!(response
        .evidence_links
        .iter()
        .any(|link| link.starts_with("impulse:")));
    assert!(response
        .evidence_links
        .iter()
        .any(|link| link.starts_with("search:local")));
    assert!(response
        .evidence_links
        .iter()
        .any(|link| link.starts_with("trace:")));
}

// Regression guard for the keyword/token split in intent-routing.lino:
// before the fix, "hello" was a greeting keyword matched via `contains_token`,
// so any multi-word prompt that mentioned "hello" (like a hello-world request)
// got misrouted to greeting. After the fix, keywords must match the whole
// prompt exactly, and only the dedicated `token "greet"` uses contains.
#[test]
fn hello_world_request_is_not_routed_to_greeting() {
    let response = UniversalSolver::default().solve("Write me hello world program in Rust");
    assert_ne!(
        response.intent, "greeting",
        "answer was: {}",
        response.answer
    );
    assert_eq!(response.intent, "write_program");
    assert!(
        response.answer.to_lowercase().contains("rust"),
        "expected Rust hello world, got: {}",
        response.answer
    );
}

#[test]
fn prime_validation_picks_seventeen_in_range() {
    let response = UniversalSolver::default().solve("Pick a prime number between 14 and 18");
    assert!(response.answer.contains("17"));
}

#[test]
fn prime_check_recognizes_seventeen() {
    assert!(is_prime(17));
    assert!(!is_prime(15));
}

#[test]
fn concept_lookup_finds_seeded_terms() {
    assert!(lookup_term("Wikipedia"));
    assert!(lookup_term("links notation"));
    assert!(lookup_term("the event log"));
    assert!(lookup_term("doublet link"));
    assert!(lookup_term("WebAssembly"));
    assert!(!lookup_term("unknown-concept-xyz"));
}

#[test]
fn concept_extraction_handles_common_prefixes() {
    assert_eq!(
        extract_term("What is Wikipedia?").as_deref(),
        Some("wikipedia"),
    );
    assert_eq!(
        extract_term("Tell me about Links Notation").as_deref(),
        Some("links notation"),
    );
    assert_eq!(
        extract_term("What does Wikidata mean?").as_deref(),
        Some("wikidata"),
    );
    assert_eq!(extract_term("Hi"), None);
    assert_eq!(extract_term("What is 2 + 2?").as_deref(), Some("2 + 2"));
}

#[test]
fn concept_extraction_handles_who_is_variants() {
    assert_eq!(
        extract_term("Tell me, who is Trump").as_deref(),
        Some("trump"),
    );
    assert_eq!(extract_term("Who Trump is").as_deref(), Some("trump"));
}

#[test]
fn concept_extraction_handles_multilingual_prefixes() {
    assert_eq!(
        extract_term("Что такое Википедия?").as_deref(),
        Some("википедия"),
    );
    assert_eq!(
        extract_term("Расскажи про Links Notation").as_deref(),
        Some("links notation"),
    );
    assert_eq!(
        extract_term("विकिपीडिया क्या है?").as_deref(),
        Some("विकिपीडिया"),
    );
    assert_eq!(extract_term("维基百科是什么?").as_deref(), Some("维基百科"),);
    assert_eq!(extract_term("什么是 Rust?").as_deref(), Some("rust"));
}

#[test]
fn concept_lookup_finds_multilingual_aliases() {
    assert!(lookup_term("Википедия"));
    assert!(lookup_term("विकिपीडिया"));
    assert!(lookup_term("维基百科"));
    assert!(lookup_term("recursive digital filter"));
    assert!(lookup_term("IIR滤波器"));
}

#[test]
fn concept_query_splits_term_and_context() {
    let query = extract_concept_query("what is IIR in ML?").expect("should extract");
    assert_eq!(query.term, "iir");
    assert_eq!(query.context.as_deref(), Some("ml"));
}

#[test]
fn concept_query_handles_russian_context_delimiter() {
    let query = extract_concept_query("что такое iir в ml").expect("should extract");
    assert_eq!(query.term, "iir");
    assert_eq!(query.context.as_deref(), Some("ml"));
}

#[test]
fn concept_query_handles_hindi_context_first() {
    let query = extract_concept_query("ML में IIR क्या है").expect("should extract");
    // Hindi puts context before the concept; the parser captures it as
    // the lexical term half. The lookup_concept_query swaps order as
    // needed when ranking against records.
    assert!(query.term == "ml" || query.term == "iir");
    assert!(query.context.is_some());
}

#[test]
fn concept_query_handles_chinese_context_first() {
    let query = extract_concept_query("ML中的IIR是什么").expect("should extract");
    assert!(query.term == "ml" || query.term == "iir");
    assert!(query.context.is_some());
}

#[test]
fn javascript_extraction_finds_fenced_program() {
    let prompt = "Please run this javascript:\n```js\nconsole.log(1 + 2);\n```";
    let body = extract_javascript_program(prompt).expect("should extract");
    assert_eq!(body, "console.log(1 + 2);");
}

#[test]
fn javascript_extraction_requires_explicit_request() {
    let prompt = "Here is some javascript:\n```js\nconsole.log(1);\n```";
    assert_eq!(extract_javascript_program(prompt), None);
}

#[test]
fn fenced_block_picks_matching_language() {
    let text = "intro\n```python\nprint(1)\n```\nthen\n```js\nconsole.log(2)\n```";
    assert_eq!(
        extract_fenced_block(text, &["js"]).as_deref(),
        Some("console.log(2)"),
    );
}

#[test]
fn universal_solver_answers_arithmetic_via_evaluator() {
    let response = UniversalSolver::default().solve("What is 7 * (3 + 4)?");
    assert_eq!(response.intent, "calculation");
    assert!(response.answer.contains("49"));
    assert!(response
        .evidence_links
        .iter()
        .any(|link| link.starts_with("calculation")));
}

#[test]
fn universal_solver_recalls_introduced_name() {
    use crate::solver::{ConversationTurn, UniversalSolver};
    let history = [ConversationTurn::user("My name is Ada.")];
    let response = UniversalSolver::default().solve_with_history("What is my name?", &history);
    assert_eq!(response.intent, "recall_name");
    assert!(response.answer.contains("Ada"));
}

#[test]
fn universal_solver_looks_up_concept() {
    let response = UniversalSolver::default().solve("What is Wikipedia?");
    assert_eq!(response.intent, "concept_lookup");
    assert!(response.answer.to_lowercase().contains("wikipedia"));
}

#[test]
fn solver_config_default_is_offline_capable() {
    let config = SolverConfig::default();
    assert!(!config.offline);
    assert!(!config.agent_mode);
}

// ---------------------------------------------------------------------
// Issue #21: humanize_url renders percent-encoded URLs as readable IRIs
// across every language while preserving query strings and falling back
// gracefully on malformed input.
// ---------------------------------------------------------------------

#[test]
fn humanize_url_decodes_cyrillic_percent_escapes() {
    let encoded = "https://ru.wikipedia.org/wiki/%D0%98%D0%B7%D1%83%D0%BC%D1%80%D1%83%D0%B4";
    assert_eq!(
        humanize_url(encoded),
        "https://ru.wikipedia.org/wiki/Изумруд",
    );
}

#[test]
fn humanize_url_decodes_devanagari_percent_escapes() {
    let encoded =
        "https://hi.wikipedia.org/wiki/%E0%A4%A8%E0%A4%AE%E0%A4%B8%E0%A5%8D%E0%A4%A4%E0%A5%87";
    assert_eq!(humanize_url(encoded), "https://hi.wikipedia.org/wiki/नमस्ते");
}

#[test]
fn humanize_url_decodes_chinese_percent_escapes() {
    let encoded = "https://zh.wikipedia.org/wiki/%E4%BD%A0%E5%A5%BD";
    assert_eq!(humanize_url(encoded), "https://zh.wikipedia.org/wiki/你好");
}

#[test]
fn humanize_url_preserves_reserved_uri_delimiters() {
    // `?`, `&`, `=`, `#`, `/`, `:` must remain percent-encoded so that
    // structural meaning of the URI is not disturbed during display.
    let encoded = "https://example.com/path?a%3Db%26c%3Dd#frag%2Fpart";
    assert_eq!(humanize_url(encoded), encoded);
}

#[test]
fn humanize_url_preserves_query_string_values_around_decoded_path() {
    // The path segment is decoded; the query stays encoded.
    let encoded = "https://ru.wikipedia.org/wiki/%D0%98%D0%B7%D1%83%D0%BC%D1%80%D1%83%D0%B4?utm_source=demo&page=1";
    assert_eq!(
        humanize_url(encoded),
        "https://ru.wikipedia.org/wiki/Изумруд?utm_source=demo&page=1",
    );
}

#[test]
fn humanize_url_returns_original_when_no_percent_escapes_present() {
    let url = "https://en.wikipedia.org/wiki/Albert_Einstein";
    assert_eq!(humanize_url(url), url);
}

#[test]
fn humanize_url_passes_through_malformed_escapes() {
    // `%ZZ` is not a valid escape — leave the bytes as-is rather than
    // throwing. This matches the JS `decodeURI` fallback strategy
    // (catch URIError → return original).
    let url = "https://example.com/%ZZbroken";
    assert_eq!(humanize_url(url), url);
}

#[test]
fn humanize_url_handles_truncated_trailing_percent() {
    let url = "https://example.com/path%";
    assert_eq!(humanize_url(url), url);
}

#[test]
fn humanize_url_accepts_lowercase_hex_digits() {
    let encoded = "https://ru.wikipedia.org/wiki/%d0%98%d0%b7%d1%83%d0%bc%d1%80%d1%83%d0%b4";
    assert_eq!(
        humanize_url(encoded),
        "https://ru.wikipedia.org/wiki/Изумруд",
    );
}

#[test]
fn humanize_url_decodes_only_invalid_utf8_returns_original() {
    // A lone continuation byte (0x80) is not valid UTF-8; fall back to
    // the original URL rather than emitting broken text.
    let url = "https://example.com/%80";
    assert_eq!(humanize_url(url), url);
}

#[test]
fn humanize_url_decodes_mixed_already_decoded_and_encoded_path() {
    let encoded = "https://ru.wikipedia.org/wiki/Изумруд_%28минерал%29";
    // `(` is `%28` and not a reserved delimiter, so it decodes.
    assert_eq!(
        humanize_url(encoded),
        "https://ru.wikipedia.org/wiki/Изумруд_(минерал)",
    );
}
