//! Numbered lines moved from one file to another (PR #1188 G85).
//!
//! `Move lines 671-690 of A to the end of B` was planned as `mv A B`, a rename
//! of the whole file. A line-range move is the range placed in the
//! destination (its end, or its start when the request says so) and then
//! removed from the source, each the computed change it would be alone: read,
//! compute, the smallest unique edit, the digest check. The destination is
//! the path after the last seeded destination or position cue that a path
//! follows (before the last cue, in a postpositional language); the source is
//! the other path. The Rust original of `js/agentic/line_range_move.mjs`.

use super::code_artifact::{source_from_agent_read_result, source_from_read_result};
use super::code_task::render_seeded_change;
use super::final_result::FinalResult;
use super::planner::{AgenticPlan, Capability, plan_one, tool_for};
use super::workspace_change::{read_arguments, result_for_path};
use super::workspace_computed_change::{
    Computation, ComputedChange, plan_computed_change_step, sentence_words,
};
use super::workspace_line_operation::{LineOperation, numbered_lines};
use super::write_request::{
    bare_surfaces, clean_cue_token, clean_path_token, looks_like_file_path, safe_relative_path,
    tokens,
};
use crate::normal_markov::quoted_segment_spans;
use crate::protocol::ChatMessage;
use crate::seed;
use crate::solver_handlers::text_outside_quoted_segments;

const LINES_SLOT: &str = concat!("{", "lines", "}");
const SOURCE_SLOT: &str = concat!("{", "source", "}");
const COUNT_SLOT: &str = concat!("{", "count", "}");

/// The roles whose words lead to the destination file.
const CUE_ROLES: [&str; 3] = [
    "file_write_destination_cue",
    "file_edit_position_end",
    "file_edit_position_start",
];

/// One numbered line range moved from `source` to `target`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct LineRangeMove {
    source: String,
    target: String,
    first: usize,
    last: usize,
    at_end: bool,
}

fn is_path(path: &str) -> bool {
    looks_like_file_path(path) && safe_relative_path(path)
}

/// The move `task` asks for, when it moves one numbered line range from one
/// file to another (mirrors `lineRangeMove`).
pub(super) fn line_range_move(task: &str) -> Option<LineRangeMove> {
    let outside = sentence_words(&text_outside_quoted_segments(task));
    let lexicon = seed::lexicon();
    if !lexicon.mentions_role("line_move_action", &outside) {
        return None;
    }
    let segments = quoted_segment_spans(task);
    if segments.iter().any(|segment| !is_path(segment.text.trim())) {
        return None;
    }
    let ranges = numbered_lines(task);
    let [(first, last)] = ranges[..] else {
        return None;
    };
    let words: Vec<_> = tokens(task)
        .into_iter()
        .filter(|token| {
            !segments
                .iter()
                .any(|segment| token.start < segment.end && token.end > segment.start)
        })
        .collect();
    let cues: Vec<String> = CUE_ROLES.into_iter().flat_map(bare_surfaces).collect();
    let paths: Vec<(usize, String)> = words
        .iter()
        .enumerate()
        .map(|(index, token)| (index, clean_path_token(token.text).to_owned()))
        .filter(|(_, path)| is_path(path))
        .collect();
    let mut distinct: Vec<&str> = paths.iter().map(|(_, path)| path.as_str()).collect();
    distinct.sort_unstable();
    distinct.dedup();
    if distinct.len() != 2 {
        return None;
    }
    let cue_at: Vec<usize> = words
        .iter()
        .enumerate()
        .filter(|(_, token)| cues.contains(&clean_cue_token(token.text)))
        .map(|(index, _)| index)
        .collect();
    let followed = cue_at
        .iter()
        .rev()
        .find(|at| paths.iter().any(|(index, _)| index > *at));
    let target = followed.map_or_else(
        || {
            let last_cue = *cue_at.last()?;
            paths
                .iter()
                .rev()
                .find(|(index, _)| *index < last_cue)
                .map(|(_, path)| path.clone())
        },
        |at| {
            paths
                .iter()
                .find(|(index, _)| index > at)
                .map(|(_, path)| path.clone())
        },
    )?;
    let source = paths.iter().find(|(_, path)| *path != target)?.1.clone();
    let at_start = lexicon.mentions_role("file_edit_position_start", &outside)
        && !lexicon.mentions_role("file_edit_position_end", &outside);
    Some(LineRangeMove {
        source,
        target,
        first,
        last,
        at_end: !at_start,
    })
}

