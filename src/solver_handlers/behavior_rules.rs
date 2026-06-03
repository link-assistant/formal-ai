//! Chat-editable behavior-rule inspection and dialog-local overrides.
//!
//! Runtime rule updates are intentionally append-only: the user message is the
//! durable record. Later turns scan prior user messages and project matching
//! instructions onto the current answer without mutating the baked seed.
//!
//! Issue #144: behavior rules are surfaced to the user as a series of
//! `When X then Y` (or `When X do Y`) statements grouped by topic so the same
//! grammar that lists the catalog can also teach new dialog-local rules. The
//! grammar is recognized in English, Russian, Hindi, and Chinese by
//! `skill_compiler`.

use std::collections::BTreeMap;

use crate::engine::{
    farewell_answer, greeting_answer, identity_answer, normalize_prompt,
    supported_program_languages, supported_program_tasks, unknown_answer, SymbolicAnswer,
};
use crate::event_log::EventLog;
use crate::language::detect as detect_language;
use crate::seed;
use crate::skill_compiler::{compile_natural_language_skill, CompiledSkillPackage};

use super::finalize_simple;
use super::self_awareness::{try_self_awareness, SelfAwarenessRuntime};

#[derive(Debug, Clone)]
struct BehaviorRuleRecord {
    id: String,
    topic: &'static str,
    intent: String,
    label: String,
    matches: String,
    response: String,
    source: String,
    when_then: String,
}

pub fn try_behavior_rules_with_runtime(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
    runtime: SelfAwarenessRuntime,
) -> Option<SymbolicAnswer> {
    let language = detect_language(prompt);
    let language = language.slug();

    if let Ok(package) = compile_natural_language_skill(prompt) {
        log.append("skill_compile:package", package.id.clone());
        log.append(
            "behavior_rule:update",
            package.legacy_behavior_rule_id.clone(),
        );
        let body = render_runtime_rule_update(&package, language);
        return Some(finalize_simple(
            prompt,
            log,
            "behavior_rule_update",
            "response:behavior_rule_update",
            &body,
            1.0,
        ));
    }

    if is_behavior_rules_list(normalized) {
        let runtime_rules = collect_runtime_rules(log);
        log.append("behavior_rules:list", "all".to_owned());
        let body = render_behavior_rule_list(&runtime_rules, language);
        return Some(finalize_simple(
            prompt,
            log,
            "behavior_rules_list",
            "response:behavior_rules_list",
            &body,
            1.0,
        ));
    }

    if let Some(query) = detail_query(prompt) {
        if let Some(rule) = find_behavior_rule(&query) {
            log.append("behavior_rule:read", rule.id.clone());
            let body = render_behavior_rule_detail(&rule, language);
            return Some(finalize_simple(
                prompt,
                log,
                "behavior_rule_detail",
                "response:behavior_rule_detail",
                &body,
                1.0,
            ));
        }
    }

    if let Some(answer) = try_self_awareness(prompt, normalized, log, runtime) {
        return Some(answer);
    }

    if let Some((package, replay)) = runtime_rule_for_prompt(prompt, log) {
        log.append("compiled_skill:package", package.links_notation());
        log.append("compiled_skill:replay", replay.rule_id.clone());
        log.append("cache_hit", replay.cache_hit);
        let response_link = format!("response:{}", package.id);
        return Some(finalize_simple(
            prompt,
            log,
            "behavior_rule_custom",
            &response_link,
            &replay.answer,
            1.0,
        ));
    }

    None
}

