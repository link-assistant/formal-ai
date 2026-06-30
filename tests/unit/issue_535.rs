use std::collections::{BTreeMap, BTreeSet};

use formal_ai::FormalAiEngine;

const ISSUE_535_REPORTED_PROMPT: &str = "Проверь данный текст на уникальность и на плагиат\n\n\
Attached files:\n\
1. variation-tech-model-manual.txt (text/plain, 160.3 KB)";

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
