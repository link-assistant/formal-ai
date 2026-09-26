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
use super::shell_command_policy::{prose_sentences, sentences};
use crate::seed::{self, Slot};
/// One whitespace token together with its byte span in the original request.
pub(super) struct Token<'a> {
    pub(super) text: &'a str,
    pub(super) start: usize,
    pub(super) end: usize,
}
/// Split a request into tokens, recording each token's byte span.
///
/// The separators are whitespace *and* the ideographic punctuation marks, for
/// the same reason the clause splitter knows `。`: a script that does not space
/// its words still separates its clauses, and a whitespace-only split glues the
/// punctuation and everything after it onto the token before. `创建文件
/// notes/attribution.md，内容为 Gemfile.lock。` produced the single token
/// `notes/attribution.md，内容为`, which is not a safe relative path, so the
/// Chinese wording of a write request named no target at all.
pub(super) fn tokens(request: &str) -> Vec<Token<'_>> {
    let mut out = Vec::new();
    let mut start: Option<usize> = None;
    for (index, character) in request.char_indices() {
        if character.is_whitespace() || is_ideographic_punctuation(character) {
            if let Some(from) = start.take() {
                out.push(Token {
                    text: &request[from..index],
                    start: from,
                    end: index,
                });
            }
            continue;
        }
        if start.is_none() {
            start = Some(index);
        }
    }
    if let Some(from) = start {
        out.push(Token {
            text: &request[from..],
            start: from,
            end: request.len(),
        });
    }
    out
}

