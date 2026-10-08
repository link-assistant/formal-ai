//! Bounded execution for normal (Markov) string-rewrite algorithms.
//!
//! A program is an ordered list of substitutions. On every step the first rule
//! whose pattern occurs is selected, and its leftmost occurrence is replaced.
//! Evaluation restarts at rule zero after each non-terminal substitution. This
//! is the standard control model of a normal algorithm; terminal rules stop the
//! run immediately. Empty patterns and replacements are ordinary data, so the
//! same representation supports creation and deletion.
//!
//! Normal algorithms are computationally universal as an abstract model. This
//! executor intentionally adds a caller-selected step bound: universality is a
//! property of the representation, while a network-facing agent must not run an
//! untrusted non-terminating rewrite forever.

/// One ordered substitution in a normal algorithm.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RewriteRule {
    /// The sequence to find. An empty pattern matches byte offset zero.
    pub pattern: String,
    /// The sequence that replaces the matched pattern. It may be empty.
    pub replacement: String,
    /// Whether applying this rule halts the program immediately.
    pub terminal: bool,
}

impl RewriteRule {
    /// Construct a non-terminal substitution.
    #[must_use]
    pub fn new(pattern: impl Into<String>, replacement: impl Into<String>) -> Self {
        Self {
            pattern: pattern.into(),
            replacement: replacement.into(),
            terminal: false,
        }
    }

    /// Mark this substitution as terminal.
    #[must_use]
    pub const fn terminal(mut self) -> Self {
        self.terminal = true;
        self
    }
}

/// An ordered normal algorithm with a resource bound.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RewriteProgram {
    /// Rules in selection-priority order.
    pub rules: Vec<RewriteRule>,
    /// Maximum substitutions allowed in one execution.
    pub max_steps: usize,
}

impl RewriteProgram {
    /// Construct a program. A zero bound is valid and immediately yields
    /// [`RewriteHalt::StepLimit`].
    #[must_use]
    pub const fn new(rules: Vec<RewriteRule>, max_steps: usize) -> Self {
        Self { rules, max_steps }
    }

    /// Execute against `input` without mutating the caller's bytes.
    #[must_use]
    pub fn execute(&self, input: &str) -> RewriteOutcome {
        let mut output = input.to_owned();
        let mut trace = Vec::new();

        for _ in 0..self.max_steps {
            let Some((rule_index, byte_offset)) = self
                .rules
                .iter()
                .enumerate()
                .find_map(|(index, rule)| output.find(&rule.pattern).map(|at| (index, at)))
            else {
                return RewriteOutcome {
                    output,
                    trace,
                    halt: RewriteHalt::NoApplicableRule,
                };
            };
            let rule = &self.rules[rule_index];
            let end = byte_offset + rule.pattern.len();
            output.replace_range(byte_offset..end, &rule.replacement);
            trace.push(RewriteStep {
                rule_index,
                byte_offset,
            });
            if rule.terminal {
                return RewriteOutcome {
                    output,
                    trace,
                    halt: RewriteHalt::TerminalRule(rule_index),
                };
            }
        }

        RewriteOutcome {
            output,
            trace,
            halt: RewriteHalt::StepLimit,
        }
    }
}

/// Why an execution stopped.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RewriteHalt {
    /// No rule matched the current sequence.
    NoApplicableRule,
    /// The indexed terminal rule was applied.
    TerminalRule(usize),
    /// The caller's substitution bound was exhausted.
    StepLimit,
}

/// One observable substitution in an execution trace.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RewriteStep {
    /// Selected rule index.
    pub rule_index: usize,
    /// Byte offset of the leftmost match.
    pub byte_offset: usize,
}

/// Immutable result and audit trace for one execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RewriteOutcome {
    /// Sequence after the final permitted substitution.
    pub output: String,
    /// Rule and match position for every applied substitution.
    pub trace: Vec<RewriteStep>,
    /// Termination reason.
    pub halt: RewriteHalt,
}

/// One literal slot, with the byte span of the delimiters that produced it.
///
/// `start` is the offset of the opening delimiter and `end` is the offset just
/// past the closing one, so `text[start..end]` is the slot *including* its
/// quotes while [`Self::text`] is the content between them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuotedSegment {
    /// The content between the delimiters.
    pub text: String,
    /// Byte offset of the opening delimiter.
    pub start: usize,
    /// Byte offset just past the closing delimiter.
    pub end: usize,
}

