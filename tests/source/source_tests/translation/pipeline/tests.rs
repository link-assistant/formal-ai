use super::*;
use std::sync::Mutex;

struct StubHttp {
    responses: Mutex<std::collections::HashMap<String, String>>,
}

impl StubHttp {
    fn new(pairs: &[(&str, &str)]) -> Self {
        Self {
            responses: Mutex::new(
                pairs
                    .iter()
                    .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
                    .collect(),
            ),
        }
    }
}

impl HttpClient for StubHttp {
    fn get(&self, url: &str) -> Result<String, HttpError> {
        self.responses
            .lock()
            .unwrap()
            .get(url)
            .cloned()
            .ok_or_else(|| HttpError::Status {
                status: 404,
                body: format!("no stubbed response for {url}"),
            })
    }
}

#[test]
fn normalize_page_title_strips_terminal_punctuation() {
    assert_eq!(normalize_page_title("Hello!"), "hello");
    assert_eq!(normalize_page_title("как у тебя дела?"), "как у тебя дела");
    assert_eq!(normalize_page_title("你好？"), "你好");
}

#[test]
fn normalize_page_title_lowercases_first_letter() {
    assert_eq!(normalize_page_title("Hello"), "hello");
    assert_eq!(normalize_page_title("Как дела"), "как дела");
}

#[test]
fn translate_identity_returns_self_with_identity_provenance() {
    let http = StubHttp::new(&[]);
    let pipeline = TranslationPipeline::new(&http);
    let translation = pipeline.translate("hello", "en", "en").unwrap();
    assert_eq!(translation.primary_surface(), Some("hello"));
    assert!(translation
        .provenance
        .iter()
        .any(|entry| entry == "identity"));
    assert_eq!(translation.meaning.slug(), "wiktionary:en:hello");
}

#[test]
fn translate_identity_upgrades_to_wikidata_meaning_when_available() {
    let search_url = "https://www.wikidata.org/w/api.php?action=wbsearchentities&search=hello&language=en&type=lexeme&format=json&uselang=en&limit=5";
    let http = StubHttp::new(&[(search_url, r#"{"search":[{"id":"L8885","label":"hello"}]}"#)]);
    let pipeline = TranslationPipeline::new(&http);

    let translation = pipeline.translate("hello", "en", "en").unwrap();

    assert_eq!(translation.primary_surface(), Some("hello"));
    assert_eq!(translation.meaning.slug(), "wikidata-sense:L8885");
    assert!(translation
        .provenance
        .iter()
        .any(|entry| entry == "wikidata:lexeme:L8885"));
}

#[test]
fn wikidata_upgrade_canonicalizes_target_english_meaning() {
    let russian_search_url = "https://www.wikidata.org/w/api.php?action=wbsearchentities&search=%D0%BF%D1%80%D0%B8%D0%B2%D0%B5%D1%82&language=ru&type=lexeme&format=json&uselang=ru&limit=5";
    let english_search_url = "https://www.wikidata.org/w/api.php?action=wbsearchentities&search=hello&language=en&type=lexeme&format=json&uselang=en&limit=5";
    let http = StubHttp::new(&[
        (
            russian_search_url,
            r#"{"search":[{"id":"L150880","label":"привет"}]}"#,
        ),
        (
            english_search_url,
            r#"{"search":[{"id":"L8885","label":"hello"}]}"#,
        ),
    ]);
    let mut candidates = vec![WiktionaryCandidate {
        surface: "hello".to_owned(),
        qualifier: None,
    }];
    let mut provenance = Vec::new();

    let meaning = upgrade_meaning_via_wikidata(
        &http,
        "привет",
        "ru",
        "en",
        &mut provenance,
        &mut candidates,
    )
    .expect("wikidata search should produce a meaning");

    assert_eq!(meaning.slug(), "wikidata-sense:L8885");
    assert!(provenance
        .iter()
        .any(|entry| entry == "wikidata:lexeme:L150880"));
    assert!(provenance
        .iter()
        .any(|entry| entry == "wikidata:canonical_lexeme:L8885"));
}

#[test]
fn translate_uses_source_edition_translation_table() {
    // English Wiktionary returns a JSON envelope around wikitext;
    // the wikitext lists the Russian translation under `{{t+|ru|...}}`.
    // Use a placeholder lemma (`blargh`) that is *not* in the offline
    // dictionary so the pipeline reaches the HTTP stage and we can
    // verify the wikitext parser end-to-end.
    let url = "https://en.wiktionary.org/w/api.php?action=parse&page=blargh&prop=wikitext&formatversion=2&format=json&redirects=1";
    let wikitext = r#"{"parse":{"title":"blargh","wikitext":"* Russian: {{t+|ru|бларг}}\n"}}"#;
    let http = StubHttp::new(&[(url, wikitext)]);
    let pipeline = TranslationPipeline::new(&http);
    let translation = pipeline.translate("blargh", "en", "ru").unwrap();
    assert_eq!(translation.primary_surface(), Some("бларг"));
    assert!(
        translation
            .provenance
            .iter()
            .any(|p| p.starts_with("wiktionary:en:blargh#translations->ru")),
        "got provenance: {:?}",
        translation.provenance,
    );
}

#[test]
fn translate_returns_translation_with_empty_candidates_when_nothing_matches() {
    // No HTTP stubs => every fetch fails. The pipeline still
    // produces a Translation, but with an empty candidates list
    // (callers detect the gap explicitly).
    let http = StubHttp::new(&[]);
    let pipeline = TranslationPipeline::new(&http);
    let translation = pipeline.translate("xyzzy", "en", "ru").unwrap();
    assert!(translation.candidates.is_empty());
    assert!(translation.primary_surface().is_none());
    assert!(translation.provenance.iter().any(|p| p.contains("error")));
}

#[test]
fn translate_uses_compositional_ru_en_fallback_for_short_phrases() {
    let http = StubHttp::new(&[]);
    let pipeline = TranslationPipeline::new(&http);

    let noun_phrase = pipeline.translate("доброе яблоко", "ru", "en").unwrap();
    assert_eq!(noun_phrase.primary_surface(), Some("Good apple"));
    assert!(noun_phrase
        .provenance
        .iter()
        .any(|p| p == "compositional:ru->en:доброе яблоко"));

    let question_phrase = pipeline.translate("что это такое?", "ru", "en").unwrap();
    assert_eq!(question_phrase.primary_surface(), Some("What is this?"));
    assert!(question_phrase
        .provenance
        .iter()
        .any(|p| p == "compositional:ru->en:что это такое"));
}

#[test]
fn translate_prefers_unqualified_candidate() {
    // Use a placeholder lemma not present in the offline dictionary so
    // the pipeline reaches the wikitext-parsing stage.
    let url = "https://en.wiktionary.org/w/api.php?action=parse&page=blargh&prop=wikitext&formatversion=2&format=json&redirects=1";
    let wikitext = r#"{"parse":{"wikitext":"* Russian: {{t|ru|здравствуйте|q=formal}}, {{t+|ru|привет|q=informal}}, {{t|ru|здорово}}\n"}}"#;
    let http = StubHttp::new(&[(url, wikitext)]);
    let pipeline = TranslationPipeline::new(&http);
    let translation = pipeline.translate("blargh", "en", "ru").unwrap();
    // The first unqualified candidate wins.
    assert_eq!(translation.primary_surface(), Some("здорово"));
}
