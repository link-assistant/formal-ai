//! "What is left to reach my goal?" answered from the dialogue's world model
//! (issue #702).
//!
//! Issue #649 built the symbolic world model: contexts that are links networks,
//! and their difference. [`crate::world_model_dialog`] drives that substrate
//! from the conversation itself — declarative turns seed the current state,
//! "I want …" turns build the target state. This handler is the last step: it
//! makes the difference *askable from chat*.
//!
//! Three properties are load-bearing.
//!
//! * **Trace-only until opted in.** With the default
//!   [`WorldModelMode::Off`](crate::world_model_dialog::WorldModelMode::Off) the
//!   handler declines immediately, so the solver behaves exactly as it did
//!   before (R13). `FORMAL_AI_WORLD_MODEL_MODE=track` opts in.
//! * **Computed, not remembered.** The answer is the current→target difference
//!   recomputed from the replayed conversation on every ask, so it is a function
//!   of the dialogue and identical on every replay.
//! * **Four languages, no phrase table in Rust.** The question is recognized
//!   through the `world_state_query` cue set in `data/meta/cue-lexicon.lino`,
//!   and the reply prose is loaded from `data/seed/multilingual-responses.lino`
//!   in the prompt's language with an English fallback.

use crate::engine::{stable_id, SymbolicAnswer};
use crate::event_log::EventLog;
use crate::language::detect as detect_language;
use crate::seed;
use crate::solver::{ConversationTurn, SolverConfig};
use crate::solver_handlers::finalize_simple;
use crate::world_model_atoms::{classify, UtteranceKind};
use crate::world_model_dialog::{record_world_model, DialogueWorldModel};

pub fn try_world_state(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
    history: &[ConversationTurn],
    config: SolverConfig,
) -> Option<SymbolicAnswer> {
    if !config.world_model_mode.emits_artifact() {
        return None;
    }
    if classify(normalized) != UtteranceKind::RemainingQuery {
        return None;
    }

    // Replay the conversation into a fresh model, then let the question itself
    // be the last observed turn: the query is recorded in the append-only
    // synchronization log like every other step.
    let mut model = DialogueWorldModel::from_turns(history);
    model.observe_user(prompt);

    let difference = model.difference();
    let remaining = model.remaining();
    // Nothing to compare against yet: with no target the difference is empty for
    // an uninteresting reason, so the question is better answered elsewhere.
    if remaining.is_empty() && difference.is_empty() && history.is_empty() {
        return None;
    }

    let diff_id = stable_id("world_diff", &difference.links_notation());
    let intent = if remaining.is_empty() {
        "world_state_reached"
    } else {
        "world_state_remaining"
    };
    let language = detect_language(prompt);
    let template = seed::response_for(intent, language.slug())
        .or_else(|| seed::response_for(intent, "en"))
        .unwrap_or_else(|| String::from("{remaining}"));
    let listed = remaining
        .iter()
        .map(crate::substitution::SubstitutionLink::pattern_text)
        .collect::<Vec<_>>()
        .join("; ");
    let body = template
        .replace("{count}", &remaining.len().to_string())
        .replace("{remaining}", &listed)
        .replace("{diff_id}", &diff_id);

    for link in &remaining {
        log.append("world_state:remaining", link.pattern_text());
    }
    log.append("world_state:difference", difference.links_notation());
    log.append("world_state:sync_events", model.events().len().to_string());
    record_world_model(log, &model, config.world_model_mode);

    Some(finalize_simple(
        prompt,
        log,
        intent,
        "response:world_state",
        &body,
        if remaining.is_empty() { 0.9 } else { 0.8 },
    ))
}
