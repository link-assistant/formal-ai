//! R1188-U20: "List the requirements of this issue: <text>" is answered in
//! chat, in every seeded language, with a localized intro and exactly the
//! requirements the extractor returns, one per line.
//!
//! The route is data: the `requirement_listing` rule set of
//! `data/seed/handler-rules.lino` reads the `requirement_listing_action` role
//! in the command head and captures `value requirements transform
//! requirement_list`, the generic text-transform value source over the
//! request's free-text payload. Twin of
//! `rust/tests/web/requirement-listing-route.test.mjs`, which pins the same
//! answers through the browser worker.

use formal_ai::FormalAiEngine;

/// Prompts and the exact answers both runtimes give them.
const ANSWERS: [(&str, &str); 5] = [
    (
        "List the requirements of this issue: The page must load in under a second. \
         Add a progress bar to the upload form. The cache works well.",
        "Requirements stated in the text:\n\
         - The page must load in under a second.\n\
         - Add a progress bar to the upload form.",
    ),
    (
        "Перечисли требования этой задачи: Страница должна загружаться быстрее секунды. \
         Добавь индикатор загрузки. Кэш работает хорошо.",
        "Требования, изложенные в тексте:\n\
         - Страница должна загружаться быстрее секунды.\n\
         - Добавь индикатор загрузки.",
    ),
    (
        "列出这个问题的需求：页面必须在一秒内加载。添加上传进度条。缓存运行良好。",
        "文本中提出的需求：\n- 页面必须在一秒内加载。\n- 添加上传进度条。",
    ),
    (
        "List the requirements of this issue\n\
         ## Acceptance criteria\n\
         - The upload resumes after a reload\n\
         - [ ] Errors are shown in the user language",
        "Requirements stated in the text:\n\
         - The upload resumes after a reload\n\
         - Errors are shown in the user language",
    ),
    (
        "Extract the requirements from \"The page must load quickly. Add a progress bar.\"",
        "Requirements stated in the text:\n- The page must load quickly.\n- Add a progress bar.",
    ),
];

#[test]
fn a_requirement_listing_request_answers_the_extracted_requirements_one_per_line() {
    for (prompt, expected) in ANSWERS {
        let response = FormalAiEngine.answer(prompt);
        assert_eq!(response.intent, "requirement_listing", "{prompt}");
        assert_eq!(response.answer, expected, "{prompt}");
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| { link.starts_with("text_transform:text_transform_") }),
            "{prompt}: transform provenance was lost"
        );
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link.contains("text_transform") && link.contains("requirement_list")),
            "{prompt}: {:?}",
            response.evidence_links
        );
    }
}

#[test]
fn without_a_payload_or_a_stated_requirement_the_rule_declines() {
    for prompt in [
        "List the requirements of this issue",
        "List the requirements of this issue: The cache was added last year. It works well.",
    ] {
        let response = FormalAiEngine.answer(prompt);
        assert_ne!(response.intent, "requirement_listing", "{prompt}");
    }
}

#[test]
fn a_listing_phrase_inside_the_pasted_text_never_routes() {
    let response = FormalAiEngine.answer(
        "Summarize: We need to list the requirements before the meeting. \
         The team met on Monday. Everyone agreed on the plan.",
    );
    assert_eq!(response.intent, "summarization_free_text");
}
