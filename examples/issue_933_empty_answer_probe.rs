//! Dump the whole `SymbolicAnswer` for the prompts that came back with an
//! empty answer while the issue #933 corpus was being recorded.
//!
//! It dumps the intent, the confidence, the evidence links and the thinking
//! steps, which is how the loss was pinned to the question-necessity pass
//! rather than to routing or to the seed. On the fixed engine every prompt
//! below answers in full.
//!
//!     cargo run --example issue_933_empty_answer_probe

use formal_ai::FormalAiEngine;

fn main() {
    for prompt in ["你好吗?", "你好吗", "谢谢", "धन्यवाद", "你好", "спасибо"]
    {
        let response = FormalAiEngine.answer(prompt);
        println!("prompt: {prompt:?}");
        println!("  intent: {}", response.intent);
        println!("  answer: {:?}", response.answer);
        println!("  confidence: {}", response.confidence);
        println!("  evidence: {:?}", response.evidence_links);
        println!("  steps: {:?}", response.thinking_steps);
    }
}