fn behavior_rule_records() -> Vec<BehaviorRuleRecord> {
    let mut records = vec![
        BehaviorRuleRecord {
            id: "rule_greeting".to_owned(),
            topic: "greetings",
            intent: "greeting".to_owned(),
            label: "Greeting rule".to_owned(),
            matches: "`Hi`, `Hello`, `Hey`, and multilingual greeting seed phrases".to_owned(),
            response: greeting_answer().to_owned(),
            source: "data/seed/intent-routing.lino + multilingual responses".to_owned(),
            when_then: format!(
                "When the user says `Hi`, `Hello`, or `Hey` then respond with `{}`.",
                greeting_answer()
            ),
        },
        BehaviorRuleRecord {
            id: "rule_farewell".to_owned(),
            topic: "farewells",
            intent: "farewell".to_owned(),
            label: "Farewell rule".to_owned(),
            matches: "`bye`, `goodbye`, `poka`, and multilingual farewell seed phrases".to_owned(),
            response: farewell_answer().to_owned(),
            source: "data/seed/intent-routing.lino + multilingual responses".to_owned(),
            when_then: format!(
                "When the user says `bye`, `goodbye`, or `пока` then respond with `{}`.",
                farewell_answer()
            ),
        },
        BehaviorRuleRecord {
            id: "rule_identity".to_owned(),
            topic: "identity",
            intent: "identity".to_owned(),
            label: "Identity rule".to_owned(),
            matches: "`Who are you?`, `Кто ты?`, and equivalent identity prompts".to_owned(),
            response: identity_answer().to_owned(),
            source: "data/seed/identity.lino + multilingual responses".to_owned(),
            when_then: format!(
                "When the user asks `Who are you?` or `Кто ты?` then respond with `{}`.",
                identity_answer()
            ),
        },
        BehaviorRuleRecord {
            id: "rule_assistant_name".to_owned(),
            topic: "assistant_name",
            intent: "assistant_name".to_owned(),
            label: "Assistant name rule".to_owned(),
            matches: "`What is your name?`, `Как тебя зовут?`, and equivalent name prompts"
                .to_owned(),
            response: "Returns the assistant-name answer; browser surfaces can override it from \
                 the assistant name setting."
                .to_owned(),
            source: "data/seed/intent-routing.lino + browser preferences".to_owned(),
            when_then: "When the user asks `What is your name?` or `Как тебя зовут?` then \
                 respond with the assistant-name answer; if a surface has an assistant-name \
                 setting, include that configured name."
                .to_owned(),
        },
        BehaviorRuleRecord {
            id: "rule_capabilities".to_owned(),
            topic: "capabilities",
            intent: "capabilities".to_owned(),
            label: "Capabilities rule".to_owned(),
            matches: "`What can you do?`, `Что ты умеешь?`, and equivalent capability prompts"
                .to_owned(),
            response: "Lists the supported symbolic chat capabilities.".to_owned(),
            source: "src/solver_handlers/user_intent.rs".to_owned(),
            when_then: "When the user asks `What can you do?` or `Что ты умеешь?` then \
                 respond with the multilingual capability listing."
                .to_owned(),
        },
    ];

    records.push(BehaviorRuleRecord {
        id: "rule_write_program".to_owned(),
        topic: "write_program",
        intent: "write_program".to_owned(),
        label: "Write-program rule".to_owned(),
        matches: format!(
            "`write_program(language, task)` with languages [{}] and tasks [{}]",
            supported_program_languages(),
            supported_program_tasks()
        ),
        response: "Returns a minimal program from the parameterized template catalog.".to_owned(),
        source: "data/seed/hello-world-programs.lino + src/coding/catalog/".to_owned(),
        when_then: "When the user requests a program with a supported `language` and `task`, \
             resolve those parameters and render the matching template through the single \
             `write_program` intent."
            .to_owned(),
    });

    records.push(BehaviorRuleRecord {
        id: "rule_unknown".to_owned(),
        topic: "unknown_fallback",
        intent: "unknown".to_owned(),
        label: "Unknown fallback rule".to_owned(),
        matches: "Any prompt that no earlier rule or handler can answer".to_owned(),
        response: unknown_answer().to_owned(),
        source: "data/seed/multilingual-responses.lino".to_owned(),
        when_then: "When no earlier rule or handler matches the prompt then respond with the \
             multilingual unknown-intent guide (`List behavior rules`, `Show behavior rule`, \
             `When I say … answer …`, `Report issue`, `Export memory`)."
            .to_owned(),
    });

    records
}

