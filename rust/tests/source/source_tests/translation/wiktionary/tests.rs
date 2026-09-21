use super::*;

#[test]
fn url_encode_keeps_ascii_alphanumerics() {
    assert_eq!(url_encode("hello"), "hello");
    assert_eq!(url_encode("how are you"), "how_are_you");
    assert_eq!(url_encode("café"), "caf%C3%A9");
}

#[test]
fn query_encode_uses_percent_twenty_for_spaces() {
    assert_eq!(query_encode("SELECT ?x"), "SELECT%20%3Fx");
    assert_eq!(query_encode("a b c"), "a%20b%20c");
    assert_eq!(query_encode("café"), "caf%C3%A9");
}

#[test]
fn clean_surface_strips_wikilinks() {
    assert_eq!(clean_surface("[[hello]]"), "hello");
    assert_eq!(clean_surface("[[hello|hi]]"), "hi");
    assert_eq!(
        clean_surface("[[how do you do]], [[how are you]]"),
        "how do you do, how are you"
    );
}

#[test]
fn clean_surface_strips_nested_templates() {
    assert_eq!(clean_surface("привет {{qualifier|informal}}"), "привет");
}

#[test]
fn clean_surface_handles_alt_key_value_pair() {
    assert_eq!(clean_surface("alt=你好嗎？"), "你好嗎？");
}

#[test]
fn extract_translations_finds_russian_from_english_page() {
    let wikitext = r"
{{trans-top|greeting}}
* Russian: {{t+|ru|привет}}, {{t|ru|здравствуйте}}
* Hindi: {{t|hi|नमस्ते}}, {{t|hi|नमस्कार}}
* Chinese:
*: Mandarin: {{t+|cmn|你好|tr=nǐhǎo}}
{{trans-bottom}}
";
    let candidates = extract_translations(wikitext, "ru");
    assert!(
        candidates.iter().any(|c| c.surface == "привет"),
        "expected привет in {candidates:?}",
    );
    assert!(candidates.iter().any(|c| c.surface == "здравствуйте"));
}

#[test]
fn extract_translations_falls_back_to_chinese_macro_language() {
    let wikitext = "* Chinese: {{t+|cmn|你好|tr=nǐhǎo}}";
    let candidates = extract_translations(wikitext, "zh");
    assert!(candidates.iter().any(|c| c.surface == "你好"));
}

#[test]
fn extract_translations_parses_russian_wiktionary_perev_blok() {
    // Russian Wiktionary uses {{перев-блок}} with |en=, |fr=, etc.
    let wikitext = r"{{перев-блок|общепринятое приветствие
|en=[[how do you do]], [[how are you]]; {{помета|разг.|}}: [[what's up]]
|fr=[[ça va]]
}}";
    let candidates = extract_translations(wikitext, "en");
    let surfaces: Vec<&str> = candidates.iter().map(|c| c.surface.as_str()).collect();
    assert!(
        surfaces.contains(&"how do you do") || surfaces.contains(&"how are you"),
        "expected how do you do / how are you in {candidates:?}",
    );
}

#[test]
fn extract_translations_descends_into_multitrans_wrapper() {
    // English Wiktionary nests translation tables inside
    // `{{multitrans|data=...}}` on heavy pages such as `yes`.
    // The outer template body must be re-scanned, not skipped.
    let wikitext = "{{multitrans|data=\n\
            {{trans-top|word used to indicate agreement}}\n\
            * Russian: {{t+|ru|да}}, {{t+|ru|так}}\n\
            {{trans-bottom}}\n\
            }}";
    let candidates = extract_translations(wikitext, "ru");
    let surfaces: Vec<&str> = candidates.iter().map(|c| c.surface.as_str()).collect();
    assert!(surfaces.contains(&"да"), "expected да in {candidates:?}");
    assert!(surfaces.contains(&"так"), "expected так in {candidates:?}");
}

#[test]
fn extract_translations_recognises_double_t_templates() {
    // English Wiktionary's `hello`, `thank you`, and similar pages use
    // `{{tt+|...}}` (rather than `{{t+|...}}`) inside
    // `{{multitrans|...}}` blocks.
    let wikitext = "{{multitrans|data=\n\
            * Russian: {{tt+|ru|привет}}, {{tt|ru|здравствуйте}}\n\
            }}";
    let candidates = extract_translations(wikitext, "ru");
    let surfaces: Vec<&str> = candidates.iter().map(|c| c.surface.as_str()).collect();
    assert!(surfaces.contains(&"привет"));
    assert!(surfaces.contains(&"здравствуйте"));
}

#[test]
fn extract_translations_strips_combining_stress_accents() {
    // Russian Wiktionary candidates carry U+0301 (combining acute
    // accent) to mark stress: `{{t+|ru|приве́т}}`. The accent must be
    // stripped so callers see the orthographic form `привет`.
    let wikitext = "* Russian: {{t+|ru|приве\u{0301}т}}";
    let candidates = extract_translations(wikitext, "ru");
    assert!(
        candidates.iter().any(|c| c.surface == "привет"),
        "expected unaccented привет in {candidates:?}",
    );
}

#[test]
fn extract_translations_ignores_irrelevant_languages() {
    let wikitext = "* French: {{t+|fr|bonjour}}";
    let candidates = extract_translations(wikitext, "ru");
    assert!(candidates.is_empty());
}

#[test]
fn extract_wikitext_decodes_json_escapes_including_unicode() {
    let body = r#"{"parse":{"title":"hello","wikitext":"Hi\nthere é"}}"#;
    let wikitext = extract_wikitext_from_parse_response(body).expect("should decode");
    assert_eq!(wikitext, "Hi\nthere é");
}

#[test]
fn extract_wikitext_decodes_surrogate_pair_for_supplementary_plane() {
    // U+1D11E (musical symbol G clef) — must be encoded as a UTF-16 pair.
    let body = "{\"wikitext\":\"\\uD834\\uDD1E\"}";
    let wikitext = extract_wikitext_from_parse_response(body).expect("should decode");
    assert_eq!(wikitext, "\u{1D11E}");
}
