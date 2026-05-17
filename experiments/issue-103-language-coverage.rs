//! Probe Hindi/Chinese conversational prompts to know which ones already
//! route to the desired intent + language tag and which need seed work.

use formal_ai::FormalAiEngine;

fn main() {
    let engine = FormalAiEngine;
    let cases: &[(&str, &str, &str)] = &[
        // group "lang:category", expected intent (or prefix), prompt
        ("hi:greeting", "greeting", "नमस्ते"),
        ("hi:greeting", "greeting", "नमस्कार"),
        ("hi:greeting", "greeting", "राम राम"),
        ("hi:greeting", "greeting", "सलाम"),
        ("hi:greeting", "greeting", "हाय"),
        ("hi:greeting", "greeting", "नमस्ते!"),
        ("zh:greeting", "greeting", "你好"),
        ("zh:greeting", "greeting", "您好"),
        ("zh:greeting", "greeting", "早上好"),
        ("zh:greeting", "greeting", "早安"),
        ("zh:greeting", "greeting", "嗨"),
        ("zh:greeting", "greeting", "哈喽"),
        ("hi:farewell", "farewell", "अलविदा"),
        ("hi:farewell", "farewell", "फिर मिलेंगे"),
        ("hi:farewell", "farewell", "विदा"),
        ("hi:farewell", "farewell", "बाय"),
        ("hi:farewell", "farewell", "टाटा"),
        ("zh:farewell", "farewell", "再见"),
        ("zh:farewell", "farewell", "拜拜"),
        ("zh:farewell", "farewell", "回见"),
        ("zh:farewell", "farewell", "改天见"),
        ("zh:farewell", "farewell", "后会有期"),
        ("hi:identity", "identity", "तुम कौन हो?"),
        ("hi:identity", "identity", "आप कौन हैं?"),
        ("hi:identity", "identity", "अपना परिचय दो"),
        ("hi:identity", "identity", "अपने बारे में बताओ"),
        ("hi:identity", "identity", "तू कौन है?"),
        ("zh:identity", "identity", "你是谁?"),
        ("zh:identity", "identity", "您是谁?"),
        ("zh:identity", "identity", "介绍一下你自己"),
        ("zh:identity", "identity", "你是什么?"),
        ("zh:identity", "identity", "告诉我你自己"),
        ("hi:concept", "concept_lookup", "विकिपीडिया क्या है?"),
        ("hi:concept", "concept_lookup", "रस्ट क्या है?"),
        ("hi:concept", "concept_lookup", "रंग क्या है?"),
        ("hi:concept", "concept_lookup", "विकिडेटा क्या है?"),
        ("hi:concept", "concept_lookup", "आईआईआर क्या है?"),
        ("zh:concept", "concept_lookup", "维基百科是什么?"),
        ("zh:concept", "concept_lookup", "颜色是什么?"),
        ("zh:concept", "concept_lookup", "维基数据是什么?"),
        ("zh:concept", "concept_lookup", "无限脉冲响应是什么?"),
        ("zh:concept", "concept_lookup", "rust语言是什么?"),
    ];

    let mut failures: Vec<(String, String, String)> = vec![];
    for (group, expected, prompt) in cases {
        let response = engine.answer(prompt);
        let intent_ok = response.intent == *expected || response.intent.starts_with(*expected);
        let language_tag = match group.split(':').next().unwrap_or("") {
            "hi" => "language:hi",
            "zh" => "language:zh",
            "ru" => "language:ru",
            "en" => "language:en",
            _ => "",
        };
        let lang_ok =
            language_tag.is_empty() || response.evidence_links.iter().any(|l| l == language_tag);
        let ok = intent_ok && lang_ok;
        let status = if ok { "ok" } else { "MISS" };
        println!(
            "{status:>4} {group:14} intent_got={:24} lang_ok={lang_ok:5} prompt={prompt}",
            response.intent,
        );
        if !ok {
            failures.push((
                (*group).to_owned(),
                (*expected).to_owned(),
                (*prompt).to_owned(),
            ));
        }
    }
    println!("\n--- {} miss(es) ---", failures.len());
    for (group, expected, prompt) in &failures {
        println!("  {group:14} want={expected:18} prompt={prompt}");
    }
}
