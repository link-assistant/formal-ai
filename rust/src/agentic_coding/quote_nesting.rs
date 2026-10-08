//! A quoted text that holds its own quote mark (PR #1188 G90, G107), and the
//! request faults the planner declines before any arm reads a payload.
//!
//! With one mark for both ends (apostrophe, double quote or backtick), `Replace 'a('x')' with 'b' in
//! f.rs.` pairs the outer opening quote with the first inner one. A closing
//! mark is followed by a space, the end, or punctuation; one followed straight
//! away by a letter, a digit or another quote mark opened an inner quote
//! instead. When the inner spans pair among themselves and the payload closes
//! at the end of the instruction, the payload runs there, whatever sentences
//! it holds. When the edit read from the request still does not hold such a
//! quote whole, the request is declined, naming the quote, rather than
//! guessed. The Rust original of `js/agentic/quote_nesting.mjs`.

use super::code_task;
use super::planner::AgenticPlan;
use crate::normal_markov::{quoted_segment_spans, wrapped_in_quote_pair};

/// The quote marks that open and close a quoted text alike.
const SAME_MARKS: [char; 3] = ['\'', '"', '`'];
/// The mark of a fenced block, whose body may hold any quote.
const FENCE: &str = "```";
/// How many characters of the request the decline quotes.
const FAULT_FRAGMENT_CHARS: usize = 32;
/// What may stand right after a closing mark.
const CLOSE_FOLLOWERS: &str = ".,;:!?)]}'\"`»";
/// What may stand right before an opening mark.
const OPEN_LEADERS: &str = "([{";
/// The seeded answer to a quote that holds its own mark.
const NESTED_INTENT: &str = "request-quote-nested";
/// The seeded answer to a clause of several edits that is no replacement.
const FRAGMENT_PLACEHOLDER: &str = "{fragment}";
const CLAUSE_PLACEHOLDER: &str = "{clause}";
const UNPLANNED_CLAUSE_INTENT: &str = "request-edit-clause-unplanned";

/// A quoted text whose closing mark opens an inner quote.
pub(super) struct NestedQuote {
    /// The seeded answer that declines it.
    pub(super) intent: &'static str,
    /// Up to 32 characters of the request from `at`, trailing space trimmed.
    pub(super) fragment: String,
}

/// Whether the character at byte `at` opens an inner quote right after a
/// close.
fn opens_inner(text: &str, at: usize) -> bool {
    text.get(at..)
        .and_then(|rest| rest.chars().next())
        .is_some_and(|after| after.is_ascii_alphanumeric() || SAME_MARKS.contains(&after))
}

/// Where a new text that starts at byte `from` ends, given the sentence end
/// `cut` (mirrors `wholePayloadEnd`).
///
/// When the payload cut there is no whole literal, or its closing mark opens
/// an inner quote, its marks are read by their shape: a mark after a space or
/// an opening bracket and before a word opens an inner quote, a mark after a
/// word and before a space or punctuation closes one, and the payload ends
/// where its own opening mark is closed. `cut` when a mark has neither shape
/// or the payload never closes.
pub(super) fn whole_payload_end(request: &str, from: usize, cut: usize) -> usize {
    let Some(payload) = request.get(from..cut) else {
        return cut;
    };
    if wrapped_in_quote_pair(payload) && !opens_inner(request, cut) {
        return cut;
    }
    let rest = &request[from..];
    let open = from + (rest.len() - rest.trim_start().len());
    let Some(mark) = request[open..].chars().next() else {
        return cut;
    };
    if !SAME_MARKS.contains(&mark) || request[open..].starts_with(FENCE) {
        return cut;
    }
    let end = super::positional_edit::instruction_end(request);
    if end <= open + mark.len_utf8() {
        return cut;
    }
    let mut depth = 1_usize;
    let mut before = mark;
    let mut characters = request[open + mark.len_utf8()..end]
        .char_indices()
        .peekable();
    while let Some((offset, character)) = characters.next() {
        let previous = before;
        before = character;
        if character != mark {
            continue;
        }
        let after = characters.peek().map(|(_, next)| *next);
        let closes = !previous.is_whitespace()
            && after.is_none_or(|next| next.is_whitespace() || CLOSE_FOLLOWERS.contains(next));
        let opens = (previous.is_whitespace() || OPEN_LEADERS.contains(previous))
            && after.is_some_and(|next| !next.is_whitespace());
        if opens && !closes {
            depth += 1;
        } else if closes {
            depth -= 1;
        } else {
            return cut;
        }
        if depth == 0 {
            return open + mark.len_utf8() + offset + mark.len_utf8();
        }
    }
    cut
}

/// The first quoted text of a request naming a file whose closing mark opens
/// an inner quote and that the request's edit, if one can be read, does not
/// hold whole (mirrors `nestedQuoteFault`).
pub(super) fn nested_quote_fault(text: &str) -> Option<NestedQuote> {
    let edit = super::write_request::compose_edit_request(text);
    // A request naming a file whose edit cannot be read at all is declined too.
    if edit.is_none() && super::module_function::paths_in(text).is_empty() {
        return None;
    }
    for segment in quoted_segment_spans(text) {
        let opening = text[segment.start..].chars().next();
        if !opening.is_some_and(|mark| SAME_MARKS.contains(&mark))
            || text[segment.start..].starts_with(FENCE)
            || !opens_inner(text, segment.end)
        {
            continue;
        }
        let next = text[segment.end..].chars().next().map_or(0, char::len_utf8);
        let across = &text[segment.start + 1..segment.end + next];
        let held = edit
            .as_ref()
            .is_some_and(|(_, old, new)| old.contains(across) || new.contains(across));
        if held {
            continue;
        }
        let fragment: String = text[segment.start..]
            .chars()
            .take(FAULT_FRAGMENT_CHARS)
            .collect();
        return Some(NestedQuote {
            intent: NESTED_INTENT,
            fragment: fragment.trim_end().to_owned(),
        });
    }
    None
}

/// The seeded answer declining a request whose quotes do not pair (PR #1188
/// G71) or nest (G90), whose edit names several files (G91, G104), or whose
/// several edits hold one that is no replacement (G106); mirrors
/// `requestFaultAnswer` in `js/agentic/planner.mjs`.
///
/// Steps joined by a sequence cue are each checked alone (G99).
pub(super) fn request_fault_answer(task: &str) -> Option<AgenticPlan> {
    let fault = crate::normal_markov::quote_fault(task)
        .map(|fault| (format!("request_quote_{}", fault.kind), fault.fragment))
        .or_else(|| {
            nested_quote_fault(task).map(|fault| (fault.intent.to_owned(), fault.fragment))
        });
    let answer = if let Some((intent, fragment)) = fault {
        code_task::render_seeded_change(&intent, task, "", &[(FRAGMENT_PLACEHOLDER, &fragment)])
    } else if super::request_sequence::sequence_steps(task).is_some() {
        return None;
    } else if let Some(targets) = super::replace_list::several_edit_targets(task) {
        code_task::render_seeded_list_change(
            "request_several_edit_targets",
            task,
            "",
            "{files}",
            &targets,
        )
    } else {
        let clause = super::replace_list::unplanned_edit_clause(task)?;
        code_task::render_seeded_change(
            UNPLANNED_CLAUSE_INTENT,
            task,
            "",
            &[(CLAUSE_PLACEHOLDER, &clause)],
        )
    };
    answer.map(AgenticPlan::Final)
}