/// Lines `first..=last` (one-based) of `source`, joined, or `None`.
pub(super) fn range_text(source: &str, first: usize, last: usize) -> Option<String> {
    let body = source.strip_suffix('\n').unwrap_or(source);
    let lines: Vec<&str> = if source.is_empty() {
        Vec::new()
    } else {
        body.split('\n').collect()
    };
    (first >= 1 && last >= first && last <= lines.len()).then(|| lines[first - 1..last].join("\n"))
}

/// The seeded answer stating the move, once both files are observed.
const fn moved_intent(single: bool, at_end: bool) -> &'static str {
    match (single, at_end) {
        (true, true) => "line_number_moved_end",
        (true, false) => "line_number_moved_start",
        (false, true) => "line_range_moved_end",
        (false, false) => "line_range_moved_start",
    }
}

/// Read the source, place the range in the destination, remove it from the
/// source, and state the move once both are observed (mirrors
/// `planLineRangeMoveStep`).
pub(super) fn plan_line_range_move_step(
    task: &str,
    current_turn: &[ChatMessage],
    tool_names: &[&str],
    order: &LineRangeMove,
    result: &mut Option<FinalResult>,
) -> Option<AgenticPlan> {
    let Some(read) = result_for_path(current_turn, Capability::Read, &order.source, None) else {
        let tool = tool_for(tool_names, Capability::Read)?;
        return Some(plan_one(tool, read_arguments(&order.source)));
    };
    let lines = if order.first == order.last {
        order.first.to_string()
    } else {
        format!("{}-{}", order.first, order.last)
    };
    // A read that came back as the client's file block is the file, whatever
    // its text says; any other failed read is a missing file.
    let missing = source_from_agent_read_result(&read).is_none()
        && super::tool_result::failure_message(&read, false, true).is_some();
    let held = (!missing).then(|| source_from_read_result(&read));
    let Some(text) = held
        .as_deref()
        .and_then(|source| range_text(source, order.first, order.last))
    else {
        let count = held.as_deref().map_or(0, |source| {
            source
                .strip_suffix('\n')
                .unwrap_or(source)
                .split('\n')
                .count()
        });
        let count = count.to_string();
        return render_seeded_change(
            "line_range_outside_file",
            task,
            &order.source,
            &[(LINES_SLOT, lines.as_str()), (COUNT_SLOT, count.as_str())],
        )
        .map(AgenticPlan::Final);
    };
    let intent = moved_intent(order.first == order.last, order.at_end);
    let placed_change = ComputedChange {
        target: order.target.clone(),
        computation: Computation::EndInsertion {
            text,
            at_end: order.at_end,
            indentation: None,
        },
        intent,
        slots: vec![
            (LINES_SLOT, lines.clone()),
            (SOURCE_SLOT, order.source.clone()),
        ],
    };
    let placed = plan_computed_change_step(task, current_turn, tool_names, &placed_change, result);
    let stated = render_seeded_change(
        intent,
        task,
        &order.target,
        &[
            (LINES_SLOT, lines.as_str()),
            (SOURCE_SLOT, order.source.as_str()),
        ],
    );
    if !matches!((&placed, &stated), (Some(AgenticPlan::Final(answer)), Some(stated)) if answer == stated)
    {
        return placed;
    }
    let removed_change = ComputedChange {
        target: order.source.clone(),
        computation: Computation::Line(LineOperation::RangeRemoval {
            first: order.first,
            last: order.last,
            from_end: false,
        }),
        intent: "numbered_lines_removed",
        slots: vec![(LINES_SLOT, lines.clone())],
    };
    *result = None;
    let removed =
        plan_computed_change_step(task, current_turn, tool_names, &removed_change, result);
    let removed_stated = render_seeded_change(
        "numbered_lines_removed",
        task,
        &order.source,
        &[(LINES_SLOT, lines.as_str())],
    );
    if matches!((&removed, &removed_stated), (Some(AgenticPlan::Final(answer)), Some(stated)) if answer == stated)
    {
        return placed;
    }
    removed
}
