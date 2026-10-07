//! Issue #1163 (E128): the internet as formal knowledge.
//!
//! The generic page formalizer turns any fetched resource into a
//! `LinkNetwork` the solver can reason over. The tests pin the contract
//! requirement by requirement:
//! - R1/R2: HTML, Markdown, JSON, plain-text, and PDF-text payloads
//!   formalize into networks whose code blocks carry correct language tags
//!   (fence annotation, HTML class prefix, heuristic, and the `unknown`
//!   fallback for an unrecognized fence),
//! - R3: the generic formalizer yields every statement the bespoke
//!   extractors yield on the same bytes (generic ⊇ bespoke),
//! - R6/R9: a page is stored under a key encoding URL and SHA-256, and its
//!   network carries the rediscovery procedure,
//! - R7: the seed-driven trust score ranks sources without excluding any,
//! - R8: the page-store queries answer with links, not strings.
//!
//! All bytes come from committed fixtures served by a fake transport, so no
//! test can reach the network; the online resolution of a need from the
//! live kotlinlang.org page is the gated `--online` run named in the issue.

#![cfg(feature = "meta-language")]

use std::sync::atomic::{AtomicUsize, Ordering};

use formal_ai::source_fetch::{CachedSourceClient, FetchError, SourceCapture, SourceTransport};
use formal_ai::web_formalize::{
    self, FormalizedPage, FormalizedPageStore, TrustFeatures, covers_statements, formalize_page,
    page_key, trust_score,
};

static TEMP_IDS: AtomicUsize = AtomicUsize::new(0);

const KOTLINLANG_URL: &str = "https://kotlinlang.org/docs/command-line.html";
const RUST_BOOK_URL: &str = "https://doc.rust-lang.org/book/ch01-02-hello-world.html";
const SCALA_DOCS_URL: &str = "https://docs.scala-lang.org/getting-started.html";

const KOTLINLANG_PAGE: &str = include_str!("../fixtures/issue-1163/command-line.html");
const RUST_BOOK_PAGE: &str = include_str!("../fixtures/issue-1163/rust-book.md");
const SCALA_DOCS_PAGE: &str = include_str!("../fixtures/issue-1163/scala-docs.html");

/// A wiktionary-shaped payload, the shape `wiktionary_entry_v1` reads.
const WIKTIONARY_PAYLOAD: &str = r#"[
  {
    "word": "variance",
    "meanings": [
      {
        "partOfSpeech": "noun",
        "definitions": [
          { "definition": "The fact of varying; a departure from a standard." }
        ]
      }
    ]
  }
]"#;

/// Serves the committed fixture shapes and refuses everything else, so no
/// test can silently reach the network.
#[derive(Clone, Copy, Default)]
struct FixtureTransport;

impl SourceTransport for FixtureTransport {
    fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
        match url {
            KOTLINLANG_URL => Ok(KOTLINLANG_PAGE.as_bytes().to_vec()),
            RUST_BOOK_URL => Ok(RUST_BOOK_PAGE.as_bytes().to_vec()),
            SCALA_DOCS_URL => Ok(SCALA_DOCS_PAGE.as_bytes().to_vec()),
            other => Err(FetchError::Transport(format!("fixture_missing:{other}"))),
        }
    }
}

fn temp_cache(label: &str) -> std::path::PathBuf {
    let id = TEMP_IDS.fetch_add(1, Ordering::SeqCst);
    std::env::temp_dir().join(format!("issue-1163-{label}-{id}"))
}

fn fetch(url: &str, label: &str) -> SourceCapture {
    CachedSourceClient::new(temp_cache(label), FixtureTransport)
        .with_online(true)
        .fetch(url)
        .expect("fixture fetch")
}

fn kotlin_page() -> FormalizedPage {
    FormalizedPage::from_capture(
        &fetch(KOTLINLANG_URL, "kotlin"),
        "how to compile a Kotlin program from the command line",
        1,
        trust_score(&TrustFeatures {
            official_site: true,
            https: true,
            primacy: Some("self_published".to_owned()),
            open_license: true,
            agreement_pages: 2,
        }),
        Some("text/html"),
    )
}

/// R1/R2: an HTML page formalizes into a network whose code block carries
/// the language from its `language-` class and the command from its text.
#[test]
fn html_page_formalizes_structure_and_code() {
    let page = kotlin_page();
    let statements = web_formalize::network_statements(&page.network);
    assert!(
        statements
            .iter()
            .any(|statement| statement == "Command-line compiler")
    );
    assert!(
        statements
            .iter()
            .any(|statement| statement.contains("Kotlin runtime"))
    );
    let code = page
        .code_blocks()
        .iter()
        .find(|block| block.language == "kotlin")
        .expect("kotlin code block");
    assert!(
        code.text
            .contains("kotlinc hello.kt -include-runtime -d hello.jar")
    );
    // The table and list formalize too (R1: tables and lists).
    assert!(statements.iter().any(|statement| statement
        == "-include-runtime | Include the Kotlin runtime into the resulting jar"));
    assert!(
        statements
            .iter()
            .any(|statement| statement.contains("JDK 8 or higher"))
    );
}

