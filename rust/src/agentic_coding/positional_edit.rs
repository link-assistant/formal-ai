//! Additive edits and the words that place them (issues #1115, #1116, #1133).
//!
//! "Add a line X directly after the line Y in F" names no old→new pair, so the
//! replacement composer in `general_planner` answered it with nothing and the
//! request went to web search. The anchor line is the text an edit tool can
//! match exactly, and the replacement is the anchor with the new line beside
//! it -- so an insertion is an edit after all, and no insert primitive is
//! needed. Which side of the position cue the anchor sits on is a fact of the
//! cue's language (`languages.lino`, `adposition`), not of the sentence.

use super::code_artifact::source_from_read_result;
use super::code_task::{render_seeded_change, render_seeded_outcome};
use super::intent_router::edit_arguments;
use super::planner::{AgenticPlan, Capability, plan_one, tool_for};
use super::workspace_change::{
    read_arguments, result_for_command, result_for_edit, result_for_path,
};
use super::write_request::{
    bare_surfaces, clean_content, clean_cue_token, clean_path_token, looks_like_file_path,
    safe_relative_path, tokens,
};
use crate::normal_markov::quoted_segment_spans;
use crate::protocol::ChatMessage;
use crate::seed;
use serde_json::json;

/// The bytes a quoted or clause-led span of edit prose stands for.
pub(super) fn literal_text(span: &str) -> Option<String> {
    quoted_verbatim(span)
        .or_else(|| described_literal(span))
        .or_else(|| clean_content(span))
        .map(|text| unescape_prose_newlines(&text))
}

/// `the heading '# Title'`: one quoted literal led only by the words that say
/// what it is stands for the literal.
fn described_literal(span: &str) -> Option<String> {
    let [segment]: [_; 1] = quoted_segment_spans(span).try_into().ok()?;
    let described = span[segment.end..].trim().is_empty()
        && span[..segment.start]
            .chars()
            .all(|character| character.is_alphabetic() || character.is_whitespace());
    described.then_some(segment.text)
}

/// An additive edit as `(target, anchor, replacement)`.
///
/// "add a line X directly after the line Y in F", "insert X before Y in F"
/// (issue #1115). The anchor line is the text replaced, and the replacement is
/// the anchor with the new line beside it, so the edit tool's exact-match
/// contract holds without a separate insert primitive. Only one insert in one
/// clause, at an anchor named by itself, composes this way.
pub(super) fn compose_positional_insert(request: &str) -> Option<(String, String, String)> {
    let inserts = positional_inserts(request)?;
    let [insert]: [PositionalInsert; 1] = inserts.try_into().ok()?;
    if insert.context.is_some() {
        return None;
    }
    let new = if insert.after {
        [insert.anchor.as_str(), insert.inserted.as_str()].join("\n")
    } else {
        [insert.inserted.as_str(), insert.anchor.as_str()].join("\n")
    };
    Some((insert.target, insert.anchor, new))
}

/// One insert a request asks for.
///
/// The lines `inserted` go beside the line holding `anchor` (after it or
/// before it), in `target`. With a `context`, the anchor is its first
/// occurrence after the context (`the line 'y' that follows 'z'`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PositionalInsert {
    pub(super) target: String,
    pub(super) anchor: String,
    pub(super) inserted: String,
    pub(super) after: bool,
    pub(super) context: Option<String>,
}

/// Every insert the request asks for, in its order.
///
/// `None` unless every clause is one. Clauses are the request cut before a
/// repeated add action that a seeded joiner or a clause mark leads (`…, and
/// insert …`); a clause that names no file takes the one file the request
/// names.
pub(super) fn positional_inserts(request: &str) -> Option<Vec<PositionalInsert>> {
    // `… after the line 'x':` followed by lines: the first line is the request
    // and the lines under it are the text inserted, whatever they quote.
    let block = introduced_block(request);
    let clauses = block
        .as_ref()
        .map_or_else(|| insert_clauses(request), |(head, _)| vec![*head]);
    let block_text = block.as_ref().map(|(_, text)| text.as_str());
    let mut inserts = Vec::new();
    for clause in clauses {
        inserts.push(clause_insert(clause, block_text)?);
    }
    let mut named: Vec<&str> = Vec::new();
    for token in unquoted_path_tokens(request) {
        let candidate = clean_path_token(token.text);
        if looks_like_file_path(candidate)
            && safe_relative_path(candidate)
            && !named.contains(&candidate)
        {
            named.push(candidate);
        }
    }
    inserts
        .into_iter()
        .map(|(target, insert)| match (target, named.as_slice()) {
            (Some(target), _) => Some(PositionalInsert { target, ..insert }),
            (None, [only]) => Some(PositionalInsert {
                target: (*only).to_owned(),
                ..insert
            }),
            (None, _) => None,
        })
        .collect()
}

