//! Behavior-rule inspection tests.

use formal_ai::{ConversationTurn, FormalAiEngine, SymbolicAnswer, UniversalSolver};

fn answer(prompt: &str) -> SymbolicAnswer {
    FormalAiEngine.answer(prompt)
}

fn answer_with_behavior_rule_list_history(prompt: &str) -> SymbolicAnswer {
    let solver = UniversalSolver::default();
    let list = solver.solve("Show rules");
    let history = [
        ConversationTurn::user("Show rules"),
        ConversationTurn::assistant(list.answer),
    ];
    solver.solve_with_history(prompt, &history)
}

pub(super) const ENGLISH_RULE_LIST: &str = "Behavior rules I can inspect in this dialog (grouped by topic, each shown as a `When X then Y` statement):\n\n### Greetings\n- `rule_greeting` -> When the user says `Hi`, `Hello`, or `Hey` then respond with `Hi, how may I help you?`.\n\n### Farewells\n- `rule_farewell` -> When the user says `bye`, `goodbye`, or `пока` then respond with `Goodbye! Feel free to return any time.`.\n\n### Small talk\n- `rule_assistant_free_time` -> When the user asks what I do in free time then respond with `I do not have free time the way a person does. Between prompts I am idle; when the dialog is active, I help with tasks, rules, and explanations.`.\n\n### Identity\n- `rule_identity` -> When the user asks `Who are you?` or `Кто ты?` then respond with `I am formal-ai, a deterministic symbolic AI implementation that answers from local Links Notation rules and OpenAI-compatible API shapes. I do not perform neural inference in this demo.`.\n\n### Assistant name\n- `rule_assistant_name` -> When the user asks `What is your name?` or `Как тебя зовут?` then respond with the assistant-name answer; if a surface has an assistant-name setting, include that configured name.\n\n### Capabilities\n- `rule_capabilities` -> When the user asks `What can you do?` or `Что ты умеешь?` then respond with the multilingual capability listing.\n\n### Program templates\n- `rule_write_program` -> When the user requests a program with a supported `language` and `task`, resolve those parameters and render the matching template through the single `write_program` intent.\n\n### Unknown fallback\n- `rule_unknown` -> When no earlier rule or handler matches the prompt then respond with the multilingual unknown-intent guide (`List behavior rules`, `Show behavior rule`, `When I say … answer …`, `Report issue`, `Export memory`).\n\nRead one with `Show behavior rule unknown` or `Show behavior rule rule_greeting`.\nTeach this dialog with: ``When `your prompt` then `your answer` ``. Equivalent forms: ``When I say `your prompt`, answer `your answer` ``; ``If I ask `your prompt`, reply `your answer` ``; ``When `your prompt` do `your answer` ``.\nMultilingual forms: Russian ``Когда `X` тогда `Y` `` / ``Когда `X` делай `Y` ``, Hindi ``जब `X` तब `Y` ``, Chinese ``当 `X` 时 `Y` ``.\nThe write is append-only: export memory to preserve the rule message with the dialog.";
pub(super) const RUSSIAN_RULE_LIST: &str = "Правила поведения, которые я могу показать в этом диалоге (сгруппированы по темам; каждое показано как инструкция `Когда X тогда Y`):\n\n### Приветствия\n- `rule_greeting` -> Когда пользователь говорит `Hi`, `Hello`, `Hey` или многоязычную фразу приветствия, ответь `Здравствуйте! Чем могу помочь?`.\n\n### Прощания\n- `rule_farewell` -> Когда пользователь говорит `bye`, `goodbye`, `poka` или многоязычную фразу прощания, ответь `До свидания! Возвращайтесь в любое время.`.\n\n### Светская беседа\n- `rule_assistant_free_time` -> Когда пользователь спрашивает, что я делаю в свободное время, ответь `У меня нет свободного времени в человеческом смысле. Между запросами я бездействую; когда диалог активен, помогаю с задачами, правилами и объяснениями.`.\n\n### Идентичность\n- `rule_identity` -> Когда пользователь спрашивает `Who are you?` или `Кто ты?`, ответь `Я formal-ai — детерминированный символьный ИИ, который отвечает на основе локальных правил Links Notation и совместимых OpenAI-форматов. В этой демонстрации я не выполняю нейросетевой инференс.`.\n\n### Имя ассистента\n- `rule_assistant_name` -> Когда пользователь спрашивает `What is your name?` или `Как тебя зовут?`, ответь сообщением об имени ассистента; если поверхность поддерживает настройку имени, включи настроенное имя.\n\n### Возможности\n- `rule_capabilities` -> Когда пользователь спрашивает `What can you do?` или `Что ты умеешь?`, ответь многоязычным списком возможностей.\n\n### Шаблоны программ\n- `rule_write_program` -> Когда пользователь просит программу с поддерживаемыми параметрами `language` и `task`, выбери соответствующий шаблон через единое намерение `write_program`.\n\n### Резервный ответ\n- `rule_unknown` -> Когда ни одно более раннее правило или обработчик не подходит к запросу, ответь многоязычной подсказкой для неизвестного намерения (`Покажи правила`, `Покажи правило`, `Когда ... тогда ...`, `Сообщить о проблеме`, `Экспорт памяти`).\n\nПрочитать одно правило можно командой `Покажи правило unknown` или `Покажи правило rule_greeting`.\nНаучить этот диалог можно так: ``Когда `ваш запрос` тогда `ваш ответ` ``. Другие формы: ``Когда я скажу `ваш запрос`, ответь `ваш ответ` ``; ``Если я спрошу `ваш запрос`, ответь `ваш ответ` ``; ``Когда `ваш запрос` делай `ваш ответ` ``.\nМногоязычные формы: английская ``When `X` then `Y` ``, хинди ``जब `X` तब `Y` ``, китайская ``当 `X` 时 `Y` ``.\nЗапись добавляется только в конец: экспортируйте память, чтобы сохранить сообщение с правилом вместе с диалогом.";
pub(super) const HINDI_RULE_LIST: &str = "व्यवहार नियम जिन्हें मैं इस संवाद में दिखा सकता हूँ (विषय के अनुसार समूहित; हर नियम `जब X तब Y` कथन के रूप में है):\n\n### अभिवादन\n- `rule_greeting` -> जब उपयोगकर्ता `Hi`, `Hello`, `Hey` या बहुभाषी greeting phrase कहे, तब `नमस्ते! मैं आपकी क्या मदद कर सकता हूँ?` उत्तर दें.\n\n### विदाई\n- `rule_farewell` -> जब उपयोगकर्ता `bye`, `goodbye`, `poka` या बहुभाषी farewell phrase कहे, तब `अलविदा! जब भी ज़रूरत हो, वापस आएं।` उत्तर दें.\n\n### हल्की बातचीत\n- `rule_assistant_free_time` -> जब उपयोगकर्ता पूछे कि मैं खाली समय में क्या करता हूँ, तब `मेरे पास इंसानों जैसा खाली समय नहीं होता। संदेशों के बीच मैं निष्क्रिय रहता हूँ; संवाद सक्रिय हो तो मैं काम, नियम और व्याख्याओं में मदद करता हूँ।` उत्तर दें.\n\n### पहचान\n- `rule_identity` -> जब उपयोगकर्ता `Who are you?` या `Кто ты?` पूछे, तब `मैं formal-ai हूँ — एक नियतात्मक प्रतीकात्मक AI प्रणाली, जो स्थानीय Links Notation नियमों और OpenAI-संगत API आकारों से उत्तर देती है। इस डेमो में मैं कोई न्यूरल इन्फेरेन्स नहीं करता।` उत्तर दें.\n\n### सहायक का नाम\n- `rule_assistant_name` -> जब उपयोगकर्ता `What is your name?` या `Как тебя зовут?` पूछे, तब assistant-name उत्तर दें; अगर surface में assistant-name setting है, तो configured name शामिल करें.\n\n### क्षमताएँ\n- `rule_capabilities` -> जब उपयोगकर्ता `What can you do?` या `Что ты умеешь?` पूछे, तब बहुभाषी capability listing दें.\n\n### Program templates\n- `rule_write_program` -> जब उपयोगकर्ता supported `language` और `task` parameter वाला program माँगे, तब single `write_program` intent से matching template दें.\n\n### अज्ञात अनुरोध का वैकल्पिक उत्तर\n- `rule_unknown` -> जब कोई पहले का rule या handler prompt से मेल न खाए, तब unknown-intent guide दें (`नियम दिखाएँ`, `rule दिखाएँ`, `जब ... तब ...`, `Report issue`, `Export memory`).\n\nएक नियम पढ़ने के लिए `Show behavior rule unknown` या `Show behavior rule rule_greeting` भेजें.\nइस संवाद को सिखाएँ: ``जब `आपका प्रश्न` तब `आपका उत्तर` ``. अन्य रूप: ``When I say `your prompt`, answer `your answer` ``; ``If I ask `your prompt`, reply `your answer` ``; ``जब `आपका प्रश्न` तो `आपका उत्तर` ``.\nबहुभाषी रूप: रूसी ``Когда `X` тогда `Y` ``, अंग्रेज़ी ``When `X` then `Y` ``, चीनी ``当 `X` 时 `Y` ``.\nलेखन केवल append-only है: नियम संदेश को संवाद के साथ रखने के लिए memory export करें.";
pub(super) const CHINESE_RULE_LIST: &str = "我可以查看的行为规则（按主题分组；每条都显示为 `当 X 时 Y` 语句）：\n\n### 问候\n- `rule_greeting` -> 当用户说 `Hi`、`Hello`、`Hey` 或多语言问候短语时，回答 `你好!请问有什么可以帮您的?`。\n\n### 告别\n- `rule_farewell` -> 当用户说 `bye`、`goodbye`、`poka` 或多语言告别短语时，回答 `再见!随时欢迎回来。`。\n\n### 闲聊\n- `rule_assistant_free_time` -> 当用户问我空闲时间做什么时，回答 `我没有人类意义上的空闲时间。两次提问之间我处于空闲状态；对话开始后,我会帮助处理任务、规则和解释。`。\n\n### 身份\n- `rule_identity` -> 当用户问 `Who are you?` 或 `Кто ты?` 时，回答 `我是 formal-ai —— 一个确定性的符号化 AI 系统,根据本地的 Links Notation 规则和兼容 OpenAI 的 API 形式作答。本演示不进行任何神经网络推理。`。\n\n### 助手名称\n- `rule_assistant_name` -> 当用户问 `What is your name?` 或 `Как тебя зовут?` 时，回答助手名称；如果界面有助手名称设置，则包含配置的名称。\n\n### 能力\n- `rule_capabilities` -> 当用户问 `What can you do?` 或 `Что ты умеешь?` 时，回答多语言能力列表。\n\n### 程序模板\n- `rule_write_program` -> 当用户请求带受支持 `language` 和 `task` 参数的程序时，通过单个 `write_program` 意图选择匹配模板。\n\n### 未知请求回退\n- `rule_unknown` -> 当前面的规则或处理器都不匹配提示时，回答未知意图指南（`显示规则`、`显示规则详情`、`当 ... 时 ...`、`报告问题`、`导出 memory`）。\n\n要读取一条规则，请发送 `Show behavior rule unknown` 或 `Show behavior rule rule_greeting`。\n可以这样教当前对话：``当 `你的提示` 时 `你的回答` ``。等价形式：``When I say `your prompt`, answer `your answer` ``；``If I ask `your prompt`, reply `your answer` ``；``当 `你的提示` 则 `你的回答` ``。\n多语言形式：俄语 ``Когда `X` тогда `Y` ``，印地语 ``जब `X` तब `Y` ``，英语 ``When `X` then `Y` ``。\n写入是 append-only：导出 memory 可把这条规则消息随对话一起保存。";

