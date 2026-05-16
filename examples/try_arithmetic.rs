//! Smoke-test the calculation handler. Run with:
//! `cargo run --example try_arithmetic` after copying this file into examples/.
//! Or run via: `rustc -L target/debug/deps --edition 2021 try_arithmetic.rs ...`
//!
//! Easier path: replicate the prompts through the test runner. This file is
//! a record of the manual probes used while developing the handler.

use formal_ai::FormalAiEngine;

fn main() {
    let prompts = [
        "What is 2 + 2?",
        "What is 8% of $50?",
        "Please calculate sqrt(16)",
        "Compute 300000 ms in seconds",
        "Посчитай 1000 рублей в долларах",
        "计算 1000 美元 换成 欧元",
        "गणना करें 1000 डॉलर में यूरो",
        "Calculate 7 * (3 + 4)",
        "What is 10 / 3",
        "Compute 100 - 25 % 7",
        "How much is 1.5 + 2.5?",
        "What is 10 plus 20 times 3?",
        "What is 5 / 0?",
        "Hi",
    ];
    for prompt in prompts {
        let response = FormalAiEngine.answer(prompt);
        println!("---");
        println!("prompt   : {prompt}");
        println!("intent   : {}", response.intent);
        println!("answer   : {}", response.answer);
        println!("confidence: {:.2}", response.confidence);
    }
}
