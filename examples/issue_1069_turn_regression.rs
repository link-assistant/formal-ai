//! Dump the agentic plan for a request, so a routing decision can be read as
//! the step it produced rather than as a count.
//!
//! `TOOLS` overrides the client's tool names (comma-separated), because which
//! tools a caller offers is an input to the routing decision.

use formal_ai::agentic_coding::plan_chat_step;
use formal_ai::protocol::ChatMessage;

fn main() {
    let prompt = std::env::args().nth(1).expect("a prompt");
    let offered = std::env::var("TOOLS")
        .unwrap_or_else(|_| "run_command,web_fetch,web_search,write_file".to_owned());
    let tools = offered.split(',').collect::<Vec<_>>();
    let messages = [ChatMessage::user(prompt)];
    println!("{:?}", plan_chat_step(&messages, &tools));
}
