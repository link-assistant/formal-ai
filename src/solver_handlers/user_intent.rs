//! Handlers for user-intent clarification, capability queries, follow-up
//! elaboration, ill-formed input, and shell-refusal policy. Extracted from
//! `solver_handlers/mod.rs` to keep individual files under 1000 lines.

use crate::concepts::extract_concept_query;
use crate::engine::{normalize_prompt, SymbolicAnswer};
use crate::event_log::EventLog;
use crate::fuzzy::typo_distance;
use crate::language::detect as detect_language;
use crate::proof_engine::{
    attempt_proof_with_config, render_outcome_with_config, ProofOutcome, ProofRenderConfig,
};
use crate::seed::{self, response_for, Slot, WordForm};
use crate::solver_handlers::finalize_simple;

/// The literal lead-in (text before the `…` slot) of every prefix-slot form of
/// a role, in lexicon declaration order. Lets the proof and who-is recognisers
/// reason about a role's opening surfaces without baking the words into code.
fn prefix_literals(role: &str) -> Vec<&'static str> {
    seed::lexicon()
        .role_word_forms(role)
        .into_iter()
        .filter(|form| form.slot() == Slot::Prefix)
        .map(WordForm::before_slot)
        .collect()
}

/// The literal tail (text after the `…` slot) of every suffix-slot form of a
/// role, in lexicon declaration order. Used for languages whose question marker
/// trails the topic (Hindi `… कौन है`, Chinese `…是谁`).
fn suffix_literals(role: &str) -> Vec<&'static str> {
    seed::lexicon()
        .role_word_forms(role)
        .into_iter()
        .filter(|form| form.slot() == Slot::Suffix)
        .map(WordForm::after_slot)
        .collect()
}

/// The surface text of every bare-slot form of a role, in lexicon declaration
/// order. A meaning's roles apply to all its forms, so we keep only the bare
/// detection tokens and drop any prefix/suffix surfaces the meaning also owns.
fn bare_literals(role: &str) -> Vec<&'static str> {
    seed::lexicon()
        .role_word_forms(role)
        .into_iter()
        .filter(|form| form.slot() == Slot::Bare)
        .map(|form| form.text.as_str())
        .collect()
}

