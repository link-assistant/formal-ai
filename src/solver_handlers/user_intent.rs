//! Handlers for user-intent clarification, capability queries, follow-up
//! elaboration, ill-formed input, and shell-refusal policy. Extracted from
//! `solver_handlers/mod.rs` to keep individual files under 1000 lines.

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

/// Handles follow-up elaboration prompts such as "how it works?", "how does
/// it work?", or "how does X work?". When the prior assistant turn mentioned a
/// named concept the solver re-runs a concept lookup for that topic; when no
/// prior context is present it redirects to the meta-explanation handler.
pub fn try_how_it_works(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let is_how_it_works = normalized == "how it works?"
        || normalized == "how it works"
        || normalized == "how does it work?"
        || normalized == "how does it work"
        || normalized.starts_with("how does it work")
        || normalized.starts_with("how it works")
        || normalized.starts_with("how does ")
            && (normalized.ends_with(" work?") || normalized.ends_with(" work"));
    if !is_how_it_works {
        return None;
    }
    log.append("followup:how_it_works", normalized.to_owned());

    // Try to extract the subject from the prompt itself ("how does Curve25519 work?").
    let subject = extract_how_it_works_subject(normalized);

    // When a subject was explicit in the prompt, do a direct concept lookup.
    if let Some(ref term) = subject {
        use crate::concepts::{extract_concept_query, lookup_concept_query};
        if let Some(query) = extract_concept_query(&format!("what is {term}")) {
            if lookup_concept_query(&query).is_some() {
                log.append("followup:subject", format!("inline:{term}"));
                // Delegate to try_concept_lookup by synthesising a standard prompt.
                return crate::solver_handlers::try_concept_lookup(
                    &format!("what is {term}"),
                    log,
                );
            }
        }
    }

    // No inline subject — look for the topic in the prior assistant reply.
    if let Some(prior) = crate::solver_helpers::last_assistant_turn(log).map(str::to_owned) {
        log.append("followup:prior_turn", "assistant".to_owned());
        // Extract the first capitalised noun phrase from the prior reply
        // (typically the term in "Term (category): …" format).
        if let Some(term) = extract_topic_from_prior_reply(&prior) {
            use crate::concepts::{extract_concept_query, lookup_concept_query};
            if let Some(query) = extract_concept_query(&format!("what is {term}")) {
                if lookup_concept_query(&query).is_some() {
                    log.append("followup:subject", format!("prior_reply:{term}"));
                    return crate::solver_handlers::try_concept_lookup(
                        &format!("what is {term}"),
                        log,
                    );
                }
            }
            // Topic is known from history but not in the concept corpus —
            // return a helpful explanation that names the topic.
            let body = format!(
                "To explain how {term} works: I know the term from the prior conversation \
                 but do not have a detailed symbolic rule for it yet. Add a Links Notation \
                 fact with the mechanism description, then ask again."
            );
            log.append("followup:subject", format!("prior_reply_no_record:{term}"));
            return Some(finalize_simple(
                prompt,
                log,
                "concept_elaboration_missing",
                "response:concept_elaboration_missing",
                &body,
                0.3,
            ));
        }
    }

    // No context at all — route to meta_explanation.
    let body = String::from(
        "I answered that way because the prompt matched a deterministic Links Notation rule. \
         To ask about a specific topic, try \"how does X work?\" where X is a concept I know \
         (e.g. \"how does Wikipedia work?\"). The evidence and trace events are appended to \
         the log; see the trace link for the full chain.",
    );
    Some(finalize_simple(
        prompt,
        log,
        "meta_explanation",
        "response:meta_explanation",
        &body,
        0.5,
    ))
}

/// Extract the explicit subject from a "how does X work?" prompt.
/// Returns `None` when the prompt is the bare "how it works?" form.
fn extract_how_it_works_subject(normalized: &str) -> Option<String> {
    // "how does X work" / "how does X work?"
    if let Some(rest) = normalized.strip_prefix("how does ") {
        let term = rest.trim_end_matches('?').trim_end_matches(" work").trim();
        if !term.is_empty() && term != "it" {
            return Some(term.to_owned());
        }
    }
    None
}

/// Extract the first meaningful topic word/phrase from a prior assistant reply.
/// Looks for "Term (category):" patterns first, then the first capitalised token.
fn extract_topic_from_prior_reply(reply: &str) -> Option<String> {
    // Match "Term (category): description" — common in concept_lookup answers.
    let first_line = reply.lines().next().unwrap_or("").trim();
    if let Some(paren_pos) = first_line.find('(') {
        let candidate = first_line[..paren_pos].trim();
        if !candidate.is_empty() {
            return Some(candidate.to_lowercase());
        }
    }
    // Fallback: first capitalised word that is not a stop word.
    let stop_words = [
        "I", "The", "A", "An", "In", "To", "For", "Of", "And", "Or", "Source",
    ];
    for word in reply.split_whitespace() {
        let clean = word.trim_matches(|c: char| !c.is_alphanumeric());
        if clean.len() >= 2
            && clean.chars().next().is_some_and(char::is_uppercase)
            && !stop_words.contains(&clean)
        {
            return Some(clean.to_lowercase());
        }
    }
    None
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
