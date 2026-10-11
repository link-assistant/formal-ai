//! Issues #1163 R14 and #1164 R12: the formalizer and the decomposer over
//! byte-for-byte captures of the real documentation pages.
//!
//! The captures under `rust/tests/fixtures/coding-discovery/captured/` were
//! fetched on 2026-10-07 from kotlinlang.org, the Rust Book (its rendered page
//! and its Markdown source), docs.scala-lang.org, go.dev and the Swift book's
//! Markdown source, and are pinned here by the SHA-256 they were recorded
//! with. Each page's code blocks carry the language tags the page itself
//! declares -- a `language-` class, kotlinlang's `data-lang` attribute, a
//! Markdown fence, or the class of the enclosing container -- and every Hello
//! World page yields its seed output call and printed literal. The browser
//! twin is `rust/tests/web/issue-1163-1164-captured-pages.test.mjs`.

#![cfg(feature = "meta-language")]

use std::sync::atomic::{AtomicUsize, Ordering};

use formal_ai::code_example_knowledge::{CodePartKind, decompose_code_node};
use formal_ai::event_log::EventLog;
use formal_ai::sha256_hex;
use formal_ai::source_fetch::{
    CachedSourceClient, CurlSourceTransport, FetchError, SourceCapture, SourceTransport,
};
use formal_ai::web_formalize::{
    FormalizedPage, FormalizedPageStore, PageBlock, formalize_page_with_context,
};

static TEMP_IDS: AtomicUsize = AtomicUsize::new(0);

const KOTLINLANG_URL: &str = "https://kotlinlang.org/docs/command-line.html";
const KOTLIN_QUERY: &str = "how to compile a Kotlin program from the command line";
const KOTLINC_COMMAND: &str = "kotlinc hello.kt -include-runtime -d hello.jar";

const KOTLIN_COMMAND_LINE: &[u8] =
    include_bytes!("../fixtures/coding-discovery/captured/issue-1163/command-line.html");
const RUST_BOOK_SOURCE: &[u8] =
    include_bytes!("../fixtures/coding-discovery/captured/issue-1163/ch01-02-hello-world.md");
const SCALA_HELLO: &[u8] =
    include_bytes!("../fixtures/coding-discovery/captured/issue-1163/taste-hello-world.html");
const KOTLIN_TOUR: &[u8] =
    include_bytes!("../fixtures/coding-discovery/captured/issue-1164/kotlin-tour-hello-world.html");
const RUST_BOOK_PAGE: &[u8] =
    include_bytes!("../fixtures/coding-discovery/captured/issue-1164/ch01-02-hello-world.html");
const GO_GETTING_STARTED: &[u8] =
    include_bytes!("../fixtures/coding-discovery/captured/issue-1164/getting-started.html");
const SWIFT_TOUR: &[u8] =
    include_bytes!("../fixtures/coding-discovery/captured/issue-1164/GuidedTour.md");

const HTML: &str = "text/html";
const MARKDOWN: &str = "text/markdown";

/// The code blocks of a captured page, as (language, text) pairs.
fn code_blocks(bytes: &[u8], mime: &str) -> Vec<(String, String)> {
    formalize_page_with_context(bytes, Some(mime), None)
        .1
        .into_iter()
        .filter_map(|(_, block)| match block {
            PageBlock::CodeBlock { language, text } => Some((language, text)),
            _ => None,
        })
        .collect()
}

