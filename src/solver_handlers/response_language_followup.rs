//! History-aware response-language retargeting.
//!
//! A follow-up such as "I do not understand English, write in Russian" is not a
//! new factual question. It asks the solver to preserve the previous user
//! request's semantic object and rerender that answer in the named response
//! language.
//!
//! Issue #556 generalizes this beyond a single hardcoded handler: rather than
//! re-running one project-lookup path, the follow-up replays the previous
//! request through the *whole* solver with the requested language forced onto
//! every localizable answer family (concept lookup, repository/project lookup,
//! …). This is the universal recursive-reasoning step — the solver re-derives
//! the prior answer, now constrained to speak the requested language — so the
//! retarget covers the entire class of answerable requests, not just one shape.
//!
//! Both the language marker and the "I cannot understand" trigger are grounded
//! in `data/seed/meanings-translation.lino`: [`detect_response_language`] reads
//! the response-language marker role and [`detect_comprehension_failure`] reads
//! the comprehension-failure marker role. This module holds no natural-language
//! phrase table of its own; widening the vocabulary is a pure seed edit.

use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::solver::{ConversationRole, ConversationTurn, SolverConfig, UniversalSolver};
use crate::translation::{detect_comprehension_failure, detect_response_language};

pub fn try_response_language_followup(
    _prompt: &str,
    normalized: &str,
    log: &mut EventLog,
    history: &[ConversationTurn],
    config: SolverConfig,
) -> Option<SymbolicAnswer> {
    // Recursion guard: a replay already carries a forced language, so it must
    // never re-enter and replay itself again.
    if config.forced_response_language.is_some() {
        return None;
    }
    let target_language = detect_response_language(normalized)?;
    if !is_language_reanswer_followup(normalized) {
        return None;
    }

    // Recover the most recent user request and the history that preceded it, so
    // the replay sees the same context the original answer did.
    let previous_index = history
        .iter()
        .rposition(|turn| turn.role == ConversationRole::User)?;
    let previous_user = history[previous_index].content.trim().to_owned();
    if previous_user.is_empty() {
        return None;
    }
    let prior_history = &history[..previous_index];

    // Replay the previous request through the whole solver with the requested
    // language forced. Every handler that can localize its answer now does, so
    // the retarget generalizes across the answerable class.
    let mut replay_config = config;
    replay_config.forced_response_language = Some(target_language);
    let replay = UniversalSolver::new(replay_config).solve_with_history(&previous_user, prior_history);

    // A replay that could not reconstruct a concrete answer is not a useful
    // re-answer; fall through so the normal handlers see the follow-up.
    if is_inconclusive(&replay.intent) {
        return None;
    }

    // Record the retarget provenance on the outer trace. The returned answer's
    // own evidence (repository slug, sources, `language_to:<code>`) already
    // comes from the replay; these markers name *why* it was produced.
    log.append(
        "response_language_followup:target",
        target_language.to_owned(),
    );
    log.append("language_to", target_language.to_owned());
    log.append("response_language_followup:prior_user", previous_user);
    log.append(
        "response_language_followup:handler",
        replay.intent.clone(),
    );

    // Splice the follow-up provenance onto the replayed answer so it survives
    // even though the outer projection is rebuilt from the replay (which does
    // not itself know it was a language retarget).
    let mut answer = replay;
    push_unique(
        &mut answer.evidence_links,
        format!("response_language_followup:target:{target_language}"),
    );
    push_unique(
        &mut answer.evidence_links,
        format!("language_to:{target_language}"),
    );
    push_unique(
        &mut answer.evidence_links,
        format!("response_language_followup:handler:{}", answer.intent),
    );
    Some(answer)
}

/// A follow-up fires when the user reports they cannot understand the prior
/// answer (seed-grounded [`detect_comprehension_failure`]) or when the prompt
/// is a terse language-switch request. In the terse case the caller has already
/// confirmed a seed-grounded response-language marker is present, so a short
/// prompt with no fresh subject of its own is enough to treat it as a retarget
/// of the previous turn rather than a new question.
fn is_language_reanswer_followup(normalized: &str) -> bool {
    let normalized = normalized.trim();
    if normalized.is_empty() {
        return false;
    }
    if detect_comprehension_failure(normalized) {
        return true;
    }
    // Chinese and other scriptio-continua markers carry no inter-word spaces,
    // so a bare "用中文" counts as one word here — still terse, still a switch.
    normalized.split_whitespace().count() <= 4
}

/// Intents that mean the replay never reached a concrete answer.
fn is_inconclusive(intent: &str) -> bool {
    matches!(intent, "unknown" | "ill_formed" | "punctuation_only_prompt")
        || intent.starts_with("clarify")
}

fn push_unique(links: &mut Vec<String>, link: String) {
    if !links.contains(&link) {
        links.push(link);
    }
}
