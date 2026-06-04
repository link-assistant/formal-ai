// Issue #395: "У меня есть числа 3, 5, 6, 7, 8 отсортируй их в JavaScript, дай
// мне код и результат" used to return the `unknown` intent. The system now
// recognizes the multilingual sort verb (seed `operation-vocabulary.lino`),
// reads the given numbers from the prompt, generates idiomatic code in the
// requested language, and — because sorting is a pure, decidable function —
// computes and shows the sorted result deterministically (no runtime needed).
//
// Run with: cargo run --example repro_issue_395
use formal_ai::FormalAiEngine;

fn main() {
    let cases = [
        // The exact prompt from issue #395 (Russian / JavaScript).
        (
            "Issue 395 (Russian / JavaScript)",
            "У меня есть числа 3, 5, 6, 7, 8 отсортируй их в JavaScript, дай мне код и результат",
        ),
        (
            "English / JavaScript",
            "I have numbers 5, 3, 8, 1, 9 — sort them in JavaScript, give me the code and the result",
        ),
        (
            "English / Python (descending)",
            "Sort the numbers 4, 2, 7, 1 in descending order in Python and show me the code and result",
        ),
        (
            "Hindi / JavaScript",
            "मेरे पास संख्याएं 3, 5, 6, 7, 8 हैं, उन्हें JavaScript में क्रमबद्ध करो और मुझे कोड और परिणाम दो",
        ),
        (
            "Chinese / Python",
            "我有数字 3, 5, 6, 7, 8，用 Python 排序，给我代码和结果",
        ),
    ];
    for (label, prompt) in &cases {
        let response = FormalAiEngine.answer(prompt);
        println!("=== {label} ===");
        println!("PROMPT: {prompt}");
        println!("INTENT: {}", response.intent);
        println!("ANSWER:\n{}", response.answer);
        println!();
    }
}
