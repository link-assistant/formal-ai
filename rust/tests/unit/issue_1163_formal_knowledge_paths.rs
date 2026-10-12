//! Issue #1163 (E128): the paths that put the generic page formalizer to work.
//!
//! - R3: on each bespoke extractor's own fixture bytes, the generic
//!   formalizer carries every statement the bespoke reader yields,
//! - R4: a need no registry row declares is satisfied through web research
//!   by the registry's own lookup, the path the need-satisfaction loop calls,
//! - R6: the research fetch path consults working memory before fetching,
//! - R7: the trust features are computed from the page, the registry, the
//!   stored pages and a Wikidata P856 claim, not supplied by the caller,
//! - R10: a page query over a page the prompt supplies,
//! - R13: `document_formats` reads HTML and Markdown through the formalizer.
//!
//! Every byte comes from a committed fixture served by a fake transport.

#![cfg(feature = "meta-language")]

use std::sync::atomic::{AtomicUsize, Ordering};

use formal_ai::concept_lookup::{LookupOutcome, RegistrySourceLookup, extractor_statements};
use formal_ai::formalization::needs::satisfy_needs;
use formal_ai::how_to_guide::ServicePreferences;
use formal_ai::memory_query_language::run_page_query;
use formal_ai::needs::{Need, NeedKind, NeedState};
use formal_ai::service_accessibility::ServiceAccessibilityCache;
use formal_ai::source_fetch::{CachedSourceClient, FetchError, SourceTransport};
use formal_ai::source_walk::{LookupBounds, SourceLookup};
use formal_ai::web_formalize::{
    FormalizedPage, FormalizedPageStore, TrustFeatures, covers_statements, formalize_page,
    official_websites, page_key, split_supplied_page, trust_features_for, trust_score,
    with_working_memory,
};
use formal_ai::{execute_source_research, formalize_document_source};

static TEMP_IDS: AtomicUsize = AtomicUsize::new(0);

const KOTLINLANG_URL: &str = "https://kotlinlang.org/docs/command-line.html";
const MIRROR_URL: &str = "https://mirror.example.org/kotlin/command-line.html";
const PYTHON_DOCS_URL: &str = "https://docs.python.org/3.12/library/functions.html";
const DUCKDUCKGO_API: &str = "https://api.duckduckgo.com/";

const KOTLINLANG_PAGE: &str = include_str!("../fixtures/issue-1163/command-line.html");
const RUST_BOOK_PAGE: &str = include_str!("../fixtures/issue-1163/rust-book.md");
const WORDNET_PAYLOAD: &str = include_str!("../fixtures/issue-1163/wordnet_sense_v1/variance.json");
const MEDIAWIKI_PAYLOAD: &str =
    include_str!("../fixtures/issue-1163/mediawiki_summary_v1/variance.json");
const WIKIDATA_PAYLOAD: &str =
    include_str!("../fixtures/issue-1163/wikidata_entity_v1/kotlin.json");
const OEIS_PAYLOAD: &str = include_str!("../fixtures/issue-1163/oeis_sequence_v1/fibonacci.json");
const PYTHON_DOCS_PAGE: &str = include_str!("../fixtures/issue-1163/python_docs_v1/functions.html");

/// Answers every search with the kotlinlang page and serves the fixture
/// pages; `pages` false refuses the pages, so only working memory can supply
/// them.
#[derive(Clone, Copy)]
struct SearchTransport {
    pages: bool,
}

impl SourceTransport for SearchTransport {
    fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
        if url.starts_with(DUCKDUCKGO_API) {
            return Ok(format!(
                r#"{{"AbstractURL":"{KOTLINLANG_URL}","Heading":"Kotlin command-line compiler","AbstractText":"Compile Kotlin programs from the command line.","RelatedTopics":[]}}"#
            )
            .into_bytes());
        }
        match url {
            KOTLINLANG_URL | MIRROR_URL if self.pages => Ok(KOTLINLANG_PAGE.as_bytes().to_vec()),
            other => Err(FetchError::Transport(format!("fixture_missing:{other}"))),
        }
    }
}

fn temp_cache(label: &str) -> std::path::PathBuf {
    let id = TEMP_IDS.fetch_add(1, Ordering::SeqCst);
    let path = std::env::temp_dir().join(format!("issue-1163-paths-{label}-{id}"));
    // A cache left by an earlier run would answer in place of working memory.
    let _ = std::fs::remove_dir_all(&path);
    path
}

fn client(label: &str, pages: bool) -> CachedSourceClient<SearchTransport> {
    CachedSourceClient::new(temp_cache(label), SearchTransport { pages }).with_online(true)
}

