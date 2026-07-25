//! Decomposition as a working task: split a task into sub-tasks, judge whether
//! a task is atomic, and name the first step (issue #847).
//!
//! Before this handler both halves were refused — "split this task into
//! subtasks" and "is this task atomic?" fell through to the unknown-prompt
//! fallback. They are two views of one recursion, so they share one handler and
//! one call into [`crate::task_decomposition`]: the atomicity answer is the
//! recursion's base case and the split answer is the recursion's result, which
//! is why the two can never disagree.
//!
//! Recognition is role-driven per issue #386: this module names the
//! language-neutral semantic gates, and every natural-language surface — the
//! splitting verbs, the sub-task nouns, the atomicity predicates, the reply
//! text — lives in `data/seed/meanings-decomposition.lino` and
//! `data/seed/multilingual-responses-decomposition.lino`. There is no
//! per-language phrase list in Rust.

use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::language::detect as detect_language;
use crate::seed::{
    self, response_for, ROLE_DECOMPOSABLE_TASK_NOUN, ROLE_SUBTASK_ENUMERATION_CUE,
    ROLE_SUBTASK_UNIT_NOUN, ROLE_TASK_ATOMICITY_PREDICATE, ROLE_TASK_DECOMPOSITION_ACTION,
    ROLE_TASK_FIRST_STEP_CUE,
};
use crate::solver::SolverConfig;
use crate::task_decomposition::{record_task_decomposition, Decomposition};

use super::finalize_simple;

/// Which of the three questions about a task the prompt is asking.
///
/// All three are answered from one decomposition, so the variant only selects
/// which part of it the reader is shown.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum DecompositionQuestion {
    /// "Is this task atomic?" — the base case, answerable standalone.
    Atomicity,
    /// "What is the first step?" — the first leaf of the same decomposition.
    FirstStep,
    /// "Split this task into subtasks." — the whole numbered list.
    Split,
}

/// Whether the prompt asks one of the three decomposition questions.
///
/// Exposed so `intent_formalization` can promote the handler ahead of the
/// how-to and text-manipulation readings of the same verbs; the routing
/// relevant and the handler therefore share one recogniser and cannot drift.
#[must_use]
pub fn looks_like_task_decomposition(normalized: &str) -> bool {
    classify(normalized).is_some()
}

/// Entry point used by the regular handler table, at the depth bound shipped in
/// the default solver configuration.
pub fn try_task_decomposition(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    try_task_decomposition_with_depth(
        prompt,
        normalized,
        log,
        SolverConfig::default().max_decomposition_depth,
    )
}

/// Entry point used by the contextual dispatch path, where the configured
/// `max_decomposition_depth` is available.
///
/// The depth is a parameter rather than a constant because the issue requires
/// the recursion to terminate at `max_decomposition_depth` *and report it*: a
/// reader who lowers the bound must see the cut in the answer.
pub fn try_task_decomposition_with_depth(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
    max_depth: u8,
) -> Option<SymbolicAnswer> {
    let question = classify(normalized)?;
    let task = extract_task(prompt);
    if task.is_empty() {
        return None;
    }

    let language = detect_language(prompt).slug();
    log.append("language", language.to_owned());
    log.append("task_decomposition:question", question.slug().to_owned());
    let decomposition = record_task_decomposition(log, &task, max_depth);

    let (intent, body) = match question {
        DecompositionQuestion::Atomicity => atomicity_answer(&decomposition, language),
        DecompositionQuestion::FirstStep => first_step_answer(&decomposition, language),
        DecompositionQuestion::Split => split_answer(&decomposition, language),
    };
    Some(finalize_simple(
        prompt,
        log,
        intent,
        "response:task_decomposition",
        &body,
        0.86,
    ))
}

impl DecompositionQuestion {
    const fn slug(self) -> &'static str {
        match self {
            Self::Atomicity => "atomicity",
            Self::FirstStep => "first_step",
            Self::Split => "split",
        }
    }
}

/// Decide which question the prompt asks, or `None` when it asks none of them.
///
/// The order matters: an atomicity question often also names sub-tasks ("can it
/// usefully be split further?"), and a first-step question often also names the
/// steps, so the more specific reading is tested first.
fn classify(normalized: &str) -> Option<DecompositionQuestion> {
    let task_noun = mentions(ROLE_DECOMPOSABLE_TASK_NOUN, normalized);
    let unit_noun = mentions(ROLE_SUBTASK_UNIT_NOUN, normalized);

    if mentions(ROLE_TASK_ATOMICITY_PREDICATE, normalized) && (task_noun || unit_noun) {
        return Some(DecompositionQuestion::Atomicity);
    }
    if mentions(ROLE_TASK_FIRST_STEP_CUE, normalized) && task_noun {
        return Some(DecompositionQuestion::FirstStep);
    }
    if mentions(ROLE_TASK_DECOMPOSITION_ACTION, normalized) && (unit_noun || task_noun) {
        return Some(DecompositionQuestion::Split);
    }
    // "What are the steps for X" names no splitting verb. It only reaches
    // decomposition when it names both the pieces and the thing being split, so
    // a plain "what are the steps to install rust" stays with the procedural
    // how-to handler.
    if mentions(ROLE_SUBTASK_ENUMERATION_CUE, normalized) && unit_noun && task_noun {
        return Some(DecompositionQuestion::Split);
    }
    None
}