/// R2: a fence annotation tags the language; an unrecognized fence falls
/// back to `unknown`, never to an empty tag.
#[test]
fn markdown_fences_tag_languages_with_unknown_fallback() {
    let network = formalize_page(RUST_BOOK_PAGE.as_bytes(), Some("text/markdown"));
    let statements = web_formalize::network_statements(&network);
    assert!(
        statements
            .iter()
            .any(|statement| statement.contains("println!(\"Hello, world!\")"))
    );

    let (_, nodes) = web_formalize::formalize_page_with_context(
        RUST_BOOK_PAGE.as_bytes(),
        Some("text/markdown"),
        None,
    );
    let languages: Vec<&str> = nodes
        .iter()
        .filter_map(|(_, block)| match block {
            web_formalize::PageBlock::CodeBlock { language, .. } => Some(language.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(languages, vec!["rust", "shell"]);

    let bare_fence = b"```\nplain text block\n```";
    let (_, nodes) =
        web_formalize::formalize_page_with_context(bare_fence, Some("text/markdown"), None);
    let language = nodes
        .iter()
        .find_map(|(_, block)| match block {
            web_formalize::PageBlock::CodeBlock { language, .. } => Some(language.clone()),
            _ => None,
        })
        .expect("code block");
    assert_eq!(language, "unknown");
}

/// R2: the class-prefix source resolves a GitHub-style `highlight
/// highlight-source-scala` class to `scala`.
#[test]
fn scala_page_tags_language_through_class_prefix() {
    let page = FormalizedPage::from_capture(
        &fetch(SCALA_DOCS_URL, "scala"),
        "scala hello world",
        1,
        trust_score(&TrustFeatures {
            https: true,
            ..TrustFeatures::default()
        }),
        Some("text/html"),
    );
    let code = page
        .code_blocks()
        .iter()
        .find(|block| block.language == "scala")
        .expect("scala code block");
    assert!(code.text.contains("println(\"Hello, world!\")"));
}

/// R3: the generic formalizer yields every statement the wiktionary
/// bespoke extractor yields on the same bytes (generic ⊇ bespoke), and a
/// statement the payload does not carry is honestly uncovered.
#[test]
fn generic_formalizer_covers_bespoke_statements() {
    let network = formalize_page(WIKTIONARY_PAYLOAD.as_bytes(), Some("application/json"));
    let bespoke = vec![
        "The fact of varying; a departure from a standard.".to_owned(),
        "noun".to_owned(),
    ];
    assert!(covers_statements(&network, &bespoke));
    assert!(!covers_statements(
        &network,
        &["the second moment of a distribution about its mean".to_owned()]
    ));
}

/// R6: the store keys pages by URL and SHA-256, so the same bytes reuse the
/// record while changed bytes make a new one.
#[test]
fn store_keys_pages_by_url_and_hash() {
    assert_eq!(
        page_key("https://example.com/a", "cafe01"),
        "page:cafe01:https://example.com/a"
    );
    let mut store = FormalizedPageStore::new();
    let first = kotlin_page();
    let key = store.insert(first.clone());
    store.insert(first.clone());
    assert_eq!(store.len(), 1);
    assert!(store.get(&key).is_some());
    assert!(store.contains_url(KOTLINLANG_URL));
    let mut changed = first;
    changed.sha256 = format!("{}0", changed.sha256);
    let second_key = store.insert(changed);
    assert_eq!(store.len(), 2);
    assert_ne!(key, second_key);
}

/// R7: the seed's weights rank sources and never exclude one -- every
/// scored page stays in range, official pages outrank citations, and a
/// low-trust page still scores above nothing.
#[test]
fn trust_score_ranks_without_excluding() {
    let trusted = trust_score(&TrustFeatures {
        official_site: true,
        https: true,
        primacy: Some("first_hand_record".to_owned()),
        open_license: true,
        agreement_pages: 3,
    });
    let cited = trust_score(&TrustFeatures {
        primacy: Some("citation".to_owned()),
        ..TrustFeatures::default()
    });
    assert_eq!(trusted, 100);
    assert!(cited < trusted);
    assert!(cited > 0, "a citation still scores, it only ranks later");
}

/// R8: "code blocks on kotlinlang.org whose text contains -d" and "the
/// command in the paragraph that mentions compiling" both answer with
/// links, not strings.
#[test]
fn page_queries_answer_with_links() {
    let mut store = FormalizedPageStore::new();
    store.insert(kotlin_page());
    let blocks = store.code_blocks_on("kotlinlang.org", "-d");
    assert!(!blocks.is_empty(), "code_blocks_on returns link values");
    let commands = store.command_mentioning("compil");
    assert!(
        !commands.is_empty(),
        "command_mentioning returns link values"
    );
}

/// R9: the network carries the rediscovery procedure -- query, rank, URL,
/// hash, timestamp -- so the record can be dropped and recovered.
#[test]
fn rediscovery_procedure_is_recorded() {
    let page = kotlin_page();
    let statements = web_formalize::network_statements(&page.network);
    assert!(
        statements
            .iter()
            .any(|statement| statement == KOTLINLANG_URL)
    );
    assert!(statements.iter().any(|statement| statement == &page.sha256));
    assert!(
        statements.iter().any(|statement| {
            statement == "how to compile a Kotlin program from the command line"
        })
    );
    assert_eq!(page.rank, 1);
}

/// R1: plain text and PDF-text payloads formalize as paragraphs.
#[test]
fn plain_and_pdf_text_formalize() {
    let network = formalize_page(b"first paragraph\n\nsecond paragraph", Some("text/plain"));
    let statements = web_formalize::network_statements(&network);
    assert!(
        statements
            .iter()
            .any(|statement| statement == "first paragraph")
    );
    let pdf = formalize_page(
        b"extracted pdf line one\n\nextracted pdf line two",
        Some("application/pdf"),
    );
    assert!(
        web_formalize::network_statements(&pdf)
            .iter()
            .any(|statement| statement == "extracted pdf line one")
    );
}
