//! Render the issue #771 session the way the agentic planner would file it.
//!
//! Replays the reported exchange — a question, a web search, a fetch that
//! returns a whole scraped page, then "report" — and prints the research answer
//! plus the `gh issue create` body. Run it to eyeball the report a human would
//! actually receive:
//!
//! ```sh
//! cargo run --example issue_771_report_format
//! ```

use std::fmt::Write as _;

use formal_ai::agentic_coding::{plan_chat_step, AgenticPlan, PlannedToolCall};
use formal_ai::protocol::{ChatMessage, ToolCall};

const TOOLS: [&str; 5] = ["websearch", "webfetch", "read", "write", "bash"];
const SOURCE: &str = "https://integral-russia.ru/2026/03/06/chastnye-kosmicheskie-kompanii/";

fn main() {
    let mut messages = vec![ChatMessage::user(
        "В каких странах есть частные космические компании?",
    )];

    let search = tool_calls(&messages).remove(0);
    answer_tool_call(
        &mut messages,
        &search,
        &format!("Частные космические компании за рубежом {SOURCE}"),
    );

    let fetch = tool_calls(&messages).remove(0);
    answer_tool_call(&mut messages, &fetch, &scraped_page());

    let answer = match plan(&messages) {
        AgenticPlan::Final(answer) => answer,
        plan => panic!("expected a final answer, got {plan:?}"),
    };
    println!("=== research answer ({} bytes) ===\n{answer}\n", answer.len());

    messages.push(ChatMessage::assistant(answer));
    messages.push(ChatMessage::user("report"));

    let command = arguments(&tool_calls(&messages)[0])["command"]
        .as_str()
        .expect("command string")
        .to_owned();
    let body = body_of(&command);
    println!("=== issue body ({} bytes) ===\n{body}", body.len());
}

/// A page shaped like the reported fetch result: site chrome and trailing
/// boilerplate around the one paragraph that answers the question.
fn scraped_page() -> String {
    let mut page = String::from(
        "ТЕХНОЛОГИИ, ИНЖИНИРИНГ, ИННОВАЦИИ\n\
         Главное меню Перейти к основному содержимому О нас Новости\n\
         Наука Техника Инжиниринг Промышленность Инновации Видео\n\
         Навигация по записям Предыдущая Следующая\n",
    );
    page.push_str(
        "Частные космические компании работают в США, Великобритании, Германии, \
         Франции, Испании, Китае и Индии.\n",
    );
    for index in 0..400 {
        writeln!(
            page,
            "Рубрика {index}: подписывайтесь на нашу рассылку и читайте другие записи блога."
        )
        .expect("writing to a String cannot fail");
    }
    page
}

/// Recover the `--body` argument from the composed shell command.
fn body_of(command: &str) -> String {
    let start = command
        .find("--body '")
        .expect("report command should carry a --body argument")
        + "--body '".len();
    command[start..]
        .strip_suffix('\'')
        .expect("body argument should be single-quoted")
        .replace("'\\''", "'")
}

fn plan(messages: &[ChatMessage]) -> AgenticPlan {
    plan_chat_step(messages, &TOOLS).expect("planner should recognise the task")
}

fn tool_calls(messages: &[ChatMessage]) -> Vec<PlannedToolCall> {
    match plan(messages) {
        AgenticPlan::ToolCalls(calls) => calls,
        plan => panic!("expected tool calls, got {plan:?}"),
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
