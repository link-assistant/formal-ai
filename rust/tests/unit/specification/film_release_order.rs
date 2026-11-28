//! Release-order questions read a verified publisher capture (issues #462 and #892).

use formal_ai::SymbolicAnswer;
use formal_ai::event_log::EventLog;
use formal_ai::source_fetch::{CachedSourceClient, FetchError, SourceTransport};
use formal_ai::try_fact_lookup_with_client;
use formal_ai::web_engine_core::normalize_prompt;
use std::sync::atomic::{AtomicUsize, Ordering};

const REPORTED_PROMPT: &str = "Перечисли фильмы про человека-паука в порядке выхода на экран?";
const SPIDER_MAN: &str = "List Spider-Man films in release order.";
const HARRY_POTTER: &str = "List Harry Potter films in release order.";
const SPIDER_EN: &str = include_str!("../../fixtures/release-timeline/spider-man.en.json");
const SPIDER_RU: &str = include_str!("../../fixtures/release-timeline/spider-man.ru.json");
const POTTER_SEARCH: &str =
    include_str!("../../fixtures/release-timeline/harry-potter-search.en.json");
const POTTER_EN: &str = include_str!("../../fixtures/release-timeline/harry-potter.en.json");

struct Publisher<'a>(&'a AtomicUsize);

impl SourceTransport for Publisher<'_> {
    fn get(&self, source: &str) -> Result<Vec<u8>, FetchError> {
        let endpoint = url::Url::parse(source).expect("source URL");
        let field = |name| {
            endpoint
                .query_pairs()
                .find_map(|(field, value)| (field == name).then(|| value.into_owned()))
                .unwrap_or_default()
        };
        let capture = match endpoint.host_str() {
            Some("www.wikidata.org") if field("search") == "harry potter film series" => {
                POTTER_SEARCH
            }
            Some("query-main.wikidata.org") => {
                let query = field("query");
                assert!(query.contains("LIMIT 101"), "{query}");
                assert_eq!(field("format"), "json");
                if query.contains("BIND(wd:Q216930") {
                    POTTER_EN
                } else if query.contains("BIND(wd:Q2307877") && query.contains("= 'ru'") {
                    SPIDER_RU
                } else if query.contains("BIND(wd:Q2307877") && query.contains("= 'en'") {
                    SPIDER_EN
                } else {
                    panic!("unexpected timeline query: {query}");
                }
            }
            _ => {
                return Err(FetchError::HttpStatus {
                    url: source.to_owned(),
                    status: 404,
                });
            }
        };
        self.0.fetch_add(1, Ordering::Relaxed);
        Ok(capture.as_bytes().to_vec())
    }
}

fn ask(prompt: &str, client: &CachedSourceClient<Publisher<'_>>) -> Option<SymbolicAnswer> {
    try_fact_lookup_with_client(
        prompt,
        &normalize_prompt(prompt),
        &mut EventLog::new(),
        client,
    )
}

#[test]
fn registered_and_open_release_requests_require_publisher_captures() {
    let cache =
        std::env::temp_dir().join(format!("formal-ai-release-order-{}", std::process::id()));
    let calls = AtomicUsize::new(0);
    let offline = CachedSourceClient::new(&cache, Publisher(&calls));
    assert!(ask(SPIDER_MAN, &offline).is_none());
    assert!(ask(HARRY_POTTER, &offline).is_none());
    assert_eq!(calls.load(Ordering::Relaxed), 0);

    let online = CachedSourceClient::new(&cache, Publisher(&calls)).with_online(true);
    for (prompt, identifier, capture, ordered_start) in [
        (
            SPIDER_MAN,
            "Q2307877",
            SPIDER_EN,
            "1. Spider-Man (2002); 2. Spider-Man 2 (2004)",
        ),
        (
            REPORTED_PROMPT,
            "Q2307877",
            SPIDER_RU,
            "1. Человек-паук (2002); 2. Человек-паук 2 (2004)",
        ),
        (
            "Назови фильмы о человеке-пауке по порядку выхода.",
            "Q2307877",
            SPIDER_RU,
            "1. Человек-паук (2002); 2. Человек-паук 2 (2004)",
        ),
        (
            "Перечисли фильмы про человека паука в порядке выхода.",
            "Q2307877",
            SPIDER_RU,
            "1. Человек-паук (2002); 2. Человек-паук 2 (2004)",
        ),
        (
            HARRY_POTTER,
            "Q216930",
            POTTER_EN,
            "1. Harry Potter and the Philosopher's Stone (2001); 2. Harry Potter and the Chamber of Secrets (2002)",
        ),
    ] {
        let answer = ask(prompt, &online).unwrap_or_else(|| panic!("unanswered: {prompt}"));
        assert_eq!(answer.intent, "fact_lookup", "{prompt}");
        assert!(answer.answer.contains(ordered_start), "{}", answer.answer);
        if prompt == HARRY_POTTER {
            let (released, _) = answer.answer.split_once(" Source: ").unwrap();
            assert_eq!(
                released,
                "Released Harry Potter films in release order: \
                1. Harry Potter and the Philosopher's Stone (2001); \
                2. Harry Potter and the Chamber of Secrets (2002); \
                3. Harry Potter and the Prisoner of Azkaban (2004); \
                4. Harry Potter and the Goblet of Fire (2005); \
                5. Harry Potter and the Order of the Phoenix (2007); \
                6. Harry Potter and the Half-Blood Prince (2009); \
                7. Harry Potter and the Deathly Hallows – Part 1 (2010); \
                8. Harry Potter and the Deathly Hallows – Part 2 (2011)."
            );
        }
        assert!(
            answer
                .evidence_links
                .contains(&format!("wikidata:{identifier}")),
            "{:?}",
            answer.evidence_links
        );
        assert!(
            answer.evidence_links.contains(&format!(
                "release_timeline:sha256:{}",
                formal_ai::sha256_hex(capture.as_bytes())
            )),
            "{:?}",
            answer.evidence_links
        );
        let replay = ask(prompt, &offline).expect("offline replay");
        assert_eq!(replay.answer, answer.answer);
    }
    assert_eq!(calls.load(Ordering::Relaxed), 4);
    std::fs::remove_dir_all(cache).expect("remove temporary captures");
}