fn topic_order(topic: &str) -> u8 {
    match topic {
        "greetings" => 0,
        "farewells" => 1,
        "identity" => 2,
        "assistant_name" => 3,
        "capabilities" => 4,
        "write_program" => 5,
        "unknown_fallback" => 6,
        _ => 7,
    }
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

fn topic_label(topic: &str, language: &str) -> &'static str {
    match topic {
        "greetings" => localized_text(language, "Greetings", "Приветствия", "अभिवादन", "问候"),
        "farewells" => localized_text(language, "Farewells", "Прощания", "विदाई", "告别"),
        "identity" => localized_text(language, "Identity", "Идентичность", "पहचान", "身份"),
        "assistant_name" => localized_text(
            language,
            "Assistant name",
            "Имя ассистента",
            "सहायक का नाम",
            "助手名称",
        ),
        "capabilities" => localized_text(language, "Capabilities", "Возможности", "क्षमताएँ", "能力"),
        "write_program" => localized_text(
            language,
            "Program templates",
            "Шаблоны программ",
            "Program templates",
            "程序模板",
        ),
        "unknown_fallback" => localized_text(
            language,
            "Unknown fallback",
            "Резервный ответ",
            "अज्ञात अनुरोध का वैकल्पिक उत्तर",
            "未知请求回退",
        ),
        _ => localized_text(language, "Other", "Другое", "अन्य", "其他"),
    }
}

fn list_intro(language: &str) -> &'static str {
    localized_text(
        language,
        "Behavior rules I can inspect in this dialog (grouped by topic, each shown as a `When X then Y` statement):",
        "Правила поведения, которые я могу показать в этом диалоге (сгруппированы по темам; каждое показано как инструкция `Когда X тогда Y`):",
        "व्यवहार नियम जिन्हें मैं इस संवाद में दिखा सकता हूँ (विषय के अनुसार समूहित; हर नियम `जब X तब Y` कथन के रूप में है):",
        "我可以查看的行为规则（按主题分组；每条都显示为 `当 X 时 Y` 语句）：",
    )
}

fn render_behavior_rule_list(runtime_rules: &[CompiledSkillPackage], language: &str) -> String {
    let mut lines = vec![list_intro(language).to_owned(), String::new()];
    let mut grouped: BTreeMap<u8, (&'static str, Vec<BehaviorRuleRecord>)> = BTreeMap::new();
    for rule in behavior_rule_records() {
        let entry = grouped
            .entry(topic_order(rule.topic))
            .or_insert_with(|| (topic_label(rule.topic, language), Vec::new()));
        entry.1.push(rule);
    }
    let group_count = grouped.len();
    for (index, (_, (label, rules))) in grouped.into_iter().enumerate() {
        lines.push(format!("### {label}"));
        for rule in rules {
            lines.push(format!(
                "- `{}` -> {}",
                rule.id,
                rule_when_then(&rule, language)
            ));
        }
        if index + 1 < group_count {
            lines.push(String::new());
        }
    }
    if !runtime_rules.is_empty() {
        lines.extend([
            String::new(),
            format!("### {}", runtime_rules_heading(language)),
        ]);
        for rule in runtime_rules {
            lines.push(format!(
                "- `{}` (`{}`) -> {}",
                rule.id,
                rule.legacy_behavior_rule_id,
                runtime_rule_when_then(rule, language),
            ));
        }
    }
    lines.extend(rule_list_footer(language));
    lines.join("\n")
}

fn runtime_rules_heading(language: &str) -> &'static str {
    localized_text(
        language,
        "Dialog-local rules taught in this conversation",
        "Правила, изученные в этом диалоге",
        "इस संवाद में सिखाए गए स्थानीय नियम",
        "本对话中学到的局部规则",
    )
}