/// One clause's insert, with its target apart.
///
/// The target is `None` when the clause names no file. The anchor is the
/// literal the position cue governs; a seeded anchor context (`the line 'y'
/// that follows 'z'`) names a literal the anchor comes after; every other
/// literal is a line inserted, in order, and those lines may be separated only
/// by seeded joiners and commas.
fn clause_insert(
    sentence: &str,
    block_text: Option<&str>,
) -> Option<(Option<String>, PositionalInsert)> {
    let lexicon = seed::lexicon();
    let normalized = crate::engine::normalize_prompt(sentence);
    // The position is the request's, not the payload's: a cue inside a quoted
    // literal ("Insert the line «… before …» after …") is text being inserted.
    let outside = crate::engine::normalize_prompt(
        &quoted_segment_spans(sentence)
            .iter()
            .rev()
            .fold(sentence.to_owned(), |text, segment| {
                [&text[..segment.start], " ", &text[segment.end..]].concat()
            }),
    );
    let after = lexicon.mentions_role(seed::ROLE_FILE_EDIT_POSITION_AFTER, &outside);
    let before = lexicon.mentions_role(seed::ROLE_FILE_EDIT_POSITION_BEFORE, &outside);
    if after == before || !lexicon.mentions_role(seed::ROLE_CODING_MEMBER_ADD_ACTION, &normalized) {
        return None;
    }
    let target = unquoted_path_tokens(sentence)
        .iter()
        .map(|token| clean_path_token(token.text))
        .find(|candidate| looks_like_file_path(candidate) && safe_relative_path(candidate))
        .map(str::to_owned);
    let literals: Vec<QuotedLiteral> = quoted_literals(sentence)
        .into_iter()
        .filter(|literal| target.as_deref() != Some(literal.text.as_str()))
        .collect();
    let contexts = cue_occurrences(sentence, ANCHOR_CONTEXT_CUE);
    let role = if after {
        seed::ROLE_FILE_EDIT_POSITION_AFTER
    } else {
        seed::ROLE_FILE_EDIT_POSITION_BEFORE
    };
    // A position word inside the context cue ("के बाद आने वाली") is the
    // context's, not the anchor's.
    let positions: Vec<CueOccurrence> = cue_occurrences(sentence, role)
        .into_iter()
        .filter(|&(start, _, _)| {
            !contexts
                .iter()
                .any(|&(from, to, _)| start >= from && start < to)
        })
        .collect();
    if let Some(text) = block_text {
        let [anchor]: [QuotedLiteral; 1] = literals.try_into().ok()?;
        if !contexts.is_empty() {
            return None;
        }
        let insert = PositionalInsert {
            target: String::new(),
            anchor: unescape_prose_newlines(&anchor.text),
            inserted: text.to_owned(),
            after,
            context: None,
        };
        return Some((target, insert));
    }
    if literals.len() < 2 {
        return None;
    }
    let indices: Vec<usize> = (0..literals.len()).collect();
    let anchor_at = governed_literal(&positions, &literals, &indices)?;
    let context_at = if contexts.is_empty() {
        None
    } else {
        let rest: Vec<usize> = indices
            .into_iter()
            .filter(|index| *index != anchor_at)
            .collect();
        Some(governed_literal(&contexts, &literals, &rest)?)
    };
    let inserted: Vec<&QuotedLiteral> = literals
        .iter()
        .enumerate()
        .filter(|(index, _)| *index != anchor_at && Some(*index) != context_at)
        .map(|(_, literal)| literal)
        .collect();
    if inserted.is_empty() || !joined_only(sentence, &inserted) {
        return None;
    }
    let insert = PositionalInsert {
        target: String::new(),
        anchor: unescape_prose_newlines(&literals[anchor_at].text),
        inserted: inserted
            .iter()
            .map(|literal| unescape_prose_newlines(&literal.text))
            .collect::<Vec<_>>()
            .join("\n"),
        after,
        context: context_at.map(|index| unescape_prose_newlines(&literals[index].text)),
    };
    Some((target, insert))
}

