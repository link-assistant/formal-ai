use crate::engine::normalize_prompt;
use crate::event_log::EventLog;
use crate::seed;

pub(super) fn built_in_rule_count() -> usize {
    super::behavior_rule_records().len()
}

pub(super) fn render_behavior_rule_count(runtime_rule_count: usize, language: &str) -> String {
    let built_in_count = built_in_rule_count();
    let total = built_in_count + runtime_rule_count;
    let summary = match language {
        "ru" => format!(
            "Всего правил: {total} (встроенных: {built_in_count}; изученных в этом диалоге: {runtime_rule_count})."
        ),
        "hi" => format!(
            "कुल व्यवहार नियम: {total} (built-in: {built_in_count}; dialog-local: {runtime_rule_count})."
        ),
        "zh" => {
            format!("行为规则总数：{total}（内置：{built_in_count}；本对话：{runtime_rule_count}）。")
        }
        _ => format!(
            "Total behavior rules: {total} (built-in: {built_in_count}; dialog-local: {runtime_rule_count})."
        ),
    };
    let reasoning = super::localized_text(
        language,
        "Reasoning: I count the built-in behavior-rule catalog and add dialog-local rules compiled from earlier user turns.",
        "Рассуждение: я считаю встроенный каталог правил поведения и добавляю правила, скомпилированные из предыдущих сообщений пользователя.",
        "Reasoning: मैं built-in behavior-rule catalog गिनता हूँ और पहले user turns से compiled dialog-local rules जोड़ता हूँ.",
        "Reasoning：我统计内置行为规则目录，并加上从此前用户消息编译出的本对话规则。",
    );

    format!(
        "{summary}\n\n{reasoning}\n\n```links\nbehavior_rules_count\n  built_in_rules \"{built_in_count}\"\n  dialog_local_rules \"{runtime_rule_count}\"\n  total_rules \"{total}\"\n  algorithm \"behavior_rule_records + collect_runtime_rules(prior_turn:user)\"\n```\n"
    )
}

pub(super) fn is_behavior_rules_count(normalized: &str, log: &EventLog) -> bool {
    let prior_rule_list_context = prior_behavior_rules_list_context(log);
    let lexicon = seed::lexicon();
    let present = |role: &str, language: &str| {
        lexicon
            .words_for_role_in_languages(role, &[language])
            .iter()
            .any(|word| normalized.contains(word.as_str()))
    };

    ["en", "ru", "hi", "zh"].into_iter().any(|language| {
        present(seed::ROLE_RULE_LISTING_SUBJECT, language)
            && present(seed::ROLE_RULE_COUNT_REQUEST, language)
            && (present(seed::ROLE_RULE_COUNT_SCOPE, language)
                || present(seed::ROLE_RULE_LISTING_SCOPE, language)
                || prior_rule_list_context)
    })
}

fn prior_behavior_rules_list_context(log: &EventLog) -> bool {
    log.events().iter().any(|event| {
        if event.kind == "prior_turn:user" {
            return super::is_behavior_rules_list(&normalize_prompt(&event.payload));
        }
        event.kind == "prior_turn:assistant"
            && event.payload.contains("rule_greeting")
            && event.payload.contains("rule_unknown")
    })
}
