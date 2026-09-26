//! Balanced binary splitting (#453, R491-C1; plan 12 leaves 7-11).
//!
//! `data/meta/task-decomposition-invariant.lino` has declared since #847 that
//! every non-leaf task is split into exactly two children, and until this module
//! nothing read the field. The split is a **regrouping**, never a discard: the
//! two byte spans are contiguous and together cover the whole task (R710-R9),
//! and the imbalance actually achieved is reported rather than optimized away by
//! dropping a segment.
//!
//! The segmentation is structural, not lexical. Boundaries come from sentence
//! terminators, list punctuation and the seed's clause-continuation markers —
//! the same three sources `src/task_decomposition.rs` already segments on — and
//! the *judgement* of whether a segment states an obligation of its own is
//! read from the seed's own action roles, so the judgement is the same in every
//! supported language. What this module adds is that every
//! boundary is kept as a byte offset, which `split_once_checkable` cannot do
//! because it rejoins its segments into fresh strings.

use crate::coding::catalog::contains_cjk;
use crate::seed::{
    self, ROLE_CLAUSE_CONTINUATION_MARKER, ROLE_FOLLOWUP_INSTRUCTION_VERB,
    ROLE_OBSERVABLE_TASK_ACTION, ROLE_SOFTWARE_AUTHORING_ACTION,
};
use crate::web_engine_core::normalize_prompt;

use super::{BinarySplit, SplitRefusal};

/// One raw segment of the task, as a byte range into it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Span {
    start: usize,
    end: usize,
}

impl Span {
    const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    fn text<'a>(&self, task: &'a str) -> &'a str {
        &task[self.start..self.end]
    }
}

/// Segment `task` the way `task_decomposition::segment` does — sentences when
/// there is more than one, clauses otherwise — but keep every boundary as a byte
/// offset so the split can be proved to cover the input.
///
/// The flag says which level answered. A sentence is a complete predication by
/// construction, so sentences are never regrouped; a clause may be a relative or
/// a subordinate one, so clauses are.
fn raw_spans(task: &str) -> (Vec<Span>, bool) {
    let sentences = sentence_spans(task);
    if sentences.len() > 1 {
        return (sentences, true);
    }
    (clause_spans(task, Span::new(0, task.len())), false)
}

/// Split on sentence terminators, keeping the terminator inside its span. A
/// period flanked by non-whitespace (`3.14`, `release.yml`) is not a terminator.
fn sentence_spans(task: &str) -> Vec<Span> {
    let mut spans = Vec::new();
    let mut start = 0;
    let mut characters = task.char_indices().peekable();
    while let Some((index, character)) = characters.next() {
        let strong = matches!(character, '?' | '!' | '。' | '！' | '？' | '।' | '॥');
        let period = character == '.'
            && characters
                .peek()
                .is_none_or(|(_, next)| next.is_whitespace());
        if strong || period {
            let end = index + character.len_utf8();
            spans.push(Span::new(start, end));
            start = end;
        }
    }
    if start < task.len() {
        spans.push(Span::new(start, task.len()));
    }
    spans.retain(|span| !span.text(task).trim().is_empty());
    spans
}

/// Split one sentence on list punctuation and on the seed's clause-continuation
/// markers. The markers are named by role, never by spelling, so the same
/// boundary rule holds in every supported language.
fn clause_spans(task: &str, sentence: Span) -> Vec<Span> {
    let markers = seed::lexicon().words_for_role(ROLE_CLAUSE_CONTINUATION_MARKER);
    let mut boundaries = vec![sentence.start];
    let text = sentence.text(task);
    for (offset, character) in text.char_indices() {
        if matches!(character, ',' | ';' | '，' | '；' | '、') {
            boundaries.push(sentence.start + offset + character.len_utf8());
        }
    }
    for marker in &markers {
        boundaries.extend(marker_boundaries(text, marker, sentence.start));
    }
    boundaries.push(sentence.end);
    boundaries.sort_unstable();
    boundaries.dedup();

    let mut spans = Vec::new();
    for pair in boundaries.windows(2) {
        let span = Span::new(pair[0], pair[1]);
        if !span.text(task).trim().is_empty() {
            spans.push(span);
        }
    }
    spans
}