fn rule_list_footer(language: &str) -> Vec<String> {
    match language {
        "ru" => vec![
            String::new(),
            "Прочитать одно правило можно командой `Покажи правило unknown` или `Покажи правило rule_greeting`.".to_owned(),
            "Научить этот диалог можно так: ``Когда `ваш запрос` тогда `ваш ответ` ``. \
             Другие формы: ``Когда я скажу `ваш запрос`, ответь `ваш ответ` ``; \
             ``Если я спрошу `ваш запрос`, ответь `ваш ответ` ``; \
             ``Когда `ваш запрос` делай `ваш ответ` ``."
                .to_owned(),
            "Многоязычные формы: английская ``When `X` then `Y` ``, хинди ``जब `X` तब `Y` ``, китайская ``当 `X` 时 `Y` ``."
                .to_owned(),
            "Запись добавляется только в конец: экспортируйте память, чтобы сохранить сообщение с правилом вместе с диалогом."
                .to_owned(),
        ],
        "hi" => vec![
            String::new(),
            "एक नियम पढ़ने के लिए `Show behavior rule unknown` या `Show behavior rule rule_greeting` भेजें.".to_owned(),
            "इस संवाद को सिखाएँ: ``जब `आपका प्रश्न` तब `आपका उत्तर` ``. \
             अन्य रूप: ``When I say `your prompt`, answer `your answer` ``; \
             ``If I ask `your prompt`, reply `your answer` ``; \
             ``जब `आपका प्रश्न` तो `आपका उत्तर` ``."
                .to_owned(),
            "बहुभाषी रूप: रूसी ``Когда `X` тогда `Y` ``, अंग्रेज़ी ``When `X` then `Y` ``, चीनी ``当 `X` 时 `Y` ``."
                .to_owned(),
            "लेखन केवल append-only है: नियम संदेश को संवाद के साथ रखने के लिए memory export करें."
                .to_owned(),
        ],
        "zh" => vec![
            String::new(),
            "要读取一条规则，请发送 `Show behavior rule unknown` 或 `Show behavior rule rule_greeting`。".to_owned(),
            "可以这样教当前对话：``当 `你的提示` 时 `你的回答` ``。\
             等价形式：``When I say `your prompt`, answer `your answer` ``；\
             ``If I ask `your prompt`, reply `your answer` ``；\
             ``当 `你的提示` 则 `你的回答` ``。"
                .to_owned(),
            "多语言形式：俄语 ``Когда `X` тогда `Y` ``，印地语 ``जब `X` तब `Y` ``，英语 ``When `X` then `Y` ``。"
                .to_owned(),
            "写入是 append-only：导出 memory 可把这条规则消息随对话一起保存。".to_owned(),
        ],
        _ => vec![
            String::new(),
            "Read one with `Show behavior rule unknown` or `Show behavior rule rule_greeting`."
                .to_owned(),
            "Teach this dialog with: ``When `your prompt` then `your answer` ``. \
             Equivalent forms: ``When I say `your prompt`, answer `your answer` ``; \
             ``If I ask `your prompt`, reply `your answer` ``; \
             ``When `your prompt` do `your answer` ``."
                .to_owned(),
            "Multilingual forms: Russian ``Когда `X` тогда `Y` `` / \
             ``Когда `X` делай `Y` ``, Hindi ``जब `X` तब `Y` ``, Chinese ``当 `X` 时 `Y` ``."
                .to_owned(),
            "The write is append-only: export memory to preserve the rule message with the dialog."
                .to_owned(),
        ],
    }
}

fn localized_response(intent: &str, language: &str, fallback: &str) -> String {
    if language == "en" {
        return fallback.to_owned();
    }
    seed::response_for(intent, language).unwrap_or_else(|| fallback.to_owned())
}