/// R3: each bespoke extractor's statements on its fixture, pinned exactly,
/// and the generic formalizer's network over the same bytes covers them.
#[test]
fn generic_formalizer_covers_every_bespoke_extractor() {
    let cases: [(&str, &str, &str, &str, Vec<&str>); 5] = [
        (
            "wordnet_sense_v1",
            WORDNET_PAYLOAD,
            "application/json",
            "variance",
            vec![
                "the quality of being subject to variation",
                "an official document that permits something not normally allowed",
            ],
        ),
        (
            "mediawiki_summary_v1",
            MEDIAWIKI_PAYLOAD,
            "application/json",
            "variance",
            vec![
                "In probability theory and statistics, variance is the expected value of the squared deviation from the mean of a random variable.",
            ],
        ),
        (
            "wikidata_entity_v1",
            WIKIDATA_PAYLOAD,
            "application/json",
            "Kotlin",
            vec!["general-purpose programming language"],
        ),
        (
            "oeis_sequence_v1",
            OEIS_PAYLOAD,
            "application/json",
            "fibonacci",
            vec![
                "Fibonacci numbers: F(n) = F(n-1) + F(n-2) with F(0) = 0 and F(1) = 1.",
                "0,1,1,2,3,5,8,13,21,34,55,89,144,233,377,610",
                "Also called Lamé's sequence.",
                "F(n+2) = number of binary sequences of length n that have no consecutive 0's.",
            ],
        ),
        (
            "python_docs_v1",
            PYTHON_DOCS_PAGE,
            "text/html",
            "sorted",
            vec![
                "sorted(iterable, /, *, key=None, reverse=False)",
                "Return a new sorted list from the items in iterable.",
                "sum(iterable, /, start=0)",
                "Sums start and the items of an iterable from left to right and returns the total.",
            ],
        ),
    ];
    for (extractor, payload, mime, surface, expected) in cases {
        let bespoke = extractor_statements(extractor, payload.as_bytes(), surface);
        assert_eq!(bespoke, expected, "{extractor} bespoke statements");
        let network = formalize_page(payload.as_bytes(), Some(mime));
        assert!(
            covers_statements(&network, &bespoke),
            "{extractor}: the generic network must carry {bespoke:?}"
        );
    }
}

/// R4: a decision need has no registry row, so the registry lookup the
/// need-satisfaction loop asks researches it through web search and reads
/// the captured page through the generic page formalizer's registry row.
#[test]
fn unmatched_need_is_satisfied_through_web_research() {
    let client = client("need", true);
    let preferences = ServicePreferences::default();
    let mut availability = ServiceAccessibilityCache::new(temp_cache("need-availability"));
    let mut lookup = RegistrySourceLookup::new(
        &client,
        &preferences,
        &mut availability,
        LookupBounds::default(),
        "en",
        0,
    );
    let need = Need::raised(NeedKind::Decision, "compiler", "en", "issue-1163-r4");
    let senses = match lookup.lookup(&need, &LookupBounds::default()) {
        LookupOutcome::Found(senses) => senses,
        LookupOutcome::NotFound { consulted } => {
            panic!("an unmatched need must be researched; consulted {consulted:?}")
        }
    };
    let glosses: Vec<&str> = senses.iter().map(|sense| sense.gloss.as_str()).collect();
    assert_eq!(
        glosses,
        vec![
            "The Kotlin compiler can be invoked from the command line.",
            "To compile a Kotlin application with the command-line compiler, use the following command.",
            "The compiler requires JDK 8 or higher.",
        ]
    );
    assert!(
        senses
            .iter()
            .all(|sense| sense.source_id == "web_page" && sense.source_url == KOTLINLANG_URL)
    );
    assert_eq!(lookup.consulted(), vec!["web_page".to_owned()]);

    let satisfied = satisfy_needs(vec![need], &mut lookup, &LookupBounds::default(), 2);
    assert_eq!(satisfied[0].state, NeedState::Satisfied);
    assert_eq!(satisfied[0].satisfied_by, Some(senses[0].content_id()));
}

/// R6: once a page is in working memory, a later research run replays it
/// from memory -- a cache hit with the same digest -- even through a client
/// whose cache is empty and whose transport refuses every page.
#[test]
fn research_consults_working_memory_before_fetching() {
    let first = execute_source_research(&client("memory-first", true), "kotlin compiler", 1)
        .expect("captured research");
    let original = first.pages[0].capture.clone();

    let second = execute_source_research(&client("memory-second", false), "kotlin compiler", 1)
        .expect("research with pages refused");
    assert_eq!(second.failures.len(), 0, "{:?}", second.failures);
    assert_eq!(second.pages.len(), 1);
    let replayed = &second.pages[0].capture;
    assert!(replayed.cached());
    assert_eq!(replayed.sha256(), original.sha256());
    assert_eq!(replayed.fetched_at(), original.fetched_at());
    assert_eq!(replayed.bytes(), original.bytes());
}