fn documented_rule_list(language: &str) -> &'static str {
    match language {
        "ru" => RUSSIAN_RULE_LIST,
        "hi" => HINDI_RULE_LIST,
        "zh" => CHINESE_RULE_LIST,
        _ => ENGLISH_RULE_LIST,
    }
}

struct PromptCase {
    language: &'static str,
    prompt: &'static str,
}

struct LocalizedListCase {
    language: &'static str,
    prompt: &'static str,
    expected: &'static str,
    rejected: &'static str,
}

#[test]
fn behavior_rules_list_possessive_list_phrase_covers_supported_languages() {
    let cases = [
        PromptCase {
            language: "en",
            prompt: "Show list of your rules",
        },
        PromptCase {
            language: "ru",
            prompt: "Покажи список своих правил",
        },
        PromptCase {
            language: "hi",
            prompt: "अपने नियमों की सूची दिखाओ",
        },
        PromptCase {
            language: "zh",
            prompt: "显示你的规则列表",
        },
    ];
    let supported_languages = formal_ai::supported_languages();

    for language in supported_languages.iter().map(String::as_str) {
        assert!(
            cases.iter().any(|case| case.language == language),
            "missing behavior_rules_list possessive-list regression case for supported language {language}"
        );
    }

    for case in cases {
        let response = answer(case.prompt);
        assert_eq!(
            response.intent, "behavior_rules_list",
            "expected behavior_rules_list for {} prompt {:?}, got {}",
            case.language, case.prompt, response.intent
        );
        assert_eq!(response.answer, documented_rule_list(case.language));
        assert!(response.answer.contains("rule_greeting"));
        assert!(response.answer.contains("rule_unknown"));
    }
}

