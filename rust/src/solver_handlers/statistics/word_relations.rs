//! Word problems as quantity relations (issue #1176 R2).
//!
//! The stated numbers of a question are joined by the relation its words
//! declare, read from the seed meanings in `data/seed/meanings-statistics.lino`:
//! `word_problem_gain` adds the next quantity (5 + 3 = 8), `word_problem_loss`
//! subtracts it (20 - 7 = 13), `word_problem_group` multiplies a per-group size
//! by a count (6 × 4 = 24), and `word_problem_share` divides a total shared
//! equally (24 ÷ 6 = 4). A question cue (`word_problem_question`) must be
//! present, and numbers no relation word joins are declined rather than
//! guessed. Twin of `tryRelationWordProblem` in
//! `js/worker/formal_ai_worker_word_relations.js`.

use super::{Decimal, mentions_marker, render_approx, stated_numbers};
use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::language::detect as detect_language;
use crate::seed::localized_response;
use crate::solver_handlers::finalize_simple;
use std::fmt::Write as _;

const QUESTION: &str = "word_problem_question";
const GAIN: &str = "word_problem_gain";
const LOSS: &str = "word_problem_loss";
const GROUP: &str = "word_problem_group";
const SHARE: &str = "word_problem_share";
const INTENT: &str = "word_problem_relation";
const MAX_VALUES: usize = 6;
const DIVISION_PRECISION: u32 = 6;

/// The sign the relation word between two stated numbers gives the later one:
/// `'+'` for a gain, `'-'` for a loss, `None` when no relation word (or both)
/// joins them.
fn relation_sign(window: &str) -> Option<char> {
    match (mentions_marker(window, GAIN), mentions_marker(window, LOSS)) {
        (true, false) => Some('+'),
        (false, true) => Some('-'),
        _ => None,
    }
}

/// Answer a word problem whose numbers are joined by seeded relation words.
pub(super) fn relation_word_problem(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let lowered = prompt.to_lowercase();
    if !mentions_marker(&lowered, QUESTION) && !mentions_marker(normalized, QUESTION) {
        return None;
    }
    let (values, positions) = stated_numbers(&lowered)?;
    if values.len() < 2 || values.len() > MAX_VALUES {
        return None;
    }
    let (result, exact, derivation, relations) =
        if values.len() == 2 && mentions_marker(&lowered, SHARE) {
            let (quotient, exact) = values[0].div(values[1], DIVISION_PRECISION)?;
            let derivation = format!(
                "{} ÷ {} = {}",
                values[0].render(),
                values[1].render(),
                render_approx(quotient, exact)
            );
            (quotient, exact, derivation, vec!["share".to_owned()])
        } else if values.len() == 2 && mentions_marker(&lowered, GROUP) {
            let product = values[0].mul(values[1])?;
            let derivation = format!(
                "{} × {} = {}",
                values[0].render(),
                values[1].render(),
                product.render()
            );
            (product, true, derivation, vec!["group".to_owned()])
        } else {
            sequential(&lowered, &values, &positions)?
        };
    for relation in relations {
        log.append("word_problem:relation", relation);
    }
    log.append("word_problem:derivation", derivation.clone());
    let language = detect_language(prompt).slug();
    let rendered = render_approx(result, exact);
    let body = localized_response(INTENT, language).map_or_else(
        || derivation.clone(),
        |template| {
            template
                .replace(concat!("{", "result}"), &rendered)
                .replace(concat!("{", "derivation}"), &derivation)
        },
    );
    Some(finalize_simple(
        prompt,
        log,
        INTENT,
        "response:word_problem_relation",
        &body,
        1.0,
    ))
}

/// Fold the numbers left to right, each joined to the running result by the
/// gain or loss word stated between it and the number before it.
fn sequential(
    lowered: &str,
    values: &[Decimal],
    positions: &[usize],
) -> Option<(Decimal, bool, String, Vec<String>)> {
    let mut result = values[0];
    let mut derivation = values[0].render();
    let mut relations = Vec::with_capacity(values.len() - 1);
    for index in 1..values.len() {
        let window = lowered.get(positions[index - 1]..positions[index])?;
        let sign = relation_sign(window)?;
        result = if sign == '+' {
            result.add(values[index])?
        } else {
            result.sub(values[index])?
        };
        let name = if sign == '+' { "gain" } else { "loss" };
        relations.push(format!("{name}:{}", values[index].render()));
        write!(derivation, " {sign} {}", values[index].render()).ok()?;
    }
    if result.is_negative() {
        return None;
    }
    write!(derivation, " = {}", result.render()).ok()?;
    Some((result, true, derivation, relations))
}
