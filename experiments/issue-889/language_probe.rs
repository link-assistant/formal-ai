//! Probe: which answer language does each candidate prompt resolve to, and how
//! does the localized trace read? (issue #889)

use formal_ai::{render_thinking_steps, thinking_answer_language, FormalAiEngine};

fn main() {
    for prompt in [
        "How are you?",
        "Hello",
        "как дела",
        "привет",
        "नमस्ते",
        "आप कैसे हैं",
        "你好",
        "hola, ¿cómo estás?",
        "2 + 2",
    ] {
        let answer = FormalAiEngine.answer(prompt);
        let language = thinking_answer_language(&answer.thinking_steps);
        println!("=== {prompt} -> [{language}]");
        println!("{}", render_thinking_steps(&answer.thinking_steps));
    }
}
