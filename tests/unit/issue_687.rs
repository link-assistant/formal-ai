//! Issue #687 — driving Formal AI as an agentic backend (`OpenCode`) must actually
//! *act* on simple natural-language requests instead of falling to the
//! unknown-reasoning blurb.
//!
//! The four prompts from the reported `OpenCode` session each name a capability the
//! agentic planner should exercise through the client's advertised tools:
//!
//! 1. a factual question → a real web search tool call (the harness runs it);
//! 2. "report [this] on GitHub" → a `gh issue create` shell tool call against the
//!    Formal AI repository;
//! 3. "what were we talking about?" → a conversational-recall final answer built
//!    from the message history;
//! 4. "learn about it." → resolve the pronoun from history and research the topic.
//!
//! These assert the *agentic* path (`plan_chat_step`), which is exactly how an
//! external agentic CLI such as `OpenCode` drives the OpenAI-compatible server.

use formal_ai::agentic_coding::{plan_chat_step, AgenticPlan, PlannedToolCall};
use formal_ai::protocol::ChatMessage;

/// The tools a typical agentic CLI (`OpenCode`) advertises.
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

// Requirement 1: factual questions go to the web instead of the unknown blurb.
mod factual_question {
    use super::*;

    #[test]
    fn future_election_question_emits_a_web_search() {
        for prompt in [
            "When are the next elections in the USA?",
            "When next elections in the USA?",
            "What is the current population of Japan?",
        ] {
            let messages = vec![ChatMessage::user(prompt)];
            let calls = tool_calls(&messages);
            assert_eq!(calls.len(), 1, "{prompt:?} should emit one call");
            assert_eq!(calls[0].tool, "websearch", "{prompt:?} should web-search");
            let query = arguments(&calls[0])["query"]
                .as_str()
                .expect("query string")
                .to_lowercase();
            assert!(!query.is_empty(), "{prompt:?} query must not be empty");
        }
    }

    #[test]
    fn election_search_then_fetch_then_answers_from_results() {
        let mut messages = vec![ChatMessage::user("When are the next elections in the USA?")];

        let search = tool_calls(&messages).remove(0);
        assert_eq!(search.tool, "websearch");
        answer_tool_call(
            &mut messages,
            &search,
            "US general elections are scheduled for November 3, 2026. \
             Source: https://www.usa.gov/election-day",
        );

        // With a fetch tool advertised and a URL in the search result, the planner
        // fetches the source before answering.
        let fetch = tool_calls(&messages).remove(0);
        assert_eq!(fetch.tool, "webfetch");
        assert_eq!(
            arguments(&fetch)["url"],
            "https://www.usa.gov/election-day",
            "planner should fetch the URL surfaced by the search"
        );
        answer_tool_call(
            &mut messages,
            &fetch,
            "The next U.S. general election is on Tuesday, November 3, 2026.",
        );

        let answer = final_answer(&messages);
        assert!(
            answer.contains("2026"),
            "final answer must carry the fetched fact, got: {answer}"
        );
    }

    #[test]
    fn research_prefers_an_official_source_and_cites_it() {
        let mut messages = vec![ChatMessage::user("When are the next elections in the USA?")];
        let search = tool_calls(&messages).remove(0);
        answer_tool_call(
            &mut messages,
            &search,
            "Commentary https://example.com/elections Official https://www.usa.gov/election-day",
        );

        let fetch = tool_calls(&messages).remove(0);
        assert_eq!(fetch.tool, "webfetch");
        assert_eq!(arguments(&fetch)["url"], "https://www.usa.gov/election-day");
        answer_tool_call(&mut messages, &fetch, "Election Day is November 3, 2026.");

        let answer = final_answer(&messages);
        assert!(answer.contains("November 3, 2026"), "{answer}");
        assert!(
            answer.contains("https://www.usa.gov/election-day"),
            "{answer}"
        );
    }
}

// Requirement 1 (generalization): a factual/answer-seeking question the symbolic
// engine cannot resolve locally must reach the web in *every* supported language,
// not only English. The reported OpenCode session was English, but the issue asks
// for the behaviour to generalise across all environments and languages.
mod multilingual_web_research {
    use super::*;

    fn web_query(prompt: &str) -> String {
        let messages = vec![ChatMessage::user(prompt)];
        let calls = tool_calls(&messages);
        assert_eq!(calls.len(), 1, "{prompt:?} should emit one call");
        assert_eq!(calls[0].tool, "websearch", "{prompt:?} should web-search");
        let query = arguments(&calls[0])["query"]
            .as_str()
            .expect("query string")
            .to_owned();
        assert!(
            !query.trim().is_empty(),
            "{prompt:?} query must not be empty"
        );
        query
    }

    #[test]
    fn english_question_reaches_the_web() {
        // language: en (english) — the original OpenCode phrasing.
        let query = web_query("When are the next elections in the USA?");
        assert!(query.to_lowercase().contains("election"), "{query}");
    }

    #[test]
    fn russian_question_reaches_the_web() {
        // language: ru — Russian (русский). Cyrillic question, full-width absent.
        let query = web_query("Когда следующие выборы в США?");
        assert!(query.contains("выборы"), "{query}");
    }