fn rule_label(rule: &BehaviorRuleRecord, language: &str) -> String {
    if rule.id == "rule_write_program" {
        return match language {
            "ru" => "Правило write-program".to_owned(),
            "hi" => "Write-program नियम".to_owned(),
            "zh" => "Write-program 规则".to_owned(),
            _ => rule.label.clone(),
        };
    }

    let label = match rule.id.as_str() {
        "rule_greeting" => localized_text(
            language,
            "Greeting rule",
            "Правило приветствия",
            "अभिवादन नियम",
            "问候规则",
        ),
        "rule_farewell" => localized_text(
            language,
            "Farewell rule",
            "Правило прощания",
            "विदाई नियम",
            "告别规则",
        ),
        "rule_identity" => localized_text(
            language,
            "Identity rule",
            "Правило идентичности",
            "पहचान नियम",
            "身份规则",
        ),
        "rule_assistant_name" => localized_text(
            language,
            "Assistant name rule",
            "Правило имени ассистента",
            "सहायक नाम नियम",
            "助手名称规则",
        ),
        "rule_capabilities" => localized_text(
            language,
            "Capabilities rule",
            "Правило возможностей",
            "क्षमता नियम",
            "能力规则",
        ),
        "rule_unknown" => localized_text(
            language,
            "Unknown fallback rule",
            "Резервное правило для неизвестного запроса",
            "अज्ञात अनुरोध का वैकल्पिक नियम",
            "未知请求回退规则",
        ),
        _ => rule.label.as_str(),
    };
    label.to_owned()
}

fn rule_matches(rule: &BehaviorRuleRecord, language: &str) -> String {
    if rule.id == "rule_write_program" {
        return match language {
            "ru" => format!("Параметры `language` и `task`: {}", rule.matches),
            "hi" => format!("`language` और `task` parameter: {}", rule.matches),
            "zh" => format!("`language` 和 `task` 参数：{}", rule.matches),
            _ => rule.matches.clone(),
        };
    }

    match rule.id.as_str() {
        "rule_greeting" => localized_text(
            language,
            "`Hi`, `Hello`, `Hey`, and multilingual greeting seed phrases",
            "`Hi`, `Hello`, `Hey` и многоязычные seed-фразы приветствия",
            "`Hi`, `Hello`, `Hey` और बहुभाषी greeting seed phrases",
            "`Hi`、`Hello`、`Hey` 以及多语言问候 seed 短语",
        ),
        "rule_farewell" => localized_text(
            language,
            "`bye`, `goodbye`, `poka`, and multilingual farewell seed phrases",
            "`bye`, `goodbye`, `poka` и многоязычные seed-фразы прощания",
            "`bye`, `goodbye`, `poka` और बहुभाषी farewell seed phrases",
            "`bye`、`goodbye`、`poka` 以及多语言告别 seed 短语",
        ),
        "rule_identity" => localized_text(
            language,
            "`Who are you?`, `Кто ты?`, and equivalent identity prompts",
            "`Who are you?`, `Кто ты?` и равнозначные вопросы об идентичности",
            "`Who are you?`, `Кто ты?` और समान identity prompts",
            "`Who are you?`、`Кто ты?` 以及等价身份提示",
        ),
        "rule_assistant_name" => localized_text(
            language,
            "`What is your name?`, `Как тебя зовут?`, and equivalent name prompts",
            "`What is your name?`, `Как тебя зовут?` и равнозначные вопросы об имени",
            "`What is your name?`, `Как тебя зовут?` और समान name prompts",
            "`What is your name?`、`Как тебя зовут?` 以及等价名称提示",
        ),
        "rule_capabilities" => localized_text(
            language,
            "`What can you do?`, `Что ты умеешь?`, and equivalent capability prompts",
            "`What can you do?`, `Что ты умеешь?` и равнозначные вопросы о возможностях",
            "`What can you do?`, `Что ты умеешь?` और समान capability prompts",
            "`What can you do?`、`Что ты умеешь?` 以及等价能力提示",
        ),
        "rule_unknown" => localized_text(
            language,
            "Any prompt that no earlier rule or handler can answer",
            "Любой запрос, на который не ответило более раннее правило или обработчик",
            "कोई भी prompt जिसका उत्तर पहले का rule या handler नहीं दे सकता",
            "任何前面的规则或处理器无法回答的提示",
        ),
        _ => &rule.matches,
    }
    .to_owned()
}

