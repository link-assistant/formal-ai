use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::language::detect as detect_language;
use crate::seed::{self, response_for, Slot};

use super::finalize_simple;
use super::self_awareness::{
    surface_label, surface_memory, surface_runtime, surface_web_search, SelfAwarenessRuntime,
};

pub fn try_meta_explanation(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    try_meta_explanation_with_runtime(prompt, normalized, log, SelfAwarenessRuntime::default())
}

pub fn try_meta_explanation_with_runtime(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
    runtime: SelfAwarenessRuntime,
) -> Option<SymbolicAnswer> {
    let is_why_question = is_why_question(normalized);
    let is_how_you_work = is_how_you_work(normalized);
    let is_architecture_question = is_architecture_question(normalized);
    if !is_why_question && !is_how_you_work && !is_architecture_question {
        return None;
    }
    let language = meta_language(prompt, normalized);
    let body = if is_why_question {
        why_explanation_body(language)
    } else if is_architecture_question {
        architecture_explanation_body(language, runtime)
    } else {
        response_for("meta_explanation", language)
            .or_else(|| response_for("meta_explanation", "en"))
            .unwrap_or_else(|| {
                String::from(
                    "I work by matching your prompt against deterministic Links Notation rules stored \
                     in memory. Each rule maps a recognized pattern to a fixed response. When no rule \
                     matches, I report intent: unknown. There is no neural inference — every answer is \
                     fully traceable to a symbolic rule.",
                )
            })
    };
    Some(finalize_simple(
        prompt,
        log,
        "meta_explanation",
        "response:meta_explanation",
        &body,
        1.0,
    ))
}

fn why_explanation_body(language: &str) -> String {
    match language {
        "ru" => String::from(
            "Я ответил так, потому что запрос совпал с детерминированным правилом Links Notation. \
             Evidence-ссылки и trace-события добавлены в журнал; по trace-ссылке можно проверить \
             всю цепочку.",
        ),
        "hi" => String::from(
            "मैंने ऐसा जवाब इसलिए दिया क्योंकि prompt deterministic Links Notation rule से मेल खाता था. \
             evidence links और trace events log में जोड़े गए हैं; पूरी chain देखने के लिए trace link देखें.",
        ),
        "zh" => String::from(
            "我这样回答是因为该提示匹配了确定性的 Links Notation 规则。evidence links 和 trace events \
             已追加到日志中; 可通过 trace link 检查完整链路。",
        ),
        _ => String::from(
            "I answered that way because the prompt matched a deterministic Links Notation rule. \
             The evidence links and trace events are appended to the log; see the trace link for the \
             full chain.",
        ),
    }
}

/// True when the prompt asks the assistant to justify its previous answer.
///
/// The English and Russian why-questions front the interrogative, so each
/// [`answer_rationale_lead`](seed::ROLE_ANSWER_RATIONALE_LEAD) surface is matched
/// directly — a [`Slot::Prefix`] form against the start of the prompt and a bare
/// form anywhere in it. The Hindi and Chinese why-questions are head-final, so
/// they are detected instead as a same-language pair of a
/// [`causal_interrogative`](seed::ROLE_CAUSAL_INTERROGATIVE) and a
/// [`prior_answer_reference`](seed::ROLE_PRIOR_ANSWER_REFERENCE); the Hindi and
/// Chinese rationale surfaces are inert completeness forms, skipped here by the
/// language filter. No question word is hardcoded in this function.
fn is_why_question(normalized: &str) -> bool {
    let lexicon = seed::lexicon();
    for meaning in lexicon.meanings_with_role(seed::ROLE_ANSWER_RATIONALE_LEAD) {
        for lexeme in &meaning.lexemes {
            if lexeme.language != "en" && lexeme.language != "ru" {
                continue;
            }
            for form in &lexeme.words {
                let matched = match form.slot() {
                    Slot::Prefix => normalized.starts_with(form.before_slot()),
                    _ => normalized.contains(form.text.as_str()),
                };
                if matched {
                    return true;
                }
            }
        }
    }
    ["hi", "zh"].into_iter().any(|language| {
        let names_cause = lexicon
            .words_for_role_in_languages(seed::ROLE_CAUSAL_INTERROGATIVE, &[language])
            .iter()
            .any(|word| normalized.contains(word.as_str()));
        let names_prior_answer = lexicon
            .words_for_role_in_languages(seed::ROLE_PRIOR_ANSWER_REFERENCE, &[language])
            .iter()
            .any(|word| normalized.contains(word.as_str()));
        names_cause && names_prior_answer
    })
}

