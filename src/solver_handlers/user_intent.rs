//! Handlers for user-intent clarification, capability queries, follow-up
//! elaboration, ill-formed input, and shell-refusal policy. Extracted from
//! `solver_handlers/mod.rs` to keep individual files under 1000 lines.

use crate::concepts::extract_concept_query;
use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::language::detect as detect_language;
use crate::seed::response_for;
use crate::solver_handlers::finalize_simple;

pub fn try_clarification(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let is_clarification = normalized == "не понял"
        || normalized == "не понимаю"
        || normalized == "не поняла"
        || normalized == "не понятно"
        || normalized == "непонятно"
        || normalized.contains("i don't understand")
        || normalized.contains("i dont understand")
        || normalized.contains("i didn't understand")
        || normalized.contains("i didnt understand")
        || normalized.contains("don't understand")
        || normalized.contains("dont understand")
        || normalized.contains("didn't understand")
        || normalized.contains("didnt understand")
        || normalized.contains("what do you mean")
        || normalized.contains("i'm confused")
        || normalized.contains("im confused")
        || normalized.contains("i am confused")
        || normalized.contains("समझ नहीं आया")
        || normalized.contains("समझ नहीं आई")
        || normalized.contains("我不明白")
        || normalized.contains("我不懂")
        || normalized.contains("听不懂");
    if !is_clarification {
        return None;
    }
    let language = detect_language(prompt);
    let body = response_for("clarification", language.slug())
        .or_else(|| response_for("clarification", "en"))
        .unwrap_or_else(|| {
            String::from(
                "I'm sorry for the confusion. I am formal-ai, a deterministic symbolic AI. \
                 I can answer greetings, identity questions, concept lookups (\"what is X?\"), \
                 arithmetic, and Hello World programs.",
            )
        });
    Some(finalize_simple(
        prompt,
        log,
        "clarification",
        "response:clarification",
        &body,
        0.9,
    ))
}

pub fn try_punctuation_only_prompt(
    prompt: &str,
    _normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let trimmed = prompt.trim();
    let sentence_marks = ['.', '?', '!', '…', '。', '？', '！'];
    let is_punctuation_only =
        !trimmed.is_empty() && trimmed.chars().all(|ch| sentence_marks.contains(&ch));
    if !is_punctuation_only {
        return None;
    }
    log.append("clarification:punctuation_only", trimmed.to_owned());
    let body =
        format!("I received only punctuation (`{trimmed}`). What would you like me to do next?");
    Some(finalize_simple(
        prompt,
        log,
        "clarification",
        "response:clarification",
        &body,
        0.8,
    ))
}

pub fn try_ill_formed(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    if !normalized.contains("teach this fact") {
        return None;
    }
    let opens = prompt.chars().filter(|c| *c == '(').count();
    let closes = prompt.chars().filter(|c| *c == ')').count();
    if opens == closes {
        return None;
    }
    log.append("error", "unbalanced links notation".to_owned());
    let body = String::from(crate::engine::unknown_answer());
    Some(finalize_simple(
        prompt,
        log,
        "unknown",
        "response:unknown",
        &body,
        0.0,
    ))
}

fn is_more_capabilities_prompt(normalized: &str, language: &str) -> bool {
    match language {
        "ru" => {
            normalized.contains("что ещё ты умеешь")
                || normalized.contains("что еще ты умеешь")
                || normalized.contains("что ещё можешь")
                || normalized.contains("что еще можешь")
                || normalized.contains("что ты ещё умеешь")
                || normalized.contains("что ты еще умеешь")
        }
        _ => {
            normalized.contains("what else can you do")
                || normalized.contains("what else do you do")
                || normalized.contains("what other things can you do")
        }
    }
}

fn prior_history_mentions_web_search(log: &EventLog) -> bool {
    log.events()
        .iter()
        .filter(|event| event.kind == "prior_turn:user" || event.kind == "prior_turn:assistant")
        .any(|event| {
            let payload = event.payload.to_lowercase();
            payload.contains("duckduckgo")
                || payload.contains("web search")
                || payload.contains("search the internet")
                || payload.contains("веб-поиск")
                || payload.contains("веб поиск")
                || payload.contains("интернет")
        })
}

