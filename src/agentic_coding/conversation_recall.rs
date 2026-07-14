//! Answer "what were we talking about?" from the conversation itself (issue #687).
//!
//! One of the reported `OpenCode` failures was a meta-question about the dialogue —
//! "What we were talking about?" — falling to the unknown-reasoning blurb. Such a
//! question is not a factual lookup and needs no tool: the answer is already in the
//! message history the client replays on every request. This module recognises the
//! recall intent and renders the prior turns back to the user.
//!
//! Detection keys on the universal shape of a recall question (a "talk/discuss"
//! verb bound to a first-person-plural "we"/"were", or a "remind me what …"
//! opener), so it generalises across phrasings rather than matching fixed strings.

use crate::protocol::ChatMessage;

/// Build a conversational-recall answer from the history, or [`None`] when the
/// latest user turn is not asking what the conversation was about.
pub(super) fn recall_answer_for(messages: &[ChatMessage]) -> Option<String> {
    let task = latest_user_text(messages)?;
    if !is_recall_intent(&task) {
        return None;
    }
    let topics = prior_topics(messages);
    if topics.is_empty() {
        return Some(String::from(
            "We have not discussed anything yet — this is the start of our conversation.",
        ));
    }
    let mut answer = String::from("Here is what we have been talking about:\n\n");
    for topic in &topics {
        answer.push_str("- ");
        answer.push_str(topic);
        answer.push('\n');
    }
    Some(answer.trim_end().to_owned())
}

/// Whether `task` asks what the conversation has been about.
fn is_recall_intent(task: &str) -> bool {
    let lower = task.to_lowercase();
    // "remind me what we discussed / were talking about"
    if lower.contains("remind me what") || lower.contains("remind me of what") {
        return true;
    }
    // A talk/discuss verb bound to first-person-plural "we"/"were".
    let talks = [
        "talking about",
        "talk about",
        "discussing",
        "discussed",
        "discuss",
    ];
    let plural = lower.contains(" we ")
        || lower.contains("we ")
        || lower.contains(" were ")
        || lower.starts_with("were ");
    talks.iter().any(|verb| lower.contains(verb)) && plural
}

/// The prior user turns, most-recent-last, excluding the recall question itself.
/// Rendered as quoted topics for the recall answer.
fn prior_topics(messages: &[ChatMessage]) -> Vec<String> {
    let mut turns: Vec<String> = messages
        .iter()
        .filter(|m| m.role.eq_ignore_ascii_case("user"))
        .map(|m| m.content.plain_text().trim().to_owned())
        .filter(|text| !text.is_empty())
        .collect();
    // Drop the current recall question (the last user turn).
    turns.pop();
    turns
        .into_iter()
        .filter(|text| !is_recall_intent(text))
        .map(|text| format!("\"{text}\""))
        .collect()
}

/// The text of the most recent `user` turn.
fn latest_user_text(messages: &[ChatMessage]) -> Option<String> {
    messages
        .iter()
        .rev()
        .find(|message| message.role.eq_ignore_ascii_case("user"))
        .map(|message| message.content.plain_text())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognises_recall_phrasings() {
        assert!(is_recall_intent("What we were talking about?"));
        assert!(is_recall_intent("What were we talking about?"));
        assert!(is_recall_intent("Remind me what we discussed."));
        assert!(is_recall_intent("what did we discuss earlier"));
    }

    #[test]
    fn ignores_non_recall_prompts() {
        assert!(!is_recall_intent("When are the next elections in the USA?"));
        assert!(!is_recall_intent("Report this issue on GitHub"));
        assert!(!is_recall_intent("let me talk to support")); // no "we"/"were"
    }

    #[test]
    fn recalls_prior_user_turn() {
        let messages = vec![
            ChatMessage::user("When are the next elections in the USA?"),
            ChatMessage::assistant("The next US general election is in November 2026."),
            ChatMessage::user("What were we talking about?"),
        ];
        let answer = recall_answer_for(&messages).expect("recall answer");
        assert!(answer.to_lowercase().contains("election"));
    }
}