/// Extract structurally delimited literal slots, including zero-length slots.
///
/// The surrounding prose is deliberately irrelevant. Callers can vary natural
/// language freely while the literal old/new values remain explicit. ASCII,
/// typographic, guillemet, and CJK quote pairs plus Markdown backticks are
/// accepted. A fenced triple-backtick block is treated as one slot.
#[must_use]
pub fn quoted_segments(text: &str) -> Vec<String> {
    quoted_segment_spans(text)
        .into_iter()
        .map(|segment| segment.text)
        .collect()
}

/// [`quoted_segments`], keeping each slot's byte span.
///
/// This is the single implementation behind every literal-slot reader. Callers
/// that only need the contents use [`quoted_segments`]; callers that must know
/// where a slot sat in the prose — to tell "replace X with Y in Z" from
/// "in Z replace X with Y" — use this.
#[must_use]
pub fn quoted_segment_spans(text: &str) -> Vec<QuotedSegment> {
    let mut result = Vec::new();
    let mut cursor = 0;
    while cursor < text.len() {
        let Some((open_at, open, close)) = next_delimiter(text, cursor) else {
            break;
        };
        let content_start = open_at + open.len();
        let Some(content_end) = closing_delimiter(text, content_start, close) else {
            break;
        };
        let segment_end = content_end + close.len();
        result.push(QuotedSegment {
            text: text[content_start..content_end].to_owned(),
            start: open_at,
            end: segment_end,
        });
        cursor = segment_end;
    }
    result
}

/// Where a request's quotes stop pairing (PR #1188 G71).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuoteFault {
    /// `escaped` for a backslash before a quote, `unpaired` for an opening
    /// quote that no literal holds.
    pub kind: &'static str,
    /// Byte offset of the backslash or of the lone opening quote.
    pub at: usize,
    /// Up to 32 characters of the request from `at`, trailing space trimmed.
    pub fragment: String,
}

const ASCII_QUOTES: [char; 3] = ['\'', '"', '`'];
const OPEN_ONLY: [char; 6] = ['«', '“', '‘', '「', '『', '《'];
const CLOSE_FOLLOWERS: &str = ".,;:!?)]}";
const FAULT_FRAGMENT_CHARS: usize = 32;

/// The first place a request's quotes stop pairing, or `None` when they pair.
///
/// `escaped` is a backslash before a quote that opens a literal, or before
/// one that closes it with a word straight after; `unpaired` is an opening
/// quote no literal holds. An apostrophe (a quote after a letter or digit)
/// never opens a literal.
#[must_use]
pub fn quote_fault(text: &str) -> Option<QuoteFault> {
    let mut faults = Vec::new();
    let mut gap_start = 0;
    for segment in quoted_segment_spans(text) {
        gap_faults(text, gap_start, segment.start, &mut faults);
        gap_start = segment.end;
        let opening = &text[segment.start..];
        if opening.starts_with(PAIRS[0].0)
            || !opening
                .chars()
                .next()
                .is_some_and(|open| ASCII_QUOTES.contains(&open))
        {
            continue;
        }
        if text[..segment.start].ends_with('\\') {
            faults.push(("escaped", segment.start - 1));
        }
        let close_at = segment.end - 1;
        if segment.end - segment.start > 2
            && text[..close_at].ends_with('\\')
            && text[segment.end..]
                .chars()
                .next()
                .is_some_and(|after| !after.is_whitespace() && !CLOSE_FOLLOWERS.contains(after))
        {
            faults.push(("escaped", close_at - 1));
        }
    }
    gap_faults(text, gap_start, text.len(), &mut faults);
    let (kind, at) = faults.into_iter().min_by_key(|&(_, at)| at)?;
    let fragment = text[at..]
        .chars()
        .take(FAULT_FRAGMENT_CHARS)
        .collect::<String>()
        .trim_end()
        .to_owned();
    Some(QuoteFault { kind, at, fragment })
}