#[test]
fn behavior_rules_short_list_phrase_covers_supported_languages() {
    let cases = [
        PromptCase {
            language: "en",
            prompt: "Show rules",
        },
        PromptCase {
            language: "ru",
            prompt: "Покажи правила",
        },
        PromptCase {
            language: "hi",
            prompt: "नियम दिखाओ",
        },
        PromptCase {
            language: "zh",
            prompt: "显示规则",
        },
    ];
    let supported_languages = formal_ai::supported_languages();

    for language in supported_languages.iter().map(String::as_str) {
        assert!(
            cases.iter().any(|case| case.language == language),
            "missing behavior_rules_list short-rule-list regression case for supported language {language}"
        );
    }

    for case in cases {
        let response = answer(case.prompt);
        assert_eq!(
            response.intent, "behavior_rules_list",
            "expected behavior_rules_list for {} prompt {:?}, got {}",
            case.language, case.prompt, response.intent
        );
        assert_eq!(response.answer, documented_rule_list(case.language));
        assert!(response.answer.contains("rule_greeting"));
        assert!(response.answer.contains("rule_unknown"));
    }
}

#[test]
fn behavior_rules_count_followup_covers_supported_languages() {
    let cases = [
        PromptCase {
            language: "en",
            prompt: "How many rules are there?",
        },
        PromptCase {
            language: "ru",
            prompt: "Сколько всего правил?",
        },
        PromptCase {
            language: "hi",
            prompt: "कुल कितने नियम हैं?",
        },
        PromptCase {
            language: "zh",
            prompt: "一共有多少规则?",
        },
    ];
    let supported_languages = formal_ai::supported_languages();

    for language in supported_languages.iter().map(String::as_str) {
        assert!(
            cases.iter().any(|case| case.language == language),
            "missing behavior_rules_count follow-up regression case for supported language {language}"
        );
    }

    for case in cases {
        let response = answer_with_behavior_rule_list_history(case.prompt);
        assert_eq!(
            response.intent, "behavior_rules_count",
            "expected behavior_rules_count for {} prompt {:?}, got {}: {}",
            case.language, case.prompt, response.intent, response.answer
        );
        let expected_answer = match case.language {
            "ru" => {
                "Всего правил: 8 (встроенных: 8; изученных в этом диалоге: 0).\n\nРассуждение: я считаю встроенный каталог правил поведения и добавляю правила, скомпилированные из предыдущих сообщений пользователя.\n\n```links\nbehavior_rules_count\n  built_in_rules \"8\"\n  dialog_local_rules \"0\"\n  total_rules \"8\"\n  algorithm \"behavior_rule_records + collect_runtime_rules(prior_turn:user)\"\n```\n"
            }
            "hi" => {
                "कुल व्यवहार नियम: 8 (built-in: 8; dialog-local: 0).\n\nReasoning: मैं built-in behavior-rule catalog गिनता हूँ और पहले user turns से compiled dialog-local rules जोड़ता हूँ.\n\n```links\nbehavior_rules_count\n  built_in_rules \"8\"\n  dialog_local_rules \"0\"\n  total_rules \"8\"\n  algorithm \"behavior_rule_records + collect_runtime_rules(prior_turn:user)\"\n```\n"
            }
            "zh" => {
                "行为规则总数：8（内置：8；本对话：0）。\n\nReasoning：我统计内置行为规则目录，并加上从此前用户消息编译出的本对话规则。\n\n```links\nbehavior_rules_count\n  built_in_rules \"8\"\n  dialog_local_rules \"0\"\n  total_rules \"8\"\n  algorithm \"behavior_rule_records + collect_runtime_rules(prior_turn:user)\"\n```\n"
            }
            _ => {
                "Total behavior rules: 8 (built-in: 8; dialog-local: 0).\n\nReasoning: I count the built-in behavior-rule catalog and add dialog-local rules compiled from earlier user turns.\n\n```links\nbehavior_rules_count\n  built_in_rules \"8\"\n  dialog_local_rules \"0\"\n  total_rules \"8\"\n  algorithm \"behavior_rule_records + collect_runtime_rules(prior_turn:user)\"\n```\n"
            }
        };
        assert_eq!(response.answer, expected_answer);
        assert!(
            response.answer.contains('8'),
            "{} behavior-rule count should include the current built-in rule count, got: {}",
            case.language,
            response.answer
        );
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link == "response:behavior_rules_count"),
            "{} behavior-rule count should expose the response link, got: {:?}",
            case.language,
            response.evidence_links
        );
    }
}

