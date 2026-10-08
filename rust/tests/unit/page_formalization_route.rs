//! R1188-U18: "formalize <url>" and "formalize this page: <url>" are answered
//! in chat, in every seeded language, with the statements of the fetched page.
//!
//! The page is fetched through the URL-fetch path's cached source client, here
//! over a fixture transport that serves the Moon page of
//! `data/benchmarks/web-formalization/en.lino`, so no test touches the
//! network. rust/tests/web/page-formalization-route.test.mjs pins the same
//! answers in the browser worker.

use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use formal_ai::{
    CachedSourceClient, EventLog, FetchError, SourceTransport, try_page_formalization_with_client,
};

const URL_UNDER_TEST: &str = "https://en.wikipedia.org/wiki/Moon";

/// The bare request for the page under test.
const URL_PROMPT: &str = "formalize https://en.wikipedia.org/wiki/Moon";

static TEMP_IDS: AtomicUsize = AtomicUsize::new(0);

/// The paragraphs of the fixture page titled `title`, as the corpus stores them.
fn fixture_paragraphs(title: &str) -> Vec<String> {
    let corpus = include_str!("../../../data/benchmarks/web-formalization/en.lino");
    let heading = format!("  title \"{title}\"");
    corpus
        .lines()
        .skip_while(|line| *line != heading)
        .skip(1)
        .take_while(|line| line.starts_with("  "))
        .filter_map(|line| line.strip_prefix("  paragraph \"")?.strip_suffix('"'))
        .map(str::to_owned)
        .collect()
}

/// Serves the fixture page for its URL and refuses every other one.
struct FixtureTransport;

impl SourceTransport for FixtureTransport {
    fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
        if url != URL_UNDER_TEST {
            return Err(FetchError::HttpStatus {
                url: url.to_owned(),
                status: 404,
            });
        }
        let paragraphs: String = fixture_paragraphs("Moon")
            .iter()
            .map(|text| format!("<p>{text}</p>"))
            .collect();
        Ok(format!("<html><body><h1>Moon</h1>{paragraphs}</body></html>").into_bytes())
    }
}

/// An empty cache directory of its own.
fn temp_cache() -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "formal-ai-page-formalization-{}-{}",
        std::process::id(),
        TEMP_IDS.fetch_add(1, Ordering::SeqCst)
    ));
    let _ = std::fs::remove_dir_all(&path);
    path
}

/// The answer to `prompt`, fetched online through the fixture transport.
fn answer(prompt: &str, online: bool) -> formal_ai::SymbolicAnswer {
    let client = CachedSourceClient::new(temp_cache(), FixtureTransport).with_online(online);
    let mut log = EventLog::new();
    try_page_formalization_with_client(prompt, &mut log, &client)
        .unwrap_or_else(|| panic!("the route should answer: {prompt}"))
}

/// The first statement line both runtimes give the Moon page.
const FIRST_STATEMENT: &str = "- The Moon is the only natural satellite of Earth. → (moon \
     wikidata_property_instance_of unknown:only unknown:natural unknown:satellite name:earth)";

