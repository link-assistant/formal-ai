//! Line removal for the computed workspace change: which lines a request that
//! names a line removes, and the literal removal for one that does not.
//!
//! Mirrors `removedLineIndices`, `removedLines`, `removedLiteral` and
//! `removedDeclarations` in `js/agentic/workspace_change.mjs` (PR #1188 G60).

use crate::seed;

/// A line without its `\n` or `\r\n` ending.
pub(super) fn bare_line(line: &str) -> &str {
    line.strip_suffix('\n')
        .map_or(line, |body| body.strip_suffix('\r').unwrap_or(body))
}

/// The lines a line removal takes: with the seeded `line_containment_cue`
/// every line containing `text`; otherwise the lines that are `text` but
/// for their indentation, else the one line containing it (PR #1188 G60).
pub(super) fn removed_line_indices(source: &str, text: &str, containing: bool) -> Vec<usize> {
    let lines: Vec<&str> = source.split_inclusive('\n').map(bare_line).collect();
    let holding: Vec<usize> = (0..lines.len())
        .filter(|&i| lines[i].contains(text))
        .collect();
    if containing {
        return holding;
    }
    let equal: Vec<usize> = holding
        .iter()
        .copied()
        .filter(|&i| lines[i].trim() == text.trim())
        .collect();
    if !equal.is_empty() {
        return equal;
    }
    if holding.len() == 1 {
        holding
    } else {
        Vec::new()
    }
}

/// `source` without the lines [`removed_line_indices`] takes; `None` when none.
pub(super) fn removed_lines(source: &str, text: &str, containing: bool) -> Option<String> {
    let taken = removed_line_indices(source, text, containing);
    let kept: String = source
        .split_inclusive('\n')
        .enumerate()
        .filter(|(index, _)| !taken.contains(index))
        .map(|(_, line)| line)
        .collect();
    (!taken.is_empty()).then_some(kept)
}

pub(super) fn removed_literal(source: &str, text: &str) -> Option<String> {
    let kept: String = source
        .split_inclusive('\n')
        .filter(|line| {
            let bare = line
                .strip_suffix('\n')
                .map_or(*line, |body| body.strip_suffix('\r').unwrap_or(body));
            bare != text
        })
        .collect();
    if kept != source {
        return Some(kept);
    }
    (source.matches(text).count() == 1).then(|| source.replacen(text, "", 1))
}

/// `source` without the function each of `names` declares, and the names that
/// were declared; `None` when none is. A name declared zero or several times
/// is left alone; a block left between two blank lines takes one of them with
/// it, and a block that ended the file the blank line before it (mirrors
/// `removedDeclarations`).
pub(super) fn removed_declarations(
    source: &str,
    names: &[String],
) -> Option<(String, Vec<String>)> {
    let lines: Vec<&str> = source.split_inclusive('\n').collect();
    let keywords = seed::lexicon().words_for_role("function_declaration_keyword");
    let mut removed = std::collections::BTreeSet::new();
    let mut declared = Vec::new();
    for name in names {
        let headers: Vec<usize> = lines
            .iter()
            .enumerate()
            .filter(|(_, line)| declares_function(line, name, &keywords))
            .map(|(index, _)| index)
            .collect();
        let [header] = headers.as_slice() else {
            continue;
        };
        let Some((start, end)) = declaration_span(&lines, *header) else {
            continue;
        };
        removed.extend(start..=end);
        declared.push(name.clone());
    }
    if declared.is_empty() {
        return None;
    }
    let blank = |index: usize| lines.get(index).is_some_and(|line| line.trim().is_empty());
    for index in removed.clone() {
        let next = index + 1;
        if removed.contains(&next) {
            continue;
        }
        let mut start = index;
        while start > 0 && removed.contains(&(start - 1)) {
            start -= 1;
        }
        if next == lines.len() {
            // A block that ended the file takes the blank line before it.
            if start > 0 && blank(start - 1) {
                removed.insert(start - 1);
            }
        } else if blank(next) && (start == 0 || blank(start - 1)) {
            removed.insert(next);
        }
    }
    let kept: String = lines
        .iter()
        .enumerate()
        .filter(|(index, _)| !removed.contains(index))
        .map(|(_, line)| *line)
        .collect();
    Some((kept, declared))
}

/// A seeded `function_declaration_keyword`, then `name`, then `(` or `<`
/// (mirrors `declaresFunction`). The line is read as identifier runs and
/// single other characters, so `fn name (` and `function name<T>(` both count.
fn declares_function(line: &str, name: &str, keywords: &[String]) -> bool {
    let is_identifier =
        |character: char| character.is_ascii_alphanumeric() || "_$".contains(character);
    let mut words: Vec<&str> = Vec::new();
    let mut run: Option<usize> = None;
    for (index, character) in line.char_indices() {
        if is_identifier(character) {
            if run.is_none() {
                run = Some(index);
            }
            continue;
        }
        if let Some(from) = run.take() {
            words.push(&line[from..index]);
        }
        words.push(&line[index..index + character.len_utf8()]);
    }
    if let Some(from) = run {
        words.push(&line[from..]);
    }
    let skip_blank = |mut at: usize| {
        while words.get(at).is_some_and(|word| word.trim().is_empty()) {
            at += 1;
        }
        at
    };
    words.iter().enumerate().any(|(index, word)| {
        if !keywords.iter().any(|keyword| keyword == word) {
            return false;
        }
        let at = skip_blank(index + 1);
        if words.get(at) != Some(&name) {
            return false;
        }
        matches!(words.get(skip_blank(at + 1)), Some(&("(" | "<")))
    })
}

/// The first and last line of the declaration whose header is line `header`,
/// with the comment and attribute lines directly above it (mirrors
/// `declarationSpan`). A braced body ends at the first later line that closes
/// at the header's indentation; an indented body (a header ending in `:`) at
/// the last line indented deeper.
fn declaration_span(lines: &[&str], header: usize) -> Option<(usize, usize)> {
    let indent = |line: &str| line.len() - line.trim_start_matches([' ', '\t']).len();
    let header_line = bare_line(lines[header]);
    let depth = indent(header_line);
    let closing = header_line.trim_end();
    let end = if closing.ends_with(['}', ';']) {
        header
    } else if closing.ends_with(':') {
        let mut end = header;
        for (index, line) in lines.iter().copied().enumerate().skip(header + 1) {
            if line.trim().is_empty() {
                continue;
            }
            if indent(line) <= depth {
                break;
            }
            end = index;
        }
        end
    } else {
        (header + 1..lines.len())
            .find(|index| indent(lines[*index]) == depth && lines[*index].trim().starts_with('}'))?
    };
    let opens_comment = |line: &str| line.trim().starts_with("/*");
    let mut start = header;
    while start > 0 {
        let above = lines[start - 1].trim();
        if above.ends_with("*/") {
            let mut open = start - 1;
            while open > 0 && !opens_comment(lines[open]) {
                open -= 1;
            }
            if !opens_comment(lines[open]) {
                break;
            }
            start = open;
        } else if ["//", "#[", "@"].iter().any(|lead| above.starts_with(lead)) {
            start -= 1;
        } else {
            break;
        }
    }
    Some((start, end))
}
