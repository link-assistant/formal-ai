//! Issue #771 — the agentic session that asked "В каких странах есть частные
//! космические компании?" produced a broken bug report.
//!
//! Two defects compose into the reported symptom, and each gets its own
//! requirement here plus a whole-task test that replays the reported session:
//!
//! 1. **The research answer dumped the whole fetched page.**
//!    `web_research::final_answer` returned the fetch tool result verbatim, so the
//!    assistant turn was the scraped site chrome (menus, navigation, every
//!    paragraph) instead of an answer to the question. The expected shape is a
//!    short, query-relevant extract that still cites the source URL.
//! 2. **The issue body's markdown was broken.** `report_issue::compose_report`
//!    inlined each turn into a `- **role:** {text}` bullet. Any turn containing a
//!    newline — which every real assistant answer does — escaped the list item, so
//!    the fetched page's own headings and lists rendered as top-level issue
//!    content and the role attribution was lost after the first line.
//!
//! Both are asserted through `plan_chat_step`, the same entry point an external
//! agentic CLI drives.

use std::fmt::Write as _;

use formal_ai::agentic_coding::{plan_chat_step, AgenticPlan, PlannedToolCall};
use formal_ai::protocol::{ChatMessage, ToolCall};

const TOOLS: [&str; 5] = ["websearch", "webfetch", "read", "write", "bash"];

fn plan(messages: &[ChatMessage]) -> AgenticPlan {
    plan_chat_step(messages, &TOOLS).expect("planner should recognise the task")
}

fn tool_calls(messages: &[ChatMessage]) -> Vec<PlannedToolCall> {
    match plan(messages) {
        AgenticPlan::ToolCalls(calls) => calls,
        plan @ AgenticPlan::Final(_) => panic!("expected tool calls, got {plan:?}"),
    }
}

fn final_answer(messages: &[ChatMessage]) -> String {
    match plan(messages) {
        AgenticPlan::Final(answer) => answer,
        plan @ AgenticPlan::ToolCalls(_) => panic!("expected final answer, got {plan:?}"),
    }
}

fn arguments(call: &PlannedToolCall) -> serde_json::Value {
    serde_json::from_str(&call.arguments).expect("tool arguments should be JSON")
}

fn answer_tool_call(messages: &mut Vec<ChatMessage>, call: &PlannedToolCall, result: &str) {
    let id = format!("call_{}", messages.len());
    messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
        id.clone(),
        call.tool.clone(),
        call.arguments.clone(),
    )]));
    messages.push(ChatMessage::tool_result(id, &call.tool, result));
}

/// The reported page: a wall of site chrome around one paragraph that actually
/// answers the question. Shaped like the real fetch result in issue #771.
fn scraped_page() -> String {
    let mut page = String::from(
        "ТЕХНОЛОГИИ, ИНЖИНИРИНГ, ИННОВАЦИИ\n\
         Главное меню Перейти к основному содержимому О нас Новости\n\
         Наука Техника Инжиниринг Промышленность Инновации Видео\n\
         Услуги Консалтинг Металлообработка Моделирование Разработки\n\
         Портфолио Продукция Партнеры Университеты НИИ Бизнес Контакты\n\
         Навигация по записям Предыдущая Следующая\n",
    );
    page.push_str(
        "Частные космические компании работают в США, Великобритании, Германии, \
         Франции, Испании, Китае и Индии.\n",
    );
    // Site boilerplate continues for a long while after the answer.
    for index in 0..400 {
        writeln!(
            page,
            "Рубрика {index}: подписывайтесь на нашу рассылку и читайте другие записи блога."
        )
        .expect("writing to a String cannot fail");
    }
    page
}

/// The confirmed `gh issue create` command the planner would run.
fn report_command(messages: &[ChatMessage]) -> String {
    let mut messages = messages.to_vec();
    messages.push(ChatMessage::user("GitHub issue"));
    messages.push(ChatMessage::user("Both logs"));
    let calls = tool_calls(&messages);
    let command = arguments(&calls[0])["command"]
        .as_str()
        .expect("command string")
        .to_owned();
    command
}

// Requirement 1: the research answer is a query-relevant extract, not a page dump.
mod research_answer {
    use super::*;

    fn answer_for_reported_question() -> String {
        let mut messages = vec![ChatMessage::user(
            "В каких странах есть частные космические компании?",
        )];
        let search = tool_calls(&messages).remove(0);
        answer_tool_call(
            &mut messages,
            &search,
            "Частные космические компании за рубежом \
             https://integral-russia.ru/2026/03/06/chastnye-kosmicheskie-kompanii/",
        );
        let fetch = tool_calls(&messages).remove(0);
        answer_tool_call(&mut messages, &fetch, &scraped_page());
        final_answer(&messages)
    }

