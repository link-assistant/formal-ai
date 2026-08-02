use std::fmt::Write as _;

use super::super::finalize_simple;
use crate::coding::contains_cjk;
use crate::engine::{normalize_prompt, SymbolicAnswer};
use crate::event_log::EventLog;
use crate::language::detect as detect_language;
use crate::seed;
use crate::summarization::{
    generate_chat_title, summarize_dialog, summarize_dialog_plain, DialogTurn, SummarizationConfig,
    SummarizationMode,
};

const RETURN_RECAP_MAX_WORDS: usize = 39;
const RETURN_RECAP_MAX_SENTENCES: usize = 2;

/// Recognise a request to summarize the running conversation by composing
/// meaning roles rather than matching raw per-language phrases (issue #386).
///
/// The universal algorithm is identical for every language: the prompt either
/// (a) carries a complete standalone conversation-summary phrasing, (b) carries
/// an objectless courtesy frame asking for a summary, (c) names a summary
/// directive *together with* a conversation reference, or (d) leads with a bare
/// summary directive (`summarize`, `резюме`, `总结`, …). The prompt is
/// re-normalised first so the boundary-aware matcher sees punctuation collapsed
/// to spaces. Mirror of `asksForConversationSummary` in the browser worker.
fn asks_for_conversation_summary(normalized: &str) -> bool {
    let cleaned = normalize_prompt(normalized);
    let lexicon = seed::lexicon();
    lexicon.mentions_role(seed::ROLE_CONVERSATION_RETURN_RECAP, &cleaned)
        || lexicon.mentions_role(seed::ROLE_CONVERSATION_SUMMARY_PHRASE, &cleaned)
        || lexicon.mentions_role(seed::ROLE_CONVERSATION_SUMMARY_COURTESY, &cleaned)
        || (lexicon.mentions_role(seed::ROLE_CONVERSATION_SUMMARY_DIRECTIVE, &cleaned)
            && lexicon.mentions_role(seed::ROLE_CONVERSATION_REFERENCE, &cleaned))
        || summary_directive_leads(&cleaned)
}

/// A bare summary directive standing alone is itself a request to summarize the
/// running conversation ("summarize", "резюме", "总结", …).
///
/// For whitespace-delimited scripts the directive must be the *whole* prompt, so
/// "summarize the article" is left for other handlers (a conversation object is
/// required via the directive∧reference arm instead). For CJK (no word spaces) a
/// leading substring suffices — mirroring the worker's historical `^总结` anchor
/// — which also keeps compounds like "工作总结" (a *work* summary) from being
/// mis-claimed. Surface words come from the `conversation_summary_directive`
/// role in the seed lexicon.
fn summary_directive_leads(cleaned: &str) -> bool {
    seed::lexicon()
        .words_for_role(seed::ROLE_CONVERSATION_SUMMARY_DIRECTIVE)
        .iter()
        .any(|word| {
            if contains_cjk(word) {
                cleaned.starts_with(word.as_str())
            } else {
                cleaned == word.as_str()
            }
        })
}

pub(super) fn try_summarize_conversation(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let is_return_recap = seed::lexicon().mentions_role(
        seed::ROLE_CONVERSATION_RETURN_RECAP,
        &normalize_prompt(normalized),
    );
    if !is_return_recap && !asks_for_conversation_summary(normalized) {
        return None;
    }
    let mut turns: Vec<DialogTurn> = log
        .events()
        .iter()
        .filter_map(|event| match event.kind {
            "prior_turn:user" => Some(DialogTurn::user(event.payload.clone())),
            "prior_turn:assistant" => Some(DialogTurn::assistant(event.payload.clone())),
            _ => None,
        })
        .collect();
    if turns.is_empty() {
        if let Some(content) = prompt
            .split_once(':')
            .map(|(_, content)| content.trim())
            .filter(|content| !content.is_empty())
        {
            turns.push(DialogTurn::user(content));
        }
    }
    let user_turn_count = turns.iter().filter(|turn| turn.role == "user").count();
    if user_turn_count == 0 {
        return None;
    }
    if is_return_recap {
        let summary =
            summarize_dialog_plain(&turns, RETURN_RECAP_MAX_WORDS, RETURN_RECAP_MAX_SENTENCES);
        if summary.is_empty() {
            return None;
        }
        log.append("filter:user", "conversation_return_recap".to_owned());
        log.append("summarization:format", "plain".to_owned());
        log.append(
            "summarization:max_words",
            RETURN_RECAP_MAX_WORDS.to_string(),
        );
        log.append(
            "summarization:max_sentences",
            RETURN_RECAP_MAX_SENTENCES.to_string(),
        );
        return Some(finalize_simple(
            prompt,
            log,
            "summarize_conversation",
            "response:summarize_conversation",
            &summary,
            0.9,
        ));
    }
    let language = detect_language(prompt).slug();
    // Standard mode keeps roughly 50% of the highest-weighted statements; with
    // the dialog bias (user +20, assistant -10) the user's questions dominate
    // the output while still keeping room for any assistant prose worth
    // remembering.
    let config = SummarizationConfig::default()
        .with_mode(SummarizationMode::Standard)
        .with_language(language);
    let summary = summarize_dialog(&turns, &config);
    let title = generate_chat_title(&turns, language);
    let user_turns: Vec<&str> = turns
        .iter()
        .filter(|turn| turn.role == "user")
        .map(|turn| turn.text.as_str())
        .collect();
    let mut body = match language {
        "ru" => {
            format!("Резюме разговора: {summary}\n\nЗаголовок: {title}\n\nРеплики пользователя:\n")
        }
        "zh" => format!("对话摘要:{summary}\n\n标题:{title}\n\n用户发言:\n"),
        _ => format!("Conversation summary: {summary}\n\nTitle: {title}\n\nUser turns:\n"),
    };
    for (index, turn) in user_turns.iter().enumerate() {
        writeln!(body, "  {}. {turn}", index + 1).expect("string write is infallible");
    }
    log.append("filter:user", "conversation_summary".to_owned());
    log.append("summarization:mode", "standard".to_owned());
    log.append("summarization:language", language.to_owned());
    log.append("chat_title", title);
    Some(finalize_simple(
        prompt,
        log,
        "summarize_conversation",
        "response:summarize_conversation",
        body.trim_end(),
        0.9,
    ))
}