#[test]
fn behavior_rules_brief_language_followup_uses_previous_rule_list() {
    let response = answer_with_behavior_rule_list_history("А по русски кратко?");
    assert_eq!(
        response.intent, "behavior_rules_brief",
        "expected behavior_rules_brief, got {}: {}",
        response.intent, response.answer
    );
    assert_eq!(
        response.answer,
        "Всего: 8 правил поведения (8 встроенных, 0 из диалога). Кратко: приветствия, прощания, светская беседа, идентичность, имя ассистента, возможности, шаблоны программ и резервный ответ."
    );
    assert!(
        response.answer.contains("Всего") && response.answer.contains('8'),
        "brief Russian follow-up should summarize the current rule count in Russian, got: {}",
        response.answer
    );
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link == "response:behavior_rules_brief"),
        "brief follow-up should expose the response link, got: {:?}",
        response.evidence_links
    );
}

#[test]
fn behavior_rules_list_answer_is_localized_for_supported_languages() {
    let cases = [
        LocalizedListCase {
            language: "en",
            prompt: "Show list of your rules",
            expected: "Behavior rules I can inspect",
            rejected: "Правила поведения",
        },
        LocalizedListCase {
            language: "ru",
            prompt: "Перечисли свои правила",
            expected: "Правила поведения, которые я могу показать",
            rejected: "Behavior rules I can inspect",
        },
        LocalizedListCase {
            language: "hi",
            prompt: "अपने नियमों की सूची दिखाओ",
            expected: "व्यवहार नियम जिन्हें मैं इस संवाद में दिखा सकता हूँ",
            rejected: "Behavior rules I can inspect",
        },
        LocalizedListCase {
            language: "zh",
            prompt: "显示你的规则列表",
            expected: "我可以查看的行为规则",
            rejected: "Behavior rules I can inspect",
        },
    ];
    let supported_languages = formal_ai::supported_languages();

    for language in supported_languages.iter().map(String::as_str) {
        assert!(
            cases.iter().any(|case| case.language == language),
            "missing localized behavior_rules_list regression case for supported language {language}"
        );
    }

    for case in cases {
        let response = answer(case.prompt);
        assert_eq!(
            response.intent, "behavior_rules_list",
            "expected behavior_rules_list for {} prompt {:?}, got {}",
            case.language, case.prompt, response.intent
        );
        assert_eq!(response.answer, documented_rule_list(case.language));
        assert!(
            response.answer.contains(case.expected),
            "{} behavior-rule list should contain localized text {:?}, got: {}",
            case.language,
            case.expected,
            response.answer
        );
        assert!(
            !response.answer.contains(case.rejected),
            "{} behavior-rule list should not use the rejected language marker {:?}, got: {}",
            case.language,
            case.rejected,
            response.answer
        );
        assert!(
            !response.answer.contains("\\`"),
            "{} behavior-rule list should not emit escaped backticks that break inline markdown, got: {}",
            case.language,
            response.answer
        );
    }
}