fn additional_capabilities_body_ru() -> String {
    String::from(
        "Кроме уже названных возможностей, могу ещё:\n\
         \n\
         - **Арифметика**: вычислять выражения вроде «Сколько будет 2 + 2?»\n\
         - **Перевод**: переводить короткие фразы между поддерживаемыми языками.\n\
         - **Поиск понятий**: объяснять термины, например «Что такое Википедия?»\n\
         - **Hello World**: генерировать минимальные программы на Rust, Python, JavaScript, Go, C и других языках.\n\
         - **Память диалога**: использовать предыдущие сообщения текущей сессии.\n\
         - **Правила поведения**: показывать встроенные правила через `List behavior rules` и `Show behavior rule unknown`.\n\
         - **Настройки и действия**: включать диагностику/демо/agent mode, менять тему, язык, стиль чата, экспортировать и импортировать память.",
    )
}

fn additional_capabilities_body_en() -> String {
    String::from(
        "Beyond the capability already discussed, I can also:\n\
         \n\
         - **Arithmetic**: evaluate expressions like `2 + 2`.\n\
         - **Translation**: translate short phrases between supported languages.\n\
         - **Concept lookup**: explain terms such as `What is Wikipedia?`.\n\
         - **Hello World**: generate small programs in Rust, Python, JavaScript, Go, C, and more.\n\
         - **Conversation memory**: use earlier messages from the current session.\n\
         - **Behavior rules**: show built-in rules with `List behavior rules` and `Show behavior rule unknown`.\n\
         - **Settings and actions**: configure diagnostics, demo mode, agent mode, theme, language, chat style, and memory import/export.",
    )
}

