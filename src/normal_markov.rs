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

/// Extract structurally delimited literal slots, including zero-length slots.
///
/// The surrounding prose is deliberately irrelevant. Callers can vary natural
/// language freely while the literal old/new values remain explicit. ASCII,
/// typographic, guillemet, and CJK quote pairs plus Markdown backticks are
/// accepted. A fenced triple-backtick block is treated as one slot.
#[must_use]
pub fn quoted_segments(text: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut cursor = 0;
    while cursor < text.len() {
        let Some((open_at, open, close)) = next_delimiter(text, cursor) else {
            break;
        };
        let content_start = open_at + open.len();
        let Some(relative_end) = text[content_start..].find(close) else {
            break;
        };
        let content_end = content_start + relative_end;
        result.push(text[content_start..content_end].to_owned());
        cursor = content_end + close.len();
    }
    result
}

fn next_delimiter(text: &str, cursor: usize) -> Option<(usize, &'static str, &'static str)> {
    const PAIRS: [(&str, &str); 9] = [
        ("```", "```"),
        ("'", "'"),
        ("\"", "\""),
        ("`", "`"),
        ("«", "»"),
        ("“", "”"),
        ("‘", "’"),
        ("「", "」"),
        ("『", "』"),
    ];
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
        if !previous_is_ascii_word && text[content_start..].contains(close) {
            return Some((open_at, open, close));
        }
        from = content_start;
    }
    None
}
