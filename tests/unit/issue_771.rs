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

/// GitHub rejects an issue body longer than this many *characters* — not bytes,
/// which matters here because the reported session is in Russian and every
/// Cyrillic character costs two bytes. The composed report must stay under it no
/// matter how large the transcribed conversation is.
const GITHUB_BODY_LIMIT: usize = 65_536;

/// The length GitHub measures a body by.
fn body_length(body: &str) -> usize {
    body.chars().count()
}

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

/// The `gh issue create --body` argument the planner would run, unquoted.
fn reported_body(messages: &[ChatMessage]) -> String {
    let calls = tool_calls(messages);
    let command = arguments(&calls[0])["command"]
        .as_str()
        .expect("command string")
        .to_owned();
    let body_flag = command
        .find("--body '")
        .expect("report command should carry a --body argument");
    let quoted = &command[body_flag + "--body '".len()..];
    let quoted = quoted
        .strip_suffix('\'')
        .expect("body argument should be single-quoted");
    quoted.replace("'\\''", "'")
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

// Requirement 2: every transcribed turn stays inside its own attributed block.
mod report_format {
    use super::*;

    fn body_for(user: &str, assistant: &str) -> String {
        let messages = vec![
            ChatMessage::user(user),
            ChatMessage::assistant(assistant),
            ChatMessage::user("report"),
        ];
        reported_body(&messages)
    }

    #[test]
    fn a_multiline_turn_does_not_escape_its_block() {
        // The reported body inlined this after `- **assistant:** `, so every line
        // but the first rendered as top-level markdown.
        let body = body_for(
            "В каких странах есть частные космические компании?",
            "# Обзор\n\nСписок:\n- SpaceX\n- Blue Origin\n\nИтого семь стран.",
        );
        // Everything between the conversation heading and the closing footer is
        // transcript, and none of it may render as top-level markdown.
        let transcript = body
            .split_once("### ")
            .and_then(|(_, rest)| rest.split_once('\n'))
            .map(|(_, rest)| rest)
            .expect("body should carry a conversation heading")
            .rsplit_once("\n\n")
            .map(|(transcript, _footer)| transcript)
            .expect("body should end with the footer");
        for line in transcript.lines() {
            let is_contained = line.trim().is_empty()
                || line.starts_with("**")
                || line.starts_with('>')
                || line.starts_with('_');
            assert!(
                is_contained,
                "every transcript line must stay inside an attributed block, \
                 but this one escaped it: {line:?}\n---\n{body}"
            );
        }
    }

    #[test]
    fn each_turn_is_attributed_to_its_role() {
        let body = body_for("first question\nsecond line", "an answer\nover two lines");
        assert!(body.contains("**user:**"), "{body}");
        assert!(body.contains("**assistant:**"), "{body}");
        // Continuation lines belong to the turn that introduced them.
        assert!(body.contains("> second line"), "{body}");
        assert!(body.contains("> over two lines"), "{body}");
    }

    #[test]
    fn a_huge_transcript_stays_within_the_github_body_limit() {
        let body = body_for("why is this broken?", &scraped_page());
        assert!(
            body_length(&body) < GITHUB_BODY_LIMIT,
            "body must fit GitHub's {GITHUB_BODY_LIMIT} character limit, got {}",
            body_length(&body)
        );
        assert!(
            body.contains("**assistant:**"),
            "the trimmed transcript must still attribute the turn:\n{body}"
        );
    }

    #[test]
    fn an_exhausted_transcript_budget_says_so_instead_of_truncating_silently() {
        // Many large turns cannot all be transcribed. The report must stop at a
        // stated boundary rather than trailing off mid-conversation.
        let mut messages = Vec::new();
        for _ in 0..12 {
            messages.push(ChatMessage::user(scraped_page()));
            messages.push(ChatMessage::assistant(scraped_page()));
        }
        messages.push(ChatMessage::user("report"));
        let body = reported_body(&messages);

        assert!(
            body_length(&body) < GITHUB_BODY_LIMIT,
            "body must stay bounded, got {}",
            body_length(&body)
        );
        assert!(
            body.contains("trimmed to keep this report within GitHub"),
            "an exhausted budget must be stated, not silent:\n{body}"
        );
        assert!(
            body.trim_end()
                .ends_with("Filed automatically by Formal AI in agentic mode."),
            "the footer must still close a trimmed report:\n{body}"
        );
    }

    #[test]
    fn the_intro_heading_and_footer_still_frame_the_transcript() {
        let body = body_for("a question", "an answer");
        assert!(
            body.starts_with("Reported from an agentic session"),
            "{body}"
        );
        assert!(body.contains("### Conversation"), "{body}");
        assert!(
            body.trim_end()
                .ends_with("Filed automatically by Formal AI in agentic mode."),
            "{body}"
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

        let body = reported_body(&messages);
        assert!(
            body_length(&body) < GITHUB_BODY_LIMIT,
            "reported body was {} characters",
            body_length(&body)
        );
        assert!(
            !body.contains("Главное меню"),
            "the scraped navigation chrome must never reach the issue:\n{body}"
        );
        assert!(body.contains("**user:**"), "{body}");
        assert!(body.contains("**assistant:**"), "{body}");
        assert!(
            body.contains("Частные космические компании работают в США"),
            "the issue must show the answer under review:\n{body}"
        );
    }
}