/// Both matchers are consulted because some surfaces are stems ("атомарн",
/// "разбе") that only a raw substring match can catch, while whole-word
/// surfaces must stay token-bounded so they don't fire inside longer words.
fn mentions(role: &str, normalized: &str) -> bool {
    let lexicon = seed::lexicon();
    lexicon.mentions_role(role, normalized) || lexicon.mentions_role_raw(role, normalized)
}

fn atomicity_answer(decomposition: &Decomposition, language: &str) -> (&'static str, String) {
    if decomposition.is_atomic() {
        return (
            "task_atomicity",
            response(language, "task_atomicity_yes", "Yes."),
        );
    }
    let lead = response(language, "task_atomicity_no", "No.");
    (
        "task_atomicity",
        with_sub_tasks(&lead, decomposition, language),
    )
}

fn first_step_answer(decomposition: &Decomposition, language: &str) -> (&'static str, String) {
    let lead = response(language, "task_first_step_lead", "The first step is:");
    let leaves = decomposition.leaves();
    let first = leaves
        .first()
        .map_or_else(|| decomposition.task.clone(), |leaf| leaf.text.clone());
    (
        "task_first_step",
        format!("{}\n{}", lead.as_str(), first.as_str()),
    )
}

fn split_answer(decomposition: &Decomposition, language: &str) -> (&'static str, String) {
    if decomposition.is_atomic() {
        return (
            "task_decomposition",
            seeded(language, "task_decomposition_atomic"),
        );
    }
    let lead = response(
        language,
        "task_decomposition_lead",
        "Sub-tasks, each with a completion criterion you can observe:",
    );
    (
        "task_decomposition",
        with_sub_tasks(&lead, decomposition, language),
    )
}

/// Render `lead`, then the numbered sub-tasks, then — only when the recursion
/// actually stopped early — the note that says so. The issue requires a depth
/// bound to be reported rather than hidden.
fn with_sub_tasks(lead: &str, decomposition: &Decomposition, language: &str) -> String {
    let marker = response(language, "task_decomposition_depth_marker", "[depth bound]");
    let mut lines = vec![lead.to_owned()];
    lines.extend(decomposition.numbered_lines(&marker));
    if decomposition.depth_bound_reached() {
        lines.push(seeded(language, "task_decomposition_depth_bound"));
    }
    lines.join("\n")
}

/// Look the reply text up in the seed response catalogue, falling back to
/// English and then to a built-in string so a missing translation degrades to a
/// readable answer instead of an empty one.
fn response(language: &str, intent: &str, fallback: &str) -> String {
    response_for(intent, language)
        .or_else(|| response_for(intent, "en"))
        .unwrap_or_else(|| fallback.to_owned())
}

/// Full-sentence replies live only in the seed (R379 forbids user-facing prose
/// in `src/`). The English seed row for every decomposition intent is guaranteed
/// present and test-pinned, so a missing translation degrades to English and
/// never to an empty answer in practice.
fn seeded(language: &str, intent: &str) -> String {
    response_for(intent, language)
        .or_else(|| response_for(intent, "en"))
        .unwrap_or_default()
}

/// Recover the task the prompt is asking about.
///
/// The prompts in the issue quote the task ("… nothing else: 'Add a
/// paths-ignore filter …'"), so a quoted span wins. Failing that, the text
/// after the last colon is the task. Failing that, the prompt itself is the
/// task — which is what makes "Split this into steps" work on a bare task.
fn extract_task(prompt: &str) -> String {
    quoted_span(prompt)
        .or_else(|| after_last_colon(prompt))
        .unwrap_or_else(|| prompt.to_owned())
        .trim()
        .to_owned()
}

/// The quote pairs used across the supported languages. Left and right differ
/// for every pair except the straight quotes, which close with themselves.
const QUOTE_PAIRS: &[(char, char)] = &[
    ('«', '»'),
    ('“', '”'),
    ('‘', '’'),
    ('「', '」'),
    ('『', '』'),
    ('"', '"'),
    ('\'', '\''),
];

fn quoted_span(prompt: &str) -> Option<String> {
    let characters: Vec<char> = prompt.chars().collect();
    let (open_index, closing) = characters
        .iter()
        .enumerate()
        .find_map(|(index, character)| {
            QUOTE_PAIRS
                .iter()
                .find(|(open, _)| open == character)
                .map(|(_, close)| (index, *close))
        })?;
    let close_index = characters
        .iter()
        .rposition(|character| *character == closing)?;
    if close_index <= open_index + 1 {
        return None;
    }
    let span: String = characters[open_index + 1..close_index].iter().collect();
    let trimmed = span.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_owned())
}

fn after_last_colon(prompt: &str) -> Option<String> {
    let index = prompt.rfind([':', '：'])?;
    let tail = prompt[index..]
        .char_indices()
        .nth(1)
        .map(|(offset, _)| &prompt[index + offset..])?;
    let trimmed = tail.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_owned())
}
