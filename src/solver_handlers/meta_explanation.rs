use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::language::detect as detect_language;
use crate::seed::response_for;

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
    let is_why_question = normalized.starts_with("why ")
        || normalized.starts_with("why did")
        || normalized.starts_with("why do you")
        || normalized.contains("why did you answer");
    let is_how_you_work = normalized.contains("how do you work")
        || normalized.contains("how does this work")
        || normalized.contains("how does it work")
        || normalized.contains("show me how you work")
        || normalized.contains("explain how you work")
        // Russian
        || normalized.contains("как ты работаешь")
        || normalized.contains("покажи как ты работаешь")
        || normalized.contains("расскажи как ты работаешь")
        || normalized.contains("объясни как ты работаешь")
        || normalized.contains("как ты устроен")
        || normalized.contains("покажи как ты устроен")
        || normalized.contains("какой принцип работы у тебя")
        || normalized.contains("принцип работы у тебя")
        || (normalized.contains("принцип работы")
            && meta_contains_any(normalized, &["ты", "теб", "твой", "твоя", "тво", "вы"]))
        || normalized.contains("какая у тебя модель окружающего мира")
        || normalized.contains("модель окружающего мира")
        || normalized.contains("идея твоей разработки")
        || normalized.contains("идея твоего проекта")
        || normalized.contains("зачем тебя разработ")
        // Hindi
        || normalized.contains("तुम कैसे काम करते हो")
        || normalized.contains("आप कैसे काम करते हैं")
        // Chinese
        || normalized.contains("你是怎么工作的")
        || normalized.contains("你怎么运作");
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
            "मैंने ऐसा जवाब इसलिए दिया because prompt deterministic Links Notation rule से मेल खाता था. \
             evidence links और trace events log में जोड़े गए हैं; पूरी chain देखने के लिए trace link देखें.",
        ),
        "zh" => String::from(
            "我这样回答 because 该提示匹配了确定性的 Links Notation 规则。evidence links 和 trace events \
             已追加到日志中; 可通过 trace link 检查完整链路。",
        ),
        _ => String::from(
            "I answered that way because the prompt matched a deterministic Links Notation rule. \
             The evidence links and trace events are appended to the log; see the trace link for the \
             full chain.",
        ),
    }
}

fn meta_contains_any(normalized: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| normalized.contains(needle))
}

fn meta_language(prompt: &str, normalized: &str) -> &'static str {
    let lower = format!("{} {}", prompt.to_lowercase(), normalized);
    if meta_has_char_in_range(&lower, '\u{0400}', '\u{04ff}')
        || meta_contains_any(
            &lower,
            &["ты", "теб", "твоя", "твой", "вы", "вас", "у тебя"],
        )
    {
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

fn is_architecture_question(normalized: &str) -> bool {
    let mentions_assistant = meta_contains_any(
        normalized,
        &[
            "you",
            "your",
            "formal ai",
            "ты",
            "теб",
            "твоя",
            "твой",
            "тво",
            "вы",
            "आप",
            "तुम",
            "你",
            "您",
        ],
    );
    if !mentions_assistant {
        return false;
    }

    meta_contains_any(
        normalized,
        &[
            "llm",
            "large language model",
            "language model",
            "openai api",
            "openai",
            "neural inference",
            "neural network",
            "links notation rules",
            "local rules",
            "world model",
            "model of the world",
            "бям",
            "языковая модель",
            "языковой моделью",
            "нейросет",
            "нейрон",
            "локальных правил",
            "локальных правилах",
            "область знаний",
            "модель окружающего мира",
            "модель мира",
            "принцип работы",
            "идея твоей разработки",
            "идея твоего проекта",
            "зачем тебя разработ",
            "ссылк",
            "न्यूरल",
            "भाषा मॉडल",
            "神经",
            "語言模型",
            "语言模型",
        ],
    )
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
