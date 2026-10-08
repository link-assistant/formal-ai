//! Another file's contents as the text an additive edit places (PR #1188
//! G50, G51): `Append the contents of a.txt to the end of b.txt`, `Insert the
//! contents of rows.txt after the line 'x' in f.lino`. The seeded
//! `file_contents_source_cue` beside a path names the source file; once it is
//! read, the request is restated with the source's lines under it as a fenced
//! block, which the additive-edit composers (end insertion, positional insert)
//! already place verbatim.
//!
//! Mirrors `js/agentic/contents_source.mjs`.

use super::write_request::{
    Token, clean_path_token, looks_like_file_path, safe_relative_path, tokens,
};
use crate::normal_markov::quoted_segment_spans;
use crate::seed;

const ROLE: &str = "file_contents_source_cue";
const FENCE: &str = "```";

/// The source path a contents cue names and the byte span of cue and path in
/// the request's first line.
pub(super) struct ContentsSource {
    pub(super) path: String,
    start: usize,
    end: usize,
}

/// The path right after a seeded contents cue (or right before it, in a
/// postpositional language), outside the request's quoted literals.
pub(super) fn contents_source(task: &str) -> Option<ContentsSource> {
    let line = task.split('\n').next().unwrap_or(task);
    // Offsets in the lowered line are the line's own while lowering keeps its
    // length.
    let lowered = Some(line.to_lowercase())
        .filter(|lowered| lowered.len() == line.len())
        .unwrap_or_else(|| line.to_owned());
    let quoted = quoted_segment_spans(line);
    let words = tokens(line);
    let is_path = |token: &Token<'_>| {
        let path = clean_path_token(token.text);
        looks_like_file_path(path) && safe_relative_path(path)
    };
    let space_between = |from: usize, to: usize| line[from..to].trim().is_empty();
    for cue in seed::lexicon().words_for_role(ROLE) {
        let surface = cue.to_lowercase();
        if surface.is_empty() {
            continue;
        }
        for (at, _) in lowered.match_indices(surface.as_str()) {
            let end = at + surface.len();
            if quoted
                .iter()
                .any(|segment| at >= segment.start && at < segment.end)
            {
                continue;
            }
            let following = words.iter().find(|token| token.start >= end);
            let preceding = words.iter().rev().find(|token| token.end <= at);
            if let Some(token) =
                following.filter(|token| is_path(token) && space_between(end, token.start))
            {
                return Some(ContentsSource {
                    path: clean_path_token(token.text).to_owned(),
                    start: at,
                    end: token.start + token.text.len(),
                });
            }
            if let Some(token) =
                preceding.filter(|token| is_path(token) && space_between(token.end, at))
            {
                return Some(ContentsSource {
                    path: clean_path_token(token.text).to_owned(),
                    start: token.start,
                    end,
                });
            }
        }
    }
    None
}

/// The request's first line without the cue and the source path, ending in
/// a colon, and the source's lines under it as a fenced block (one final
/// line break is the file's, not a line).
pub(super) fn with_contents(task: &str, source: &ContentsSource, contents: &str) -> String {
    let line = task.split('\n').next().unwrap_or(task);
    let joined = [&line[..source.start], &line[source.end..]].concat();
    let trimmed = joined.trim_end();
    let head = trimmed
        .strip_suffix(['.', '!', '\u{3002}'])
        .unwrap_or(trimmed);
    let body = contents.strip_suffix('\n').unwrap_or(contents);
    format!("{head}:\n{FENCE}\n{body}\n{FENCE}")
}