/// Whether a character is one of the ideographic punctuation marks that
/// separate words and clauses in a script written without spaces.
const fn is_ideographic_punctuation(character: char) -> bool {
    matches!(character,
        '\u{3001}'..='\u{3003}'
            | '\u{3008}'..='\u{3011}'
            | '\u{3014}'..='\u{301f}'
            | '\u{ff01}'
            | '\u{ff08}'
            | '\u{ff09}'
            | '\u{ff0c}'
            | '\u{ff1a}'
            | '\u{ff1b}'
            | '\u{ff1f}')
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
///
/// The Devanagari full stop `।` ends a Hindi sentence exactly as `.` ends an
/// English one, so a cue that closes a clause — `लिखो।` — is the same cue as
/// `लिखो`. Without it the verb of every sentence-final Hindi request went
/// unrecognised.
pub(super) fn clean_cue_token(word: &str) -> String {
    word.trim_matches(|c: char| {
        matches!(
            c,
            '`' | '"' | '\'' | ',' | ':' | ';' | '.' | '!' | '?' | '\u{0964}' | '\u{0965}'
        )
    })
    .to_lowercase()
}
/// The byte span just past the leftmost `file_write_content_lead` marker in the
/// lowercased request, honouring whole-word boundaries for space-delimited
/// scripts and substring matches for CJK (which has no inter-word spaces).
pub(super) fn first_content_lead_end(lowered: &str) -> Option<(usize, usize)> {
    first_prefix_lead_end(lowered, seed::ROLE_FILE_WRITE_CONTENT_LEAD)
}
/// Where the payload a circumfix content marker opened has to stop.
///
/// "जिसमें Gemfile.lock हो" states its content between two literals, and so does
/// "把 Gemfile.lock 写入": the closing literal is a grammatical boundary, not
/// part of the bytes. Returns [`None`] when the marker that ended at `from` was
/// not a circumfix one, which is every marker in the languages that put their
/// content last.
pub(super) fn content_lead_close(lowered: &str, from: usize) -> Option<usize> {
    seed::lexicon()
        .role_word_forms(seed::ROLE_FILE_WRITE_CONTENT_LEAD)
        .iter()
        .filter(|form| form.slot() == Slot::Circumfix)
        .filter_map(|form| {
            let opener = form.before_slot().trim().to_lowercase();
            let closer = form.after_slot().trim().to_lowercase();
            if opener.is_empty()
                || closer.is_empty()
                || !lowered.get(..from)?.trim_end().ends_with(&opener)
            {
                return None;
            }
            lowered
                .get(from..)?
                .find(&closer)
                .map(|relative| from + relative)
        })
        .min()
}
pub(super) fn first_prefix_lead_end(lowered: &str, role: &str) -> Option<(usize, usize)> {
    let markers: Vec<(String, String)> = seed::lexicon()
        .role_word_forms(role)
        .iter()
        .filter(|form| matches!(form.slot(), Slot::Prefix | Slot::Circumfix))
        .map(|form| {
            (
                form.before_slot().trim().to_lowercase(),
                form.after_slot().trim().to_lowercase(),
            )
        })
        .filter(|(marker, _)| !marker.is_empty())
        .collect();
    let mut best: Option<(usize, usize)> = None;
    for (marker, closer) in &markers {
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
            // A circumfix form holds its subject *between* two literals, so its
            // opener means nothing on its own: "把" opens a Chinese object
            // phrase only when the verb that closes it follows. Requiring the
            // closer keeps a one-character opener from claiming every sentence
            // that happens to contain it.
            let closed = closer.is_empty() || lowered[end..].contains(closer.as_str());
            if before_ok && after_ok && closed {
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
/// Which family of cue named a path, which decides what the rest of the
/// sentence is allowed to mean.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum CueFamily {
    /// A `file_write_target_cue` — the file the work is about.
    Target,
    /// A `file_write_destination_cue` — somewhere to deliver bytes.
    Destination,
    /// A bare `file_write_action_cue`, which names a file only incidentally.
    Action,
}

/// One path a sentence names, together with the cue that named it.
#[derive(Debug, Clone)]
pub(super) struct WriteBinding {
    /// Token index of the path.
    pub(super) index: usize,
    /// The cleaned, safe relative path.
    pub(super) path: String,
    /// Byte offset at which the binding cue begins.
    pub(super) cue_start: usize,
    /// Byte offset just past the binding cue.
    pub(super) cue_end: usize,
    /// Which family of cue bound it.
    pub(super) family: CueFamily,
    /// Whether the cue stands *before* the path. A postposition stands after it.
    pub(super) cue_precedes: bool,
}

/// The target file of a write, as `(token index, path)`.
///
/// A sentence can offer more than one file-shaped token behind a cue, and which
/// one it *delivers to* is not the leftmost: "Write Gemfile.lock into
/// notes/attribution.md" names the content first, behind the bare action cue
/// *Write*, and the destination second, behind *into*. Binding the leftmost
/// bound the content as the target, the destination-led branch below then failed
/// its `action_end <= clause_start` test, and the whole clause composed to
/// nothing — in all five languages.
///
/// So the candidates are ranked by what named them rather than by where they
/// stand: a path a *target* or *destination* cue points at outranks one only an
/// action cue stands beside, and a cue that follows its path (a postposition)
/// ranks last. A sentence that offers exactly one candidate is unaffected, which
/// is every sentence the earlier rule read correctly.
pub(super) fn cued_write_target(toks: &[Token<'_>]) -> Option<(usize, String)> {
    preferred_binding(toks).map(|binding| (binding.index, binding.path))
}

/// Every candidate binding, best first.
///
/// A cue that stands *between* two file-shaped tokens could belong to either of
/// them, and no ranking settles that on its own: `notes/attribution.md में
/// Gemfile.lock लिखो` puts the destination postposition after the path it marks,
/// so the same rule that reads `into notes/attribution.md` correctly reads the
/// Hindi sentence backwards. The reading is settled by the rest of the sentence
/// instead — the caller takes these in order and keeps the first that yields a
/// payload — so the ranking only has to decide which reading is *tried* first.
pub(super) fn ranked_bindings(toks: &[Token<'_>]) -> Vec<WriteBinding> {
    let mut bindings = write_bindings(toks);
    bindings.sort_by_key(binding_rank);
    bindings
}

/// The binding a write request is read through: the highest-ranked candidate.
pub(super) fn preferred_binding(toks: &[Token<'_>]) -> Option<WriteBinding> {
    ranked_bindings(toks).into_iter().next()
}

/// Lower ranks are preferred. Token order breaks a tie, which `min_by_key`
/// preserves because the bindings are produced in token order.
const fn binding_rank(binding: &WriteBinding) -> u8 {
    match (binding.cue_precedes, binding.family) {
        (true, CueFamily::Target | CueFamily::Destination) => 0,
        (true, CueFamily::Action) => 1,
        (false, _) => 2,
    }
}

/// Every cued target in token order, so a caller that has a further question to
/// ask about a candidate can go on to the next one instead of losing the
/// sentence with it.
pub(super) fn cued_write_targets(toks: &[Token<'_>]) -> Vec<(usize, String)> {
    write_bindings(toks)
        .into_iter()
        .filter(|binding| binding.cue_precedes)
        .map(|binding| (binding.index, binding.path))
        .collect()
}

/// Every path a cue binds, in token order.
///
/// A cue stands before its path in the languages that put it there and after it
/// in the languages that do not: Hindi writes `notes/attribution.md फ़ाइल` and
/// `notes/attribution.md में`, so a reader that only ever looks one token back
/// finds no target in a Hindi request at all. A postposition is therefore read
/// too, but only for the target and destination families — an action cue after a
/// path closes the clause rather than naming the file — and only when the path
/// has no cue in front of it.
fn write_bindings(toks: &[Token<'_>]) -> Vec<WriteBinding> {
    let target_cues = bare_surfaces(seed::ROLE_FILE_WRITE_TARGET_CUE);
    let dest_cues = bare_surfaces(seed::ROLE_FILE_WRITE_DESTINATION_CUE);
    let action_cues = bare_surfaces(seed::ROLE_FILE_WRITE_ACTION_CUE);
    // The fused reading is for the nouns and particles that a script without
    // spaces writes against their neighbour — `创建文件` is "create" and "file",
    // `文件` is the cue. A verb is not read that way: `输出内容 sample.txt` opens
    // with the write verb `输出` and means "output the contents of", a read, and
    // reading the verb out of the fused token claimed the file the request asked
    // to be shown.
    let families = [
        (CueFamily::Destination, &dest_cues, true),
        (CueFamily::Target, &target_cues, true),
        (CueFamily::Action, &action_cues, false),
    ];
    toks.iter()
        .enumerate()
        .filter_map(|(index, token)| {
            let cleaned = clean_path_token(token.text);
            if !looks_like_file_path(cleaned) || !safe_relative_path(cleaned) {
                return None;
            }
            let path = cleaned.to_owned();
            if let Some(previous) = index.checked_sub(1).map(|before| &toks[before])
                && let Some((family, cue_start, cue_end)) =
                    families.iter().find_map(|(family, cues, fused)| {
                        trailing_cue(previous, cues, *fused).map(|span| (*family, span.0, span.1))
                    })
            {
                return Some(WriteBinding {
                    index,
                    path,
                    cue_start,
                    cue_end,
                    family,
                    cue_precedes: true,
                });
            }
            let next = toks.get(index + 1)?;
            families
                .iter()
                .take(2)
                .find_map(|(family, cues, fused)| {
                    leading_cue(next, cues, *fused).map(|span| (*family, span.0, span.1))
                })
                .map(|(family, cue_start, cue_end)| WriteBinding {
                    index,
                    path,
                    cue_start,
                    cue_end,
                    family,
                    cue_precedes: false,
                })
        })
        .collect()
}

/// The span of a cue standing at the *end* of `token`, if one does.
///
/// An unspaced script writes the cue and the word before it as one token —
/// `创建文件` is "create" and "file" with nothing between them — so for a cue in
/// such a script the token's tail is the boundary the language actually has.
fn trailing_cue(token: &Token<'_>, cues: &[String], fused: bool) -> Option<(usize, usize)> {
    let cleaned = clean_cue_token(token.text);
    cues.iter().find_map(|cue| {
        if cleaned == *cue {
            return Some((token.start, token.end));
        }
        (fused && is_unspaced_surface(cue) && cleaned.ends_with(cue.as_str()))
            .then(|| (token.end.saturating_sub(cue.len()), token.end))
    })
}

/// The span of a cue standing at the *start* of `token`, if one does.
fn leading_cue(token: &Token<'_>, cues: &[String], fused: bool) -> Option<(usize, usize)> {
    let cleaned = clean_cue_token(token.text);
    cues.iter().find_map(|cue| {
        if cleaned == *cue {
            return Some((token.start, token.end));
        }
        (fused && is_unspaced_surface(cue) && cleaned.starts_with(cue.as_str()))
            .then(|| (token.start, token.start + cue.len()))
    })
}

/// Whether a surface belongs to a script that does not separate its words with
/// spaces, so a token boundary is not where the word ends.
fn is_unspaced_surface(surface: &str) -> bool {
    crate::coding::catalog::contains_cjk(surface)
}

/// Whether a token is written as a double-quoted literal.
///
/// A path is *mentioned* -- bare, or set in the backticks this repository's
/// prose uses for code -- whereas a double-quoted token is a *value* the
/// sentence is handing to something else: the member "add \"bun.lock\" to the
/// `IGNORE_FILES` list" inserts, the marker the ladder asks a record to contain.
/// Both can be file-shaped, and `clean_path_token` peels either quoting away, so
/// the distinction has to be read before the peeling.
fn quoted_as_value(word: &str) -> bool {
    let bare = peel_sentence_punctuation(word, |token| token.trim_end_matches([',', ':', ';']));
    bare.len() >= 2 && bare.starts_with('"') && bare.ends_with('"')
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
/// Bind a source destination using the ordinary cue/path safety rules, with
/// the language's artifact type as a further constraint. Output operands and
/// workflow paths do not become source destinations by appearing first.
pub fn typed_write_target(request: &str, extension: &str) -> Option<String> {
    cued_write_targets(&tokens(request))
        .into_iter()
        .map(|(_, path)| path)
        .find(|path| {
            std::path::Path::new(path)
                .extension()
                .and_then(|value| value.to_str())
                == Some(extension)
        })
}
/// Whether the request applies a seed-defined write action to anything.
///
/// Used with [`stated_write_target`] to tell "record it in FILE" (a write whose
/// content is composed) from "read the first line in FILE" (a read that merely
/// mentions a file after a positional cue).
pub(super) fn states_write_action(request: &str) -> bool {
    first_action_cue_end(&tokens(request)).is_some()
}

/// The path a sentence names as the *destination* of a write it states.
///
/// [`stated_write_target`] answers a weaker question -- which path a cue points
/// at -- and a sentence can point a cue at a path it is not delivering to,
/// because the same cues introduce the file the work is done *in*. Both of these
/// name a path after a target cue and state a write action:
///
/// > In the file `src/solver_handlers/document_request.rs`, add "toward " to the
/// > TARGET_MARKERS list.
///
/// > Then create `agent-ladder-effects/node-1.1.2.2.1.lino` recording what you
/// > changed.
///
/// Only the second is somewhere to put an answer. In the first the thing being
/// added goes into a list *inside* the file, and reading it as a delivery
/// destroys the file it was asked to edit -- the planner writes its own status
/// line over the source and the write reports success.
///
/// What separates them is order, the same adjacency principle
/// [`cued_write_target`] already uses to bind a cue to a path. A delivery
/// composes something and then says where to put it, so its path *follows* the
/// action; a sentence that says where the work happens states the place first
/// and the action afterwards.
///
/// Measured over the 1 118 request sentences this repository records that name a
/// file and carry a cue (`experiments/issue_1069_paths_in_prose/cue-order-survey.py`),
/// the path follows the action in 21.56% of them, precedes it in 65.74%, and in
/// the remaining 12.70% there is no write cue at all -- an edit, which is never a
/// delivery. So this reads roughly a fifth of them as destinations, where the
/// weaker "cues a path and states a write" test reads all 1 118.
///
/// The rule that suggests itself first is the cue family: the seed declares a
/// second family for edits, so a sentence mentioning one could be doing work
/// rather than delivering. That fails on this corpus. Thirty of the sentences
/// carry both families, and the ladder's own delivery sentence is one of them --
/// "Then create `agent-ladder-effects/node-1.1.2.2.1.lino` … followed by at
/// least four words that state the **change** you made" -- so the family rule
/// discards the record the node exists to produce. Position separates what
/// mention cannot.
///
/// Position alone still admits a value that merely *looks* like a path, because
/// the operand of an insertion follows the action too: "Edit `scripts/metric.rs`:
/// add \"bun.lock\" to the `IGNORE_FILES` list" names the file before the action
/// and the member after it. So the candidates are taken in order and a
/// double-quoted one is skipped rather than accepted -- see [`quoted_as_value`].
/// The same survey measures what that costs: of the 241 sentences the position
/// rule admits, 2 (0.83%) quote their path, and both are member insertions of
/// exactly this shape. No delivery destination in the corpus is double-quoted.
pub(super) fn delivered_write_target(sentence: &str) -> Option<String> {
    let toks = tokens(sentence);
    let action = first_action_cue_start(&toks)?;
    cued_write_targets(&toks)
        .into_iter()
        .find(|(index, _)| toks[*index].start > action && !quoted_as_value(toks[*index].text))
        .map(|(_, path)| path)
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
                (!candidate.contains(char::is_whitespace) && candidate.contains(['=', ':']))
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
/// The span of the first `file_write_action_cue` in the request.
fn first_action_cue(toks: &[Token<'_>]) -> Option<(usize, usize)> {
    let actions = bare_surfaces(seed::ROLE_FILE_WRITE_ACTION_CUE);
    toks.iter()
        .find_map(|token| leading_cue(token, &actions, false))
}
/// The byte offset just past the first `file_write_action_cue` token.
pub(super) fn first_action_cue_end(toks: &[Token<'_>]) -> Option<usize> {
    first_action_cue(toks).map(|(_, end)| end)
}
/// Where the first `file_write_action_cue` at or after `from` begins.
///
/// A language that closes its clause with the verb states the action *after* the
/// destination it was given, so the payload is what stands between them. The
/// prepositional shape reads the same span from the other side — see
/// `parse_write_request`.
pub(super) fn action_cue_start_after(toks: &[Token<'_>], from: usize) -> Option<usize> {
    let actions = bare_surfaces(seed::ROLE_FILE_WRITE_ACTION_CUE);
    toks.iter()
        .filter(|token| token.start >= from)
        .find_map(|token| leading_cue(token, &actions, false))
        .map(|(start, _)| start)
}
/// The byte offset at which the first `file_write_action_cue` begins.
///
/// [`first_action_cue_end`] answers where a payload may start; this answers
/// where the delivery clause does, and the two questions have different callers.
/// A sentence can carry both halves of a request -- "Break the customer import
/// rewrite into sub-tasks and record what you work out in `import-split.md`"
/// states the work and then, mid-sentence, says where its result goes -- so a
/// reader that hands the whole sentence to delivery throws the work away with
/// it (issue #1066).
pub(super) fn first_action_cue_start(toks: &[Token<'_>]) -> Option<usize> {
    first_action_cue(toks).map(|(start, _)| start)
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

/// Recover the `(target, old, new)` of a file-edit request from its wording.
///
/// Every cue comes from the seed lexicon. A target may be a safe relative path
/// or an unambiguous self-AST census reference; ambiguous spans fail closed.
#[must_use]
pub fn compose_edit_request(request: &str) -> Option<(String, String, String)> {
    if let Some(edit) = super::positional_edit::compose_positional_insert(request) {
        return Some(edit);
    }
    let toks = tokens(request);
    let action_cues = bare_surfaces(seed::ROLE_FILE_EDIT_ACTION_CUE);
    let new_leads = bare_surfaces(seed::ROLE_FILE_EDIT_NEW_LEAD_CUE);
    let target_cues = bare_surfaces(seed::ROLE_FILE_EDIT_TARGET_CUE);
    let is_target_cue = |index: usize| target_cues.contains(&clean_cue_token(toks[index].text));
    let is_action_cue = |index: usize| action_cues.contains(&clean_cue_token(toks[index].text));
    let (file_index, target) = toks.iter().enumerate().find_map(|(index, token)| {
        let cleaned = clean_path_token(token.text);
        let resolved = super::general_planner::resolve_census_target(cleaned);
        if resolved.is_none() && (!looks_like_file_path(cleaned) || !safe_relative_path(cleaned)) {
            return None;
        }
        let prev_is_cue = index
            .checked_sub(1)
            .is_some_and(|previous| is_target_cue(previous) || is_action_cue(previous));
        let next_is_cue =
            (index + 1 < toks.len()) && (is_target_cue(index + 1) || is_action_cue(index + 1));
        let target = resolved.map_or_else(|| cleaned.to_owned(), |census| census.module_path);
        (prev_is_cue || next_is_cue).then_some((index, target))
    })?;
    let mut clause_start_index = file_index;
    while clause_start_index > 0 && is_target_cue(clause_start_index - 1) {
        clause_start_index -= 1;
    }
    let file_clause_start = toks[clause_start_index].start;
    let action = toks
        .iter()
        .filter(|token| action_cues.contains(&clean_cue_token(token.text)))
        .find(|token| token.start > toks[file_index].end)
        .or_else(|| {
            toks.iter()
                .find(|token| action_cues.contains(&clean_cue_token(token.text)))
        })?;
    let action_end = action.end;
    let new_lead = toks.iter().find(|token| {
        token.start >= action_end && new_leads.contains(&clean_cue_token(token.text))
    })?;
    if file_clause_start >= action_end && file_clause_start < new_lead.start {
        return None;
    }
    let old_span = request.get(action_end..new_lead.start)?;
    let sentence_end = prose_sentences(request)
        .into_iter()
        .find(|sentence| sentence.span.contains(&new_lead.end))
        .map_or_else(
            || request.len(),
            |sentence| {
                let raw = &request[sentence.span.clone()];
                sentence.span.start + (raw.len() - raw.trim_start().len()) + sentence.text.len()
            },
        );
    let new_end = if file_clause_start > new_lead.end {
        file_clause_start.min(sentence_end)
    } else {
        sentence_end
    };
    let new_span = request.get(new_lead.end..new_end)?;
    let old = super::positional_edit::literal_text(old_span)?;
    let new = super::positional_edit::literal_text(new_span)?;
    Some((target, old, new))
}

/// Whether the payload starting at `from` is a block that outlives its first line.
///
/// A marker alone on its line already introduces the whole block below it, and
/// that is the shape this route was built for. The same request written with the
/// payload starting on the marker's own line -- "with exactly this content: #
/// Title\n\nbody..." -- means exactly the same thing, but the sentence bound cut
/// it at the first line and delivered a 58-byte file for a 1478-byte document.
///
/// That is not a bound the author of the request can see. It made every
/// multi-line literal write silently lossy: `alpha\nbeta\ngamma` was written as
/// `alpha`, and the self-authoring loop produced a one-line stub of the document
/// it was handed, which is how it looked broken while reporting success.
///
/// A payload whose first line ends but whose block continues is therefore read
/// to `limit`, exactly as the marker-only spelling always was. The sentence bound
/// still governs a payload that stays on one line, so "write the following: hello
/// to `x.txt`" is unaffected -- there is no continuation to find.
pub(super) fn payload_continues_past_its_first_line(
    request: &str,
    from: usize,
    sentence_end: usize,
) -> bool {
    let Some(tail) = request.get(from..) else {
        return false;
    };
    let Some(break_at) = tail.find('\n') else {
        return false;
    };
    // Only a payload the sentence bound would actually truncate qualifies: the
    // newline has to fall inside the sentence being cut, and real content has to
    // follow it. Otherwise the sentence bound is already returning the whole
    // payload and there is nothing to widen.
    from + break_at < sentence_end && tail[break_at..].chars().any(char::is_alphanumeric)
}
