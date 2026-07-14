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