/// The seeded role of the words that name the line an anchor follows.
const ANCHOR_CONTEXT_CUE: &str = "file_edit_anchor_context_cue";
/// The seeded role of the words that join listed lines or clauses.
const JOINER_CUE: &str = "file_edit_joiner_cue";

/// The literal a seeded anchor context names, and the span naming it.
///
/// `that follows 'z'`: `(start, end, text)`, the span from the cue through the
/// literal (the literal through the cue, in a postpositional language).
pub(super) fn anchor_context(sentence: &str) -> Option<(usize, usize, String)> {
    let contexts = cue_occurrences(sentence, ANCHOR_CONTEXT_CUE);
    if contexts.is_empty() {
        return None;
    }
    let literals = quoted_literals(sentence);
    let indices: Vec<usize> = (0..literals.len()).collect();
    let literal = &literals[governed_literal(&contexts, &literals, &indices)?];
    // The cue that governs the literal: the nearest on its language's side.
    let (cue_start, cue_end, postpositional) = contexts
        .iter()
        .copied()
        .filter_map(|cue @ (cue_start, cue_end, postpositional)| {
            let distance = if postpositional {
                cue_start.checked_sub(literal.end)
            } else {
                literal.start.checked_sub(cue_end)
            };
            distance.map(|distance| (distance, cue))
        })
        .min_by_key(|(distance, _)| *distance)?
        .1;
    let (start, end) = if postpositional {
        (literal.start, cue_end)
    } else {
        (cue_start, literal.end)
    };
    Some((start, end, unescape_prose_newlines(&literal.text)))
}

/// The request cut into insert clauses.
///
/// A cut falls before every repeated add action that a seeded joiner or a
/// clause mark leads -- after it, in a postpositional language, whose verb
/// closes its clause.
fn insert_clauses(request: &str) -> Vec<&str> {
    let segments = quoted_segment_spans(request);
    let toks = tokens(request);
    let quoted = |token: &super::write_request::Token<'_>| {
        segments
            .iter()
            .any(|segment| token.start < segment.end && token.end > segment.start)
    };
    let joiners = bare_surfaces(JOINER_CUE);
    let joins = |token: Option<&super::write_request::Token<'_>>| {
        token.is_some_and(|token| !quoted(token) && joiners.contains(&clean_cue_token(token.text)))
    };
    let closes =
        |token: &super::write_request::Token<'_>| token.text.ends_with([',', ';', '\u{0964}']);
    let mut actions: Vec<(String, bool)> = Vec::new();
    for meaning in seed::lexicon().meanings_with_role(seed::ROLE_CODING_MEMBER_ADD_ACTION) {
        for lexeme in &meaning.lexemes {
            let postpositional = crate::language::uses_postpositions(&lexeme.language);
            for word in &lexeme.words {
                actions.push((word.text.to_lowercase(), postpositional));
            }
        }
    }
    // Only a repeated action opens a clause: the first one (the last, where the
    // verb closes its clause) belongs to the clause before it.
    let verbs: Vec<(usize, bool)> = toks
        .iter()
        .enumerate()
        .filter(|(_, token)| !quoted(token))
        .filter_map(|(index, token)| {
            let cleaned = clean_cue_token(token.text);
            actions
                .iter()
                .rev()
                .find(|(surface, _)| *surface == cleaned)
                .map(|&(_, postpositional)| (index, postpositional))
        })
        .collect();
    let mut cuts: Vec<(usize, usize)> = Vec::new();
    for (at, &(index, postpositional)) in verbs.iter().enumerate() {
        let token = &toks[index];
        if !postpositional && at > 0 {
            let previous = &toks[index - 1];
            if joins(Some(previous)) {
                cuts.push((previous.start, previous.end));
            } else if closes(previous) {
                cuts.push((token.start, token.start));
            }
        } else if postpositional && at + 1 < verbs.len() {
            if joins(toks.get(index + 1)) {
                cuts.push((toks[index + 1].start, toks[index + 1].end));
            } else if closes(token) {
                cuts.push((token.end, token.end));
            }
        }
    }
    let mut clauses = Vec::new();
    let mut from = 0;
    for (start, end) in cuts {
        // Two verbs may claim one joiner; the second claim is already made.
        if start < from {
            continue;
        }
        clauses.push(&request[from..start]);
        from = end;
    }
    clauses.push(&request[from..]);
    clauses
        .into_iter()
        .map(str::trim)
        .filter(|clause| !clause.is_empty())
        .collect()
}

