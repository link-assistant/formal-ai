use std::collections::{BTreeMap, BTreeSet};

use formal_ai::{handle_telegram_webhook, FormalAiEngine};

const ISSUE_535_REPORTED_PROMPT: &str = "Проверь данный текст на уникальность и на плагиат\n\n\
Attached files:\n\
1. variation-tech-model-manual.txt (text/plain, 160.3 KB)";

/// The generalized verification class: comment 4754747438 asks us to cover the
/// *whole class of similar questions* — not only plagiarism/originality, but
/// authenticity, factual accuracy, and veracity checks — in every supported
/// language. Each prompt uses the broadened action/subject cues (verify /
/// authenticity / factual accuracy / достоверность / सत्यता / 真实性) rather than
/// the plagiarism stems, and must still route to the same grounded workflow.
const VERIFICATION_CLASS_CASES: &[(&str, &str, &str)] = &[
    (
        "en",
        "Verify the authenticity and factual accuracy of this attached document\n\n\
Attached files:\n\
1. claim.txt (text/plain, 3.0 KB)",
        "claim.txt",
    ),
    (
        "ru",
        "Проверь достоверность этого приложенного материала\n\n\
Attached files:\n\
1. novost.txt (text/plain, 4.0 KB)",
        "novost.txt",
    ),
    (
        "hi",
        "इस संलग्न दस्तावेज़ की सत्यता जांचें\n\n\
Attached files:\n\
1. lekh.txt (text/plain, 5.0 KB)",
        "lekh.txt",
    ),
    (
        "zh",
        "核实这篇附件文章的真实性\n\n\
Attached files:\n\
1. news.txt (text/plain, 6.0 KB)",
        "news.txt",
    ),
];

const DOCUMENT_ORIGINALITY_CHECK_CASES: &[(&str, &str, &str)] = &[
    (
        "en",
        "Check this attached text for uniqueness and plagiarism\n\n\
Attached files:\n\
1. article.txt (text/plain, 12.0 KB)",
        "article.txt",
    ),
    (
        "ru",
        ISSUE_535_REPORTED_PROMPT,
        "variation-tech-model-manual.txt",
    ),
    (
        "hi",
        "संलग्न पाठ की मौलिकता और plagiarism जांचें\n\n\
Attached files:\n\
1. report.txt (text/plain, 8.0 KB)",
        "report.txt",
    ),
    (
        "zh",
        "检查这个附件文本的原创性和抄袭情况\n\n\
Attached files:\n\
1. manuscript.txt (text/plain, 9.0 KB)",
        "manuscript.txt",
    ),
];

#[test]
fn issue_535_russian_attachment_prompt_keeps_russian_language() {
    assert_eq!(
        formal_ai::detect_language(ISSUE_535_REPORTED_PROMPT).slug(),
        "ru",
        "the Latin file metadata must not make the reported Russian request look English",
    );
}

#[test]
fn issue_535_russian_attachment_originality_request_is_not_unknown() {
    let response = FormalAiEngine.answer(ISSUE_535_REPORTED_PROMPT);

    assert_eq!(
        response.intent, "document_originality_check",
        "reported prompt should route to document_originality_check, got {} with answer {}",
        response.intent, response.answer,
    );
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link == "language:ru"),
        "reported prompt should stay localized as Russian: {:?}",
        response.evidence_links,
    );
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link
                == "document_originality_check:attachment:variation-tech-model-manual.txt"),
        "the attached filename should be preserved as evidence: {:?}",
        response.evidence_links,
    );
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link == "read_local_file:request:variation-tech-model-manual.txt"),
        "document originality checks over attachments should request local file text: {:?}",
        response.evidence_links,
    );
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link == "web_search:query_kind:document_originality_check"),
        "document originality checks must be grounded in web-search evidence: {:?}",
        response.evidence_links,
    );
    assert!(
        response.answer.contains("уникаль") && response.answer.contains("плагиат"),
        "Russian answer should explain the originality/plagiarism workflow, got: {}",
        response.answer,
    );
}

#[test]
fn document_originality_check_cases_cover_every_supported_language() {
    let languages = formal_ai::supported_languages();
    let supported_languages = languages
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let mut case_languages = BTreeMap::<&str, usize>::new();
    for &(language, _, _) in DOCUMENT_ORIGINALITY_CHECK_CASES {
        *case_languages.entry(language).or_insert(0) += 1;
    }
    assert_eq!(
        case_languages.keys().copied().collect::<BTreeSet<_>>(),
        supported_languages,
        "document-originality prompts must cover every supported language",
    );
    assert!(
        case_languages.values().all(|count| *count == 1),
        "document-originality prompts should add one case per supported language: {case_languages:?}",
    );
}