#[test]
fn behavior_rules_count_followup_answers_reported_russian_prompt() {
    let solver = UniversalSolver::default();
    let list = answer("Покажи правила");
    assert_eq!(list.intent, "behavior_rules_list");

    let response = solver.solve_with_history(
        "Сколько всего правил?",
        &[
            ConversationTurn::user("Покажи правила"),
            ConversationTurn::assistant(list.answer),
        ],
    );

    assert_eq!(response.intent, "behavior_rules_count");
    assert!(response.answer.contains("Всего правил"));
    assert!(response.answer.contains("total_rules \"8\""));
    assert!(!response.answer.contains("Мне не удалось тебя понять"));
}

#[test]
fn behavior_rules_count_query_covers_supported_languages() {
    let cases = [
        LocalizedListCase {
            language: "en",
            prompt: "How many behavior rules are there?",
            expected: "Total behavior rules: 8",
            rejected: "Всего правил",
        },
        LocalizedListCase {
            language: "ru",
            prompt: "Сколько всего правил?",
            expected: "Всего правил: 8",
            rejected: "Total behavior rules",
        },
        LocalizedListCase {
            language: "hi",
            prompt: "कुल कितने नियम हैं?",
            expected: "कुल व्यवहार नियम: 8",
            rejected: "Total behavior rules",
        },
        LocalizedListCase {
            language: "zh",
            prompt: "总共有多少规则?",
            expected: "行为规则总数：8",
            rejected: "Total behavior rules",
        },
    ];
    let supported_languages = formal_ai::supported_languages();

    for language in supported_languages.iter().map(String::as_str) {
        assert!(
            cases.iter().any(|case| case.language == language),
            "missing behavior_rules_count regression case for supported language {language}"
        );
    }

    for case in cases {
        let response = answer(case.prompt);
        assert_eq!(
            response.intent, "behavior_rules_count",
            "expected behavior_rules_count for {} prompt {:?}, got {}",
            case.language, case.prompt, response.intent
        );
        let expected_answer = match case.language {
            "ru" => {
                "Всего правил: 8 (встроенных: 8; изученных в этом диалоге: 0).\n\nРассуждение: я считаю встроенный каталог правил поведения и добавляю правила, скомпилированные из предыдущих сообщений пользователя.\n\n```links\nbehavior_rules_count\n  built_in_rules \"8\"\n  dialog_local_rules \"0\"\n  total_rules \"8\"\n  algorithm \"behavior_rule_records + collect_runtime_rules(prior_turn:user)\"\n```\n"
            }
            "hi" => {
                "कुल व्यवहार नियम: 8 (built-in: 8; dialog-local: 0).\n\nReasoning: मैं built-in behavior-rule catalog गिनता हूँ और पहले user turns से compiled dialog-local rules जोड़ता हूँ.\n\n```links\nbehavior_rules_count\n  built_in_rules \"8\"\n  dialog_local_rules \"0\"\n  total_rules \"8\"\n  algorithm \"behavior_rule_records + collect_runtime_rules(prior_turn:user)\"\n```\n"
            }
            "zh" => {
                "行为规则总数：8（内置：8；本对话：0）。\n\nReasoning：我统计内置行为规则目录，并加上从此前用户消息编译出的本对话规则。\n\n```links\nbehavior_rules_count\n  built_in_rules \"8\"\n  dialog_local_rules \"0\"\n  total_rules \"8\"\n  algorithm \"behavior_rule_records + collect_runtime_rules(prior_turn:user)\"\n```\n"
            }
            _ => {
                "Total behavior rules: 8 (built-in: 8; dialog-local: 0).\n\nReasoning: I count the built-in behavior-rule catalog and add dialog-local rules compiled from earlier user turns.\n\n```links\nbehavior_rules_count\n  built_in_rules \"8\"\n  dialog_local_rules \"0\"\n  total_rules \"8\"\n  algorithm \"behavior_rule_records + collect_runtime_rules(prior_turn:user)\"\n```\n"
            }
        };
        assert_eq!(response.answer, expected_answer);
        assert!(
            response.answer.contains(case.expected),
            "{} behavior-rule count should contain localized text {:?}, got: {}",
            case.language,
            case.expected,
            response.answer
        );
        assert!(
            !response.answer.contains(case.rejected),
            "{} behavior-rule count should not use the rejected language marker {:?}, got: {}",
            case.language,
            case.rejected,
            response.answer
        );
        assert!(response.answer.contains("built_in_rules \"8\""));
        assert!(response.answer.contains("dialog_local_rules \"0\""));
        assert!(response.answer.contains("total_rules \"8\""));
    }
}