/// Whether listed literals are joined only by joiners.
///
/// Consecutive inserted literals may be separated by nothing but whitespace,
/// commas and seeded joiners (`'a' and 'b'`).
fn joined_only(sentence: &str, literals: &[&QuotedLiteral]) -> bool {
    let joiners = bare_surfaces(JOINER_CUE);
    literals.windows(2).all(|pair| {
        sentence[pair[0].end..pair[1].start]
            .split_whitespace()
            .map(clean_cue_token)
            .all(|word| word.is_empty() || joiners.contains(&word))
    })
}

/// A cue's byte span in a sentence and whether its language is postpositional.
type CueOccurrence = (usize, usize, bool);

/// Every surface of `role` in `sentence` outside its quoted literals.
///
/// Each comes with the side its own language's adposition governs, because
/// the side a cue governs is a fact of *its* language and a mixed prompt
/// (`config.yml में … "on:" के बाद`) cannot be trusted to detect as one
/// (`languages.lino`, `adposition`).
fn cue_occurrences(sentence: &str, role: &str) -> Vec<CueOccurrence> {
    let lowered = sentence.to_lowercase();
    // A cue word inside a quoted literal is payload ("Insert the line «… before
    // …» after …"), never the position the request asks for.
    let literals = quoted_literals(sentence);
    let inside_literal = |start: usize| {
        literals
            .iter()
            .any(|literal| start >= literal.start && start < literal.end)
    };
    let mut occurrences = Vec::new();
    for meaning in seed::lexicon().meanings_with_role(role) {
        for lexeme in &meaning.lexemes {
            let postpositional = crate::language::uses_postpositions(&lexeme.language);
            for word in &lexeme.words {
                let surface = word.text.to_lowercase();
                if surface.is_empty() {
                    continue;
                }
                occurrences.extend(
                    lowered
                        .match_indices(surface.as_str())
                        .filter(|(start, _)| !inside_literal(*start))
                        .map(|(start, matched)| (start, start + matched.len(), postpositional)),
                );
            }
        }
    }
    occurrences
}

/// Which of the `candidates` (literal indices) a cue governs.
///
/// The nearest on the side its language's adposition faces -- a preposition
/// the literal after it, a postposition the literal before it -- and a later
/// one on a tie. A cue that occurs only on the far side of every candidate
/// places none ("… renders "x" before it reads the list" is a time, not a
/// position: issue #1069); with no cue at all, the last candidate (the
/// prepositional default).
fn governed_literal(
    occurrences: &[CueOccurrence],
    literals: &[QuotedLiteral],
    candidates: &[usize],
) -> Option<usize> {
    let gap = |literal: &QuotedLiteral| {
        occurrences
            .iter()
            // `checked_sub` is the guard as well as the arithmetic: a cue on
            // the wrong side of the literal yields `None` rather than a
            // subtraction that underflows.
            .filter_map(|&(cue_start, cue_end, postpositional)| {
                if postpositional {
                    cue_start.checked_sub(literal.end)
                } else {
                    literal.start.checked_sub(cue_end)
                }
            })
            .min()
    };
    let mut governed: Option<(usize, usize)> = None;
    for &index in candidates {
        if let Some(distance) = gap(&literals[index])
            && governed.is_none_or(|(_, nearest)| distance <= nearest)
        {
            governed = Some((index, distance));
        }
    }
    governed.map(|(index, _)| index).or_else(|| {
        candidates
            .last()
            .copied()
            .filter(|_| occurrences.is_empty())
    })
}

/// A request whose first line ends in a colon and is followed by lines: that
/// first line, and the lines with their shared indentation removed (one quoted
/// literal stands for itself; a fenced block keeps its own).
pub(super) fn introduced_block(request: &str) -> Option<(&str, String)> {
    let (head, rest) = request.split_once('\n')?;
    let head = head.trim_end();
    if !head.ends_with(':') {
        return None;
    }
    let mut lines: Vec<&str> = rest.split('\n').map(str::trim_end).collect();
    while lines.first().is_some_and(|line| line.is_empty()) {
        lines.remove(0);
    }
    while lines.last().is_some_and(|line| line.is_empty()) {
        lines.pop();
    }
    let indent_of = |line: &str| line.len() - line.trim_start().len();
    // A fenced block (```` ``` ```` lines around it) is kept as written, less
    // the fence's own indentation: its indentation is the text's.
    let fenced = lines.len() >= 2
        && lines[0].trim_start().starts_with(FENCE)
        && lines[lines.len() - 1].trim() == FENCE;
    let (body, shared) = if fenced {
        (&lines[1..lines.len() - 1], indent_of(lines[0]))
    } else {
        let shared = lines
            .iter()
            .copied()
            .filter(|line| !line.is_empty())
            .map(indent_of)
            .min()?;
        (lines.as_slice(), shared)
    };
    let text = body
        .iter()
        .copied()
        .map(|line| &line[shared.min(indent_of(line))..])
        .collect::<Vec<_>>()
        .join("\n");
    let text = if fenced {
        text
    } else {
        quoted_verbatim(&text).unwrap_or(text)
    };
    Some((head, text))
}

