//! Meanings-driven decomposition of two-sided comparisons (issue #840).

use serde_json::json;

use super::planner::{plan_one, tool_for, AgenticPlan, Capability};
use crate::protocol::ChatMessage;
use crate::seed;

const LEFT_SLOT: &str = concat!("{", "left", "}");
const RIGHT_SLOT: &str = concat!("{", "right", "}");
const EVIDENCE_SLOT: &str = concat!("{", "evidence", "}");

pub(super) fn plan_comparison_step(
    task: &str,
    messages: &[ChatMessage],
    tool_names: &[&str],
) -> Option<AgenticPlan> {
    let (left, right) = comparison_sides(task)?;
    let evidence = comparison_evidence(messages);
    let search = tool_for(tool_names, Capability::Search);
    match evidence.len() {
        0 => search.map(|tool| plan_one(tool, json!({ "query": left }).to_string())),
        1 => search.map(|tool| plan_one(tool, json!({ "query": right }).to_string())),
        _ => Some(AgenticPlan::Final(comparison_answer(
            task, &left, &right, &evidence,
        ))),
    }
}

pub(super) fn comparison_sides(task: &str) -> Option<(String, String)> {
    let normalized = crate::engine::normalize_prompt(task);
    let lexicon = seed::lexicon();
    let mut markers = lexicon.words_for_role(seed::ROLE_COMPARISON_INFIX);
    markers.sort_by_key(|marker| std::cmp::Reverse(marker.chars().count()));
    for marker in markers {
        let marker = crate::engine::normalize_prompt(&marker);
        let Some(position) = bounded_find(&normalized, &marker) else {
            continue;
        };
        let before = normalized[..position].trim();
        let after = normalized[position + marker.len()..].trim();

        // Inflected frames such as “чем отличается A от B” put both operands
        // after the comparison verb. The right-hand separator is another role.
        for rhs_marker in lexicon.words_for_role(seed::ROLE_COMPARISON_RHS_MARKER) {
            let rhs_marker = crate::engine::normalize_prompt(&rhs_marker);
            if let Some(rhs_position) = bounded_find(after, &rhs_marker) {
                let left = clean_side(&after[..rhs_position]);
                let right = clean_side(&after[rhs_position + rhs_marker.len()..]);
                if !left.is_empty() && !right.is_empty() {
                    return Some((left, right));
                }
            }
        }
        if !before.is_empty() && !after.is_empty() {
            return Some((clean_side(before), clean_side(after)));
        }
    }
    None
}

fn bounded_find(text: &str, marker: &str) -> Option<usize> {
    text.match_indices(marker).find_map(|(position, _)| {
        let before = text[..position].chars().next_back();
        let after = text[position + marker.len()..].chars().next();
        let left_boundary = before.is_none_or(|character| !character.is_alphanumeric());
        let right_boundary = after.is_none_or(|character| !character.is_alphanumeric());
        (left_boundary && right_boundary).then_some(position)
    })
}

fn clean_side(side: &str) -> String {
    let mut side = side
        .trim_matches(|character: char| {
            character.is_whitespace()
                || character.is_ascii_punctuation()
                || matches!(character, '？' | '。' | '«' | '»')
        })
        .to_owned();
    let mut noise = seed::lexicon().words_for_role(seed::ROLE_COMPARISON_QUERY_NOISE);
    noise.sort_by_key(|surface| std::cmp::Reverse(surface.chars().count()));
    for surface in noise {
        remove_bounded_surface(&mut side, &crate::engine::normalize_prompt(&surface));
    }
    side.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn remove_bounded_surface(text: &mut String, surface: &str) {
    let mut offset = 0;
    while let Some(relative) = text[offset..].find(surface) {
        let start = offset + relative;
        let end = start + surface.len();
        let left = start == 0
            || text[..start]
                .chars()
                .next_back()
                .is_some_and(|character| !character.is_alphanumeric());
        let right = end == text.len()
            || text[end..]
                .chars()
                .next()
                .is_some_and(|character| !character.is_alphanumeric());
        if left && right {
            text.replace_range(start..end, " ");
            offset = start + 1;
        } else {
            offset = end;
        }
    }
}

fn comparison_evidence(messages: &[ChatMessage]) -> Vec<String> {
    let current_turn = messages
        .iter()
        .rposition(|message| message.role.eq_ignore_ascii_case("user"))
        .map_or(0, |index| index + 1);
    messages
        .iter()
        .enumerate()
        .skip(current_turn)
        .filter(|(_, message)| message.role.eq_ignore_ascii_case("tool"))
        .filter(|(index, _)| {
            super::progress::result_capability(messages, *index) == Some(Capability::Search)
        })
        .filter_map(|(_, message)| {
            super::tool_result::normalized_payload(&message.content.plain_text())
                .filter(|text| !text.trim().is_empty())
        })
        .collect()
}

fn comparison_answer(task: &str, left: &str, right: &str, evidence: &[String]) -> String {
    let language = crate::language::detect(task).slug();
    let intent = if evidence.is_empty() {
        "comparison_decomposed_no_evidence"
    } else {
        "comparison_decomposed_evidence"
    };
    seed::localized_response(intent, language)
        .unwrap_or_default()
        .replace(LEFT_SLOT, left)
        .replace(RIGHT_SLOT, right)
        .replace(EVIDENCE_SLOT, &evidence.join("\n\n"))
}