/// Prompts in every seeded language, the first line of the answer and its last.
const SUMMARIES: &[(&str, &str, &str)] = &[
    (
        "formalize this page: https://en.wikipedia.org/wiki/Moon",
        "Formalized `https://en.wikipedia.org/wiki/Moon`: 25 sentences became 67 statements. 3 \
         sentences are fully formal; 197 of 423 terms have no meaning yet; 61 of 62 facts survive \
         the round trip back to text.",
        "… and 27 more statements.",
    ),
    (
        "формализуй эту страницу: https://en.wikipedia.org/wiki/Moon",
        "Страница `https://en.wikipedia.org/wiki/Moon` формализована: из 25 предложений получено \
         67 утверждений. Полностью формальны предложений: 3; терминов без значения: 197 из 423; \
         фактов, переживших обратный перевод в текст: 61 из 62.",
        "… и ещё утверждений: 27.",
    ),
    (
        "इस पेज को औपचारिक बनाओ: https://en.wikipedia.org/wiki/Moon",
        "`https://en.wikipedia.org/wiki/Moon` को औपचारिक बनाया गया: 25 वाक्यों से 67 कथन बने। 3 \
         वाक्य पूरी तरह औपचारिक हैं; 423 में से 197 पदों का अभी कोई अर्थ नहीं है; 62 में से 61 \
         तथ्य पाठ में वापस बदलने पर बचे रहते हैं।",
        "… और 27 कथन।",
    ),
    (
        "形式化这个网页 https://en.wikipedia.org/wiki/Moon",
        "已形式化 `https://en.wikipedia.org/wiki/Moon`：25 个句子变为 67 条陈述。3 \
         个句子完全形式化；423 个术语中有 197 个尚无含义；62 个事实中有 61 个在转回文本后保留。",
        "……另有 27 条陈述。",
    ),
    (
        "formaliza esta página: https://en.wikipedia.org/wiki/Moon",
        "Página `https://en.wikipedia.org/wiki/Moon` formalizada: 25 oraciones se convirtieron en \
         67 enunciados. Oraciones totalmente formales: 3; términos aún sin significado: 197 de \
         423; hechos que sobreviven a la vuelta al texto: 61 de 62.",
        "… y 27 enunciados más.",
    ),
];

#[test]
fn formalize_url_answers_the_statements_of_the_fetched_page_in_every_language() {
    for (prompt, summary, more) in SUMMARIES {
        let answer = answer(prompt, true);
        assert_eq!(answer.intent, "page_formalization", "{prompt}");
        let lines: Vec<&str> = answer.answer.split('\n').collect();
        assert_eq!(lines[0], *summary, "{prompt}");
        assert_eq!(lines[1], "", "{prompt}");
        assert_eq!(lines[2], FIRST_STATEMENT, "{prompt}");
        assert_eq!(lines.len(), 43, "{prompt}");
        assert_eq!(lines.last().copied(), Some(*more), "{prompt}");
    }
}

#[test]
fn the_bare_form_formalizes_too() {
    let answer = answer(URL_PROMPT, true);
    assert_eq!(answer.intent, "page_formalization");
    assert_eq!(answer.answer.split('\n').nth(2), Some(FIRST_STATEMENT));
}

#[test]
fn a_refused_fetch_says_so_and_formalizes_nothing() {
    let answer = answer("formalize https://example.org/missing", true);
    assert_eq!(
        answer.answer,
        "I could not fetch `https://example.org/missing` to formalize it: source answered HTTP \
         404 for https://example.org/missing."
    );
}

#[test]
fn offline_with_no_cached_capture_the_answer_says_nothing_was_fetched() {
    let answer = answer(URL_PROMPT, false);
    assert_eq!(
        answer.answer,
        "I cannot formalize `https://en.wikipedia.org/wiki/Moon`: the page is not in the source \
         cache and live fetching is off, so nothing was fetched."
    );
}

#[test]
fn offline_a_cached_capture_is_formalized() {
    let cache = temp_cache();
    let online = CachedSourceClient::new(&cache, FixtureTransport).with_online(true);
    let mut log = EventLog::new();
    try_page_formalization_with_client(URL_PROMPT, &mut log, &online);
    let offline = CachedSourceClient::new(&cache, FixtureTransport).with_online(false);
    let mut log = EventLog::new();
    let answer = try_page_formalization_with_client(URL_PROMPT, &mut log, &offline)
        .expect("the cached page is formalized offline");
    assert_eq!(answer.answer.split('\n').nth(2), Some(FIRST_STATEMENT));
}

#[test]
fn a_request_without_a_url_or_a_formalize_cue_is_not_this_route() {
    let client = CachedSourceClient::new(temp_cache(), FixtureTransport).with_online(true);
    for prompt in [
        "formalize every man is mortal",
        "fetch https://en.wikipedia.org/wiki/Moon",
    ] {
        let mut log = EventLog::new();
        assert!(
            try_page_formalization_with_client(prompt, &mut log, &client).is_none(),
            "{prompt}"
        );
    }
}