/// Every capture is the recorded bytes and keeps the page's language tags.
#[test]
fn captures_are_the_recorded_bytes_with_the_pages_language_tags() {
    let cases: [(&[u8], &str, &str, Vec<&str>); 6] = [
        (
            KOTLIN_COMMAND_LINE,
            HTML,
            "0847a47edc7136be89ddb103aed1178c4eb95a0ba03bdfe396f1b207ab806e8e",
            vec![
                "bash", "bash", "bash", "kotlin", "bash", "bash", "bash", "bash", "bash", "kotlin",
                "bash", "bash",
            ],
        ),
        (
            RUST_BOOK_SOURCE,
            MARKDOWN,
            "645189892417cf5431fafb29c4e9b413db2c8cb07fb99e993efaf3c3e9d7a143",
            vec![
                "console",
                "cmd",
                "rust",
                "console",
                "powershell",
                "rust",
                "rust",
                "console",
                "console",
                "cmd",
                "console",
            ],
        ),
        (
            SCALA_HELLO,
            HTML,
            "913485460cbb046dab070e46eca56fc97cc3f1bc43fb57ae931a2f711145ef2b",
            vec![
                "scala",
                "scala",
                "bash",
                "plaintext",
                "scala",
                "scala",
                "scala",
                "bash",
                "bash",
                "scala",
            ],
        ),
        (
            KOTLIN_TOUR,
            HTML,
            "916f03525029483ef10ca64b75b7c110c538aadce2c7021dbb6a56e8ef8bd92a",
            vec!["kotlin", "kotlin", "kotlin", "kotlin", "kotlin"],
        ),
        (
            RUST_BOOK_PAGE,
            HTML,
            "98d3e31d5c41a6572a3536ee16c51a1058b3974e6f22e15ba5fb2a5e964ceb4e",
            vec![
                "console",
                "cmd",
                "rust",
                "console",
                "powershell",
                "rust",
                "rust",
                "console",
                "console",
                "cmd",
                "console",
            ],
        ),
        (
            GO_GETTING_STARTED,
            HTML,
            "a7048d2cce6a94ae835c9503d6af5e96f88954a459d831fe03f9a7d82b60ef2c",
            vec!["unknown"; 10],
        ),
    ];
    for (bytes, mime, sha256, languages) in cases {
        assert_eq!(sha256_hex(bytes), sha256);
        let tags: Vec<String> = code_blocks(bytes, mime)
            .into_iter()
            .map(|(language, _)| language)
            .collect();
        assert_eq!(tags, languages, "{sha256}");
    }
}

/// R1163-14: kotlinlang's numbered steps keep the compile and run commands,
/// each a `data-lang` block inside a list item that is stepped into.
#[test]
fn kotlinlang_steps_keep_the_compile_and_run_commands() {
    let texts: Vec<String> = code_blocks(KOTLIN_COMMAND_LINE, HTML)
        .into_iter()
        .map(|(_, text)| text.trim().to_owned())
        .collect();
    assert!(
        texts
            .iter()
            .any(|text| text == "kotlinc hello.kt -include-runtime -d hello.jar")
    );
    assert!(texts.iter().any(|text| text == "java -jar hello.jar"));
}

/// One captured page: bytes, mime, language, output call, printed literal,
/// and whether the page tags its example block.
type CapturedHelloWorld = (
    &'static [u8],
    &'static str,
    &'static str,
    &'static str,
    &'static str,
    bool,
);

/// R1164-12: every captured Hello World page decomposes into its seed output
/// call and printed literal. The page's example is its first block, tagged
/// with the language or untagged (go.dev declares no tag), that decomposes
/// into an output operation.
#[test]
fn every_captured_hello_world_page_decomposes() {
    let cases: [CapturedHelloWorld; 6] = [
        (
            KOTLIN_COMMAND_LINE,
            HTML,
            "kotlin",
            "println",
            "Hello, World!",
            true,
        ),
        (
            KOTLIN_TOUR,
            HTML,
            "kotlin",
            "println",
            "Hello, world!",
            true,
        ),
        (
            RUST_BOOK_PAGE,
            HTML,
            "rust",
            "println!",
            "Hello, world!",
            true,
        ),
        (
            GO_GETTING_STARTED,
            HTML,
            "go",
            "fmt.Println",
            "Hello, World!",
            true,
        ),
        (
            SWIFT_TOUR,
            MARKDOWN,
            "swift",
            "print",
            "Hello, world!",
            false,
        ),
        (SCALA_HELLO, HTML, "scala", "println", "Hello, World!", true),
    ];
    for (bytes, mime, language, call, literal, entry_point) in cases {
        let node = code_blocks(bytes, mime)
            .into_iter()
            .filter(|(tag, _)| tag == language || tag == "unknown")
            .find_map(|(_, text)| {
                decompose_code_node(&text, language, &[])
                    .ok()
                    .filter(|node| {
                        node.parts
                            .iter()
                            .any(|part| part.kind == CodePartKind::OutputOperation)
                    })
            })
            .unwrap_or_else(|| panic!("{language}: the captured page decomposes"));
        let has = |kind: CodePartKind, text: &str| {
            node.parts
                .iter()
                .any(|part| part.kind == kind && part.source_text == text)
        };
        assert!(has(CodePartKind::OutputOperation, call), "{language} call");
        assert!(
            has(CodePartKind::StringLiteral, literal),
            "{language} literal"
        );
        assert_eq!(
            node.parts
                .iter()
                .any(|part| part.kind == CodePartKind::EntryPoint),
            entry_point,
            "{language} entry point"
        );
    }
}

/// Serves the kotlinlang capture and refuses everything else.
#[derive(Clone, Copy)]
struct CapturedTransport;

impl SourceTransport for CapturedTransport {
    fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
        if url == KOTLINLANG_URL {
            Ok(KOTLIN_COMMAND_LINE.to_vec())
        } else {
            Err(FetchError::Transport(format!("fixture_missing:{url}")))
        }
    }
}

