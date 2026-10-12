//! PR #1188 dogfooding: `t.md से drop शब्द हटाओ।` ran `rm t.md`. When no edit
//! step can ground a request about text inside a file, the destructive shell
//! intents (`destructive true` in data/seed/shell-intents.lino) are declined
//! with the seeded `file_text_unit` sentence instead of composed; a request
//! that plainly deletes the file is still a deletion. Twin of the JS cases in
//! `rust/tests/web/pull-request-1188-dogfood.test.mjs`.

use formal_ai::agentic_coding::{AgenticPlan, plan_chat_step};
use formal_ai::protocol::ChatMessage;

const TOOLS: [&str; 4] = ["read", "edit", "bash", "write"];

fn first_plan(prompt: &str) -> Option<AgenticPlan> {
    plan_chat_step(&[ChatMessage::user(prompt)], &TOOLS)
}

#[test]
fn an_edit_shaped_request_is_declined_never_composed_as_rm() {
    for (prompt, expected) in [
        (
            "Delete the file t.md word drop.",
            "No edit I can verify reads this request against `t.md`, so nothing was changed. The request names text inside the file, not the file itself, so I did not delete it.",
        ),
        (
            "Удалить файл t.md слово drop",
            "Ни одна проверяемая правка не прочитала этот запрос для `t.md`, поэтому ничего не изменено. Запрос говорит о тексте внутри файла, а не о самом файле, поэтому файл не удалён.",
        ),
        (
            "t.md से drop शब्द हटाओ।",
            "`t.md` के लिए इस अनुरोध को कोई सत्यापन-योग्य संपादन नहीं पढ़ सका, इसलिए कुछ नहीं बदला गया। अनुरोध फ़ाइल के भीतर के पाठ के बारे में है, फ़ाइल के बारे में नहीं, इसलिए फ़ाइल नहीं हटाई गई।",
        ),
        (
            "删除文件 t.md 里的文字 drop",
            "没有可验证的编辑能针对 `t.md` 读懂这个请求，因此未做任何更改。请求说的是文件中的文本，而不是文件本身，所以没有删除该文件。",
        ),
    ] {
        match first_plan(prompt) {
            Some(AgenticPlan::Final(answer)) => assert_eq!(answer, expected, "{prompt}"),
            other => panic!("{prompt} planned {other:?}"),
        }
    }
}

#[test]
fn a_request_that_plainly_deletes_the_file_is_still_a_deletion() {
    for prompt in [
        "Delete the file t.md",
        "Удали файл t.md",
        "t.md हटाओ",
        "删除文件 t.md",
    ] {
        let Some(AgenticPlan::ToolCalls(calls)) = first_plan(prompt) else {
            panic!("{prompt} planned no tool call");
        };
        assert_eq!(calls[0].tool, "bash", "{prompt}");
        let arguments: serde_json::Value =
            serde_json::from_str(&calls[0].arguments).expect("tool arguments are JSON");
        assert_eq!(arguments["command"], "rm t.md", "{prompt}");
    }
}

#[test]
fn the_destructive_commands_are_read_from_the_seed() {
    let destructive: Vec<String> = formal_ai::seed::shell_intent_vocabulary()
        .intents
        .iter()
        .filter(|intent| intent.destructive)
        .map(|intent| intent.command.clone())
        .collect();
    assert_eq!(destructive, ["rmdir", "rm"]);
}