/// The quote faults between two literals: an escaped quote, or an opening
/// quote that is not an apostrophe.
fn gap_faults(text: &str, from: usize, to: usize, faults: &mut Vec<(&'static str, usize)>) {
    for (offset, character) in text[from..to].char_indices() {
        let index = from + offset;
        let ascii = ASCII_QUOTES.contains(&character);
        if ascii && text[..index].ends_with('\\') {
            faults.push(("escaped", index - 1));
        } else if (ascii
            && !text[..index]
                .chars()
                .next_back()
                .is_some_and(|before| before.is_ascii_alphanumeric()))
            || OPEN_ONLY.contains(&character)
        {
            faults.push(("unpaired", index));
        }
    }
}

/// Remove one pair of client-added framing quotes without consuming literal
/// operands such as `'old' -> 'new'`.
#[must_use]
pub fn unwrap_transport_quotes(text: &str) -> &str {
    let trimmed = text.trim();
    for quote in ['"', '\''] {
        if let Some(inner) = trimmed
            .strip_prefix(quote)
            .and_then(|value| value.strip_suffix(quote))
            && !inner.contains(quote)
        {
            return inner;
        }
    }
    trimmed
}

/// The quote pairs a literal slot is delimited by, the fenced block first.
const PAIRS: [(&str, &str); 10] = [
    ("```", "```"),
    ("'", "'"),
    ("\"", "\""),
    ("`", "`"),
    ("«", "»"),
    ("“", "”"),
    ("‘", "’"),
    ("「", "」"),
    ("『", "』"),
    ("《", "》"),
];

/// Whether `text`, trimmed, opens with a quote and closes with its pair.
///
/// Such text is one literal however the quotes inside it pair (PR #1188 G63).
#[must_use]
pub fn wrapped_in_quote_pair(text: &str) -> bool {
    let trimmed = text.trim();
    PAIRS.iter().any(|(open, close)| {
        trimmed.len() >= open.len() + close.len()
            && trimmed.starts_with(open)
            && trimmed.ends_with(close)
    })
}

/// Whether `text` holds `literal` inside one quote pair.
///
/// The quotes inside `literal` may pair among themselves (PR #1188 G63).
#[must_use]
pub fn quotes_whole(text: &str, literal: &str) -> bool {
    !literal.is_empty()
        && PAIRS
            .iter()
            .any(|(open, close)| text.contains(&format!("{open}{literal}{close}")))
}

fn next_delimiter(text: &str, cursor: usize) -> Option<(usize, &'static str, &'static str)> {
    PAIRS
        .iter()
        .filter_map(|&(open, close)| next_complete_pair(text, cursor, open, close))
        .min_by_key(|(at, open, _)| (*at, usize::MAX - open.len()))
}

fn next_complete_pair(
    text: &str,
    cursor: usize,
    open: &'static str,
    close: &'static str,
) -> Option<(usize, &'static str, &'static str)> {
    let mut from = cursor;
    while let Some(relative) = text[from..].find(open) {
        let open_at = from + relative;
        let previous_is_ascii_word = open == "'"
            && text[..open_at]
                .chars()
                .next_back()
                .is_some_and(|character| character.is_ascii_alphanumeric());
        let content_start = open_at + open.len();
        if !previous_is_ascii_word && closing_delimiter(text, content_start, close).is_some() {
            return Some((open_at, open, close));
        }
        from = content_start;
    }
    None
}

/// The first close that is not an apostrophe inside a word, or, for a single
/// quote, the one that closes a payload quoting a single-quoted literal of its
/// own (PR #1188 G61): a quote with a space before it and a word character
/// after it opens a nested literal, and the payload closes after that literal
/// does. When such nesting never closes, the first close stands.
fn closing_delimiter(text: &str, cursor: usize, close: &str) -> Option<usize> {
    if close == "'"
        && let Some(close_at) = nested_closing_delimiter(text, cursor)
    {
        return Some(close_at);
    }
    plain_closing_delimiter(text, cursor, close)
}

/// The close of a single-quoted payload that holds nested single-quoted
/// literals, or `None` when the nesting never closes.
fn nested_closing_delimiter(text: &str, cursor: usize) -> Option<usize> {
    let mut depth = 0_usize;
    let mut from = cursor;
    while let Some(relative) = text[from..].find('\'') {
        let close_at = from + relative;
        from = close_at + 1;
        let before = text[..close_at].chars().next_back();
        let after_is_word = text[from..]
            .chars()
            .next()
            .is_some_and(|character| character.is_ascii_alphanumeric());
        if before.is_some_and(|character| character.is_ascii_alphanumeric()) && after_is_word {
            continue;
        }
        if before.is_some_and(char::is_whitespace) && after_is_word {
            depth += 1;
        } else if depth == 0 {
            return Some(close_at);
        } else {
            depth -= 1;
        }
    }
    None
}

fn plain_closing_delimiter(text: &str, cursor: usize, close: &str) -> Option<usize> {
    let mut from = cursor;
    while let Some(relative) = text[from..].find(close) {
        let close_at = from + relative;
        let after_close = close_at + close.len();
        let is_ascii_apostrophe = close == "'"
            && text[..close_at]
                .chars()
                .next_back()
                .is_some_and(|character| character.is_ascii_alphanumeric())
            && text[after_close..]
                .chars()
                .next()
                .is_some_and(|character| character.is_ascii_alphanumeric());
        if !is_ascii_apostrophe {
            return Some(close_at);
        }
        from = after_close;
    }
    None
}
