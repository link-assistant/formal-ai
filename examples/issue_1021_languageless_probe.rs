//! Issue #1021 / #862: what Formal AI answers for a coding request that names
//! no implementation language.
use formal_ai::FormalAiEngine;

fn main() {
    for prompt in [
        "мне нужен код",
        "дай мне код",
        "I need code",
        "give me code",
        "I want code",
        "मुझे कोड चाहिए",
        "我需要代码",
        "给我代码",
        "write me a program",
        "write me some code",
        "напиши программу",
        "give me the code of this repository",
        "I need a code review",
        "I need information about Rust",
        "I need to find a python tutorial",
        "дай мне код этого репозитория",
        "мне нужен код на пхп",
        "give me python code",
        "I need PHP code",
    ] {
        let response = FormalAiEngine.answer(prompt);
        println!("{:<44} -> {}", prompt, response.intent);
    }

    // The intent alone does not show that the reply is useful. Print the whole
    // answer for the reported request and for its English counterpart, so the
    // record shows the reply asks which language -- in the language it was
    // asked in -- rather than reciting what the catalogue happens to hold.
    for prompt in ["мне нужен код", "I need code"] {
        println!("\n=== {prompt}\n{}", FormalAiEngine.answer(prompt).answer);
    }
}