pub fn try_capabilities(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let language = detect_language(prompt);
    let more_capabilities = is_more_capabilities_prompt(normalized, language.slug());
    let is_capabilities = match language.slug() {
        "ru" => {
            more_capabilities
                || normalized.contains("что ты умеешь")
                || normalized.contains("чем ты можешь")
                || normalized.contains("что ты можешь")
                || normalized.contains("что умеет")
                || normalized.contains("что можешь")
                || normalized.contains("твои возможности")
                || normalized.contains("что за дичь")
                || normalized.contains("что это такое")
                || normalized.contains("что происходит")
                || normalized.contains("что ты делаешь")
        }
        "zh" => {
            normalized.contains("你能做什么")
                || normalized.contains("你会做什么")
                || normalized.contains("你有什么功能")
                || normalized.contains("你能干什么")
        }
        "hi" => {
            normalized.contains("आप क्या कर सकते")
                || normalized.contains("तुम क्या कर सकते")
                || normalized.contains("क्या क्या कर सकते")
        }
        _ => {
            more_capabilities
                || normalized.contains("what can you do")
                || normalized.contains("what you can do")
                || normalized.contains("what are your capabilities")
                || normalized.contains("what are you capable of")
                || normalized.contains("what do you do")
                || normalized.contains("show me what you can do")
                || normalized.contains("what features do you have")
                || normalized.contains("how can you help")
                || normalized.contains("what are your features")
        }
    };
    if !is_capabilities {
        return None;
    }
    if more_capabilities {
        if prior_history_mentions_web_search(log) {
            log.append("capabilities:history", "prior_web_search".to_owned());
        }
        let body = if language.slug() == "ru" {
            additional_capabilities_body_ru()
        } else {
            additional_capabilities_body_en()
        };
        return Some(finalize_simple(
            prompt,
            log,
            "capabilities",
            "response:capabilities",
            &body,
            1.0,
        ));
    }
    let body = match language.slug() {
        "ru" => String::from(
            "Я formal-ai — детерминированный символьный ИИ. Вот что я умею:\n\
             \n\
             - **Приветствия**: отвечаю на «Привет», «Здравствуйте» и т.п.\n\
             - **Hello World**: генерирую программы на Rust, Python, JavaScript, Go, C и других языках.\n\
             - **Веб-поиск**: ищу в интернете через DuckDuckGo, Wikipedia и Wikidata, когда поиск доступен.\n\
             - **Поиск понятий**: объясняю термины — попробуйте «Что такое Википедия?»\n\
             - **Арифметика**: вычисляю выражения — например, «Сколько будет 2 + 2?»\n\
             - **Перевод**: перевожу фразы между языками.\n\
             - **Память**: помню контекст разговора в рамках сессии.\n\
             - **Правила поведения**: отправьте `List behavior rules`, чтобы увидеть встроенные правила, и `Show behavior rule unknown`, чтобы прочитать одно правило.\n\
             - **Обучение в диалоге**: отправьте «When I say \\`ваш запрос\\`, answer \\`ваш ответ\\`», чтобы добавить правило, действующее только в этом диалоге.\n\
             - **Факты о себе**: отправьте `List all facts you know about yourself`, чтобы увидеть, что я знаю о себе.\n\
             - **Сообщение об ошибке**: используйте кнопку «Report issue» сверху или ссылку на странице сообщения, чтобы попросить разработчиков добавить встроенное правило.\n\
             - **Настройки и действия**: через сообщения можно включать диагностику/демо/agent mode, менять тему, язык, стиль чата и экспортировать или импортировать память.\n\
             \n\
             Я работаю на основе локальных символьных правил, без нейросетевого инференса.",
        ),
        "zh" => String::from(
            "我是 formal-ai —— 一个确定性的符号化 AI。以下是我的功能：\n\
             \n\
             - **问候**：回应「你好」等问候语。\n\
             - **Hello World**：生成 Rust、Python、JavaScript、Go、C 等语言的示例程序。\n\
             - **Web search**：在可用时通过 DuckDuckGo、Wikipedia 和 Wikidata 搜索互联网。\n\
             - **概念查找**：解释术语，例如「什么是维基百科？」\n\
             - **算术**：计算表达式，例如「2 + 2 等于多少？」\n\
             - **翻译**：在语言之间翻译短语。\n\
             - **记忆**：在会话中记住上下文。\n\
             - **行为规则**：发送 `List behavior rules` 查看内置规则，并发送 `Show behavior rule unknown` 阅读某条规则。\n\
             - **对话内教学**：发送「When I say \\`prompt\\`, answer \\`answer\\`」可以在本轮对话中添加一条本地规则。\n\
             - **自我事实**：发送 `List all facts you know about yourself` 查看我知道的关于自己的事实。\n\
             - **问题反馈**：使用顶部的 「Report issue」按钮或消息中的链接,请开发者把规则加入种子文件。\n\
             - **设置和操作**：可通过消息开启诊断、演示、agent mode，切换主题、语言、聊天样式，并导出或导入记忆。\n\
             \n\
             我基于本地符号规则运行，不进行神经网络推理。",
        ),
        "hi" => String::from(
            "मैं formal-ai हूँ — एक नियतात्मक प्रतीकात्मक AI। मैं यह कर सकता हूँ:\n\
             \n\
             - **अभिवादन**: «नमस्ते» आदि का जवाब देना।\n\
             - **Hello World**: Rust, Python, JavaScript, Go, C आदि में प्रोग्राम बनाना।\n\
             - **Web search**: उपलब्ध होने पर DuckDuckGo, Wikipedia, और Wikidata से इंटरनेट में खोजना।\n\
             - **अवधारणा खोज**: शब्दों को समझाना — जैसे «विकिपीडिया क्या है?»\n\
             - **अंकगणित**: गणनाएँ — जैसे «2 + 2 क्या है?»\n\
             - **अनुवाद**: भाषाओं के बीच अनुवाद।\n\
             - **स्मृति**: सत्र में संदर्भ याद रखना।\n\
             - **व्यवहार नियम**: `List behavior rules` भेजकर अंतर्निहित नियम देखें और `Show behavior rule unknown` से कोई नियम पढ़ें।\n\
             - **संवाद-स्तर पर सिखाना**: «When I say \\`prompt\\`, answer \\`answer\\`» भेजकर इस संवाद के लिए स्थानीय नियम जोड़ें।\n\
             - **स्व-तथ्य**: `List all facts you know about yourself` भेजें ताकि मैं अपने बारे में जो जानता हूँ वह सूचीबद्ध करूँ।\n\
             - **समस्या रिपोर्ट**: ऊपर के «Report issue» बटन या मैसेज लिंक का उपयोग करके डेवलपर्स से built-in नियम जोड़वा सकते हैं।\n\
             - **Settings और actions**: messages से diagnostics/demo/agent mode बदलना, theme/language/chat style बदलना, और memory export/import करना।\n\
             \n\
             मैं स्थानीय प्रतीकात्मक नियमों पर चलता हूँ, कोई न्यूरल इन्फेरेन्स नहीं।",
        ),
        _ => String::from(
            "I am formal-ai, a deterministic symbolic AI. Here is what I can do:\n\
             \n\
             - **Greetings**: respond to «Hi», «Hello», and similar.\n\
             - **Hello World**: generate programs in Rust, Python, JavaScript, Go, C, and more.\n\
             - **Web search**: search the internet through DuckDuckGo, Wikipedia, and Wikidata when available.\n\
             - **Concept lookup**: explain terms — try «What is Wikipedia?»\n\
             - **Arithmetic**: evaluate expressions — try «What is 2 + 2?»\n\
             - **Translation**: translate phrases between languages.\n\
             - **Memory**: recall context within the current session.\n\
             - **Behavior rules**: send `List behavior rules` to see the built-in routing rules, and `Show behavior rule unknown` to read one in Links Notation.\n\
             - **Teach this dialog**: send «When I say \\`your prompt\\`, answer \\`your answer\\`» to add a dialog-local rule for the current conversation.\n\
             - **Self facts**: send `List all facts you know about yourself` to see what I know about myself.\n\
             - **Report a missing rule**: use the top-bar **Report issue** button or any message's Report issue link to ask developers to add a built-in rule.\n\
             - **Settings and actions**: configure diagnostics, demo mode, agent mode, theme, language, chat style, and memory import/export from messages.\n\
             \n\
             I run on local symbolic rules, without any neural network inference.",
        ),
    };
    Some(finalize_simple(
        prompt,
        log,
        "capabilities",
        "response:capabilities",
        &body,
        1.0,
    ))
}
pub fn try_shell_refusal(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    if normalized.contains("[agent]") || normalized.contains("agent mode") {
        return None;
    }
    let mentions_shell = (normalized.contains("run `") || normalized.contains("execute `"))
        && (normalized.contains("rm ")
            || normalized.contains("sudo")
            || normalized.contains("on my behalf"));
    if !mentions_shell {
        return None;
    }
    log.append("policy:chat_bounded_autonomy", prompt.to_owned());
    let body = String::from(
        "I can only respond with a chat reply. Running shell commands on your behalf is not \
         allowed without explicit agent mode opt-in, and even then only inside an isolated \
         sandbox.",
    );
    Some(finalize_simple(
        prompt,
        log,
        "policy_bounded_autonomy",
        "response:policy:bounded_autonomy",
        &body,
        0.5,
    ))
}