/// R7: the features come from the page and what is known about it. The
/// kotlinlang page is HTTPS, has no registered source on its domain and no
/// license marker; Wikidata's P856 names it the official site; a mirror on
/// another domain stating the same paragraphs is one agreeing page.
#[test]
fn trust_features_are_computed_from_the_page() {
    assert_eq!(
        official_websites(WIKIDATA_PAYLOAD.as_bytes()),
        vec!["https://kotlinlang.org/".to_owned()]
    );
    let mirror = client("mirror", true)
        .fetch(MIRROR_URL)
        .expect("mirror fetch");
    let mut store = FormalizedPageStore::new();
    store.insert(FormalizedPage::from_capture(&mirror, "kotlin", 1, 0, None));
    let features = trust_features_for(
        KOTLINLANG_URL,
        KOTLINLANG_PAGE.as_bytes(),
        &store,
        &official_websites(WIKIDATA_PAYLOAD.as_bytes()),
    );
    assert_eq!(
        features,
        TrustFeatures {
            official_site: true,
            https: true,
            primacy: None,
            open_license: false,
            agreement_pages: 1,
        }
    );
    assert_eq!(trust_score(&features), 48);

    let python = trust_features_for(
        PYTHON_DOCS_URL,
        PYTHON_DOCS_PAGE.as_bytes(),
        &FormalizedPageStore::new(),
        &[],
    );
    assert_eq!(python.primacy, Some("self_published".to_owned()));
    let licensed = trust_features_for(
        "http://example.org/notes",
        b"<p>Text is available under https://creativecommons.org/licenses/by-sa/4.0/ terms.</p>",
        &FormalizedPageStore::new(),
        &[],
    );
    assert_eq!(
        licensed,
        TrustFeatures {
            open_license: true,
            ..TrustFeatures::default()
        }
    );

    // The research path stores each page under its computed score: HTTPS
    // alone, 0.10 of the seed's weights.
    let research = execute_source_research(&client("scored", true), "kotlin compiler", 1)
        .expect("captured research");
    let key = page_key(KOTLINLANG_URL, research.pages[0].capture.sha256());
    let stored = with_working_memory(|memory| memory.get(&key).map(|page| page.trust));
    assert_eq!(stored, Some(10));
    assert_eq!(research.pages_by_trust().len(), 1);
}

/// R10: the first line is the page query, the rest is the page; the
/// command tied to the paragraph that mentions the phrase answers.
#[test]
fn supplied_page_query_answers_from_the_page() {
    let prompt = "The command in the paragraph that mentions compiler:\n\nThe compiler turns hello.kt into a runnable jar with this line.\n\nkotlinc hello.kt -include-runtime -d hello.jar\n\nThe launcher starts that jar with this line.\n\njava -jar hello.jar";
    let (query, page_text) = split_supplied_page(prompt).expect("a query and a page");
    assert_eq!(query, "The command in the paragraph that mentions compiler");
    let page = FormalizedPage::from_supplied_text(&page_text, &query);
    let mut store = FormalizedPageStore::new();
    let key = store.insert(page);
    let links = run_page_query(&store, &query).expect("the query parses");
    let stored = store.get(&key).expect("the stored page");
    let texts: Vec<&str> = links
        .iter()
        .filter_map(|link| stored.block_text(link))
        .collect();
    assert_eq!(
        texts,
        vec!["kotlinc hello.kt -include-runtime -d hello.jar"]
    );
    assert_eq!(
        split_supplied_page("the command in the paragraph that mentions compiler"),
        None
    );
}

/// R13: HTML and Markdown are document-conversion sources read through the
/// formalizer; a format the seed lists no `document_source` row for is not.
#[test]
fn document_formats_read_html_and_markdown_through_the_formalizer() {
    let markdown = formalize_document_source("Markdown", RUST_BOOK_PAGE).expect("markdown source");
    assert_eq!(markdown.source_format, "markdown");
    assert_eq!(markdown.mime_hint, "text/markdown");
    assert_eq!(markdown.code_languages, vec!["rust", "shell"]);
    let html = formalize_document_source("html", KOTLINLANG_PAGE).expect("html source");
    assert_eq!(html.source_format, "html");
    assert_eq!(html.code_languages, vec!["kotlin"]);
    assert_eq!(
        html.blocks.first(),
        Some(&("heading".to_owned(), "Command-line compiler".to_owned()))
    );
    assert_eq!(
        formalize_document_source("md", "# Title\n\nbody").map(|source| source.blocks),
        Some(vec![
            ("heading".to_owned(), "Title".to_owned()),
            ("paragraph".to_owned(), "body".to_owned()),
        ])
    );
    assert_eq!(formalize_document_source("docx", "body"), None);
}