    #[test]
    fn does_not_dump_the_whole_fetched_page() {
        let page = scraped_page();
        let answer = answer_for_reported_question();
        assert!(
            answer.len() < page.len() / 4,
            "answer must extract, not dump the {} byte page; got {} bytes:\n{answer}",
            page.len(),
            answer.len()
        );
        assert!(
            !answer.contains("Главное меню"),
            "site navigation chrome must not survive into the answer:\n{answer}"
        );
        assert!(
            !answer.contains("подписывайтесь на нашу рассылку"),
            "trailing site boilerplate must not survive into the answer:\n{answer}"
        );
    }

    #[test]
    fn keeps_the_sentence_that_answers_the_question_and_cites_the_source() {
        let answer = answer_for_reported_question();
        assert!(
            answer.contains("Частные космические компании работают в США"),
            "the answering sentence must survive extraction:\n{answer}"
        );
        assert!(
            answer
                .contains("https://integral-russia.ru/2026/03/06/chastnye-kosmicheskie-kompanii/"),
            "the answer must cite the fetched source:\n{answer}"
        );
    }

    #[test]
    fn a_short_page_is_still_answered_verbatim() {
        // Extraction must not damage the already-concise case covered by #687.
        let mut messages = vec![ChatMessage::user("When are the next elections in the USA?")];
        let search = tool_calls(&messages).remove(0);
        answer_tool_call(
            &mut messages,
            &search,
            "Election day: https://www.usa.gov/election-day",
        );
        let fetch = tool_calls(&messages).remove(0);
        answer_tool_call(
            &mut messages,
            &fetch,
            "The next US general election is on November 3, 2026.",
        );
        let answer = final_answer(&messages);
        assert!(answer.contains("November 3, 2026"), "{answer}");
        assert!(
            answer.contains("https://www.usa.gov/election-day"),
            "{answer}"
        );
    }
}

// Requirement 1, in every supported language. Extraction depends on splitting a
// page into sentences and ranking them against the query, and both steps are
// script-sensitive: Hindi ends its sentences with a danda rather than a full
// stop, and Chinese writes without spaces so word tokenization never overlaps.
// A page dump in one language is the same defect as issue #771 in another.
mod multilingual_extraction {
    use super::*;

    /// One reported session per language: a question, a page whose chrome and
    /// trailing boilerplate bury the one sentence that answers it, and the
    /// phrase the extract must keep.
    struct Case {
        language: &'static str,
        query: &'static str,
        chrome: &'static str,
        answer: &'static str,
        boilerplate: &'static str,
        expected: &'static str,
    }

    const CASES: [Case; 4] = [
        Case {
            language: "en",
            query: "What countries have private space companies?",
            chrome: "TECHNOLOGY, ENGINEERING, INNOVATION\n\
                     Main menu Skip to content About us News\n",
            answer: "Private space companies operate in the USA, the United Kingdom and India.\n",
            boilerplate: "Subscribe to our newsletter and read other blog posts.",
            expected: "Private space companies operate in the USA",
        },
        Case {
            language: "ru",
            query: "В каких странах есть частные космические компании?",
            chrome: "ТЕХНОЛОГИИ, ИНЖИНИРИНГ, ИННОВАЦИИ\n\
                     Главное меню Перейти к основному содержимому О нас Новости\n",
            answer: "Частные космические компании работают в США, Великобритании и Индии.\n",
            boilerplate: "Подписывайтесь на нашу рассылку и читайте другие записи блога.",
            expected: "Частные космические компании работают в США",
        },
        Case {
            language: "hi",
            query: "किन देशों में निजी अंतरिक्ष कंपनियाँ हैं?",
            chrome: "प्रौद्योगिकी, इंजीनियरिंग, नवाचार\n\
                     मुख्य मेनू हमारे बारे में समाचार।\n",
            answer: "निजी अंतरिक्ष कंपनियाँ अमेरिका, ब्रिटेन और भारत में हैं।\n",
            boilerplate: "सदस्यता लें और अन्य ब्लॉग पोस्ट पढ़ें।",
            expected: "निजी अंतरिक्ष कंपनियाँ अमेरिका",
        },
        Case {
            language: "zh",
            query: "哪些国家有私营航天公司？",
            chrome: "技术、工程、创新\n主菜单 关于我们 新闻。\n",
            answer: "私营航天公司位于美国、英国和印度。\n",
            boilerplate: "订阅我们的通讯并阅读其他博客文章。",
            expected: "私营航天公司位于美国",
        },
    ];

    /// The same shape as [`scraped_page`], in the case's language.
    fn page(case: &Case) -> String {
        let mut page = String::from(case.chrome);
        page.push_str(case.answer);
        for index in 0..400 {
            writeln!(page, "{index}: {}", case.boilerplate)
                .expect("writing to a String cannot fail");
        }
        page
    }