/// Byte offsets inside `text` where `marker` opens a new clause.
///
/// A marker written in a script with word boundaries must stand as its own word;
/// one written without them (`并`) is matched as a substring, exactly as
/// `task_decomposition::split_piece_on_marker` does.
fn marker_boundaries(text: &str, marker: &str, base: usize) -> Vec<usize> {
    let mut offsets = Vec::new();
    if marker.is_empty() {
        return offsets;
    }
    let substring_match = contains_cjk(marker);
    let bytes = text.as_bytes();
    let mut search = 0;
    while let Some(found) = text[search..].find(marker) {
        let at = search + found;
        search = at + marker.len();
        if substring_match {
            offsets.push(base + at);
            continue;
        }
        let before_is_boundary = at == 0
            || text[..at]
                .chars()
                .next_back()
                .is_some_and(char::is_whitespace);
        let after = at + marker.len();
        let after_is_boundary = after >= bytes.len()
            || text[after..]
                .chars()
                .next()
                .is_some_and(char::is_whitespace);
        if before_is_boundary && after_is_boundary && at > 0 && !joins_two_numbers(text, at, after)
        {
            offsets.push(base + at);
        }
    }
    offsets
}

/// Whether the marker occupying `start..end` sits between two numbers.
///
/// "between 860 and 864", "entre 860 y 864", "от 860 до 864" are one quantity,
/// not two obligations: a range names its bounds with the same word a request
/// uses to add a second demand, and cutting there invents a clause with no verb.
fn joins_two_numbers(text: &str, start: usize, end: usize) -> bool {
    let preceding = text[..start]
        .split_whitespace()
        .next_back()
        .is_some_and(|token| token.chars().any(|character| character.is_ascii_digit()));
    let following = text[end..]
        .split_whitespace()
        .next()
        .is_some_and(|token| token.chars().any(|character| character.is_ascii_digit()));
    preceding && following
}

/// The observable action the first segment opens with, when it opens with one.
///
/// Carried into later segments for the *checkability judgement only*: a bare
/// continuation ("write a benchmark for it") is judged with the head action in
/// front of it, exactly as `task_decomposition::distribute_head_action` does,
/// and the span is untouched.
fn head_action(segment: &str) -> Option<&str> {
    if contains_cjk(segment) {
        return None;
    }
    let first = segment.split_whitespace().next()?;
    let normalized = normalize_prompt(first);
    seed::lexicon()
        .mentions_role(ROLE_OBSERVABLE_TASK_ACTION, &normalized)
        .then_some(first)
}

/// The roles whose surfaces make a clause an obligation of its own: an action a
/// reader can observe the completion of, an authoring action, or a follow-up
/// instruction verb. A clause evidencing none of them is a relative or
/// subordinate one — "which learns by itself", "how the benchmark is run" — and
/// belongs to the obligation beside it rather than standing as a sibling.
const OBLIGATION_ROLES: [&str; 3] = [
    ROLE_OBSERVABLE_TASK_ACTION,
    ROLE_SOFTWARE_AUTHORING_ACTION,
    ROLE_FOLLOWUP_INSTRUCTION_VERB,
];

/// Fold every raw span that is not independently checkable into a neighbour,
/// keeping the byte ranges contiguous. A leading fragment belongs to the span
/// that follows it; a trailing fragment to the one before it. Nothing is
/// dropped, so the checkable segments still cover the whole task.
fn checkable_spans(task: &str) -> Vec<Span> {
    let (raw, sentence_level) = raw_spans(task);
    if raw.len() < 2 {
        return Vec::new();
    }
    if sentence_level {
        // A sentence is already a complete predication; regrouping sentences
        // would second-guess the punctuation the request was written with.
        return raw;
    }
    let head = head_action(raw[0].text(task)).map(str::to_owned);
    let mut out: Vec<Span> = Vec::new();
    let mut pending: Option<Span> = None;
    for span in raw {
        let merged = pending.map_or(span, |open| Span::new(open.start, span.end));
        if carries_an_obligation(task, span, head.as_deref()) {
            out.push(merged);
            pending = None;
        } else {
            pending = Some(merged);
        }
    }
    if let Some(tail) = pending {
        match out.last_mut() {
            Some(last) => last.end = tail.end,
            None => out.push(tail),
        }
    }
    if out.len() < 2 {
        // Merging is a *refinement* of the raw segmentation, not a veto over it.
        // A clause that evidences no action role is still a clause, so when the
        // refinement would erase the split entirely the raw boundaries stand --
        // otherwise a prompt whose verbs are not yet in the seed would silently
        // stop being decomposable at all.
        return raw_spans(task).0;
    }
    out
}

