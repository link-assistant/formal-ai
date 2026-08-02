//! Probe which multilingual prompts trigger the issue-687 web-research recipe.
use formal_ai::agentic_coding::{plan_chat_step, AgenticPlan};
use formal_ai::protocol::ChatMessage;

fn main() {
    let tools = ["websearch", "webfetch", "read", "write", "bash"];
    let prompts = [
        ("en question", "When are the next elections in the USA?"),
        ("en research", "Research quantum computing"),
        ("ru research", "Изучи квантовые компьютеры"),
        ("ru question", "Когда следующие выборы в США?"),
        ("hi research", "क्वांटम कंप्यूटिंग के बारे में जानकारी दें"),
        ("hi question", "संयुक्त राज्य अमेरिका में अगले चुनाव कब हैं?"),
        ("zh research", "研究量子计算"),
        ("zh question", "美国下次选举是什么时候？"),
    ];
    for (label, prompt) in prompts {
        let messages = vec![ChatMessage::user(prompt)];
        match plan_chat_step(&messages, &tools) {
            Some(AgenticPlan::ToolCalls(calls)) => {
                println!("[{label}] {prompt}\n   -> tool={} args={}", calls[0].tool, calls[0].arguments);
            }
            Some(AgenticPlan::Final(a)) => println!("[{label}] {prompt}\n   -> FINAL: {}", a.replace('\n', " ").chars().take(80).collect::<String>()),
            None => println!("[{label}] {prompt}\n   -> None"),
        }
    }
}
