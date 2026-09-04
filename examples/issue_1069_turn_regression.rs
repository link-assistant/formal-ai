//! Dump the agentic plan for a request, so a routing decision can be read as
//! the step it produced rather than as a count.

use formal_ai::agentic_coding::plan_chat_step;
use formal_ai::protocol::ChatMessage;

fn main() {
    let prompt = std::env::args().nth(1).expect("a prompt");
    let tools = ["run_command", "web_fetch", "web_search", "write_file"];
    let messages = [ChatMessage::user(prompt)];
    println!("{:?}", plan_chat_step(&messages, &tools));
}