/// Whether one raw span states an obligation of its own, judged with the head
/// action carried in front of it when it carries none.
fn carries_an_obligation(task: &str, span: Span, head: Option<&str>) -> bool {
    let text = span.text(task).trim();
    if evidences_an_obligation(text) {
        return true;
    }
    match head {
        Some(head) if !contains_cjk(text) => evidences_an_obligation(&with_head(head, text)),
        _ => false,
    }
}

/// `head`, a space, then `text`. Built rather than formatted so no prose
/// template literal lives in `src/` (R379).
fn with_head(head: &str, text: &str) -> String {
    let mut joined = String::with_capacity(head.len() + 1 + text.len());
    joined.push_str(head);
    joined.push(' ');
    joined.push_str(text);
    joined
}

/// Whether `text` evidences any of [`OBLIGATION_ROLES`].
fn evidences_an_obligation(text: &str) -> bool {
    let normalized = normalize_prompt(text);
    let lexicon = seed::lexicon();
    OBLIGATION_ROLES.iter().any(|role| {
        lexicon.mentions_role(role, &normalized) || lexicon.mentions_role_raw(role, &normalized)
    })
}

/// Fold the n-ary segmentation into the balanced binary tree the invariant
/// declares, by cutting at the boundary that minimizes the imbalance.
///
/// The left child takes the larger half, so three segments become 2/1 and seven
/// become 4/3 — the shape #491 asks for, with the imbalance reported.
pub(super) fn balanced_split(task: &str) -> Option<BinarySplit> {
    let spans = checkable_spans(task);
    if spans.len() < 2 {
        return None;
    }
    let left_weight = spans.len().div_ceil(2);
    let right_weight = spans.len() - left_weight;
    let cut = spans[left_weight].start;
    let left_span = (0, cut);
    let right_span = (cut, task.len());
    Some(BinarySplit {
        left: task[left_span.0..left_span.1].trim().to_owned(),
        right: task[right_span.0..right_span.1].trim().to_owned(),
        left_weight,
        right_weight,
        left_span,
        right_span,
    })
}

/// Why a split was refused, so `None` is never read as "the task is easy".
pub(super) fn split_refusal(task: &str) -> Option<SplitRefusal> {
    if balanced_split(task).is_some() {
        return None;
    }
    if task.trim().is_empty() {
        return Some(SplitRefusal::SingleClause {
            reason: String::from("empty_task"),
        });
    }
    // One clause is not the same judgement as "atomic". The decomposition
    // approaches published for a goal are a lookup (plan 01), and until that
    // lookup lands the honest report names it as the blocker.
    Some(SplitRefusal::Underivable {
        blocker: String::from("concept_lookup:decomposition_approaches"),
    })
}

/// How many obligation segments `task` carries.
///
/// Zero means the task states one obligation or none, and therefore cannot be
/// split at the text level at all.
pub(super) fn segment_count(task: &str) -> usize {
    checkable_spans(task).len()
}

/// The number of layers a complete binary tree over `task`'s segments has.
///
/// `data/meta/task-decomposition-invariant.lino` declares that "1, 2, 4, 8, 16,
/// 32 and so on leaves are valid complete layers", so the recursion descends to
/// the deepest layer that is *complete*: `floor(log2(segments))`. Seven segments
/// therefore become four leaves holding 2, 2, 2 and 1 segments rather than seven
/// leaves in an unfinished layer -- every segment still survives on exactly one
/// side, and the leaf that carries two of them says so through its span.
pub(super) fn complete_layers(task: &str) -> u8 {
    let segments = segment_count(task);
    if segments < 2 {
        return 0;
    }
    u8::try_from(usize::BITS - segments.leading_zeros() - 1).unwrap_or(u8::MAX)
}