pub fn try_opinion_question(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let is_opinion_request = normalized.starts_with("do you think")
        || normalized.starts_with("what do you think")
        || normalized.starts_with("what is your opinion")
        || normalized.starts_with("what's your opinion")
        || normalized.starts_with("in your opinion")
        || normalized.starts_with("do you believe")
        || normalized.starts_with("what do you believe")
        || normalized.starts_with("do you feel")
        || normalized.starts_with("what do you feel")
        || normalized.starts_with("would you say")
        || normalized.starts_with("how do you feel")
        || normalized.starts_with("give me your opinion")
        || normalized.starts_with("share your opinion")
        || normalized.starts_with("share your thoughts")
        || normalized.starts_with("what are your thoughts");
    if !is_opinion_request {
        return None;
    }
    log.append("policy:no_opinion", prompt.to_owned());
    let body = String::from(
        "I am a deterministic symbolic AI. I do not hold opinions, beliefs, or feelings — \
         every answer I give is derived from an explicit Links Notation rule. \
         If you are looking for factual information on this topic, try asking \
         \"what is <topic>\" and I will look it up in my knowledge base.",
    );
    Some(finalize_simple(
        prompt,
        log,
        "opinion_question",
        "response:opinion_question",
        &body,
        1.0,
    ))
}

