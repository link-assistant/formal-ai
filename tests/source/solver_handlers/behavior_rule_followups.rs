//! Localized follow-up wording for behavior-rule inspection.

use crate::seed;

pub(super) fn behavior_rule_response_language(normalized: &str, detected_language: &str) -> String {
    response_language_from_prompt(normalized)
        .unwrap_or(detected_language)
        .to_owned()
}

pub(super) fn render_behavior_rule_count(
    built_in: usize,
    runtime: usize,
    language: &str,
) -> String {
    let total = built_in + runtime;
    match language {
        "ru" => format!(
            "Всего правил поведения в этом диалоге: {total}. Встроенных: {built_in}; правил, изученных в диалоге: {runtime}. Я посчитал записи каталога правил и добавил правила из истории диалога."
        ),
        "hi" => format!(
            "इस संवाद में कुल {total} व्यवहार नियम हैं. Built-in: {built_in}; संवाद में सिखाए गए नियम: {runtime}. मैंने rule catalog की entries गिनीं और dialog history से मिले rules जोड़े."
        ),
        "zh" => format!(
            "这个对话里共有 {total} 条行为规则。内置规则：{built_in}；本对话学到的规则：{runtime}。我统计了规则目录记录，并加上对话历史中的规则。"
        ),
        _ => format!(
            "There are {total} behavior rules in this dialog: {built_in} built-in and {runtime} dialog-local. I counted the behavior-rule catalog entries and added rules learned from the dialog history."
        ),
    }
}

pub(super) fn render_behavior_rules_brief(
    built_in: usize,
    runtime: usize,
    language: &str,
) -> String {
    let total = built_in + runtime;
    let groups = localized_text(
        language,
        "greetings, farewells, small talk, identity, assistant name, capabilities, program templates, and the unknown fallback",
        "приветствия, прощания, светская беседа, идентичность, имя ассистента, возможности, шаблоны программ и резервный ответ",
        "अभिवादन, विदाई, हल्की बातचीत, पहचान, सहायक का नाम, क्षमताएँ, program templates, और unknown fallback",
        "问候、告别、闲聊、身份、助手名称、能力、程序模板和未知请求回退",
    );
    match language {
        "ru" => format!(
            "Всего: {total} правил поведения ({built_in} встроенных, {runtime} из диалога). Кратко: {groups}."
        ),
        "hi" => format!(
            "कुल: {total} व्यवहार नियम ({built_in} built-in, {runtime} dialog-local). संक्षेप में: {groups}."
        ),
        "zh" => format!(
            "总计：{total} 条行为规则（{built_in} 条内置，{runtime} 条来自对话）。简要：{groups}。"
        ),
        _ => format!(
            "Briefly: {total} behavior rules ({built_in} built-in, {runtime} dialog-local): {groups}."
        ),
    }
}

fn response_language_from_prompt(normalized: &str) -> Option<&'static str> {
    seed::lexicon()
        .meanings_with_role(seed::ROLE_RESPONSE_LANGUAGE_MARKER)
        .filter(|meaning| meaning.words().any(|word| normalized.contains(word)))
        .find_map(language_code_of)
}

fn language_code_of(meaning: &seed::Meaning) -> Option<&'static str> {
    meaning
        .defined_by
        .iter()
        .find_map(|slug| match slug.as_str() {
            "language_english" => Some("en"),
            "language_russian" => Some("ru"),
            "language_hindi" => Some("hi"),
            "language_chinese" => Some("zh"),
            _ => None,
        })
}

fn localized_text(
    language: &str,
    en: &'static str,
    ru: &'static str,
    hi: &'static str,
    zh: &'static str,
) -> &'static str {
    match language {
        "ru" => ru,
        "hi" => hi,
        "zh" => zh,
        _ => en,
    }
}
