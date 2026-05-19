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

pub fn try_capabilities(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let language = detect_language(prompt);
    let is_capabilities = match language.slug() {
        "ru" => {
            normalized.contains("что ты умеешь")
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
            normalized.contains("what can you do")
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
    let body = match language.slug() {
        "ru" => String::from(
            "Я formal-ai — детерминированный символьный ИИ. Вот что я умею:\n\
             \n\
             - **Приветствия**: отвечаю на «Привет», «Здравствуйте» и т.п.\n\
             - **Hello World**: генерирую программы на Rust, Python, JavaScript, Go, C и других языках.\n\
             - **Поиск понятий**: объясняю термины — попробуйте «Что такое Википедия?»\n\
             - **Арифметика**: вычисляю выражения — например, «Сколько будет 2 + 2?»\n\
             - **Перевод**: перевожу фразы между языками.\n\
             - **Память**: помню контекст разговора в рамках сессии.\n\
             - **Правила поведения**: отправьте `List behavior rules`, чтобы увидеть встроенные правила, и `Show behavior rule unknown`, чтобы прочитать одно правило.\n\
             - **Обучение в диалоге**: отправьте «When I say \\`ваш запрос\\`, answer \\`ваш ответ\\`», чтобы добавить правило, действующее только в этом диалоге.\n\
             - **Факты о себе**: отправьте `List all facts you know about yourself`, чтобы увидеть, что я знаю о себе.\n\
             - **Сообщение об ошибке**: используйте кнопку «Report issue» сверху или ссылку на странице сообщения, чтобы попросить разработчиков добавить встроенное правило.\n\
             \n\
             Я работаю на основе локальных символьных правил, без нейросетевого инференса.",
        ),
        "zh" => String::from(
            "我是 formal-ai —— 一个确定性的符号化 AI。以下是我的功能：\n\
             \n\
             - **问候**：回应「你好」等问候语。\n\
             - **Hello World**：生成 Rust、Python、JavaScript、Go、C 等语言的示例程序。\n\
             - **概念查找**：解释术语，例如「什么是维基百科？」\n\
             - **算术**：计算表达式，例如「2 + 2 等于多少？」\n\
             - **翻译**：在语言之间翻译短语。\n\
             - **记忆**：在会话中记住上下文。\n\
             - **行为规则**：发送 `List behavior rules` 查看内置规则，并发送 `Show behavior rule unknown` 阅读某条规则。\n\
             - **对话内教学**：发送「When I say \\`prompt\\`, answer \\`answer\\`」可以在本轮对话中添加一条本地规则。\n\
             - **自我事实**：发送 `List all facts you know about yourself` 查看我知道的关于自己的事实。\n\
             - **问题反馈**：使用顶部的 「Report issue」按钮或消息中的链接,请开发者把规则加入种子文件。\n\
             \n\
             我基于本地符号规则运行，不进行神经网络推理。",
        ),
        "hi" => String::from(
            "मैं formal-ai हूँ — एक नियतात्मक प्रतीकात्मक AI। मैं यह कर सकता हूँ:\n\
             \n\
             - **अभिवादन**: «नमस्ते» आदि का जवाब देना।\n\
             - **Hello World**: Rust, Python, JavaScript, Go, C आदि में प्रोग्राम बनाना।\n\
             - **अवधारणा खोज**: शब्दों को समझाना — जैसे «विकिपीडिया क्या है?»\n\
             - **अंकगणित**: गणनाएँ — जैसे «2 + 2 क्या है?»\n\
             - **अनुवाद**: भाषाओं के बीच अनुवाद।\n\
             - **स्मृति**: सत्र में संदर्भ याद रखना।\n\
             - **व्यवहार नियम**: `List behavior rules` भेजकर अंतर्निहित नियम देखें और `Show behavior rule unknown` से कोई नियम पढ़ें।\n\
             - **संवाद-स्तर पर सिखाना**: «When I say \\`prompt\\`, answer \\`answer\\`» भेजकर इस संवाद के लिए स्थानीय नियम जोड़ें।\n\
             - **स्व-तथ्य**: `List all facts you know about yourself` भेजें ताकि मैं अपने बारे में जो जानता हूँ वह सूचीबद्ध करूँ।\n\
             - **समस्या रिपोर्ट**: ऊपर के «Report issue» बटन या मैसेज लिंक का उपयोग करके डेवलपर्स से built-in नियम जोड़वा सकते हैं।\n\
             \n\
             मैं स्थानीय प्रतीकात्मक नियमों पर चलता हूँ, कोई न्यूरल इन्फेरेन्स नहीं।",
        ),
        _ => String::from(
            "I am formal-ai, a deterministic symbolic AI. Here is what I can do:\n\
             \n\
             - **Greetings**: respond to «Hi», «Hello», and similar.\n\
             - **Hello World**: generate programs in Rust, Python, JavaScript, Go, C, and more.\n\
             - **Concept lookup**: explain terms — try «What is Wikipedia?»\n\
             - **Arithmetic**: evaluate expressions — try «What is 2 + 2?»\n\
             - **Translation**: translate phrases between languages.\n\
             - **Memory**: recall context within the current session.\n\
             - **Behavior rules**: send `List behavior rules` to see the built-in routing rules, and `Show behavior rule unknown` to read one in Links Notation.\n\
             - **Teach this dialog**: send «When I say \\`your prompt\\`, answer \\`your answer\\`» to add a dialog-local rule for the current conversation.\n\
             - **Self facts**: send `List all facts you know about yourself` to see what I know about myself.\n\
             - **Report a missing rule**: use the top-bar **Report issue** button or any message's Report issue link to ask developers to add a built-in rule.\n\
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