/// True when the prompt asks the assistant to explain how it works.
///
/// Most phrasings are complete clauses carried by
/// [`assistant_mechanism_inquiry`](seed::ROLE_ASSISTANT_MECHANISM_INQUIRY) and
/// matched as raw substrings. The Russian principle-of-operation phrasing
/// (принцип работы … тебя) is compositional, so it is recognised by requiring an
/// [`operating_principle`](seed::ROLE_OPERATING_PRINCIPLE) surface together with
/// an [`assistant_self_reference`](seed::ROLE_ASSISTANT_SELF_REFERENCE) surface,
/// both read in Russian only.
fn is_how_you_work(normalized: &str) -> bool {
    let lexicon = seed::lexicon();
    if lexicon.mentions_role_raw(seed::ROLE_ASSISTANT_MECHANISM_INQUIRY, normalized) {
        return true;
    }
    let names_principle = lexicon
        .words_for_role_in_languages(seed::ROLE_OPERATING_PRINCIPLE, &["ru"])
        .iter()
        .any(|word| normalized.contains(word.as_str()));
    let addresses_assistant = lexicon
        .words_for_role_in_languages(seed::ROLE_ASSISTANT_SELF_REFERENCE, &["ru"])
        .iter()
        .any(|word| normalized.contains(word.as_str()));
    names_principle && addresses_assistant
}

/// True when the prompt asks how the assistant itself is built rather than
/// requesting a task.
///
/// Decomposes exactly like the original two-list screen: the prompt must address
/// the assistant — carry an
/// [`assistant_self_reference`](seed::ROLE_ASSISTANT_SELF_REFERENCE) surface —
/// *and* name an [`architecture_concept`](seed::ROLE_ARCHITECTURE_CONCEPT) such
/// as a language model, neural network, or the project's local rules. Both are
/// matched as raw substrings across all four languages.
fn is_architecture_question(normalized: &str) -> bool {
    let lexicon = seed::lexicon();
    lexicon.mentions_role_raw(seed::ROLE_ASSISTANT_SELF_REFERENCE, normalized)
        && lexicon.mentions_role_raw(seed::ROLE_ARCHITECTURE_CONCEPT, normalized)
}

fn meta_language(prompt: &str, normalized: &str) -> &'static str {
    let lower = format!("{} {}", prompt.to_lowercase(), normalized);
    if meta_has_char_in_range(&lower, '\u{0400}', '\u{04ff}') {
        return "ru";
    }
    if meta_has_char_in_range(&lower, '\u{0900}', '\u{097f}') {
        return "hi";
    }
    if meta_has_char_in_range(&lower, '\u{4e00}', '\u{9fff}') {
        return "zh";
    }
    detect_language(prompt).slug()
}

fn meta_has_char_in_range(text: &str, start: char, end: char) -> bool {
    text.chars().any(|ch| (start..=end).contains(&ch))
}

fn architecture_explanation_body(language: &str, runtime: SelfAwarenessRuntime) -> String {
    match language {
        "ru" => format!(
            "Я не LLM-рантайм и не выполняю нейросетевой инференс. \
             Текущая среда: {} (`{}`). Рантайм: {}. У проекта есть \
             OpenAI-совместимые API-форматы, но ответы строит детерминированный solver: \
             сначала он проверяет локальный seed Links Notation, правила и память ({}); \
             затем веб-поиск используется только с учетом среды: {}. Весь интернет не \
             загружен в локальные правила целиком.",
            surface_label(runtime),
            runtime.surface.slug(),
            surface_runtime(runtime),
            surface_memory(runtime),
            surface_web_search(runtime)
        ),
        "hi" => format!(
            "मैं LLM runtime नहीं हूँ और neural inference नहीं चलाता. Current environment: {} (`{}`). \
             Runtime: {}. Project OpenAI-compatible API shapes देता है, लेकिन जवाब deterministic solver \
             बनाता है: पहले local Links Notation seed, rules और memory ({}) देखता है; फिर web search \
             केवल environment अनुमति दे तो उपयोग करता है: {}. पूरा internet local rules में preload नहीं है.",
            surface_label(runtime),
            runtime.surface.slug(),
            surface_runtime(runtime),
            surface_memory(runtime),
            surface_web_search(runtime)
        ),
        "zh" => format!(
            "我不是 LLM runtime, 也不执行神经网络推理。当前环境: {} (`{}`)。Runtime: {}。\
             项目提供 OpenAI-compatible API 形状, 但回答由确定性的 solver 生成: 先检查本地 \
             Links Notation seed、规则和记忆 ({}); 然后只在当前环境允许时使用 web search: {}。\
             整个互联网不会预加载到本地规则中。",
            surface_label(runtime),
            runtime.surface.slug(),
            surface_runtime(runtime),
            surface_memory(runtime),
            surface_web_search(runtime)
        ),
        _ => format!(
            "I am not an LLM runtime and I do not perform neural inference. \
             Current environment: {} (`{}`). Runtime: {}. The project exposes \
             OpenAI-compatible API shapes, but answers come from a deterministic solver: \
             it checks the local Links Notation seed, rules, and memory ({}) first; \
             web search is used only when this environment allows it: {}. The whole \
             internet is not preloaded into local rules.",
            surface_label(runtime),
            runtime.surface.slug(),
            surface_runtime(runtime),
            surface_memory(runtime),
            surface_web_search(runtime)
        ),
    }
}
