//! Reproduce issue #499: the "learn from this data source" directive returns unknown.
use formal_ai::engine::FormalAiEngine;

fn main() {
    let engine = FormalAiEngine;
    let prompts = [
        "Обратясь сюда ты узнаешь актуальные темы https://trends.google.com/trending?hl=ru&&geo=US",
        "Тут видны темы кототорые интересуют людей https://trends.google.com/trending?hl=ru&&geo=US",
        "Here you can learn the current trending topics https://trends.google.com/trending?geo=US",
        "Learn from popular Google searches at https://trends.google.com/trending/rss?geo=US",
    ];
    for p in prompts {
        let a = engine.answer(p);
        println!("PROMPT: {p}\n  intent = {}\n  answer = {}\n", a.intent, a.answer.replace('\n', " ").chars().take(120).collect::<String>());
    }
}