pub fn try_clarification(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    if !is_clarification_request(normalized) {
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

/// Does `normalized` signal the user did not understand and wants the prior
/// answer made clear?
///
/// Issue #386: recognised by the `clarification_request` meaning role rather
/// than a hardcoded per-language phrase list — the surface words ("i don t
/// understand", "не понял", "समझ नहीं आया", "我不明白", …) live once in
/// `data/seed/meanings-intent.lino`. The prompt is re-normalised so trailing
/// punctuation ("what do you mean?") and apostrophes ("i don't understand")
/// collapse to the same canonical spacing the seed stores.
fn is_clarification_request(normalized: &str) -> bool {
    seed::lexicon().mentions_role(
        seed::ROLE_CLARIFICATION_REQUEST,
        &normalize_prompt(normalized),
    )
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

/// Is `normalized` a follow-up asking what *else* the assistant can do?
///
/// Issue #386: recognised by the `capability_query_more` meaning role rather
/// than a hardcoded per-language phrase list — the surface words ("what else
/// can you do", "что ещё ты умеешь", "और क्या कर सकते", "你还能做什么", …) live
/// once in `data/seed/meanings-intent.lino`. Recognition is language-agnostic
/// because the surface words are script-specific; the response body is still
/// chosen by the caller from `detect_language`. The prompt is re-normalised so
/// trailing punctuation collapses to the canonical spacing the seed stores.
fn is_more_capabilities_prompt(normalized: &str) -> bool {
    seed::lexicon().mentions_role(
        seed::ROLE_CAPABILITY_QUERY_MORE,
        &normalize_prompt(normalized),
    )
}

/// Is `normalized` asking what the assistant is able to do?
///
/// Issue #386: recognised by the `capability_query` meaning role — plus its
/// follow-up [`is_more_capabilities_prompt`], so "what else can you do" still
/// counts as a capability query — rather than a hardcoded per-language phrase
/// list. The surface words ("what can you do", "что ты умеешь", "что за дичь",
/// "आप क्या कर सकते", "你能做什么", …) live once in
/// `data/seed/meanings-intent.lino`. The prompt is re-normalised so trailing
/// punctuation ("what can you do?", "你能做什么？") collapses to the canonical
/// spacing the seed stores.
fn is_capability_query(normalized: &str) -> bool {
    let cleaned = normalize_prompt(normalized);
    let lexicon = seed::lexicon();
    lexicon.mentions_role(seed::ROLE_CAPABILITY_QUERY, &cleaned)
        || lexicon.mentions_role(seed::ROLE_CAPABILITY_QUERY_MORE, &cleaned)
}

fn prior_history_mentions_web_search(log: &EventLog) -> bool {
    log.events()
        .iter()
        .filter(|event| event.kind == "prior_turn:user" || event.kind == "prior_turn:assistant")
        .any(|event| {
            let payload = event.payload.to_lowercase();
            seed::lexicon().mentions_role_raw(seed::ROLE_WEB_SEARCH_HISTORY_SIGNAL, &payload)
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
    let more_capabilities = is_more_capabilities_prompt(normalized);
    if !is_capability_query(normalized) {
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
             - **Сообщение об ошибке**: используйте кнопку «Report issue» сверху; для неизвестных запросов ссылка в сообщении добавит diagnostic trace.\n\
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
             - **问题反馈**：使用顶部的 「Report issue」按钮；未知请求的消息链接会带上 diagnostic trace。\n\
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
             - **समस्या रिपोर्ट**: ऊपर के «Report issue» बटन का उपयोग करें; unknown prompts के message link में diagnostic trace शामिल होता है।\n\
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
             - **Report a missing rule**: use the top-bar **Report issue** button; unknown-prompt message links include the diagnostic trace for maintainers.\n\
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
/// …" / "证明 …" prompts and route them through the universal proof
/// engine (`crate::proof_engine`).
///
/// Every branch of the engine returns a real outcome — `Proven` for a
/// theorem we can discharge by direct calculation or by quoting the
/// classical proof, `Disproven` with a worked counterexample,
/// `PartialPlan` that walks the user through the proof plan and asks
/// for the missing axiom set or definitions, or `Inconclusive` with a
/// concrete reason. The handler never falls back to the generic
/// "unknown intent" opener.
pub fn try_proof_request(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    try_proof_request_with_config(prompt, normalized, log, ProofRenderConfig::default())
}

/// Configuration-aware variant of [`try_proof_request`].
///
/// The two sliders in [`ProofRenderConfig`] control how the proof is
/// surfaced to the user:
///
/// * High `guess_probability` → the engine explains how it interpreted the
///   prompt (an "Interpretation" header), commits to a formal translation,
///   and runs the proof through to a conclusion.
/// * High `follow_up_probability` → the engine appends a "Clarifying
///   questions" footer listing every input it still needs before the final
///   research execution.
///
/// The two sliders are independent so all four combinations work.
pub fn try_proof_request_with_config(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
    config: ProofRenderConfig,
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
    // A proof request is recognised structurally from the meaning lexicon, not
    // from words baked into this file: a clause-initial bare directive verb
    // (`proof_directive`, with the verb-boundary check above), an English
    // request-frame lead that needs no `that` clause (`proof_request_lead`), or
    // a mid-prompt proof assertion marker in any language (`proof_marker`).
    let is_proof_request = bare_literals(seed::ROLE_PROOF_DIRECTIVE)
        .iter()
        .any(|&verb| starts_with_verb(verb))
        || prefix_literals(seed::ROLE_PROOF_REQUEST_LEAD)
            .iter()
            .any(|&lead| normalized.starts_with(lead))
        || seed::lexicon().mentions_role_raw(seed::ROLE_PROOF_MARKER, normalized);
    if !is_proof_request {
        return None;
    }
    if is_known_unsolved_bounded_proof_request(normalized) {
        return None;
    }
    let language = detect_language(prompt).slug();
    let mentions_godel =
        seed::lexicon().mentions_role_raw(seed::ROLE_PROOF_CONCEPT_GODEL, normalized);
    let mentions_determinism =
        seed::lexicon().mentions_role_raw(seed::ROLE_PROOF_CONCEPT_DETERMINISM, normalized);
    log.append("policy:proof_request", prompt.to_owned());
    log.append(
        "policy:proof_guess_probability",
        format!("{:.2}", config.guess_probability),
    );
    log.append(
        "policy:proof_follow_up_probability",
        format!("{:.2}", config.follow_up_probability),
    );
    if mentions_godel {
        log.append("concept", "godel_incompleteness".to_owned());
    }
    if mentions_determinism {
        log.append("concept", "determinism".to_owned());
    }
    log.append("pipeline:planned", "relative-meta-logic".to_owned());
    let claim = extract_claim_from_prompt(normalized);
    let outcome = attempt_proof_with_config(
        prompt,
        &claim,
        language,
        mentions_godel,
        mentions_determinism,
        config,
    );
    log.append("proof_outcome", outcome.status_slug().to_owned());
    if let Some(method) = outcome.method() {
        log.append("proof_method", method.slug().to_owned());
    }
    if config.show_interpretation() {
        log.append("proof_render:interpretation", "shown".to_owned());
    }
    if config.ask_follow_ups() {
        log.append("proof_render:follow_ups", "shown".to_owned());
    }
    let mut body = render_outcome_with_config(&outcome, language, config);
    if matches!(outcome, ProofOutcome::PartialPlan { .. }) {
        body.push_str(&pipeline_footer(language));
    }
    let confidence = match &outcome {
        ProofOutcome::Proven { .. } | ProofOutcome::Disproven { .. } => 0.85,
        ProofOutcome::PartialPlan { .. } => 0.6,
        ProofOutcome::Inconclusive { .. } => 0.4,
    };
    Some(finalize_simple(
        prompt,
        log,
        "proof_request",
        "response:proof_request",
        &body,
        confidence,
    ))
}

fn is_known_unsolved_bounded_proof_request(normalized: &str) -> bool {
    let asks_for_terse_final_proof = normalized.contains("in two sentences")
        || normalized.contains("in 2 sentences")
        || normalized.contains("briefly prove")
        || normalized.contains("short proof");
    let names_open_problem = normalized.contains("p=np")
        || normalized.contains("p = np")
        || normalized.contains("p versus np")
        || normalized.contains("p vs np");
    asks_for_terse_final_proof && names_open_problem
}

/// Strip the surrounding "prove that …" / "докажи, что …" / "证明 …"
/// scaffolding so the proof engine receives the bare claim. We err on
/// the side of returning the full normalized prompt — the engine
/// tolerates extra text in its keyword search and arithmetic split.
fn extract_claim_from_prompt(normalized: &str) -> String {
    let trimmed = normalized.trim();
    // The claim scaffolds (each ending in the `…` slot) are sourced from the
    // `proof_claim_scaffold` role in declaration order, so the first matching
    // prefix wins exactly as before — every `that`/`что`/`कि` variant is listed
    // ahead of its shorter sibling in the lexicon. Comma variants such as
    // `докажи, что` are intentionally absent: `normalize_prompt` rewrites the
    // comma to a space, so they are unreachable here.
    let prefixes = prefix_literals(seed::ROLE_PROOF_CLAIM_SCAFFOLD);
    for prefix in &prefixes {
        if let Some(rest) = trimmed.strip_prefix(prefix) {
            return strip_claim_prefix_noise(rest).to_owned();
        }
    }
    for prefix in &prefixes {
        if let Some(index) = trimmed.find(prefix) {
            let before = &trimmed[..index];
            if before.chars().last().is_some_and(is_claim_intro_boundary) {
                let rest = &trimmed[index + prefix.len()..];
                return strip_claim_prefix_noise(rest).to_owned();
            }
        }
    }
    trimmed.to_owned()
}

fn is_claim_intro_boundary(ch: char) -> bool {
    ch.is_whitespace()
        || matches!(
            ch,
            '.' | ',' | ':' | ';' | '!' | '?' | '…' | '。' | '，' | '：' | '；' | '！' | '？'
        )
}

fn strip_claim_prefix_noise(text: &str) -> &str {
    text.trim_start_matches(|c: char| c == ',' || c == ':' || c.is_whitespace())
}

fn pipeline_footer(language: &str) -> String {
    match language {
        "ru" => String::from(
            "\n\nПоддерживаемый конвейер: impulse → formalize (Викиданные) → context → \
             план доказательства → проверка в relative-meta-logic → deformalize → finalize.",
        ),
        "hi" => String::from(
            "\n\nसमर्थित पाइपलाइन: impulse → formalize (Wikidata) → context → प्रमाण योजना \
             → relative-meta-logic में सत्यापन → deformalize → finalize।",
        ),
        "zh" => String::from(
            "\n\n所支持的流程:impulse → formalize(Wikidata)→ context → 证明计划 → \
             relative-meta-logic 校验 → deformalize → finalize。",
        ),
        _ => String::from(
            "\n\nSupported pipeline: impulse → formalize (Wikidata-backed) → context → \
             proof plan → verification in relative-meta-logic → deformalize → finalize.",
        ),
    }
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
    // "who is …" detection reasons over the `who_question` meaning: a
    // language whose marker leads the name occupies the `who_question_lead`
    // prefix slot (English `who is …`, Russian `кто такой …`), while one whose
    // marker trails it occupies the `who_question_tail` suffix slot (Hindi
    // `… कौन है`, Chinese `…是谁`). No question word is baked into this file.
    let is_who_question = prefix_literals(seed::ROLE_WHO_QUESTION_LEAD)
        .iter()
        .any(|&lead| normalized.starts_with(lead))
        || suffix_literals(seed::ROLE_WHO_QUESTION_TAIL)
            .iter()
            .any(|&tail| normalized.ends_with(tail));
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
        let is_close = variants.iter().any(|v| typo_distance(&lower, v) == 1)
            || typo_distance(&lower, &canonical_lower) == 1;
        if is_close {
            return Some((*canonical).to_owned());
        }
    }
    None
}
