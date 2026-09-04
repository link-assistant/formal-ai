//! Recovering the parts of a write request from its prose (issue #654).
//!
//! A request that asks for a file to be produced carries the same four parts
//! however it is worded: whitespace tokens, the seed-defined cues that mark a
//! target or an action, the shape that tells a path from an ordinary word, and
//! the span that holds the payload.  [`general_planner`](super::general_planner)
//! composes them into a plan; this module answers only what the prose says, so
//! a second route can ask the same questions without re-deriving them and
//! drifting from the parse the planner will actually execute (issue #1066).
use super::file_path_shape::{is_dotted_number, peel_sentence_punctuation};
use super::shell_command_policy::sentences;
use crate::seed::{self, Slot};
/// One whitespace token together with its byte span in the original request.
pub(super) struct Token<'a> {
    pub(super) text: &'a str,
    pub(super) start: usize,
    pub(super) end: usize,
}
/// Split a request into whitespace tokens, recording each token's byte span.
pub(super) fn tokens(request: &str) -> Vec<Token<'_>> {
    let mut cursor = 0;
    request
        .split_whitespace()
        .map(|word| {
            let start = request[cursor..]
                .find(word)
                .map_or(cursor, |offset| cursor + offset);
            let end = start + word.len();
            cursor = end;
            Token {
                text: word,
                start,
                end,
            }
        })
        .collect()
}
/// The bare (whole-word) surface forms for a role, lowercased for token matching.
pub(super) fn bare_surfaces(role: &str) -> Vec<String> {
    seed::lexicon()
        .role_word_forms(role)
        .iter()
        .filter(|form| form.slot() == Slot::Bare)
        .map(|form| form.text.to_lowercase())
        .collect()
}
/// Trim the quoting/edge punctuation from a token that may be a file path,
/// preserving the interior dots that make it look like a file. Trailing sentence
/// punctuation is stripped too, so a plain word that merely *ends a sentence*
/// ("… add the plural to томат.") is not mistaken for a file whose only dot is the
/// terminal period — a real filename never ends in a bare `.`/`!`/`?`.
pub(super) fn clean_path_token(word: &str) -> &str {
    peel_sentence_punctuation(word, |token| {
        token
            .trim_matches(|c: char| matches!(c, '`' | '"' | '\'' | ',' | ':' | ';'))
            .trim_end_matches(['!', '?'])
    })
}
/// Whether a safe-looking token names a file rather than merely using the
/// conventional `./` prefix for a directory.
///
/// Checking the whole token for a dot made policy prose such as "keep examples
/// in ./examples" file-shaped. When a later sentence contained a write-content
/// marker, the generic planner consequently tried to overwrite that directory.
/// File shape belongs to the final path component; dots in parent components or
/// in the relative-path prefix do not make the target a file.
pub(super) fn looks_like_file_path(path: &str) -> bool {
    !path.contains("://")
        && !is_dotted_number(path)
        && path
            .rsplit('/')
            .next()
            .is_some_and(|file_name| file_name.contains('.'))
}
/// Lowercase a token stripped of edge punctuation, for cue/action comparison.
pub(super) fn clean_cue_token(word: &str) -> String {
    word.trim_matches(|c: char| matches!(c, '`' | '"' | '\'' | ',' | ':' | ';' | '.' | '!' | '?'))
        .to_lowercase()
}
/// The byte span just past the leftmost `file_write_content_lead` marker in the
/// lowercased request, honouring whole-word boundaries for space-delimited
/// scripts and substring matches for CJK (which has no inter-word spaces).
pub(super) fn first_content_lead_end(lowered: &str) -> Option<(usize, usize)> {
    first_prefix_lead_end(lowered, seed::ROLE_FILE_WRITE_CONTENT_LEAD)
}
pub(super) fn first_prefix_lead_end(lowered: &str, role: &str) -> Option<(usize, usize)> {
    let markers: Vec<String> = seed::lexicon()
        .role_word_forms(role)
        .iter()
        .filter(|form| form.slot() == Slot::Prefix)
        .map(|form| form.before_slot().trim().to_lowercase())
        .filter(|marker| !marker.is_empty())
        .collect();
    let mut best: Option<(usize, usize)> = None;
    for marker in &markers {
        let mut from = 0;
        while let Some(relative) = lowered[from..].find(marker.as_str()) {
            let start = from + relative;
            let end = start + marker.len();
            let cjk = !marker.contains(' ') && !marker.is_ascii();
            let before_ok = cjk
                || start == 0
                || lowered[..start]
                    .chars()
                    .next_back()
                    .is_some_and(char::is_whitespace);
            let after_ok = cjk
                || end == lowered.len()
                || lowered[end..]
                    .chars()
                    .next()
                    .is_some_and(|c| c.is_whitespace() || c.is_ascii_punctuation());
            if before_ok && after_ok {
                if best.is_none_or(|(best_start, best_end)| {
                    start < best_start || (start == best_start && end > best_end)
                }) {
                    best = Some((start, end));
                }
                break;
            }
            from = end;
        }
    }
    best
}
/// The target file of a write, as `(token index, path)`: the first safe,
/// file-looking token that directly follows a target cue, a destination cue, or
/// an action cue. Requiring a cue keeps an incidental dotted token (a version,
/// an abbreviation) out of the write path.
pub(super) fn cued_write_target(toks: &[Token<'_>]) -> Option<(usize, String)> {
    let target_cues = bare_surfaces(seed::ROLE_FILE_WRITE_TARGET_CUE);
    let dest_cues = bare_surfaces(seed::ROLE_FILE_WRITE_DESTINATION_CUE);
    let action_cues = bare_surfaces(seed::ROLE_FILE_WRITE_ACTION_CUE);
    toks.iter().enumerate().find_map(|(index, token)| {
        let cleaned = clean_path_token(token.text);
        if !looks_like_file_path(cleaned) || !safe_relative_path(cleaned) {
            return None;
        }
        let previous = index.checked_sub(1).map(|i| &toks[i])?;
        let previous_word = clean_cue_token(previous.text);
        (target_cues.contains(&previous_word)
            || dest_cues.contains(&previous_word)
            || action_cues.contains(&previous_word))
        .then(|| (index, cleaned.to_owned()))
    })
}
/// The path a request names as the destination of a write, whether or not it
/// also spells the bytes out.
///
/// [`parse_write_request`] answers a narrower question — it recovers a write it
/// can execute verbatim — and deliberately declines a request whose payload has
/// still to be *composed*. The evidence-record route needs the target of
/// exactly that declined shape ("record what you find in FILE"), so the target
/// half of the parse is exposed on its own rather than duplicated (issue #1066).
pub(super) fn stated_write_target(request: &str) -> Option<String> {
    cued_write_target(&tokens(request)).map(|(_, target)| target)
}
/// Whether the request applies a seed-defined write action to anything.
///
/// Used with [`stated_write_target`] to tell "record it in FILE" (a write whose
/// content is composed) from "read the first line in FILE" (a read that merely
/// mentions a file after a positional cue).
pub(super) fn states_write_action(request: &str) -> bool {
    first_action_cue_end(&tokens(request)).is_some()
}

/// Whether the sentence leads with an *edit* action rather than a write one.
///
/// The seed declares two families of action cue, and they are two different
/// relations to a file: an answer is composed and delivered *into* a
/// destination, while an edit changes the content *of* a file that already has
/// some. Only the first relation can be a delivery, so a sentence stating the
/// second names an operand of the work and never a place to put some other
/// sentence's finding.
///
/// The families cannot be told apart by mere mention. Measured over the 1 118
/// request sentences this repository records that name a file and carry a cue
/// (`experiments/issue_1069_delivery_vs_operand/cue-order-survey.py`), 30 of
/// them -- 2.68% -- use both, and the ladder's own delivery sentence is one:
/// "Then create `agent-ladder-effects/node-1.1.2.2.1.lino` … followed by at
/// least four words that state the **change** you made". Reading that as an
/// edit loses the record the node exists to produce.
///
/// What separates them is position, the same way [`cued_write_target`] binds a
/// cue to a path by adjacency: the cue a sentence leads with is the one that
/// governs it, and any later cue belongs to a clause the leading one already
/// took as its object. On the real node prompt this reads all four
/// file-naming sentences correctly, including both that carry both families.
pub(super) fn leads_with_edit_action(sentence: &str) -> bool {
    let writes = bare_surfaces(seed::ROLE_FILE_WRITE_ACTION_CUE);
    let edits = bare_surfaces(seed::ROLE_FILE_EDIT_ACTION_CUE);
    tokens(sentence)
        .iter()
        .find_map(|token| {
            let word = clean_cue_token(token.text);
            if writes.contains(&word) {
                Some(false)
            } else if edits.contains(&word) {
                Some(true)
            } else {
                None
            }
        })
        .unwrap_or(false)
}

/// Whether `request` names `path` as the destination of a write it states.
///
/// The read routes need this to keep from opening the file they were asked to
/// create. "Leave observable evidence in `.agent-ladder/node-1.2-proof.md`. The
/// first line must be exactly `node_path=1.2`" names one path and one read cue
/// -- *first line* -- and the two belong to different obligations: the path is
/// where the answer goes, the cue describes how it has to open. Read the prompt
/// as a whole and the cue captures the path, and the run opens the file it was
/// supposed to write.
///
/// Scoped to a sentence, for the same reason [`super::evidence_record`] scopes
/// its own split: "Read the file `Cargo.toml`. Record what you find in
/// `notes/report.md`." states a write of the second path only, and the first
/// must stay readable.
pub(super) fn is_stated_write_target(request: &str, path: &str) -> bool {
    sentences(request).into_iter().any(|sentence| {
        states_write_action(sentence.text)
            && stated_write_target(sentence.text).is_some_and(|target| target == path)
    })
}

/// The opening line a sentence pins, read through
/// [`seed::ROLE_FILE_LEADING_LINE_CONSTRAINT_LEAD`].
///
/// The lowercased copy is byte-length preserving for every supported language,
/// so the marker's end offset slices the original sentence and the recovered
/// line keeps its case.
pub(super) fn pinned_first_line(sentence: &str) -> Option<String> {
    let lowered = sentence.to_lowercase();
    let (_, end) = first_prefix_lead_end(&lowered, seed::ROLE_FILE_LEADING_LINE_CONSTRAINT_LEAD)?;
    let raw = sentence
        .get(end..)?
        .trim()
        .trim_start_matches([':', '-', '\u{2014}', '\u{2013}'])
        .trim();
    let line = delimited_first_line(raw)
        .or_else(|| unquoted_machine_first_line(raw))
        .unwrap_or(raw)
        .trim_matches(['`', '"', '\'']);
    (!line.is_empty()).then(|| line.to_owned())
}

/// An explicitly delimited opening line, without the presentation delimiters.
///
/// The closing delimiter is also a grammatical boundary: in "exactly `id=7`
/// and the body ...", the coordinated body constraint is not part of the
/// machine-readable header.
fn delimited_first_line(raw: &str) -> Option<&str> {
    let delimiter = raw.chars().next()?;
    if !matches!(delimiter, '`' | '"' | '\'') {
        return None;
    }
    let after_open = raw.get(delimiter.len_utf8()..)?;
    let close = after_open.find(delimiter)?;
    after_open.get(..close)
}

/// Stop an unquoted machine header before a coordinated second requirement.
///
/// Natural-language lines such as "ready and waiting" remain whole. A compact
/// assignment/header token (`name=value`, `name:value`) followed by a
/// seed-defined procedure separator is unambiguous: the separator begins the
/// next clause, as it did in the live Agent ladder prompt.
fn unquoted_machine_first_line(raw: &str) -> Option<&str> {
    let lowered = raw.to_lowercase();
    bare_surfaces(seed::ROLE_SKILL_PROCEDURE_CLAUSE_SEPARATOR)
        .into_iter()
        .filter_map(|separator| {
            let marker = format!(" {separator} ");
            lowered.find(&marker).and_then(|boundary| {
                let candidate = raw.get(..boundary)?.trim();
                (!candidate.contains(char::is_whitespace)
                    && candidate.contains(['=', ':']))
                .then_some(candidate)
            })
        })
        .min_by_key(|candidate| candidate.len())
}
/// The opening line the request pins, wherever in the request it pins it.
///
/// A pinned first line constrains the finished file, not the sentence that
/// happens to state it. [`super::evidence_record`] reads the constraint one
/// sentence at a time because it is separating delivery from work; a route that
/// only has to *obey* the constraint needs nothing more than its text.
pub(super) fn pinned_first_line_of_request(request: &str) -> Option<String> {
    sentences(request)
        .into_iter()
        .find_map(|sentence| pinned_first_line(sentence.text))
}
/// The same content, rewritten to open with the line `request` pinned.
///
/// A literal write is only literal when it satisfies every stated constraint on
/// the file it writes, and a request can state two: what the file contains, and
/// how it begins. "Write `notes/x.md` containing alpha, beta and gamma. The
/// first line must be exactly `id=7`" reads as a literal write to the broad
/// content parser -- *containing* cues content -- and the bytes it recovers are
/// the prose that followed the cue, which neither open with the pinned line nor
/// leave the pinning sentence out. Writing them satisfies the sentence that was
/// parsed and violates the one that was not.
///
/// Both corrections come out of a parse that has already happened -- the pinned
/// line, and which sentence pinned it -- so the plan is repaired rather than
/// dropped. Dropping it was the earlier fix, and it moved the failure instead of
/// removing it: a request that states every byte it wants and no work to do has
/// no other route to reach, so declining sent the plainest write there is to the
/// open-web routers.
///
/// The pinning sentence leaves the body with it. It was recovered only because
/// the content lead swept up the rest of the prose; the caller stated it as a
/// constraint on the file, never as something to put inside the file.
///
/// Returns `None` when there is nothing to repair -- `request` pins no line, or
/// `content` already opens with it (issue #1066).
pub(super) fn honouring_pinned_first_line(request: &str, content: &str) -> Option<String> {
    let line = pinned_first_line_of_request(request)?;
    if content.starts_with(&line) {
        return None;
    }
    let body = without_pinning_sentences(content);
    Some(if body.is_empty() {
        format!("{line}\n")
    } else {
        format!("{line}\n\n{body}\n")
    })
}

/// `content` without any sentence that is itself a leading-line constraint.
fn without_pinning_sentences(content: &str) -> String {
    sentences(content)
        .into_iter()
        .filter(|sentence| pinned_first_line(sentence.text).is_none())
        .map(|sentence| content[sentence.span].trim())
        .filter(|kept| !kept.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}
/// The first `file_write_action_cue` token, with its span.
fn first_action_cue<'a, 'b>(toks: &'a [Token<'b>]) -> Option<&'a Token<'b>> {
    let actions = bare_surfaces(seed::ROLE_FILE_WRITE_ACTION_CUE);
    toks.iter()
        .find(|token| actions.contains(&clean_cue_token(token.text)))
}
/// The byte offset just past the first `file_write_action_cue` token.
pub(super) fn first_action_cue_end(toks: &[Token<'_>]) -> Option<usize> {
    first_action_cue(toks).map(|token| token.end)
}
/// The byte offset at which the first `file_write_action_cue` token begins.
///
/// [`first_action_cue_end`] answers where a payload may start; this answers
/// where the delivery clause does, and the two questions have different callers.
/// A sentence can carry both halves of a request -- "Break the customer import
/// rewrite into sub-tasks and record what you work out in `import-split.md`"
/// states the work and then, mid-sentence, says where its result goes -- so a
/// reader that hands the whole sentence to delivery throws the work away with
/// it (issue #1066).
pub(super) fn first_action_cue_start(toks: &[Token<'_>]) -> Option<usize> {
    first_action_cue(toks).map(|token| token.start)
}
/// Trim a recovered content span down to its literal payload, dropping the
/// leading clause separator ("… the following: hello") and any surrounding
/// quoting. A delimiter is removed only when the entire payload has a matching
/// opening and closing delimiter. This matters for generated source and Links
/// Notation: a lone terminal quote is data, not presentation punctuation.
/// Returns [`None`] when nothing is left.
pub(super) fn clean_content(raw: &str) -> Option<String> {
    let led = strip_clause_lead(raw);
    let result = if led.len() >= 6 && led.starts_with("```") && led.ends_with("```") {
        led[3..led.len() - 3].trim()
    } else if led.len() >= 2 {
        let first = led.as_bytes()[0];
        let last = led.as_bytes()[led.len() - 1];
        if first == last && matches!(first, b'`' | b'"' | b'\'') {
            led[1..led.len() - 1].trim()
        } else {
            led
        }
    } else {
        led
    };
    (!result.is_empty()).then(|| result.to_owned())
}
/// Strip everything a recovered span carries *before* its literal payload: the
/// clause separators, and the seed-defined adverbs that qualify the requirement
/// rather than naming content.
///
/// "…containing exactly: Hello World" delimits the content with `exactly:`, so
/// slicing after the content lead captured `exactly: Hello World` as the bytes
/// to write and as the evidence to verify against — the file would never have
/// matched (issue #905 §3).
fn strip_clause_lead(raw: &str) -> &str {
    let qualifiers = bare_surfaces(seed::ROLE_FILE_WRITE_CONTENT_QUALIFIER);
    let mut led = raw.trim();
    loop {
        let separated = led.trim_start_matches([':', '-', '—', '–']).trim();
        let shortened = strip_leading_qualifier(separated, &qualifiers);
        if shortened.len() == led.len() {
            return led;
        }
        led = shortened;
    }
}
/// Drop one leading qualifier, but only when a clause separator follows it. The
/// separator is what marks the adverb as introducing the payload rather than
/// opening it, so content that genuinely starts with "exactly what I asked for"
/// keeps its first word.
fn strip_leading_qualifier<'a>(text: &'a str, qualifiers: &[String]) -> &'a str {
    let lowered = text.to_lowercase();
    qualifiers
        .iter()
        .filter(|qualifier| lowered.starts_with(qualifier.as_str()))
        .filter_map(|qualifier| {
            let rest = text.get(qualifier.len()..)?.trim_start();
            rest.starts_with([':', '-', '—', '–']).then_some(rest)
        })
        .min_by_key(|rest| rest.len())
        .unwrap_or(text)
}
pub(super) fn safe_relative_path(path: &str) -> bool {
    !path.starts_with('/')
        && !path.starts_with('-')
        && !path.split('/').any(|part| part == ".." || part.is_empty())
        && path
            .chars()
            .all(|c| c.is_alphanumeric() || matches!(c, '/' | '.' | '_' | '-'))
}
