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
//! A code span can also be the operand of an obligation: "The program should
//! print exactly: `Hello, World!`". The span asks for nothing by itself, but
//! the clause that binds it does. The binding of a recipe reads the
//! obligation text with those operand spans kept — the output literals an
//! obligation clause binds
//! ([`crate::intent_formalization::bound_output_literals`]) — so it reads the
//! clause whole.
//!
//! Mirrored by `js/agentic/work_item_citations.mjs`.

/// Fence openers of a fenced code block.
const FENCES: [&str; 2] = ["```", "~~~"];

/// The scheme separator that marks an address (`https://…`).
const SCHEME_SEPARATOR: &str = "://";

/// The host prefix that marks an address written without a scheme.
const WEB_HOST: &str = "www.";

/// The opener of a Markdown image (`![text](target)`).
const IMAGE_OPENER: &str = "![";

/// The work item with every citation span replaced by spaces of the same byte
/// length; line breaks are kept, so byte offsets are preserved.
#[must_use]
pub fn obligation_text(text: &str) -> String {
    obligation_text_keeping(text, &[])
}

/// The obligation text with every code span whose content is one of
/// `operands` kept. Mirrored by `obligationTextKeeping`.
fn obligation_text_keeping(text: &str, operands: &[String]) -> String {
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
            kept.push_str(&without_inline_citations(body, operands));
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

/// One line without its code spans, links and addresses; a code span whose
/// content is one of `operands` is kept whole.
fn without_inline_citations(line: &str, operands: &[String]) -> String {
    let bytes = line.as_bytes();
    let mut kept = String::with_capacity(line.len());
    let mut index = 0;
    while index < bytes.len() {
        if let Some((end, content)) = code_span_end(line, index)
            && operands.iter().any(|operand| operand == content)
        {
            kept.push_str(&line[index..end]);
            index = end;
            continue;
        }
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

/// The end of the code span that starts at byte `start`, and its content,
/// when one does.
fn code_span_end(line: &str, start: usize) -> Option<(usize, &str)> {
    let rest = &line[start..];
    if !rest.starts_with('`') {
        return None;
    }
    let run = rest.len() - rest.trim_start_matches('`').len();
    let fence = &rest[..run];
    let close = rest[run..].find(fence)?;
    Some((start + run + close + run, &rest[run..run + close]))
}

/// The end of the citation that starts at byte `start`, or `start` when none
/// does.
fn citation_end(line: &str, start: usize) -> usize {
    let rest = &line[start..];
    if rest.starts_with('`') {
        return code_span_end(line, start).map_or(start, |(end, _)| end);
    }
    if (rest.starts_with('[') || rest.starts_with(IMAGE_OPENER))
        && let Some(end) = link_end(rest)
    {
        return start + end;
    }
    if rest.starts_with('<')
        && let Some(close) = rest.find('>')
        && rest[1..close].contains(SCHEME_SEPARATOR)
        && !rest[1..close].contains(char::is_whitespace)
    {
        return start + close + 1;
    }
    let token_start = line[..start]
        .chars()
        .next_back()
        .is_none_or(|previous| previous.is_whitespace() || "([{<\"'".contains(previous));
    if token_start {
        let token_length = rest.find(char::is_whitespace).unwrap_or(rest.len());
        let token = &rest[..token_length];
        if token.contains(SCHEME_SEPARATOR) || token.to_ascii_lowercase().starts_with(WEB_HOST) {
            return start + token_length;
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

/// Drop the solver's recipe when no obligation of the work item names it, and
/// return the path it would have written. Mirrored by `unbindCitedRecipe`.
///
/// The recipe is bound when the work item's own words — its [`binding_text`]
/// — select a recipe writing the same file in the same language. A work item
/// that cites nothing is its own obligation text and is not solved twice. The decision is traced as route
/// `work_item_recipe_binding` under `FORMAL_AI_TRACE_REQUESTS`.
pub(super) fn unbind_cited_recipe(
    answer: &mut crate::engine::SymbolicAnswer,
    objective: &str,
) -> Option<String> {
    let recipe = answer.execution_recipe.as_ref()?;
    let obligations = binding_text(objective);
    if obligations == objective {
        return None;
    }
    let bound = crate::solver::UniversalSolver::default()
        .solve(&obligations)
        .execution_recipe
        .is_some_and(|obliged| obliged.path == recipe.path && obliged.language == recipe.language);
    super::planner::trace_route_with("work_item_recipe_binding", || {
        Some(serde_json::json!({
            "path": recipe.path,
            "language": recipe.language,
            "bound": bound,
        }))
    });
    if bound {
        return None;
    }
    answer.execution_recipe.take().map(|recipe| recipe.path)
}

/// The [`obligation_text`] with the code spans that are operands of an
/// obligation clause kept: the output literals its clauses bind
/// ([`crate::intent_formalization::bound_output_literals`]). A code span is
/// no obligation by itself; "print exactly: `Hello, World!`" is one, and the
/// binding reads it whole. Mirrored by `bindingText`.
#[must_use]
pub fn binding_text(text: &str) -> String {
    obligation_text_keeping(
        text,
        &crate::intent_formalization::bound_output_literals(text),
    )
}