#[test]
fn telegram_document_attachment_routes_to_originality_check() {
    // A Telegram user forwards a file with a Russian caption asking for an
    // originality check — the shared attachment-context builder must fold the
    // document into an `Attached files:` block so the same handler fires.
    let body = r#"{
        "update_id": 1,
        "message": {
            "message_id": 10,
            "chat": {"id": 42, "type": "private"},
            "caption": "Проверь этот файл на уникальность и на плагиат",
            "document": {
                "file_name": "variation-tech-model-manual.txt",
                "mime_type": "text/plain",
                "file_size": 164096
            }
        }
    }"#;

    let reply = handle_telegram_webhook(body)
        .expect("valid webhook body")
        .expect("private message should get a reply");
    assert!(
        reply.text.contains("уникаль") && reply.text.contains("плагиат"),
        "Telegram document attachment should route to the originality workflow: {}",
        reply.text,
    );
    assert!(
        reply.text.contains("variation-tech-model-manual.txt"),
        "the attached filename should survive into the Telegram reply: {}",
        reply.text,
    );
}

#[test]
fn telegram_attachment_only_message_still_reaches_the_solver() {
    // Even with no caption, an attached file must reach the solver rather than
    // hitting the text-only fallback message.
    let body = r#"{
        "update_id": 2,
        "message": {
            "message_id": 11,
            "chat": {"id": 42, "type": "private"},
            "document": {
                "file_name": "notes.txt",
                "mime_type": "text/plain",
                "file_size": 2048
            }
        }
    }"#;

    let reply = handle_telegram_webhook(body)
        .expect("valid webhook body")
        .expect("private message should get a reply");
    assert!(
        !reply
            .text
            .contains("I can only process Telegram text messages"),
        "an attachment-only message must not hit the text-only fallback: {}",
        reply.text,
    );
    assert!(
        reply.text.contains("notes.txt"),
        "the attachment name should reach the solver context: {}",
        reply.text,
    );
}

#[test]
fn document_originality_grounds_each_statement_with_relative_meta_logic() {
    let prompt = "Check this attached text for uniqueness and plagiarism\n\n\
Attached files:\n\
1. article.txt (text/plain, 12.0 KB)\n\
Text excerpt: The tower opened in 1889. It stands 300 metres tall.";
    let response = FormalAiEngine.answer(prompt);

    assert_eq!(response.intent, "document_originality_check");
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link == "statement_verification:statement_count:2"),
        "each statement in the excerpt should be planned for grounding: {:?}",
        response.evidence_links,
    );
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("statement_verification:statement:The tower opened")),
        "individual statements should be recorded for verification: {:?}",
        response.evidence_links,
    );
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("relative_meta_logic:assumed_prior:")),
        "statements should start from an assumed-true relative-meta-logic prior: {:?}",
        response.evidence_links,
    );
    assert!(
        response.evidence_links.iter().any(|link| link
            == "relative_meta_logic:trusted_source_tier:original_first_party:weight=1.000000"),
        "the trusted-source policy must rank original first sources highest: {:?}",
        response.evidence_links,
    );
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link == "relative_meta_logic:ignored_source_tier:unoriginal"),
        "unoriginal reposts must be recorded as ignored: {:?}",
        response.evidence_links,
    );
}

#[test]
fn document_originality_requests_route_to_grounded_attachment_workflow() {
    for &(language, prompt, file_name) in DOCUMENT_ORIGINALITY_CHECK_CASES {
        let response = FormalAiEngine.answer(prompt);

        assert_eq!(
            response.intent, "document_originality_check",
            "{language} originality prompt should route to document_originality_check, got {} with answer {}",
            response.intent, response.answer,
        );
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link == &format!("document_originality_check:attachment:{file_name}")),
            "{language} prompt should preserve the attachment name {file_name:?}: {:?}",
            response.evidence_links,
        );
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link == &format!("read_local_file:request:{file_name}")),
            "{language} prompt should request local file text for the attachment: {:?}",
            response.evidence_links,
        );
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link == "web_search:query_kind:document_originality_check"),
            "{language} prompt should record web-search grounding: {:?}",
            response.evidence_links,
        );
        assert_ne!(response.intent, "unknown");
    }
}

#[test]
fn verification_class_generalizes_beyond_plagiarism_in_every_language() {
    // The class must recognize authenticity / factual-accuracy / veracity
    // requests (not only plagiarism) and ground them the same way.
    let languages = formal_ai::supported_languages();
    let supported_languages = languages
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let case_languages = VERIFICATION_CLASS_CASES
        .iter()
        .map(|&(language, _, _)| language)
        .collect::<BTreeSet<_>>();
    assert_eq!(
        case_languages, supported_languages,
        "the generalized verification class must cover every supported language",
    );

    for &(language, prompt, file_name) in VERIFICATION_CLASS_CASES {
        let response = FormalAiEngine.answer(prompt);

        assert_eq!(
            response.intent, "document_originality_check",
            "{language} authenticity/veracity prompt should route to the grounded \
             verification workflow, got {} with answer {}",
            response.intent, response.answer,
        );
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link == &format!("document_originality_check:attachment:{file_name}")),
            "{language} prompt should preserve the attachment name {file_name:?}: {:?}",
            response.evidence_links,
        );
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link == "web_search:query_kind:document_originality_check"),
            "{language} prompt should record web-search grounding: {:?}",
            response.evidence_links,
        );
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link == &format!("language:{language}")),
            "{language} prompt should stay localized: {:?}",
            response.evidence_links,
        );
    }
}
