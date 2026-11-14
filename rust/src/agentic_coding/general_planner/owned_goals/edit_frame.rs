//! Owned source-edit frames reuse the primitive parsers and consumed modifiers.
use super::compose_edit_clauses;
use crate::agentic_coding::edit_scope::without_all_occurrence_cues;
use crate::agentic_coding::replace_list::{
    replace_clauses, replace_list, several_edit_targets, unplanned_edit_clause,
};
use crate::agentic_coding::request_sequence::{request_sequence, sequence_steps};
use crate::agentic_coding::write_request::bare_surfaces;

fn grammar(text: &str) -> bool {
    text.chars()
        .all(|character| " \t\r\n.!?。！？।;；".contains(character))
}
fn frame_prefix(text: &str) -> bool {
    let cue = text
        .trim()
        .trim_end_matches(|character| ",.!?。！？।;；".contains(character))
        .trim()
        .to_lowercase();
    grammar(text) || super::seeded_action_prologue(&cue)
}
pub(super) fn complete_edit_frame(request: &str, spans: [(usize, usize); 2]) -> bool {
    let view = without_all_occurrence_cues(request);
    let mut ordered = spans;
    ordered.sort_by_key(|span| span.0);
    let mut end = 0;
    for (start, next) in ordered {
        if next < start || next > request.len() || request.get(start..next).is_none() {
            return false;
        }
        let Some(gap) = view.get(end..end.max(start)) else {
            return false;
        };
        if !(if end == 0 {
            frame_prefix(gap)
        } else {
            grammar(gap)
        }) {
            return false;
        }
        end = end.max(next);
    }
    view.get(end..).is_some_and(grammar)
}
pub(super) fn effective_edit_spans(
    request: &str,
    edit: &crate::agentic_coding::write_request::EditClauses,
) -> Option<[(usize, usize); 2]> {
    let [file, action] = edit.spans?;
    let mut operands = crate::normal_markov::quoted_segment_spans(request)
        .into_iter()
        .filter(|span| {
            span.start >= action.0
                && span.end <= action.1
                && !(span.start < file.1 && span.end > file.0)
        });
    let (Some(old), Some(next)) = (operands.next(), operands.next()) else {
        return edit.spans;
    };
    if old.text != edit.edit.1 {
        return edit.spans;
    }
    let lead = if file.0 >= old.end && file.1 <= next.start {
        format!(
            "{} {}",
            request.get(old.end..file.0)?,
            request.get(file.1..next.start)?
        )
    } else {
        request.get(old.end..next.start)?.to_owned()
    };
    if !bare_surfaces("file_edit_new_lead_cue").contains(&lead.trim().to_lowercase()) {
        return edit.spans;
    }
    Some([
        file,
        (
            action.0,
            crate::agentic_coding::quote_nesting::whole_payload_end(request, next.start, next.end),
        ),
    ])
}

pub(super) fn owns_edit_request_prefix(request: &str) -> bool {
    let Some(spans) =
        compose_edit_clauses(request).and_then(|edit| effective_edit_spans(request, &edit))
    else {
        return false;
    };
    let end = spans.iter().map(|span| span.1).max().unwrap_or(0);
    request
        .get(..end)
        .is_some_and(|prefix| complete_edit_frame(prefix, spans))
}
pub(super) fn owned_block_edit(request: &str) -> bool {
    let Some(block) = crate::agentic_coding::positional_edit::introduced_block(request) else {
        return false;
    };
    let Some(edit) = compose_edit_clauses(request) else {
        return false;
    };
    let Some(head) = compose_edit_clauses(block.head) else {
        return false;
    };
    edit.spans.is_none()
        && edit.edit.2 == block.text
        && head.edit.0 == edit.edit.0
        && head.edit.1 == edit.edit.1
        && effective_edit_spans(block.head, &head)
            .is_some_and(|spans| complete_edit_frame(block.head, spans))
}

pub(super) fn supported_edit_frame(request: &str) -> bool {
    if owned_block_edit(request) {
        return true;
    }
    if compose_edit_clauses(request)
        .and_then(|edit| effective_edit_spans(request, &edit))
        .is_some_and(|spans| complete_edit_frame(request, spans))
    {
        return true;
    }
    let Some((target, _)) = replace_list(request) else {
        return false;
    };
    let Some(clauses) = replace_clauses(request) else {
        return false;
    };
    let Some((named, spans)) = clauses.iter().find_map(|clause| {
        let edit = compose_edit_clauses(clause)?;
        (edit.edit.0 == target).then_some((clause, edit.spans?))
    }) else {
        return false;
    };
    let Some(file_clause) = named.get(spans[0].0..spans[0].1) else {
        return false;
    };
    clauses.iter().all(|clause| {
        let bound = if compose_edit_clauses(clause).is_none() {
            format!("{file_clause} {clause}")
        } else {
            clause.clone()
        };
        compose_edit_clauses(&bound).is_some_and(|edit| {
            edit.edit.0 == target
                && effective_edit_spans(&bound, &edit)
                    .is_some_and(|spans| complete_edit_frame(&bound, spans))
        })
    })
}
pub(super) fn delegated_edit_request(request: &str) -> bool {
    owned_block_edit(request)
        || several_edit_targets(request).is_some()
        || unplanned_edit_clause(request).is_some()
        || sequence_steps(request)
            .is_some_and(|parts| parts.iter().all(|part| supported_edit_frame(part)))
        || request_sequence(request)
            .is_some_and(|parts| parts.iter().skip(1).all(|part| supported_edit_frame(part)))
}
