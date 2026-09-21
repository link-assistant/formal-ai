use formal_ai::FormalAiEngine;

fn main() {
    let cases = [
        ("Issue 216 (no quotes)", "translate apple to russian"),
        ("Issue 217 (« quotes)", "переведи «яблоко» на английский"),
        ("Issue 217 (\" quotes)", "переведи \"яблоко\" на английский"),
        ("Apple no quotes en", "translate apple to english"),
        (
            "Doброе яблоко (no fix yet for compositional)",
            "Переведи \"доброе яблоко\" на английский.",
        ),
    ];
    for (label, prompt) in &cases {
        let response = FormalAiEngine.answer(prompt);
        println!("=== {label} ===");
        println!("PROMPT: {prompt}");
        println!("INTENT: {}", response.intent);
        println!("ANSWER: {}", response.answer);
        println!("EVIDENCE: {:?}", response.evidence_links);
        println!();
    }
}
