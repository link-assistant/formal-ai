//! History-aware response-language retargeting.
//!
//! A follow-up such as "I do not understand English, write in Russian" is not a
//! new factual question. It asks the solver to preserve the previous user
//! request's semantic object and rerender that answer in the named response
//! language. The language marker itself comes from
//! `data/seed/meanings-translation.lino`; this module only decides when the
//! conversation history supplies a replayable request.

use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::solver::{ConversationRole, ConversationTurn};
use crate::translation::detect_response_language;

use super::web_requests::try_project_lookup_with_response_language;

pub fn try_response_language_followup(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
    history: &[ConversationTurn],
    promote_associative_repositories: bool,
) -> Option<SymbolicAnswer> {
    let target_language = detect_response_language(normalized)?;
    if !is_language_reanswer_followup(normalized) {
        return None;
    }
    let previous_user = history
        .iter()
        .rev()
        .find(|turn| turn.role == ConversationRole::User)?
        .content
        .trim();
    if previous_user.is_empty() {
        return None;
    }

    let mut candidate_log = log.clone();
    candidate_log.append(
        "response_language_followup:target",
        target_language.to_owned(),
    );
    candidate_log.append("language_to", target_language.to_owned());
    candidate_log.append(
        "response_language_followup:prior_user",
        previous_user.to_owned(),
    );
    candidate_log.append("response_language_followup:handler", "project_lookup");

    let answer = try_project_lookup_with_response_language(
        prompt,
        previous_user,
        &mut candidate_log,
        promote_associative_repositories,
        false,
        target_language,
    )?;
    *log = candidate_log;
    Some(answer)
}

fn is_language_reanswer_followup(normalized: &str) -> bool {
    let normalized = normalized.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        return false;
    }
    if normalized.contains("do not understand")
        || normalized.contains("don't understand")
        || normalized.contains("dont understand")
        || normalized.contains("can't understand")
        || normalized.contains("cant understand")
        || normalized.contains("cannot understand")
        || normalized.contains("не понимаю")
        || normalized.contains("не понял")
        || normalized.contains("не поняла")
        || normalized.contains("समझ नहीं")
        || normalized.contains("नहीं आती")
        || normalized.contains("不懂")
        || normalized.contains("看不懂")
        || normalized.contains("听不懂")
    {
        return true;
    }

    let word_count = normalized.split_whitespace().count();
    word_count <= 4
        && (normalized.contains("in english")
            || normalized.contains("in russian")
            || normalized.contains("in hindi")
            || normalized.contains("in chinese")
            || normalized.contains("на английском")
            || normalized.contains("по-английски")
            || normalized.contains("на русском")
            || normalized.contains("по-русски")
            || normalized.contains("по русски")
            || normalized.contains("на хинди")
            || normalized.contains("на китайском")
            || normalized.contains("अंग्रेजी में")
            || normalized.contains("अंग्रेज़ी में")
            || normalized.contains("रूसी में")
            || normalized.contains("हिंदी में")
            || normalized.contains("हिन्दी में")
            || normalized.contains("चीनी में")
            || normalized.contains("用英文")
            || normalized.contains("用英语")
            || normalized.contains("用英語")
            || normalized.contains("用俄语")
            || normalized.contains("用俄語")
            || normalized.contains("用印地语")
            || normalized.contains("用印地文")
            || normalized.contains("用中文")
            || normalized.contains("用汉语")
            || normalized.contains("用漢語"))
}
