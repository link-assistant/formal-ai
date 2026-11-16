//! The obligations of a work item, apart from what it only cites (issue #1562).
//!
//! A bug report about another tool cites the evidence it was read from: the
//! address of a test repository such as `test-hello-world-…`, a source file in
//! a code span, a table of evidence, a quoted log. None of those spans asks for
//! anything. The solver normalizes every non-alphanumeric byte to a space, so
//! an address `…/test-hello-world-01` reads as "test hello world" and selected
//! the stored Hello World program for a report that asked for no program.
//!
//! [`obligation_text`] keeps the work item's own words and blanks every
//! citation span with spaces of the same byte length, so a byte span of the
//! obligation text is the byte span of the same words in the work item. The
//! citations are: fenced code blocks, block quotations (`>`), table rows
//! (`|`), inline code spans, Markdown links and images (`[text](target)`),
//! autolinks (`<scheme://…>`) and bare addresses (a token holding `://` or
//! starting with `www.`). A quoted phrase inside a sentence ("a \"Hello
//! World\" program") is the work item's own naming of what it asks for and is
//! kept.
//!
//! Mirrored by `js/agentic/work_item_citations.mjs`.

/// Fence openers of a fenced code block.
const FENCES: [&str; 2] = ["```", "~~~"];

/// The work item with every citation span replaced by spaces of the same byte
/// length; line breaks are kept, so byte offsets are preserved.
#[must_use]
pub fn obligation_text(text: &str) -> String {
    let mut kept = String::with_capacity(text.len());
    let mut fenced = false;
    for line in text.split_inclusive('\n') {
        let (body, ending) = line
            .strip_suffix("\r\n")
            .map(|body| (body, "\r\n"))
            .or_else(|| line.strip_suffix('\n').map(|body| (body, "\n")))
            .unwrap_or((line, ""));
        let trimmed = body.trim_start();
        let fence = FENCES.iter().any(|fence| trimmed.starts_with(*fence));
        if fence || fenced || trimmed.starts_with('>') || trimmed.starts_with('|') {
            if fence {
                fenced = !fenced;
            }
            blank(&mut kept, body);
        } else {
            kept.push_str(&without_inline_citations(body));
        }
        kept.push_str(ending);
    }
    kept
}

/// Whether the work item cites anything at all: its obligation text differs
/// from the text itself.
#[must_use]
pub fn cites_anything(text: &str) -> bool {
    obligation_text(text) != text
}

/// Push one space per byte of `span`.
fn blank(kept: &mut String, span: &str) {
    kept.extend(std::iter::repeat_n(' ', span.len()));
}

/// One line without its code spans, links and addresses.
fn without_inline_citations(line: &str) -> String {
    let bytes = line.as_bytes();
    let mut kept = String::with_capacity(line.len());
    let mut index = 0;
    while index < bytes.len() {
        let end = citation_end(line, index);
        if end > index {
            blank(&mut kept, &line[index..end]);
            index = end;
            continue;
        }
        let character = line[index..].chars().next().unwrap_or(' ');
        kept.push(character);
        index += character.len_utf8();
    }
    kept
}

/// The end of the citation that starts at byte `start`, or `start` when none
/// does.
fn citation_end(line: &str, start: usize) -> usize {
    let rest = &line[start..];
    if rest.starts_with('`') {
        let run = rest.len() - rest.trim_start_matches('`').len();
        let fence = &rest[..run];
        return rest[run..]
            .find(fence)
            .map_or(start, |close| start + run + close + run);
    }
    if (rest.starts_with('[') || rest.starts_with("!["))
        && let Some(end) = link_end(rest)
    {
        return start + end;
    }
    if rest.starts_with('<')
        && let Some(close) = rest.find('>')
        && rest[1..close].contains("://")
        && !rest[1..close].contains(char::is_whitespace)
    {
        return start + close + 1;
    }
    let token_start = line[..start]
        .chars()
        .next_back()
        .is_none_or(|previous| previous.is_whitespace() || "([{<\"'".contains(previous));
    if token_start {
        let token_len = rest.find(char::is_whitespace).unwrap_or(rest.len());
        let token = &rest[..token_len];
        if token.contains("://") || token.to_ascii_lowercase().starts_with("www.") {
            return start + token_len;
        }
    }
    start
}

/// The byte length of a Markdown link or image `[text](target)` at the start
/// of `rest`, when one starts there.
fn link_end(rest: &str) -> Option<usize> {
    let open = rest.find('[')?;
    let mut depth = 0usize;
    let mut close = None;
    for (offset, character) in rest[open..].char_indices() {
        match character {
            '[' => depth += 1,
            ']' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    close = Some(open + offset);
                    break;
                }
            }
            _ => {}
        }
    }
    let close = close?;
    let target = rest[close + 1..].strip_prefix('(')?;
    let end = target.find(')')?;
    Some(close + 2 + end + 1)
}

/// Whether the byte span `(start, end)` of the work item holds only citations,
/// given its obligation text. Mirrored by `isCitation`.
#[must_use]
pub fn is_citation(obligations: &str, (start, end): (usize, usize)) -> bool {
    start < end
        && obligations.as_bytes().get(start..end).is_some_and(|span| {
            span.iter()
                .all(|byte| matches!(byte, b' ' | b'\n' | b'\r' | b'\t'))
        })
}
