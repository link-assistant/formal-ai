//! Composing the document a request describes, instead of writing its
//! description (issue #1066).
//!
//! "Produce a final evidence note containing the selected tree level, node
//! outcomes, test results, and session id." names no bytes. It names the
//! *headings* the finished note has to have. The literal-write parser reads
//! `containing` as its content lead and takes the words after it for the file,
//! which is right for "create `list.txt` containing apples, bananas and
//! cherries" and wrong here: the result is a file whose whole content is the
//! sentence that asked for it.
//!
//! Three seed-declared signals together say which of the two readings applies:
//! a composition verb ([`seed::ROLE_DOCUMENT_COMPOSITION_ACTION`]), the noun for
//! a document whose content is described rather than supplied
//! ([`seed::ROLE_COMPOSED_DOCUMENT_KIND`]), and a content lead
//! ([`seed::ROLE_FILE_WRITE_CONTENT_LEAD`]) followed by an enumeration of two or
//! more parts. All three have to be in the same sentence, the same scoping that
//! separates a named command from an ordered one (issue #907) and a delivery
//! path from a read path ([`super::evidence_record`]).
//!
//! What this route composes is a note, not a verdict. It records what the
//! request asked the note to cover, what the session actually observed, and --
//! plainly -- which requested parts nothing has established yet. A session that
//! observed nothing gets a note that says so. That is the honest deliverable for
//! a request of this shape, and it is deliberately not a claim of success: the
//! parts stay listed as outstanding until something in the conversation answers
//! them.

use super::planner::AgenticPlan;
use super::shell_command_policy::sentences;
use super::write_request::first_content_lead_end;
use crate::protocol::ChatMessage;
use crate::seed;

/// Plan the answer to a "produce a document covering A, B and C" request.
///
/// The plan is a [`AgenticPlan::Final`]: composing a note needs no tool. Where
/// the note has to *land* is a separate obligation that
/// [`super::evidence_record`] owns, so this route never writes a file itself --
/// which is what lets the two compose, the record route delivering what this one
/// composed.
pub(super) fn plan_note_composition_step(
    task: &str,
    messages: &[ChatMessage],
) -> Option<AgenticPlan> {
    let specification = parse_specification(task)?;
    Some(AgenticPlan::Final(compose(&specification, messages)))
}

/// What a request said its document has to cover.
struct Specification {
    /// The sentence that asked for the document, as the caller wrote it.
    request: String,
    /// The parts named after the content lead, in the order they were named.
    parts: Vec<String>,
}

/// Read the composition request out of `task`, one sentence at a time.
fn parse_specification(task: &str) -> Option<Specification> {
    sentences(task).into_iter().find_map(|sentence| {
        let normalized = crate::engine::normalize_prompt(sentence.text);
        let lexicon = seed::lexicon();
        if !lexicon.mentions_role(seed::ROLE_DOCUMENT_COMPOSITION_ACTION, &normalized)
            || !lexicon.mentions_role(seed::ROLE_COMPOSED_DOCUMENT_KIND, &normalized)
        {
            return None;
        }
        let parts = enumerated_parts(sentence.text)?;
        Some(Specification {
            request: sentence.text.trim().to_owned(),
            parts,
        })
    })
}

/// The two-or-more parts a sentence enumerates after its content lead.
///
/// One part is not an enumeration: "produce a report containing the exchange
/// rate" describes a single thing to find out, which the ordinary routes already
/// answer. Two or more is a specification of a document's structure, and that is
/// what this route composes.
fn enumerated_parts(sentence: &str) -> Option<Vec<String>> {
    let lowered = sentence.to_lowercase();
    let (_, end) = first_content_lead_end(&lowered)?;
    let separators = seed::lexicon().words_for_role(seed::ROLE_CLAUSE_CONTINUATION_MARKER);
    let parts: Vec<String> = sentence
        .get(end..)?
        .split(',')
        .flat_map(|span| split_on_separators(span, &separators))
        .map(|part| trimmed_part(&part))
        .filter(|part| !part.is_empty())
        .collect();
    (parts.len() >= 2).then_some(parts)
}

/// One enumerated part, with the punctuation that only joined it to the sentence.
///
/// A content lead can be followed by a colon or a dash before the first part
/// begins -- "with the content: the tree level, ..." -- and the last part carries
/// the sentence's full stop. Neither belongs to the part itself, and leaving them
/// on would make the first and last bullets read differently from the ones in
/// between for no reason the caller stated.
fn trimmed_part(part: &str) -> String {
    part.trim()
        .trim_start_matches([':', '-', '\u{2014}', '\u{2013}'])
        .trim_end_matches('.')
        .trim()
        .to_owned()
}

/// Split one comma-delimited span on any seed-declared clause continuation.
///
/// English writes the last item of a list behind "and", Russian behind "и",
/// Chinese behind "并". Asking the lexicon for those surfaces rather than
/// listing them keeps the enumeration reader working in every registered
/// language, and keeps "commands and options" -- where the word is inside a part
/// rather than between two -- from splitting, because a separator only counts
/// when it stands alone as a word.
fn split_on_separators(span: &str, separators: &[String]) -> Vec<String> {
    let mut parts = vec![String::new()];
    for word in span.split_whitespace() {
        if separators
            .iter()
            .any(|separator| word.eq_ignore_ascii_case(separator))
        {
            parts.push(String::new());
            continue;
        }
        let current = parts.last_mut().expect("parts is never empty");
        if !current.is_empty() {
            current.push(' ');
        }
        current.push_str(word);
    }
    parts
}

/// Render the note: what was asked for, what was observed, what is outstanding.
fn compose(specification: &Specification, messages: &[ChatMessage]) -> String {
    let observations = observations(messages);
    let mut note = format!("{}\n\nRequested parts:\n", specification.request);
    for part in &specification.parts {
        note.push_str("- ");
        note.push_str(part);
        note.push('\n');
    }
    note.push_str("\nObserved in this session:\n");
    if observations.is_empty() {
        note.push_str("- nothing: no tool result was recorded before this note.\n");
    } else {
        for observation in &observations {
            note.push_str("- ");
            note.push_str(observation);
            note.push('\n');
        }
    }
    if observations.is_empty() {
        note.push_str(
            "\nNo requested part above is backed by an observation from this session.\n",
        );
    }
    note
}

/// One line per tool result in the current turn: what ran, and what it returned.
///
/// The note reports observations, not conclusions, so each line stays close to
/// the result it came from -- the tool's own name and the first line of its
/// payload. A note that summarized further would be asserting more than the
/// session established.
fn observations(messages: &[ChatMessage]) -> Vec<String> {
    let turn = messages
        .iter()
        .rposition(|message| message.role.eq_ignore_ascii_case("user"))
        .map_or(0, |index| index + 1);
    messages
        .iter()
        .skip(turn)
        .filter(|message| message.role.eq_ignore_ascii_case("tool"))
        .map(|message| {
            let name = message.name.as_deref().unwrap_or("tool");
            let raw = message.content.plain_text();
            let head = raw.lines().find(|line| !line.trim().is_empty()).unwrap_or("");
            format!("{name}: {}", head.trim())
        })
        .collect()
}
