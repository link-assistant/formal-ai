//! Issue #1172 R1172-5: "What does X mean?" resolves through the registry's
//! dictionary source (Wiktionary), read from committed captures in
//! `rust/tests/fixtures/issue-1172-definitions/source-cache` (fetched
//! 2026-10-07 through the production URLs). The browser twin is pinned by
//! `rust/tests/web/issue-1172-word-definition.test.mjs` over the same bytes.

use std::path::{Path, PathBuf};

use formal_ai::event_log::EventLog;
use formal_ai::{CachedSourceClient, CurlSourceTransport};

fn fixture_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the repository root sits one level above the crate")
        .join("rust/tests/fixtures/issue-1172-definitions")
}

/// The dictionary answer over the committed captures, offline.
fn definition(prompt: &str) -> Option<String> {
    let client = CachedSourceClient::new(fixture_dir(), CurlSourceTransport).with_online(false);
    let mut log = EventLog::new();
    formal_ai::try_word_definition_with_client(prompt, &mut log, &client).map(|answer| answer.answer)
}

#[test]
fn english_definition_is_read_from_the_wiktionary_capture() {
    assert_eq!(
        definition("What does ephemeral mean?").as_deref(),
        Some("ephemeral, as Wiktionary defines it:\n  1. (noun) Something which lasts for a short period of time.\n  2. (adjective) Lasting for a short period of time.\n  3. (adjective) Existing for only one day, as with some flowers, insects, and diseases.\nSource: https://api.dictionaryapi.dev/api/v2/entries/en/ephemeral (sha256 1069f27b1e20a804; CC BY-SA 3.0)")
    );
}

#[test]
fn russian_definition_is_read_from_the_wiktionary_capture() {
    assert_eq!(
        definition("Что означает эфемерный?").as_deref(),
        Some("эфемерный — значения по словарю Wiktionary:\n  1. книжн. скоропреходящий, непрочный, мимолётный, временный\n  2. книжн. мнимый, воображаемый, призрачный\nИсточник: https://ru.wiktionary.org/w/api.php?action=query&format=json&prop=extracts&explaintext=1&titles=%D1%8D%D1%84%D0%B5%D0%BC%D0%B5%D1%80%D0%BD%D1%8B%D0%B9 (sha256 79a720af10980ed5; CC BY-SA 3.0)")
    );
}

#[test]
fn hindi_definition_is_read_from_the_wiktionary_capture() {
    assert_eq!(
        definition("क्षणिक का क्या अर्थ है?").as_deref(),
        Some("क्षणिक — Wiktionary के अनुसार अर्थ:\n  1. क्षणिक ^१ वि॰ [सं॰] एक क्षण रहनेवाला । क्षणभंगुर । अनित्य ।\n  2. क्षणिक ^२ संज्ञा पुं॰ [सं॰] दे॰ 'क्षणिकवाद' ।\nस्रोत: https://hi.wiktionary.org/w/api.php?action=query&format=json&prop=extracts&explaintext=1&titles=%E0%A4%95%E0%A5%8D%E0%A4%B7%E0%A4%A3%E0%A4%BF%E0%A4%95 (sha256 fbf4884f705fb060; CC BY-SA 3.0)")
    );
}

#[test]
fn spanish_definition_is_read_from_the_wiktionary_capture() {
    assert_eq!(
        definition("¿Qué significa efímero?").as_deref(),
        Some("efímero, según Wiktionary:\n  1. Que dura un solo día.\n  2. Que dura poco tiempo.\n  3. Que comienza y acaba rápido, de forma fugaz.\nFuente: https://es.wiktionary.org/w/api.php?action=query&format=json&prop=extracts&explaintext=1&titles=ef%C3%ADmero (sha256 aa9f241595cf51db; CC BY-SA 3.0)")
    );
}

#[test]
fn chinese_frame_reads_the_word_and_a_page_without_senses_falls_through() {
    assert_eq!(
        formal_ai::definition_term("苹果是什么意思？"),
        Some(("苹果".to_owned(), "zh".to_owned()))
    );
    // The captured zh page states no Chinese sense, so the row falls through
    // to the next one instead of inventing a definition.
    assert_eq!(definition("苹果是什么意思？"), None);
}

#[test]
fn an_uncaptured_word_and_a_non_definition_prompt_fall_through() {
    assert_eq!(definition("What does banana mean?"), None);
    assert_eq!(formal_ai::definition_term("What is the capital of France?"), None);
    assert_eq!(
        formal_ai::definition_term("What does the word \"ephemeral\" mean?"),
        Some(("ephemeral".to_owned(), "en".to_owned()))
    );
}