/// Issue #185: catch "prove …" / "show that …" / "доказать …" / "साबित कर
/// …" / "证明 …" prompts and return a structured response that names the
/// formalization pipeline and the planned `relative-meta-logic` integration
/// instead of falling through to the unknown-prompt opener.
///
/// The handler deliberately does **not** attempt to synthesise a proof
/// itself: discharging a proof requires the `relative-meta-logic` Rust
/// prover from `link-foundation/relative-meta-logic`, which is currently
/// only published as a git repository (no crates.io release) and which
/// requires a Wikidata-backed formalization step before it can be invoked
/// with a concrete axiom set. Wiring that integration is tracked in the
/// case study at `docs/case-studies/issue-185/README.md` and will land in
/// a follow-up PR.
pub fn try_proof_request(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    // A proof verb may be followed by whitespace or punctuation (",", ":",
    // "!", "."). Avoid false positives on longer words that just happen to
    // start with the verb (e.g. "prover" or "proven") by checking the
    // following character is non-alphabetic. End-of-string is treated as a
    // boundary (so `normalized == verb` still matches).
    let starts_with_verb = |verb: &str| -> bool {
        normalized
            .strip_prefix(verb)
            .is_some_and(|tail| !tail.chars().next().unwrap_or(' ').is_alphabetic())
    };
    let is_proof_request = starts_with_verb("prove")
        || starts_with_verb("proof")
        || normalized.starts_with("can you prove")
        || normalized.starts_with("could you prove")
        || normalized.starts_with("please prove")
        || normalized.starts_with("give me a proof")
        || normalized.starts_with("give a proof")
        || normalized.starts_with("show that ")
        || normalized.starts_with("demonstrate that ")
        || normalized.contains(" prove that ")
        || normalized.contains(" proof of ")
        // Russian
        || starts_with_verb("докажи")
        || starts_with_verb("докажите")
        || starts_with_verb("доказать")
        || starts_with_verb("доказательство")
        || normalized.contains(" докажи ")
        // Hindi
        || normalized.contains("साबित कर")
        || normalized.contains("सिद्ध कर")
        || normalized.contains("प्रमाण")
        // Chinese
        || normalized.contains("证明")
        || normalized.contains("證明");
    if !is_proof_request {
        return None;
    }
    let language = detect_language(prompt).slug();
    let mentions_godel = normalized.contains("godel")
        || normalized.contains("gödel")
        || normalized.contains("гёдел")
        || normalized.contains("гёделя")
        || normalized.contains("гедел")
        || normalized.contains("哥德尔")
        || normalized.contains("गोडेल");
    let mentions_determinism = normalized.contains("determinism")
        || normalized.contains("deterministic")
        || normalized.contains("детерминизм")
        || normalized.contains("决定论")
        || normalized.contains("निर्धारणवाद");
    log.append("policy:proof_request", prompt.to_owned());
    if mentions_godel {
        log.append("concept", "godel_incompleteness".to_owned());
    }
    if mentions_determinism {
        log.append("concept", "determinism".to_owned());
    }
    log.append("pipeline:planned", "relative-meta-logic".to_owned());
    let body = proof_request_body(language, mentions_godel, mentions_determinism);
    Some(finalize_simple(
        prompt,
        log,
        "proof_request",
        "response:proof_request",
        &body,
        0.6,
    ))
}