#[test]
fn behavior_rules_count_includes_dialog_local_runtime_rules() {
    let solver = UniversalSolver::default();
    let history = [ConversationTurn::user(
        "When `synthetic-prompt` then `synthetic-answer`.",
    )];

    let response = solver.solve_with_history("How many behavior rules are there?", &history);

    assert_eq!(response.intent, "behavior_rules_count");
    assert_eq!(
        response.answer,
        "Total behavior rules: 9 (built-in: 8; dialog-local: 1).\n\nReasoning: I count the built-in behavior-rule catalog and add dialog-local rules compiled from earlier user turns.\n\n```links\nbehavior_rules_count\n  built_in_rules \"8\"\n  dialog_local_rules \"1\"\n  total_rules \"9\"\n  algorithm \"behavior_rule_records + collect_runtime_rules(prior_turn:user)\"\n```\n"
    );
    assert!(response.answer.contains("Total behavior rules: 9"));
    assert!(response.answer.contains("built_in_rules \"8\""));
    assert!(response.answer.contains("dialog_local_rules \"1\""));
    assert!(response.answer.contains("total_rules \"9\""));
}

#[test]
fn behavior_rule_detail_answer_is_localized_for_russian() {
    let response = answer("Покажи правило unknown");
    assert_eq!(response.intent, "behavior_rule_detail");
    assert_eq!(
        response.answer,
        "Резервное правило для неизвестного запроса\n\nКогда ни одно более раннее правило или обработчик не подходит к запросу, ответь многоязычной подсказкой для неизвестного намерения (`Покажи правила`, `Покажи правило`, `Когда ... тогда ...`, `Сообщить о проблеме`, `Экспорт памяти`).\n\n```links\nrule_unknown\n  topic \"unknown_fallback\"\n  intent \"unknown\"\n  matches \"Любой запрос, на который не ответило более раннее правило или обработчик\"\n  response \"Я пока не знаю, как ответить на это. Я пока не могу ответить на это по локальным правилам связей. Чтобы посмотреть текущие правила, отправьте `Покажи правила поведения`, затем `Покажи правило unknown`. Чтобы научить этот диалог ответу, отправьте: Когда я скажу `ваш запрос`, ответь `ваш ответ`. Если после этих проверок всё ещё нужен общий seed-факт или правило связей в формате Links Notation, сообщите о недостающем правиле с диагностической трассировкой или экспортируйте память, чтобы сохранить правило этого диалога.\"\n  source \"data/seed/multilingual-responses.lino\"\n  when_then \"Когда ни одно более раннее правило или обработчик не подходит к запросу, ответь многоязычной подсказкой для неизвестного намерения (`Покажи правила`, `Покажи правило`, `Когда ... тогда ...`, `Сообщить о проблеме`, `Экспорт памяти`).\"\n```\n\nЧтобы изменить это поведение в текущем диалоге, отправьте: ``Когда `ваш запрос` тогда `ваш ответ` ``. Также можно: ``Когда я скажу `ваш запрос`, ответь `ваш ответ` ``."
    );
    assert!(response.answer.contains("Резервное правило"));
    assert!(!response.answer.contains("Unknown fallback rule"));
    assert!(!response.answer.contains("To change this behavior"));
}