/// The line that opens and closes a fenced block.
const FENCE: &str = "```";

/// One quoted span of a request and where it sits.
struct QuotedLiteral {
    start: usize,
    end: usize,
    text: String,
}

/// The delimited literal slots of `request`, verbatim: a quoted line keeps its
/// indentation. Every quote pair [`quoted_segment_spans`] reads counts —
/// `'single'` quotes included — so `Insert 'x' after the line 'y'` places the
/// line exactly as its double-quoted form does.
fn quoted_literals(request: &str) -> Vec<QuotedLiteral> {
    quoted_segment_spans(request)
        .into_iter()
        .map(|segment| QuotedLiteral {
            start: segment.start,
            end: segment.end,
            text: segment.text,
        })
        .collect()
}

/// A span that is one quoted literal, kept byte for byte: the author who
/// quoted `"  schedule:"` quoted its indentation on purpose (issue #1116).
fn quoted_verbatim(span: &str) -> Option<String> {
    let trimmed = span.trim().trim_start_matches([':', '-', '—', '–']).trim();
    // One quoted literal of any delimiter (single quotes included) is verbatim.
    if let [segment] = quoted_segment_spans(trimmed).as_slice()
        && segment.start == 0
        && segment.end == trimmed.len()
    {
        return Some(segment.text.clone());
    }
    let mut chars = trimmed.chars();
    let (first, last) = (chars.next()?, chars.next_back()?);
    (first == last && matches!(first, '"' | '`') && trimmed.len() >= 2)
        .then(|| trimmed[1..trimmed.len() - 1].to_owned())
        .filter(|inner| !inner.contains(first))
}

/// The request's tokens outside every quoted literal; a literal that is
/// exactly one path still names the file (mirrors `unquotedPathTokens`).
pub(super) fn unquoted_path_tokens(request: &str) -> Vec<super::write_request::Token<'_>> {
    let segments = quoted_segment_spans(request);
    tokens(request)
        .into_iter()
        .filter(|token| {
            !segments.iter().any(|segment| {
                token.start >= segment.start
                    && token.end <= segment.end
                    && clean_path_token(token.text) != segment.text
            })
        })
        .collect()
}

/// The words a request says itself.
///
/// The head of an edit request whose lines follow it (the lines are its
/// payload); otherwise the whole request.
pub(super) fn own_text(request: &str) -> &str {
    match introduced_block(request) {
        Some((head, _)) if names_local_edit(head) => head,
        _ => request,
    }
}

/// Whether `task` is an instruction to change a named local file: an edit or
/// insert action beside a workspace path. Such a request is never a web
/// question, whatever else the router makes of it (issues #1115, #1133).
#[must_use]
pub(super) fn names_local_edit(task: &str) -> bool {
    let lexicon = seed::lexicon();
    let normalized = crate::engine::normalize_prompt(task);
    let edits = lexicon.mentions_role(seed::ROLE_FILE_EDIT_ACTION_CUE, &normalized)
        || lexicon.mentions_role(seed::ROLE_CODING_MEMBER_ADD_ACTION, &normalized);
    edits
        && tokens(task).iter().any(|token| {
            let candidate = clean_path_token(token.text);
            looks_like_file_path(candidate) && safe_relative_path(candidate)
        })
}

/// A newline an author spelled as `\n` inside prose means a newline in the
/// file (issue #1116); the edit tool would otherwise write the two characters.
pub(super) fn unescape_prose_newlines(text: &str) -> String {
    text.replace("\\n", "\n").replace("\\t", "\t")
}