    fn answer_for(case: &Case) -> String {
        let mut messages = vec![ChatMessage::user(case.query)];
        let search = tool_calls(&messages).remove(0);
        answer_tool_call(&mut messages, &search, "https://example.invalid/space");
        let fetch = tool_calls(&messages).remove(0);
        answer_tool_call(&mut messages, &fetch, &page(case));
        final_answer(&messages)
    }

    #[test]
    fn every_supported_language_gets_an_extract_and_not_a_page_dump() {
        for case in &CASES {
            let language = case.language;
            let page = page(case);
            let answer = answer_for(case);
            assert!(
                answer.chars().count() < page.chars().count() / 4,
                "[{language}] answer must extract, not dump the {} character page; \
                 got {}:\n{answer}",
                page.chars().count(),
                answer.chars().count()
            );
            assert!(
                answer.contains(case.expected),
                "[{language}] the answering sentence must survive extraction:\n{answer}"
            );
            assert!(
                !answer.contains(case.boilerplate),
                "[{language}] trailing site boilerplate must not survive:\n{answer}"
            );
        }
    }
}

// Requirement 2 after #822: the issue command uploads the canonical complete
// Links Notation context instead of rebuilding a lossy Markdown transcript.
mod report_format {
    use super::*;

    fn command_for(user: &str, assistant: &str) -> String {
        let messages = vec![
            ChatMessage::user(user),
            ChatMessage::assistant(assistant),
            ChatMessage::user("report"),
        ];
        report_command(&messages)
    }

    #[test]
    fn a_multiline_turn_is_not_interpolated_into_the_shell_command() {
        let command = command_for(
            "В каких странах есть частные космические компании?",
            "# Обзор\n\nСписок:\n- SpaceX\n- Blue Origin\n\nИтого семь стран.",
        );
        assert!(!command.contains("# Обзор"), "{command}");
        assert!(command.contains("curl -fsS"), "{command}");
        assert!(command.contains("--body-file"), "{command}");
    }

    #[test]
    fn complete_context_is_requested_in_links_notation() {
        let command = command_for("first question", "an answer");
        assert!(command.contains("include=both"), "{command}");
        assert!(
            command.contains("formal-ai-report.XXXXXX.lino"),
            "{command}"
        );
        assert!(command.contains("```lino"), "{command}");
    }

    #[test]
    fn a_huge_transcript_does_not_grow_the_command() {
        let short = command_for("why?", "brief");
        let huge = command_for("why?", &scraped_page());
        assert_eq!(short, huge, "conversation bytes belong in the context file");
    }

    #[test]
    fn oversized_context_links_the_full_file_and_keeps_a_bounded_latest_preview() {
        let mut messages = Vec::new();
        for _ in 0..12 {
            messages.push(ChatMessage::user(scraped_page()));
            messages.push(ChatMessage::assistant(scraped_page()));
        }
        messages.push(ChatMessage::user("report"));
        let command = report_command(&messages);
        assert!(command.contains("wc -c"), "{command}");
        assert!(command.contains("gh gist create"), "{command}");
        assert!(command.contains("tail -c 12000"), "{command}");
        assert!(command.contains("sed '1d'"), "{command}");
        assert!(!command.contains("head -c 12000"), "{command}");
        assert!(
            command.contains("complete Links Notation context"),
            "{command}"
        );
    }

    #[test]
    fn the_intro_and_complete_context_heading_frame_the_upload() {
        let command = command_for("a question", "an answer");
        assert!(
            command.contains("Reported from an agentic session"),
            "{command}"
        );
        assert!(
            command.contains("### Complete agentic context"),
            "{command}"
        );
    }
}

// The whole task: replay the reported session end to end.
mod reported_session {
    use super::*;

    #[test]
    fn the_reported_session_files_a_readable_issue() {
        let mut messages = vec![ChatMessage::user(
            "В каких странах есть частные космические компании?",
        )];
        let search = tool_calls(&messages).remove(0);
        answer_tool_call(
            &mut messages,
            &search,
            "Частные космические компании за рубежом \
             https://integral-russia.ru/2026/03/06/chastnye-kosmicheskie-kompanii/",
        );
        let fetch = tool_calls(&messages).remove(0);
        answer_tool_call(&mut messages, &fetch, &scraped_page());
        let answer = final_answer(&messages);
        messages.push(ChatMessage::assistant(answer));
        messages.push(ChatMessage::user("report"));

        let command = report_command(&messages);
        assert!(!command.contains("Главное меню"), "{command}");
        assert!(command.contains("include=both"), "{command}");
        assert!(command.contains("gh issue create"), "{command}");
        assert!(command.contains("--body-file"), "{command}");
    }
}