    #[test]
    fn hindi_question_reaches_the_web() {
        // language: hi — Hindi (हिंदी). Devanagari question ending in `?`.
        let query = web_query("संयुक्त राज्य अमेरिका में अगले चुनाव कब हैं?");
        assert!(query.contains("चुनाव"), "{query}");
    }

    #[test]
    fn chinese_question_reaches_the_web() {
        // language: zh — Chinese (中文). Ends with the full-width `？`, which must be
        // recognised as a question mark the same way ASCII `?` is.
        let query = web_query("美国下次选举是什么时候？");
        assert!(query.contains("选举"), "{query}");
    }

    #[test]
    fn research_imperative_carries_the_bare_topic() {
        // A direct English research imperative reduces to its bare topic.
        let messages = vec![ChatMessage::user("Research quantum computing")];
        let calls = tool_calls(&messages);
        assert_eq!(calls[0].tool, "websearch");
        assert_eq!(arguments(&calls[0])["query"], "quantum computing");
    }
}

// Requirement 2: report the issue to the Formal AI repository in natural language.
mod report_issue {
    use super::*;

    #[test]
    fn report_request_emits_gh_issue_create() {
        for prompt in [
            "Report this issue on GitHub",
            "Please file a bug report for the Formal AI repository",
            "open an issue about this",
        ] {
            let messages = vec![
                ChatMessage::user("When next elections in the USA?"),
                ChatMessage::assistant("I could not determine that."),
                ChatMessage::user(prompt),
            ];
            let calls = tool_calls(&messages);
            assert_eq!(calls.len(), 1, "{prompt:?} should emit one call");
            assert_eq!(calls[0].tool, "bash", "{prompt:?} should shell out to gh");
            let args = arguments(&calls[0]);
            let command = args["command"].as_str().expect("command string");
            assert!(
                command.contains("gh issue create"),
                "{prompt:?} should run `gh issue create`, got: {command}"
            );
            assert!(
                command.contains("link-assistant/formal-ai"),
                "{prompt:?} should target the Formal AI repo, got: {command}"
            );
        }
    }

    #[test]
    fn bare_report_after_conversation_files_the_issue() {
        let messages = vec![
            ChatMessage::user("What we were talking about?"),
            ChatMessage::assistant("We discussed the next US elections."),
            ChatMessage::user("Report"),
        ];
        let calls = tool_calls(&messages);
        assert_eq!(calls[0].tool, "bash");
        let args = arguments(&calls[0]);
        let command = args["command"].as_str().unwrap();
        assert!(command.contains("gh issue create"));
    }

    #[test]
    fn report_then_confirms_with_created_url() {
        let mut messages = vec![ChatMessage::user("Report this to GitHub")];
        let create = tool_calls(&messages).remove(0);
        answer_tool_call(
            &mut messages,
            &create,
            "https://github.com/link-assistant/formal-ai/issues/999",
        );
        let answer = final_answer(&messages);
        assert!(
            answer.contains("issues/999"),
            "final answer should surface the created issue URL, got: {answer}"
        );
    }

    #[test]
    fn bare_russian_report_files_the_issue() {
        // language: ru — Russian (русский) bare "сообщи" is the minimal report.
        let messages = vec![
            ChatMessage::user("Когда следующие выборы в США?"),
            ChatMessage::assistant("Я не смог это определить."),
            ChatMessage::user("сообщи"),
        ];
        let calls = tool_calls(&messages);
        assert_eq!(calls[0].tool, "bash");
        let args = arguments(&calls[0]);
        let command = args["command"].as_str().unwrap();
        assert!(command.contains("gh issue create"), "{command}");
    }

    #[test]
    fn every_supported_language_routes_report_intent_from_seed_data() {
        for prompt in [
            "Please report this problem",
            "Пожалуйста, сообщи об этой ошибке",
            "कृपया इस समस्या की रिपोर्ट करें",
            "请报告这个问题",
        ] {
            let messages = vec![
                ChatMessage::user("The answer did not use the available tool."),
                ChatMessage::assistant("I could not determine that."),
                ChatMessage::user(prompt),
            ];
            let calls = tool_calls(&messages);
            assert_eq!(calls[0].tool, "bash", "{prompt:?}");
            let command = arguments(&calls[0])["command"]
                .as_str()
                .expect("command string")
                .to_owned();
            assert!(command.contains("gh issue create"), "{prompt:?}: {command}");
        }
    }

    #[test]
    fn conversation_text_is_shell_escaped_in_the_command() {
        // An apostrophe in the conversation must not break out of the shell quoting
        // when the report body transcribes it.
        let messages = vec![
            ChatMessage::user("it's broken and I can't proceed"),
            ChatMessage::assistant("I could not determine that."),
            ChatMessage::user("Report this issue on GitHub"),
        ];
        let calls = tool_calls(&messages);
        let args = arguments(&calls[0]);
        let command = args["command"].as_str().unwrap();
        assert!(command.contains("gh issue create --repo link-assistant/formal-ai"));
        // POSIX single-quote escaping renders a literal `'` as `'\''`.
        assert!(
            command.contains("'\\''"),
            "apostrophe must be escaped: {command}"
        );
    }

