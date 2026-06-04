use super::*;

#[test]
fn parse_sparql_lemmas_extracts_values_and_languages() {
    let body = r#"{
            "results": {
                "bindings": [
                    {"lemma": {"xml:lang": "ru", "type": "literal", "value": "привет"}},
                    {"lemma": {"xml:lang": "fr", "type": "literal", "value": "bonjour"}}
                ]
            }
        }"#;
    let rows = parse_sparql_lemmas(body);
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].value, "привет");
    assert_eq!(rows[0].language.as_deref(), Some("ru"));
    assert_eq!(rows[1].value, "bonjour");
    assert_eq!(rows[1].language.as_deref(), Some("fr"));
}

#[test]
fn parse_sparql_lemmas_handles_empty_bindings() {
    let body = r#"{"results": {"bindings": []}}"#;
    let rows = parse_sparql_lemmas(body);
    assert!(rows.is_empty());
}

#[test]
fn parse_wbsearch_hits_extracts_lexeme_ids_and_labels() {
    let body = r#"{"searchinfo": {"search": "hello"}, "search": [
            {"id": "L8485", "label": "hello", "description": "..."},
            {"id": "L52", "label": "hello", "description": "..."}
        ]}"#;
    let hits = parse_wbsearch_hits(body);
    assert_eq!(hits.len(), 2);
    assert_eq!(hits[0].id, "L8485");
    assert_eq!(hits[0].label, "hello");
    assert_eq!(hits[1].id, "L52");
}

#[test]
fn read_json_string_decodes_unicode_escapes() {
    let input = "\\u0041BC\"rest";
    assert_eq!(read_json_string(input), "ABC");
}

#[test]
fn escaped_string_advance_skips_to_closing_quote() {
    let input = "hello\"world";
    assert_eq!(escaped_string_advance(input), 6);
}

#[test]
fn escaped_string_advance_handles_unicode_escapes() {
    let input = "\\u0041\"after";
    assert_eq!(escaped_string_advance(input), 7);
}