fn rule_response(rule: &BehaviorRuleRecord, language: &str) -> String {
    if rule.id == "rule_write_program" {
        return match language {
            "ru" => "Возвращает минимальную программу из параметризованного каталога шаблонов."
                .to_owned(),
            "hi" => "Parameterized template catalog से न्यूनतम program लौटाता है.".to_owned(),
            "zh" => "从参数化模板目录返回一个最小程序。".to_owned(),
            _ => rule.response.clone(),
        };
    }

    match rule.id.as_str() {
        "rule_greeting" => localized_response("greeting", language, greeting_answer()),
        "rule_farewell" => localized_response("farewell", language, farewell_answer()),
        "rule_identity" => localized_response("identity", language, identity_answer()),
        "rule_assistant_name" => localized_text(
            language,
            "Returns the assistant-name answer; browser surfaces can override it from the assistant name setting.",
            "Возвращает ответ об имени ассистента; браузерные поверхности могут переопределить его настройкой имени ассистента.",
            "assistant-name उत्तर लौटाता है; browser surfaces assistant name setting से इसे बदल सकते हैं.",
            "返回助手名称回答；浏览器界面可通过助手名称设置覆盖它。",
        )
        .to_owned(),
        "rule_capabilities" => localized_text(
            language,
            "Lists the supported symbolic chat capabilities.",
            "Перечисляет поддерживаемые возможности символьного чата.",
            "समर्थित symbolic chat क्षमताओं को सूचीबद्ध करता है.",
            "列出支持的符号聊天能力。",
        )
        .to_owned(),
        "rule_unknown" => localized_response("unknown", language, unknown_answer()),
        _ => rule.response.clone(),
    }
}