fn temp_cache(label: &str) -> std::path::PathBuf {
    let id = TEMP_IDS.fetch_add(1, Ordering::SeqCst);
    let path = std::env::temp_dir().join(format!("issue-1163-captured-{label}-{id}"));
    let _ = std::fs::remove_dir_all(&path);
    path
}

/// The command the page query answers on the formalized kotlinlang page: the
/// code block the paragraph mentioning `Compile` introduces.
fn compile_commands(capture: &SourceCapture) -> Vec<String> {
    let mut store = FormalizedPageStore::new();
    store.insert(FormalizedPage::from_capture(
        capture,
        KOTLIN_QUERY,
        1,
        0,
        Some("text/html"),
    ));
    let page = store.page_for_url(KOTLINLANG_URL).expect("stored page");
    store
        .command_mentioning("Compile")
        .iter()
        .filter_map(|link| page.block_text(link))
        .map(|text| text.trim().to_owned())
        .collect()
}

/// R1163-11, offline half: on the captured kotlinlang page the paragraph that
/// mentions compiling introduces exactly the `kotlinc` command.
#[test]
fn captured_kotlinlang_page_answers_the_compile_command() {
    let capture = CachedSourceClient::new(temp_cache("offline"), CapturedTransport)
        .with_online(true)
        .fetch(KOTLINLANG_URL)
        .expect("captured page");
    assert_eq!(compile_commands(&capture), vec![KOTLINC_COMMAND.to_owned()]);
}

fn live_fetch_requested() -> bool {
    matches!(
        std::env::var("FORMAL_AI_LIVE_FETCH")
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase()
            .as_str(),
        "1" | "true" | "yes" | "on"
    )
}

/// R1163-11: fetch kotlinlang.org live, formalize it, and resolve the compile
/// command with the page URL and the SHA-256 of the fetched bytes in the
/// trace. Gated by `FORMAL_AI_LIVE_FETCH`, so the normal suite stays offline.
#[test]
fn live_kotlinlang_compile_command_with_url_and_hash() {
    if !live_fetch_requested() {
        eprintln!("skipped: set FORMAL_AI_LIVE_FETCH=1 to fetch kotlinlang.org");
        return;
    }
    let capture = CachedSourceClient::new(temp_cache("live"), CurlSourceTransport)
        .with_online(true)
        .fetch(KOTLINLANG_URL)
        .expect("kotlinlang.org answers");
    let mut log = EventLog::new();
    capture.record(&mut log);
    let trace = log
        .events()
        .iter()
        .find(|event| event.kind == "source:http")
        .map(|event| event.payload.clone())
        .expect("the capture is traced");
    assert!(trace.starts_with(KOTLINLANG_URL), "{trace}");
    assert!(
        trace.contains(&format!("sha256={}", sha256_hex(capture.bytes()))),
        "{trace}"
    );
    assert_eq!(compile_commands(&capture), vec![KOTLINC_COMMAND.to_owned()]);
}

/// R1165-1: a code block keeps its indentation while prose still collapses.
///
/// The shared indentation and the blank lines around a block are the page's
/// markup, a highlighter span around leading spaces is indentation, and the
/// captured Hello World examples keep the indentation their pages show.
#[test]
fn a_code_block_keeps_its_indentation() {
    let rust: Vec<String> = code_blocks(RUST_BOOK_PAGE, HTML)
        .into_iter()
        .map(|(_, text)| text)
        .collect();
    assert!(
        rust.iter()
            .any(|text| text == "fn main() {\n    println!(\"Hello, world!\");\n}"),
        "{rust:?}"
    );
    let kotlin: Vec<String> = code_blocks(KOTLIN_COMMAND_LINE, HTML)
        .into_iter()
        .map(|(_, text)| text)
        .collect();
    assert!(
        kotlin
            .iter()
            .any(|text| text == "fun main() {\n    println(\"Hello, World!\")\n}"),
        "{kotlin:?}"
    );
    let page = "<p>Some   spaced\n   prose</p><pre><code>\n\n      if x {\n<span>      </span>    y()\n      }\n\n</code></pre>";
    let texts: Vec<String> = formalize_page_with_context(page.as_bytes(), Some(HTML), None)
        .1
        .into_iter()
        .filter_map(|(_, block)| match block {
            PageBlock::Paragraph { text, .. } | PageBlock::CodeBlock { text, .. } => Some(text),
            _ => None,
        })
        .collect();
    assert_eq!(texts, ["Some spaced prose", "if x {\n    y()\n}"]);
}
