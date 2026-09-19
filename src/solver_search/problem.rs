//! Seed-grounded recognition of bounded arithmetic reachability problems.

use crate::seed;

use super::{Op, SearchProblem};

/// Recognize an arithmetic-reachability search problem across every supported
/// language. Returns `None` for any prompt that is not clearly of this shape so
/// the stage stays inert for the overwhelming majority of impulses.
///
/// Recognition is grounded entirely in the seed lexicon (issue #386): the
/// "combine numbers" framing, the search verb, and the target marker are read by
/// semantic role from `data/seed/meanings-search.lino`, and the operator
/// vocabulary comes from `data/seed/meanings-calculator.lino` — no per-language
/// phrase table lives here. Only the digits and the notation symbols they anchor
/// are language-neutral and matched directly.
pub(super) fn parse_search_problem(prompt: &str) -> Option<SearchProblem> {
    let lower = prompt.to_lowercase();
    let lexicon = seed::lexicon();
    if !lexicon.mentions_role_raw(seed::ROLE_REACHABILITY_OPERAND_FRAMING, &lower)
        || !lexicon.mentions_role_raw(seed::ROLE_REACHABILITY_SEARCH_CUE, &lower)
    {
        return None;
    }

    let integers = extract_integers_with_positions(&lower);
    if integers.len() < 3 {
        return None;
    }
    let marker_positions = target_marker_positions(&lower);
    if marker_positions.is_empty() {
        return None;
    }
    let target_index = integers
        .iter()
        .enumerate()
        .min_by_key(|(_, (_, position))| distance_to_nearest(*position, &marker_positions))
        .map(|(index, _)| index)?;

    let target = integers[target_index].0;
    let numbers: Vec<i64> = integers
        .iter()
        .enumerate()
        .filter(|(index, _)| *index != target_index)
        .map(|(_, (value, _))| *value)
        .collect();
    if numbers.len() < 2 || numbers.len() > MAX_OPERANDS {
        return None;
    }

    Some(SearchProblem {
        numbers,
        target,
        ops: parse_ops(&lower),
    })
}

/// Upper bound on operand count so the search space and per-call cost stay
/// bounded regardless of the prompt.
const MAX_OPERANDS: usize = 6;

fn target_marker_positions(lower: &str) -> Vec<usize> {
    let mut positions = Vec::new();
    for marker in seed::lexicon().words_for_role(seed::ROLE_REACHABILITY_TARGET_MARKER) {
        let mut from = 0;
        while let Some(offset) = lower[from..].find(&marker) {
            let absolute = from + offset;
            positions.push(absolute);
            from = absolute + marker.len();
        }
    }
    positions
}

fn distance_to_nearest(position: usize, marker_positions: &[usize]) -> usize {
    marker_positions
        .iter()
        .map(|&marker| position.abs_diff(marker))
        .min()
        .unwrap_or(usize::MAX)
}

fn parse_ops(lower: &str) -> Vec<Op> {
    let operators = seed::lexicon().arithmetic_operators();
    let mut ops: Vec<Op> = operators
        .iter()
        .filter(|operator| {
            symbol_present(lower, operator.symbol)
                || operator.spelled.iter().any(|word| lower.contains(word))
        })
        .map(|operator| Op::new(operator.symbol))
        .collect();
    if ops.is_empty() {
        ops = operators
            .iter()
            .map(|operator| Op::new(operator.symbol))
            .collect();
    }
    ops
}

fn symbol_present(lower: &str, symbol: char) -> bool {
    let chars: Vec<char> = lower.chars().collect();
    let arithmetic_context =
        |neighbor: Option<char>| neighbor.is_none_or(|c| c.is_ascii_digit() || c.is_whitespace());
    for (index, &current) in chars.iter().enumerate() {
        if current != symbol {
            continue;
        }
        let before = index.checked_sub(1).map(|prev| chars[prev]);
        let after = chars.get(index + 1).copied();
        if arithmetic_context(before) || arithmetic_context(after) {
            return true;
        }
    }
    false
}

fn extract_integers_with_positions(span: &str) -> Vec<(i64, usize)> {
    let mut numbers = Vec::new();
    let mut current = String::new();
    let mut start = 0;
    for (offset, ch) in span.char_indices() {
        if ch.is_ascii_digit() {
            if current.is_empty() {
                start = offset;
            }
            current.push(ch);
        } else if !current.is_empty() {
            if let Ok(value) = current.parse::<i64>() {
                numbers.push((value, start));
            }
            current.clear();
        }
    }
    if !current.is_empty()
        && let Ok(value) = current.parse::<i64>()
    {
        numbers.push((value, start));
    }
    numbers
}