fn rule_when_then(rule: &BehaviorRuleRecord, language: &str) -> String {
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

fn runtime_rule_when_then(rule: &CompiledSkillPackage, language: &str) -> String {
    match language {
        "ru" => format!(
            "Когда пользователь говорит `{}`, ответь `{}`.",
            rule.trigger, rule.response
        ),
        "hi" => format!(
            "जब उपयोगकर्ता `{}` कहे, तब `{}` उत्तर दें.",
            rule.trigger, rule.response
        ),
        "zh" => format!("当用户说 `{}` 时，回答 `{}`。", rule.trigger, rule.response),
        _ => format!(
            "When the user says `{}` then respond with `{}`.",
            rule.trigger, rule.response
        ),
    }
}

fn collect_runtime_rules(log: &EventLog) -> Vec<CompiledSkillPackage> {
    let mut seen = std::collections::HashSet::new();
    let mut rules = Vec::new();
    for event in log.events().iter().filter(|e| e.kind == "prior_turn:user") {
        if let Ok(rule) = compile_natural_language_skill(&event.payload) {
            if seen.insert(rule.id.clone()) {
                rules.push(rule);
            }
        }
    }
    rules
}

fn render_behavior_rule_detail(rule: &BehaviorRuleRecord, language: &str) -> String {
    let label = rule_label(rule, language);
    let when_then = rule_when_then(rule, language);
    let matches = rule_matches(rule, language);
    let response = rule_response(rule, language);
    let change_hint = match language {
        "ru" => "Чтобы изменить это поведение в текущем диалоге, отправьте: ``Когда `ваш запрос` тогда `ваш ответ` ``. Также можно: ``Когда я скажу `ваш запрос`, ответь `ваш ответ` ``.",
        "hi" => "इस व्यवहार को वर्तमान संवाद में बदलने के लिए भेजें: ``जब `आपका प्रश्न` तब `आपका उत्तर` ``. दूसरा रूप: ``When I say `your prompt`, answer `your answer` ``.",
        "zh" => "要在当前对话中改变此行为，请发送：``当 `你的提示` 时 `你的回答` ``。也可以发送：``When I say `your prompt`, answer `your answer` ``。",
        _ => "To change this behavior in the current dialog, send: ``When `your prompt` then `your answer` ``. Equivalent: ``When I say `your prompt`, answer `your answer` ``.",
    };
    format!(
        concat!(
            "{}\n\n",
            "{}\n\n",
            "```links\n",
            "{}\n",
            "  topic \"{}\"\n",
            "  intent \"{}\"\n",
            "  matches \"{}\"\n",
            "  response \"{}\"\n",
            "  source \"{}\"\n",
            "  when_then \"{}\"\n",
            "```\n\n",
            "{}"
        ),
        label,
        when_then,
        rule.id,
        escape_lino_value(rule.topic),
        escape_lino_value(&rule.intent),
        escape_lino_value(&matches),
        escape_lino_value(&response),
        escape_lino_value(&rule.source),
        escape_lino_value(&when_then),
        change_hint,
    )
}

fn render_runtime_rule_update(rule: &CompiledSkillPackage, language: &str) -> String {
    let when_then = runtime_rule_when_then(rule, language);
    let (title, send_hint) = match language {
        "ru" => (
            "Правило поведения скомпилировано для этого диалога.",
            format!(
                "Отправьте `{}` сейчас, и я отвечу настроенным ответом. Экспортируйте память, чтобы сохранить это правило вместе с диалогом.",
                rule.trigger
            ),
        ),
        "hi" => (
            "इस संवाद के लिए व्यवहार नियम compile किया गया.",
            format!(
                "`{}` अभी भेजें और मैं configured response से उत्तर दूँगा. इस rule message को dialog के साथ रखने के लिए memory export करें.",
                rule.trigger
            ),
        ),
        "zh" => (
            "已为本对话编译行为规则。",
            format!(
                "现在发送 `{}`，我会使用配置的回答。导出 memory 可把这条规则消息随对话一起保存。",
                rule.trigger
            ),
        ),
        _ => (
            "Behavior rule compiled for this dialog.",
            format!(
                "Send `{}` now and I will answer with the configured response. Export memory to keep this rule message with the dialog.",
                rule.trigger
            ),
        ),
    };
    format!(
        concat!(
            "{}\n\n",
            "{}\n\n",
            "```links\n",
            "{}\n",
            "  type \"compiled_skill_package\"\n",
            "  legacy_behavior_rule_id \"{}\"\n",
            "  match_prompt \"{}\"\n",
            "  answer \"{}\"\n",
            "  when_then \"{}\"\n",
            "  compiled_handler \"{}\"\n",
            "  replay_mode \"exact_normalized_prompt\"\n",
            "  source \"user_message\"\n",
            "```\n\n",
            "{}"
        ),
        title,
        when_then,
        rule.id,
        rule.legacy_behavior_rule_id,
        escape_lino_value(&rule.trigger),
        escape_lino_value(&rule.response),
        escape_lino_value(&when_then),
        rule.handler_id,
        send_hint,
    )
}

fn is_behavior_rules_list(normalized: &str) -> bool {
    matches_behavior_rules_list_seed_pattern(normalized)
        || mentions_behavior_rule_set_phrase(normalized)
        || is_supported_language_behavior_rules_list_query(normalized)
}

/// True when the prompt contains one of the fixed phrases that name the
/// behavior-rule set outright (role [`seed::ROLE_RULE_LISTING_PHRASE`]) — e.g.
/// the bare compound "behavior rules" / "行为规则" / "व्यवहार के नियम" recognised
/// as a list request without a separate enumerate verb. The phrases live in
/// `data/seed/meanings-behavior-rules.lino`, not in this code; raw `contains`
/// (not the token-bounded [`seed::Lexicon::mentions_role`]) preserves the
/// original substring match byte-for-byte.
fn mentions_behavior_rule_set_phrase(normalized: &str) -> bool {
    seed::lexicon()
        .words_for_role(seed::ROLE_RULE_LISTING_PHRASE)
        .iter()
        .any(|phrase| normalized.contains(phrase.as_str()))
}

fn matches_behavior_rules_list_seed_pattern(normalized: &str) -> bool {
    seed::prompt_patterns()
        .into_iter()
        .filter(|pattern| pattern.intent == "behavior_rules_list")
        .any(|pattern| {
            let text = normalize_prompt(&pattern.text);
            if text.is_empty() {
                return false;
            }
            match pattern.kind.as_str() {
                "keyword" | "phrase" => normalized == text || normalized.contains(&text),
                "prefix" => normalized.starts_with(&text),
                "suffix" => normalized.ends_with(&text),
                _ => false,
            }
        })
}

/// True when the prompt, within one supported language's vocabulary, names the
/// rule subject, asks to enumerate it, and scopes the request to the assistant's
/// own behavior. The three compositional dimensions are read from the meaning
/// lexicon (roles [`seed::ROLE_RULE_LISTING_SUBJECT`],
/// [`seed::ROLE_RULE_LISTING_REQUEST`], [`seed::ROLE_RULE_LISTING_SCOPE`]) rather
/// than hardcoded per-language word lists, so the English/Russian/Hindi/Chinese
/// coverage now lives entirely in `data/seed/meanings-behavior-rules.lino`.
///
/// The per-language AND is preserved: every dimension must be evidenced *within
/// the same language* ([`seed::Lexicon::words_for_role_in_languages`]), so an
/// English verb cannot satisfy a Russian-scoped query. Matching uses raw
/// `contains` to keep the legacy stem match (`правил` catching `правила`, `नियम`
/// catching `नियमों`) byte-for-byte; the language codes are the legitimate
/// code-resident bridge while every surface word stays in the seed.
fn is_supported_language_behavior_rules_list_query(normalized: &str) -> bool {
    let lexicon = seed::lexicon();
    let present = |role: &str, language: &str| {
        lexicon
            .words_for_role_in_languages(role, &[language])
            .iter()
            .any(|word| normalized.contains(word.as_str()))
    };
    ["en", "ru", "hi", "zh"].into_iter().any(|language| {
        present(seed::ROLE_RULE_LISTING_SUBJECT, language)
            && present(seed::ROLE_RULE_LISTING_REQUEST, language)
            && present(seed::ROLE_RULE_LISTING_SCOPE, language)
    })
}

fn detail_query(prompt: &str) -> Option<String> {
    let lower = prompt.to_lowercase();
    for prefix in [
        "show behavior rule",
        "read behavior rule",
        "describe behavior rule",
        "show rule",
        "read rule",
        "details for rule",
        "детали правила",
        "покажи правило",
        "прочитай правило",
    ] {
        if lower.starts_with(prefix) {
            let original_tail = prompt.get(prefix.len()..).unwrap_or_default();
            return Some(clean_rule_query(original_tail));
        }
    }
    if lower.contains("rule_unknown") {
        return Some("unknown".to_owned());
    }
    None
}

fn clean_rule_query(raw: &str) -> String {
    raw.trim()
        .trim_matches(|ch: char| {
            ch.is_whitespace()
                || matches!(
                    ch,
                    '`' | '"' | '\'' | ':' | '-' | '_' | '.' | ',' | '?' | '!'
                )
        })
        .to_lowercase()
}

fn find_behavior_rule(query: &str) -> Option<BehaviorRuleRecord> {
    let cleaned = clean_rule_query(query);
    let without_prefix = cleaned.strip_prefix("rule_").unwrap_or(&cleaned);
    behavior_rule_records().into_iter().find(|rule| {
        rule.id == cleaned
            || rule.id == format!("rule_{without_prefix}")
            || rule.intent == cleaned
            || rule.intent == without_prefix
            || rule.label.to_lowercase().contains(without_prefix)
    })
}

fn runtime_rule_for_prompt(
    prompt: &str,
    log: &EventLog,
) -> Option<(
    CompiledSkillPackage,
    crate::skill_compiler::CompiledSkillReplay,
)> {
    log.events()
        .iter()
        .rev()
        .filter(|event| event.kind == "prior_turn:user")
        .filter_map(|event| compile_natural_language_skill(&event.payload).ok())
        .find_map(|package| package.replay(prompt).map(|replay| (package, replay)))
}

fn escape_lino_value(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}