/// One insert as `(start, old, new)` over `source`.
///
/// `old` is the whole lines from the anchor context (or the anchor) through
/// the anchor, `new` those lines with the inserted text beside the anchor's
/// line; `None` unless the context (with none, the anchor) occurs once and the
/// anchor occurs after it.
fn insert_edit(source: &str, insert: &PositionalInsert) -> Option<(usize, String, String)> {
    let anchor = insert.anchor.as_str();
    let lead = insert.context.as_deref().unwrap_or(anchor);
    let mut leads = source.match_indices(lead).map(|(at, _)| at);
    let (Some(lead_at), None) = (leads.next(), leads.next()) else {
        return None;
    };
    if anchor.is_empty() {
        return None;
    }
    // After a context, a line that is exactly the anchor outranks one that only
    // contains it (`text set` is not `text setting`).
    let at = if insert.context.is_some() {
        let from = lead_at + lead.len();
        let whole = |index: usize| {
            (index == 0 || source.as_bytes()[index - 1] == b'\n')
                && (anchor.ends_with('\n')
                    || source
                        .as_bytes()
                        .get(index + anchor.len())
                        .is_none_or(|byte| *byte == b'\n'))
        };
        let found: Vec<usize> = source[from..]
            .match_indices(anchor)
            .map(|(index, _)| from + index)
            .collect();
        found
            .iter()
            .copied()
            .find(|index| whole(*index))
            .or_else(|| found.first().copied())?
    } else {
        lead_at
    };
    let line_start = |index: usize| source[..index].rfind('\n').map_or(0, |newline| newline + 1);
    let start = line_start(lead_at);
    let tail = at + anchor.len();
    let end = if anchor.ends_with('\n') {
        tail - 1
    } else {
        source[tail..]
            .find('\n')
            .map_or(source.len(), |newline| tail + newline)
    };
    let old = &source[start..end];
    let anchor_line = line_start(at);
    let new = if insert.after {
        [old, "\n", insert.inserted.as_str()].concat()
    } else {
        [
            &source[start..anchor_line],
            insert.inserted.as_str(),
            "\n",
            &source[anchor_line..end],
        ]
        .concat()
    };
    Some((start, old.to_owned(), new))
}

/// Make a request's inserts one anchored edit at a time.
///
/// Each file is read once; each insert is its own anchored edit over the file
/// as the earlier inserts left it; then every file's digest is checked and
/// every insert stated.
pub(super) fn plan_insert_sequence_step(
    task: &str,
    current_turn: &[ChatMessage],
    tool_names: &[&str],
    inserts: &[PositionalInsert],
) -> Option<AgenticPlan> {
    let edit_tool = tool_for(tool_names, Capability::Edit)?;
    let mut expected: Vec<(&str, String)> = Vec::new();
    let mut stated = Vec::new();
    for insert in inserts {
        let target = insert.target.as_str();
        if !expected.iter().any(|(path, _)| *path == target) {
            let Some(read) = result_for_path(current_turn, Capability::Read, target, None) else {
                let tool = tool_for(tool_names, Capability::Read)?;
                return Some(plan_one(tool, read_arguments(target)));
            };
            expected.push((target, source_from_read_result(&read)));
        }
        let index = expected.iter().position(|(path, _)| *path == target)?;
        let source = expected[index].1.as_str();
        let Some((start, lines, widened)) = insert_edit(source, insert) else {
            return Some(AgenticPlan::Final(render_seeded_outcome(
                "coding_workspace_verification_failed",
                task,
                target,
            )?));
        };
        if result_for_edit(current_turn, target, &lines, &widened).is_none() {
            return Some(plan_one(
                edit_tool,
                edit_arguments(target, &lines, &widened),
            ));
        }
        let updated = [
            &source[..start],
            widened.as_str(),
            &source[start + lines.len()..],
        ]
        .concat();
        expected[index].1 = updated;
        let intent = if insert.after {
            "file_edit_position_after"
        } else {
            "file_edit_position_before"
        };
        stated.push(render_seeded_change(
            intent,
            task,
            target,
            &[
                ("{new}", insert.inserted.as_str()),
                ("{anchor}", insert.anchor.as_str()),
            ],
        ));
    }
    for (target, content) in &expected {
        let command = ["sha256sum -- ", *target].concat();
        let Some(observed) = result_for_command(current_turn, &command) else {
            let tool = tool_for(tool_names, Capability::Run)?;
            return Some(plan_one(tool, json!({"command": command}).to_string()));
        };
        let digest = crate::source_fetch::sha256_hex(content.as_bytes());
        if observed.split_whitespace().next() != Some(digest.as_str()) {
            return Some(AgenticPlan::Final(render_seeded_outcome(
                "coding_workspace_verification_failed",
                task,
                target,
            )?));
        }
    }
    let stated: Vec<String> = stated.into_iter().collect::<Option<_>>()?;
    Some(AgenticPlan::Final(stated.join("\n")))
}