    #[test]
    fn unrelated_report_shaped_prompt_is_not_an_issue_filing() {
        // "report the file sizes here" uses "report" as a plain verb, not an intent
        // to file a GitHub issue, so it must not shell out to `gh issue create`.
        for prompt in [
            "report the file sizes here",
            "create file notes.txt with hello",
        ] {
            let messages = vec![ChatMessage::user(prompt)];
            if let AgenticPlan::ToolCalls(calls) =
                plan_chat_step(&messages, &TOOLS).unwrap_or(AgenticPlan::Final(String::new()))
            {
                for call in &calls {
                    let args = arguments(call);
                    let command = args["command"].as_str().unwrap_or_default();
                    assert!(
                        !command.contains("gh issue create"),
                        "{prompt:?} must not file an issue, got: {command}"
                    );
                }
            }
        }
    }
}

// Requirement 3: talk about the conversation itself.
mod conversation_recall {
    use super::*;

    #[test]
    fn what_were_we_talking_about_recalls_prior_turns() {
        for prompt in [
            "What we were talking about?",
            "What were we talking about?",
            "Remind me what we discussed.",
        ] {
            let messages = vec![
                ChatMessage::user("When are the next elections in the USA?"),
                ChatMessage::assistant("The next US general election is in November 2026."),
                ChatMessage::user(prompt),
            ];
            let answer = final_answer(&messages);
            assert!(
                answer.to_lowercase().contains("election"),
                "{prompt:?} should recall the prior topic, got: {answer}"
            );
        }
    }

    #[test]
    fn non_recall_prompts_do_not_hijack_the_recall_recipe() {
        // "let me talk to support" mentions "talk" but no first-person-plural "we",
        // so it is not a recall question and must not answer from history.
        let messages = vec![
            ChatMessage::user("When are the next elections in the USA?"),
            ChatMessage::assistant("The next US general election is in November 2026."),
            ChatMessage::user("let me talk to support"),
        ];
        let plan = plan_chat_step(&messages, &TOOLS);
        if let Some(AgenticPlan::Final(answer)) = plan {
            assert!(
                !answer.contains("Here is what we have been talking about"),
                "must not be answered as recall, got: {answer}"
            );
        }
    }

    #[test]
    fn multilingual_summary_phrases_use_the_shared_history_solver() {
        for prompt in [
            "What have we talked about?",
            "О чём мы разговаривали?",
            "हमने किस बारे में बात की?",
            "我们聊了什么？",
        ] {
            let messages = vec![
                ChatMessage::user("Associative auto-learning"),
                ChatMessage::assistant("We inspected the durable links network."),
                ChatMessage::user(prompt),
            ];
            let answer = final_answer(&messages);
            assert!(
                answer.contains("Associative auto-learning"),
                "{prompt:?} should recall prior dialog, got: {answer}"
            );
        }
    }
}

// Requirement 4: "learn about it" resolves the pronoun and researches the topic.
mod learn_about {
    use super::*;

    #[test]
    fn learn_about_it_researches_the_prior_topic() {
        let messages = vec![
            ChatMessage::user("Tell me about the Rust borrow checker."),
            ChatMessage::assistant("It enforces ownership at compile time."),
            ChatMessage::user("Learn about it."),
        ];
        let calls = tool_calls(&messages);
        assert_eq!(calls[0].tool, "websearch");
        let query = arguments(&calls[0])["query"]
            .as_str()
            .expect("query")
            .to_lowercase();
        assert!(
            query.contains("borrow") || query.contains("rust"),
            "learn-about should carry the resolved topic, got: {query}"
        );
    }

    #[test]
    fn whole_reported_sequence_keeps_the_last_substantive_topic() {
        let messages = vec![
            ChatMessage::user("When are the next elections in the USA?"),
            ChatMessage::assistant("The next general election is in 2026."),
            ChatMessage::user("Report this problem"),
            ChatMessage::assistant("Filed the issue on GitHub."),
            ChatMessage::user("What we were talking about?"),
            ChatMessage::assistant("We discussed the next elections and reporting the failure."),
            ChatMessage::user("Learn about it."),
        ];
        let calls = tool_calls(&messages);
        assert_eq!(calls[0].tool, "websearch");
        let query = arguments(&calls[0])["query"]
            .as_str()
            .expect("query")
            .to_lowercase();
        assert!(query.contains("election"), "resolved query: {query}");
        assert!(!query.contains("talking"), "resolved query: {query}");
        assert!(!query.contains("report"), "resolved query: {query}");
    }
}

fn answer_tool_call(messages: &mut Vec<ChatMessage>, call: &PlannedToolCall, result: &str) {
    use formal_ai::protocol::ToolCall;
    let id = format!("call_{}", messages.len());
    messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
        id.clone(),
        call.tool.clone(),
        call.arguments.clone(),
    )]));
    messages.push(ChatMessage::tool_result(id, &call.tool, result));
}
