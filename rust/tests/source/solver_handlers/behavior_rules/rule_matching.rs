use super::*;

pub(super) fn rule_when_then(rule: &BehaviorRuleRecord, language: &str) -> String {
    if rule.id == "rule_write_program" {
        return match language {
            "ru" => "Когда пользователь просит программу с поддерживаемыми параметрами `language` и `task`, выбери соответствующий шаблон через единое намерение `write_program`.".to_owned(),
            "hi" => "जब उपयोगकर्ता supported `language` और `task` parameter वाला program माँगे, तब single `write_program` intent से matching template दें.".to_owned(),
            "zh" => "当用户请求带受支持 `language` 和 `task` 参数的程序时，通过单个 `write_program` 意图选择匹配模板。".to_owned(),
            _ => rule.when_then.clone(),
        };
    }

    match rule.id.as_str() {
        "rule_greeting" => match language {
            "ru" => format!(
                "Когда пользователь говорит `Hi`, `Hello`, `Hey` или многоязычную фразу приветствия, ответь `{}`.",
                rule_response(rule, language)
            ),
            "hi" => format!(
                "जब उपयोगकर्ता `Hi`, `Hello`, `Hey` या बहुभाषी greeting phrase कहे, तब `{}` उत्तर दें.",
                rule_response(rule, language)
            ),
            "zh" => format!(
                "当用户说 `Hi`、`Hello`、`Hey` 或多语言问候短语时，回答 `{}`。",
                rule_response(rule, language)
            ),
            _ => rule.when_then.clone(),
        },
        "rule_farewell" => match language {
            "ru" => format!(
                "Когда пользователь говорит `bye`, `goodbye`, `poka` или многоязычную фразу прощания, ответь `{}`.",
                rule_response(rule, language)
            ),
            "hi" => format!(
                "जब उपयोगकर्ता `bye`, `goodbye`, `poka` या बहुभाषी farewell phrase कहे, तब `{}` उत्तर दें.",
                rule_response(rule, language)
            ),
            "zh" => format!(
                "当用户说 `bye`、`goodbye`、`poka` 或多语言告别短语时，回答 `{}`。",
                rule_response(rule, language)
            ),
            _ => rule.when_then.clone(),
        },
        "rule_assistant_free_time" => match language {
            "ru" => format!(
                "Когда пользователь спрашивает, что я делаю в свободное время, ответь `{}`.",
                rule_response(rule, language)
            ),
            "hi" => format!(
                "जब उपयोगकर्ता पूछे कि मैं खाली समय में क्या करता हूँ, तब `{}` उत्तर दें.",
                rule_response(rule, language)
            ),
            "zh" => format!(
                "当用户问我空闲时间做什么时，回答 `{}`。",
                rule_response(rule, language)
            ),
            _ => rule.when_then.clone(),
        },
        "rule_identity" => match language {
            "ru" => format!(
                "Когда пользователь спрашивает `Who are you?` или `Кто ты?`, ответь `{}`.",
                rule_response(rule, language)
            ),
            "hi" => format!(
                "जब उपयोगकर्ता `Who are you?` या `Кто ты?` पूछे, तब `{}` उत्तर दें.",
                rule_response(rule, language)
            ),
            "zh" => format!(
                "当用户问 `Who are you?` 或 `Кто ты?` 时，回答 `{}`。",
                rule_response(rule, language)
            ),
            _ => rule.when_then.clone(),
        },
        "rule_assistant_name" => match language {
            "ru" => "Когда пользователь спрашивает `What is your name?` или `Как тебя зовут?`, ответь сообщением об имени ассистента; если поверхность поддерживает настройку имени, включи настроенное имя.".to_owned(),
            "hi" => "जब उपयोगकर्ता `What is your name?` या `Как тебя зовут?` पूछे, तब assistant-name उत्तर दें; अगर surface में assistant-name setting है, तो configured name शामिल करें.".to_owned(),
            "zh" => "当用户问 `What is your name?` 或 `Как тебя зовут?` 时，回答助手名称；如果界面有助手名称设置，则包含配置的名称。".to_owned(),
            _ => rule.when_then.clone(),
        },
        "rule_capabilities" => match language {
            "ru" => "Когда пользователь спрашивает `What can you do?` или `Что ты умеешь?`, ответь многоязычным списком возможностей.".to_owned(),
            "hi" => "जब उपयोगकर्ता `What can you do?` या `Что ты умеешь?` पूछे, तब बहुभाषी capability listing दें.".to_owned(),
            "zh" => "当用户问 `What can you do?` 或 `Что ты умеешь?` 时，回答多语言能力列表。".to_owned(),
            _ => rule.when_then.clone(),
        },
        "rule_unknown" => match language {
            "ru" => "Когда ни одно более раннее правило или обработчик не подходит к запросу, ответь многоязычной подсказкой для неизвестного намерения (`Покажи правила`, `Покажи правило`, `Когда ... тогда ...`, `Сообщить о проблеме`, `Экспорт памяти`).".to_owned(),
            "hi" => "जब कोई पहले का rule या handler prompt से मेल न खाए, तब unknown-intent guide दें (`नियम दिखाएँ`, `rule दिखाएँ`, `जब ... तब ...`, `Report issue`, `Export memory`).".to_owned(),
            "zh" => "当前面的规则或处理器都不匹配提示时，回答未知意图指南（`显示规则`、`显示规则详情`、`当 ... 时 ...`、`报告问题`、`导出 memory`）。".to_owned(),
            _ => rule.when_then.clone(),
        },
        _ => rule.when_then.clone(),
    }
}
