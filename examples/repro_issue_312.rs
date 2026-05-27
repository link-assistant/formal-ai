// Issue #312: "list the files in the current directory" used to return the
// `unknown` intent because `list_files` was never an implemented `write_program`
// task. This example shows the prompt resolving to a real Rust program (using
// `std::fs::read_dir`) across every supported language, plus the same task in
// other catalog languages.
//
// Run with: cargo run --example repro_issue_312
use formal_ai::FormalAiEngine;

fn main() {
    let cases = [
        // The exact prompt from issue #312 (Russian).
        (
            "Issue 312 (Russian / Rust)",
            "Напиши мне программу на Rust, которая выдаёт список файлов в текущей директории",
        ),
        (
            "English / Rust",
            "Write a Rust program that lists files in the current directory",
        ),
        (
            "Hindi / Rust",
            "Rust में फ़ाइलों की सूची दिखाने वाला प्रोग्राम लिखो",
        ),
        ("Chinese / Rust", "用 Rust 编写一个列出当前目录中文件的程序"),
        (
            "English / Python",
            "Write a Python program that lists files in the current directory",
        ),
        (
            "English / Go",
            "Write a Go program that lists files in the current directory",
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
