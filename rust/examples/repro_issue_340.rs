// Issue #340: a composite `write_program` request (HTTP GET -> parse JSON ->
// compute mean and median) used to dead-end at `write_program_unsupported`
// because no verified catalog template matched the task. This example shows the
// same request now resolving to a curated *blueprint*: the full program, its
// decomposition plan, the libraries it needs, and an honest "not run" report
// (the program needs external libraries and network access the offline sandbox
// cannot provide, so it never claims "compiled and ran").
//
// Run with: cargo run --example repro_issue_340
use formal_ai::FormalAiEngine;

fn main() {
    let cases = [
        // The exact prompt from issue #340 (English / Rust).
        (
            "Issue 340 (English / Rust)",
            "Write a Rust program that makes an HTTP GET request, parses the JSON \
             response, calculates the mean and median, and outputs the results, \
             with error handling and comments.",
        ),
        (
            "English / Python",
            "Write a Python program that makes an HTTP GET request, parses the JSON, \
             and computes the mean and median of the values.",
        ),
        (
            "English / JavaScript",
            "Write a JavaScript program that fetches JSON over HTTP and reports the \
             mean and median.",
        ),
        (
            "Russian / Rust",
            "Напиши программу на Rust, которая делает HTTP запрос, разбирает JSON и \
             считает среднее и медиану.",
        ),
        (
            "Hindi / Python",
            "Python में एक प्रोग्राम लिखो जो http अनुरोध करे, json पार्स करे और औसत और \
             माध्यिका की गणना करे।",
        ),
        (
            "Chinese / JavaScript",
            "用 JavaScript 编写一个程序，发起 http 请求，解析 json，并计算平均值和中位数。",
        ),
        // A partial request (no statistics) stays honestly unsupported.
        (
            "Partial (no statistics) stays unsupported",
            "Write a Rust program that makes an HTTP GET request and parses the JSON \
             response.",
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