fn proof_request_body(language: &str, mentions_godel: bool, mentions_determinism: bool) -> String {
    let mut body = match language {
        "ru" => String::from(
            "Я пока не могу самостоятельно вывести это доказательство: библиотека-доказатель \
             relative-meta-logic (github.com/link-foundation/relative-meta-logic) ещё не \
             подключена к этому сборочному графу. Когда подключение появится, конвейер \
             будет работать так: impulse → formalize (с использованием Викиданных) → context \
             (math / logic / science) → план доказательства → выполнение в relative-meta-logic \
             → deformalize → finalize. Чтобы продвинуться сейчас, переформулируйте утверждение \
             как формальное высказывание и явно перечислите аксиомы и контекст.",
        ),
        "hi" => String::from(
            "मैं अभी स्वयं प्रमाण नहीं दे सकता: relative-meta-logic \
             (github.com/link-foundation/relative-meta-logic) प्रूवर पुस्तकालय अभी इस बिल्ड \
             ग्राफ़ में नहीं जुड़ा है। जब जुड़ जाएगा तो पाइपलाइन इस तरह चलेगी: impulse → \
             formalize (Wikidata के साथ) → context (math / logic / science) → प्रमाण योजना \
             → relative-meta-logic में निष्पादन → deformalize → finalize। अभी आगे बढ़ने के \
             लिए, कथन को औपचारिक प्रस्ताव के रूप में फिर से लिखें और अपने अभिगृहीत \
             (axioms) तथा संदर्भ स्पष्ट रूप से बताएँ।",
        ),
        "zh" => String::from(
            "我目前还无法自己完成这个证明:relative-meta-logic\
             (github.com/link-foundation/relative-meta-logic) 证明库尚未集成到本次构建中。\
             集成后,流程将是:impulse → formalize(借助 Wikidata)→ context\
             (math / logic / science)→ 证明计划 → 在 relative-meta-logic 中执行 → \
             deformalize → finalize。现在要推进的话,请把陈述改写为形式化命题,并明确给出\
             公理与上下文。",
        ),
        _ => String::from(
            "I cannot discharge that proof yet because the relative-meta-logic prover \
             (github.com/link-foundation/relative-meta-logic) is not wired into this build \
             as a library. When the integration lands, the pipeline will run: impulse → \
             formalize (Wikidata-backed) → context (math / logic / science) → proof plan → \
             execution in relative-meta-logic → deformalize → finalize. To move forward \
             today, restate the claim as a formal proposition and supply the axiom set and \
             context you want the proof to live in.",
        ),
    };
    if mentions_godel && mentions_determinism {
        let note = match language {
            "ru" => {
                "\n\nЗамечание про Гёделя и детерминизм: «детерминизм» сам по себе \
                     не является формальным высказыванием. Чтобы свести его к проверяемому \
                     утверждению, выберите конкретную формулировку — например, «лапласовский \
                     детерминизм совместим с классической механикой при наборе аксиом A» — и \
                     укажите аксиомы A. Теоремы Гёделя о неполноте применимы только к \
                     достаточно богатым формальным системам, поэтому контекст (PA, ZFC и т.д.) \
                     должен быть выбран явно перед запуском доказательства."
            }
            "hi" => {
                "\n\nगोडेल और निर्धारणवाद पर टिप्पणी: \"निर्धारणवाद\" अपने आप में \
                     औपचारिक प्रस्ताव नहीं है। इसे जाँचने योग्य कथन तक घटाने के लिए एक \
                     विशेष रूप चुनें — जैसे \"Laplace का निर्धारणवाद अभिगृहीत समुच्चय A के \
                     साथ शास्त्रीय यांत्रिकी के अनुकूल है\" — और A स्पष्ट रूप से बताएँ। \
                     गोडेल के अपूर्णता प्रमेय केवल पर्याप्त समृद्ध औपचारिक तंत्र (PA, ZFC \
                     आदि) पर लागू होते हैं, इसलिए संदर्भ पहले स्पष्ट करना ज़रूरी है।"
            }
            "zh" => {
                "\n\n关于哥德尔与决定论的说明:\"决定论\"本身并不是一个形式命题。\
                     要把它化简为可检验的陈述,请选择一种具体表述——例如\
                     \"拉普拉斯式决定论在公理集 A 下与经典力学相容\"——并明确给出 A。\
                     哥德尔不完备性定理只适用于足够丰富的形式系统(如 PA、ZFC),\
                     因此在启动证明之前必须显式选择上下文。"
            }
            _ => {
                "\n\nGödel-and-determinism note: \"determinism\" is not itself a formal \
                  proposition. To reduce it to a checkable claim, pick a concrete reading — \
                  for example, \"Laplacian determinism is consistent with classical \
                  mechanics under axiom set A\" — and spell out A. Gödel's incompleteness \
                  theorems only apply to sufficiently rich formal systems (PA, ZFC, …), so \
                  the context (PA, ZFC, …) must be chosen explicitly before the proof is \
                  attempted."
            }
        };
        body.push_str(note);
    }
    body
}

