use formal_ai::agentic_coding::{plan_chat_step, AgenticPlan};
use formal_ai::protocol::ChatMessage;

fn main() {
    let cases = [
        "In greeting.txt, change hello to goodbye",
        "Replace foo with bar in notes.txt",
        "Correct teh to the in doc.txt",
        "замени привет на пока в файле заметки.txt",
        "modify the value old with new in config/app.yaml",
        "read the file alpha.txt",              // must NOT edit
        "add hello to config.txt",              // must NOT edit (write intent)
        "fix the bug in server.rs",             // no new-lead => None
    ];
    for c in cases {
        let msgs = vec![ChatMessage::user(c)];
        let plan = plan_chat_step(&msgs, &["edit", "read_file", "write_file"]);
        match plan {
            Some(AgenticPlan::ToolCalls(calls)) => {
                println!("[{}] -> {} :: {}", c, calls[0].tool, calls[0].arguments);
            }
            other => println!("[{}] -> {:?}", c, other),
        }
    }
}
