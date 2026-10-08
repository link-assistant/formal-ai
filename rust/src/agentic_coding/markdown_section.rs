//! A Markdown section as the scope of an additive edit (PR #1188 G35):
//! `Insert the line 'x' at the end of the section '## Usage' in README.md`.
//! The section runs from its heading to the line before the next heading of
//! the same or a higher level (fenced code is not a heading).
//!
//! Mirrors `js/agentic/markdown_section.mjs`.

use super::workspace_line_operation::named_target_and_payloads;
use crate::seed;

const FENCE: &str = "```";

/// The level of a Markdown ATX heading line, or `None`.
pub(super) fn heading_level(line: &str) -> Option<usize> {
    let trimmed = line.trim();
    let level = trimmed
        .chars()
        .take_while(|&character| character == '#')
        .count();
    if level == 0 || level > 6 {
        return None;
    }
    trimmed[level..]
        .chars()
        .next()
        .is_none_or(char::is_whitespace)
        .then_some(level)
}

/// The section an additive edit is scoped to: its file, heading and text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SectionScope {
    pub(super) target: String,
    pub(super) heading: String,
    pub(super) text: String,
}

/// The seeded `file_section_noun` outside the request's quotes (`outside`,
/// lowered), two quoted payloads of which one is a Markdown heading (the
/// later, when both are) and the other the text placed.
pub(super) fn section_scope(task: &str, outside: &str) -> Option<SectionScope> {
    if !seed::lexicon().mentions_role("file_section_noun", outside) {
        return None;
    }
    let (target, payloads) = named_target_and_payloads(task)?;
    if payloads.len() != 2 {
        return None;
    }
    let heading = payloads
        .iter()
        .rev()
        .find(|payload| heading_level(payload).is_some())?
        .clone();
    let text = payloads
        .iter()
        .find(|payload| **payload != heading)?
        .clone();
    Some(SectionScope {
        target,
        heading,
        text,
    })
}

/// `source` with `text` placed after the section's last non-blank line
/// (`at_end`) or after its heading and the blank line under it, and the line
/// it now follows; `None` when the heading is not one line of the file.
pub(super) fn inserted_in_section(
    source: &str,
    heading: &str,
    text: &str,
    at_end: bool,
) -> Option<(String, String)> {
    let lines: Vec<&str> = source.split('\n').collect();
    let found: Vec<usize> = (0..lines.len())
        .filter(|&index| lines[index].trim() == heading.trim())
        .collect();
    let [start] = found.as_slice() else {
        return None;
    };
    let start = *start;
    let level = heading_level(lines[start])?;
    let mut end = lines.len();
    let mut fenced = false;
    for (index, line) in lines.iter().enumerate().skip(start + 1) {
        if line.trim_start().starts_with(FENCE) {
            fenced = !fenced;
        }
        let nested = if fenced { None } else { heading_level(line) };
        if nested.is_some_and(|nested| nested <= level) {
            end = index;
            break;
        }
    }
    let after = if at_end {
        (start + 1..end)
            .rev()
            .find(|&index| !lines[index].trim().is_empty())
            .unwrap_or(start)
    } else if start + 1 < end && lines[start + 1].trim().is_empty() {
        start + 1
    } else {
        start
    };
    let mut updated: Vec<&str> = lines[..=after].to_vec();
    updated.extend(text.split('\n'));
    updated.extend_from_slice(&lines[after + 1..]);
    let anchor = if at_end { lines[after] } else { lines[start] };
    Some((updated.join("\n"), anchor.to_owned()))
}