/// Detects "who is X" / "who was X" prompts (and multilingual equivalents)
/// that were not claimed by the concept-lookup handler because the entity is
/// not in the knowledge base.  Returns a deterministic response that
/// (a) acknowledges the question form, (b) reports the knowledge-base miss,
/// and (c) offers a typo correction when the queried term is close to a known
/// concept term.
pub fn try_who_is_question(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let is_who_question = normalized.starts_with("who is ")
        || normalized.starts_with("who was ")
        || normalized.starts_with("who are ")
        || normalized.starts_with("кто такой ")
        || normalized.starts_with("кто такая ")
        || normalized.starts_with("кто это ")
        || normalized.starts_with("кто ")
        || normalized.ends_with(" कौन है")
        || normalized.ends_with(" कौन हैं")
        || normalized.ends_with("是谁")
        || normalized.ends_with("是誰");
    if !is_who_question {
        return None;
    }
    let query = extract_concept_query(prompt)?;
    let term = &query.term;
    log.append("concept_lookup:miss", term.clone());
    let body = suggest_correction(term).map_or_else(
        || {
            format!(
                "I don't have a Links Notation fact for \"{term}\" yet. \
                 Add a fact or rule in Links Notation and run the request again."
            )
        },
        |corrected| {
            format!(
                "I don't have a Links Notation fact for \"{term}\" yet. \
                 Did you mean \"{corrected}\"? \
                 Add a fact or rule in Links Notation and run the request again."
            )
        },
    );
    Some(finalize_simple(
        prompt,
        log,
        "who_is_question",
        "response:who_is_question",
        &body,
        0.5,
    ))
}

/// Return a suggested correction for `term` when one token in `term` is
/// within edit-distance 1 of a known variant.  Returns `None` when no close
/// match is found.
fn suggest_correction(term: &str) -> Option<String> {
    let candidates: &[(&str, &[&str])] = &[
        ("Elon Musk", &["elon musk", "elon mask", "elon muск"]),
        (
            "Donald Trump",
            &["donald trump", "donald tramp", "donald tromp"],
        ),
        ("Joe Biden", &["joe biden", "joe bidan", "joe bidon"]),
        (
            "Barack Obama",
            &["barack obama", "barak obama", "barrack obama"],
        ),
        (
            "Vladimir Putin",
            &["vladimir putin", "vladimir puting", "vladmir putin"],
        ),
        (
            "Albert Einstein",
            &["albert einstein", "albert einstien", "albert enstien"],
        ),
        (
            "Isaac Newton",
            &["isaac newton", "isaak newton", "issac newton"],
        ),
        (
            "Nikola Tesla",
            &["nikola tesla", "nicolas tesla", "nikolai tesla"],
        ),
    ];
    let lower = term.to_lowercase();
    for (canonical, variants) in candidates {
        if variants.iter().any(|v| *v == lower) {
            return Some((*canonical).to_owned());
        }
    }
    for (canonical, variants) in candidates {
        let canonical_lower = canonical.to_lowercase();
        let is_close = variants.iter().any(|v| edit_distance(&lower, v) == 1)
            || edit_distance(&lower, &canonical_lower) == 1;
        if is_close {
            return Some((*canonical).to_owned());
        }
    }
    None
}

/// Compute the Levenshtein edit distance between two strings.
fn edit_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let m = a_chars.len();
    let n = b_chars.len();
    let mut dp = vec![vec![0usize; n + 1]; m + 1];
    for (i, row) in dp.iter_mut().enumerate() {
        row[0] = i;
    }
    for (j, cell) in dp[0].iter_mut().enumerate() {
        *cell = j;
    }
    for i in 1..=m {
        for j in 1..=n {
            dp[i][j] = if a_chars[i - 1] == b_chars[j - 1] {
                dp[i - 1][j - 1]
            } else {
                1 + dp[i - 1][j - 1].min(dp[i - 1][j]).min(dp[i][j - 1])
            };
        }
    }
    dp[m][n]
}
