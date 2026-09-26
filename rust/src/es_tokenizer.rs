//! One general ECMAScript lexer and token tree (issue #1138, plan 16 L2a).
//!
//! The tokenizer is engine code under the engine/data split: a single lexer
//! serves the `./js` and `./ts` roots, knows nothing about translation, and
//! normalizes one source text into a tree of tokens with byte spans. Which
//! token class may project into which target is a per-language seed rule
//! (plan 16 L2c), never a lexer decision, so the lexer only states what the
//! bytes say. Ambiguities that ES itself resolves by parser context — a `/`
//! starting a regex or being division, a `{` opening a block or an object
//! literal — are resolved by the standard previous-significant-token
//! heuristic, and every construct the heuristic cannot close honestly is an
//! error with a byte span, not a guess.

extern crate alloc;

use alloc::vec::Vec;
use core::fmt;
use core::ops::Range;

/// Byte range of a token, group, or template inside the tokenized source.
pub type Span = Range<usize>;

/// The class of one leaf token. Keywords stay `Identifier`; classifying a
/// word is [`is_keyword`]'s job so the table, not the lexer loop, carries
/// the language's reserved vocabulary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenKind {
    /// An identifier, a keyword, `true`/`false`/`null`, or a private name
    /// (`#field`): everything the lexer cannot tell apart without context.
    Identifier,
    /// A numeric literal in any base, including the `BigInt` `n` suffix.
    Numeric,
    /// A single- or double-quoted string literal.
    String,
    /// A template-literal chunk between interpolations.
    TemplateChunk,
    /// A regular-expression literal with its flags.
    RegExp,
    /// An operator or delimiter; `text` is the matched punctuator.
    Punctuator,
}

/// One leaf: the slice of source it came from, its class, and its span.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Token<'src> {
    /// The token's text, always a substring of the tokenized source.
    pub text: &'src str,
    /// The token's class.
    pub kind: TokenKind,
    /// The token's byte range in the source.
    pub span: Span,
}

/// The delimiter of a balanced group.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Delimiter {
    /// `( ... )`
    Paren,
    /// `[ ... ]`
    Bracket,
    /// `{ ... }`
    Brace,
}

impl Delimiter {
    /// The opening byte of this delimiter's pair.
    #[must_use]
    pub const fn opens(self) -> u8 {
        match self {
            Self::Paren => b'(',
            Self::Bracket => b'[',
            Self::Brace => b'{',
        }
    }

    /// The closing byte of this delimiter's pair.
    #[must_use]
    pub const fn closes(self) -> u8 {
        match self {
            Self::Paren => b')',
            Self::Bracket => b']',
            Self::Brace => b'}',
        }
    }
}

/// One node of the normalized token tree.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Tree<'src> {
    /// A single token outside any interpolation.
    Leaf(Token<'src>),
    /// A balanced `(...)`, `[...]`, or `{...}` group with its subtree.
    Group {
        /// Which delimiter pair enclosed the subtree.
        delim: Delimiter,
        /// The group's byte range, delimiters included.
        span: Span,
        /// The tokens and nested groups inside the delimiters.
        trees: Vec<Self>,
    },
    /// A template literal: ordered chunks and interpolations.
    Template {
        /// The template's byte range, backticks included.
        span: Span,
        /// Chunks and interpolations in source order.
        parts: Vec<TemplatePart<'src>>,
    },
}

impl Tree<'_> {
    /// The node's byte range in the source.
    #[must_use]
    pub fn span(&self) -> Span {
        match self {
            Self::Leaf(token) => token.span.clone(),
            Self::Group { span, .. } | Self::Template { span, .. } => span.clone(),
        }
    }
}

/// One part of a template literal.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TemplatePart<'src> {
    /// Literal text between interpolations; escapes are kept verbatim.
    Chunk(Token<'src>),
    /// A `${ ... }` interpolation with its own token subtree.
    Interpolation {
        /// The interpolation's byte range, `${` through `}` included.
        span: Span,
        /// The tokens and nested groups inside the interpolation.
        trees: Vec<Tree<'src>>,
    },
}

/// Why a source could not be normalized into a token tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenizeError {
    /// The failure class.
    pub kind: TokenizeErrorKind,
    /// Where the failure was detected (unterminated constructs point at
    /// their opening byte; closers point at themselves).
    pub span: Span,
}

/// The failure classes the lexer reports instead of guessing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenizeErrorKind {
    /// A quote was never closed before the line or the input ended.
    UnterminatedString,
    /// A backtick was never closed.
    UnterminatedTemplate,
    /// A `/` that read as a regex start was never closed.
    UnterminatedRegExp,
    /// A block comment was never closed.
    UnterminatedComment,
    /// The input ended with this group still open; the span holds the
    /// opener's byte.
    UnclosedGroup(Delimiter),
    /// A closing delimiter arrived with no matching opener, or against the
    /// wrong opener.
    UnexpectedClosing(Delimiter),
    /// A byte that starts no token this lexer knows.
    UnexpectedChar(char),
}

impl fmt::Display for TokenizeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let what = match self.kind {
            TokenizeErrorKind::UnterminatedString => "unterminated string literal",
            TokenizeErrorKind::UnterminatedTemplate => "unterminated template literal",
            TokenizeErrorKind::UnterminatedRegExp => "unterminated regular expression",
            TokenizeErrorKind::UnterminatedComment => "unterminated block comment",
            TokenizeErrorKind::UnclosedGroup(_) => "unclosed group",
            TokenizeErrorKind::UnexpectedClosing(_) => "unexpected closing delimiter",
            TokenizeErrorKind::UnexpectedChar(_) => "unexpected character",
        };
        write!(f, "{what} at byte {}", self.span.start)
    }
}

/// ES keywords and reserved literals, one table for both classification and
/// the regex-context decision, so a new word is added in exactly one place.
const KEYWORDS: [&str; 38] = [
    "await",
    "break",
    "case",
    "catch",
    "class",
    "const",
    "continue",
    "debugger",
    "default",
    "delete",
    "do",
    "else",
    "enum",
    "export",
    "extends",
    "false",
    "finally",
    "for",
    "function",
    "if",
    "import",
    "in",
    "instanceof",
    "let",
    "new",
    "null",
    "return",
    "static",
    "super",
    "switch",
    "this",
    "throw",
    "true",
    "try",
    "typeof",
    "var",
    "void",
    "while",
];

/// Words after which a `/` starts a regex rather than a division: the
/// expression-position keywords. Every other identifier (including
/// `true`/`false`/`null`/`this`/`super`, which are values) is followed by
/// division.
const REGEX_CONTEXT_KEYWORDS: [&str; 14] = [
    "await",
    "case",
    "delete",
    "do",
    "else",
    "in",
    "instanceof",
    "new",
    "of",
    "return",
    "throw",
    "typeof",
    "void",
    "yield",
];

/// Punctuators this lexer matches, longest first so the match is greedy in
/// bytes and `>>>=` can never fall apart into `>` comparisons.
const PUNCTUATORS: [&str; 59] = [
    ">>>=", "...", "===", "!==", "**=", "<<=", ">>=", ">>>", "&&=", "||=", "??=", "=>", "==", "!=",
    "<=", ">=", "&&", "||", "??", "?.", "++", "--", "+=", "-=", "*=", "/=", "%=", "&=", "|=", "^=",
    "<<", ">>", "**", "{", "}", "(", ")", "[", "]", ";", ",", "<", ">", "+", "-", "*", "/", "%",
    "&", "|", "^", "!", "~", "?", ":", "=", ".", "@", "#",
];

/// Whether a word is an ES keyword or reserved literal.
#[must_use]
pub fn is_keyword(word: &str) -> bool {
    KEYWORDS.contains(&word)
}

/// Whether a `/` after this identifier starts a regex. Only the
/// expression-position keywords allow it; every other identifier is a value
/// a division can follow.
#[must_use]
pub fn regex_may_follow(word: &str) -> bool {
    REGEX_CONTEXT_KEYWORDS.contains(&word)
}

/// A stack frame while building the tree.
enum Frame<'src> {
    Group {
        delim: Delimiter,
        start: usize,
        trees: Vec<Tree<'src>>,
    },
    Interp {
        start: usize,
        trees: Vec<Tree<'src>>,
    },
    Template {
        start: usize,
        chunk_start: usize,
        parts: Vec<TemplatePart<'src>>,
    },
}

impl Frame<'_> {
    const fn start(&self) -> usize {
        match self {
            Self::Group { start, .. }
            | Self::Interp { start, .. }
            | Self::Template { start, .. } => *start,
        }
    }
}

/// What a closing delimiter does to the innermost open frame.
enum Close {
    /// It closes a matching group.
    Group,
    /// It closes a template interpolation (only `}` can).
    Interp,
    /// No open frame matches it.
    Unexpected,
}

/// Tokenize one ECMAScript source into its normalized token tree.
///
/// Comments and whitespace are dropped (they are not part of the normalized
/// tree the cycle compares); everything else survives as leaves, groups, or
/// templates with byte spans into `source`.
///
/// # Errors
///
/// Returns the first unterminated construct, mismatched delimiter, or
/// unlexable byte, with its span, instead of guessing a reading.
pub fn tokenize(source: &str) -> Result<Vec<Tree<'_>>, TokenizeError> {
    let bytes = source.as_bytes();
    let mut pos = 0;
    let mut root: Vec<Tree> = Vec::new();
    let mut stack: Vec<Frame> = Vec::new();
    // Whether a `/` at the current position starts a regex: true at the
    // start of the input and after every token except values and `)`/`]`.
    let mut regex_allowed = true;

    if bytes.first() == Some(&b'#') && bytes.get(1) == Some(&b'!') {
        pos = skip_line(bytes, 2);
    }

    loop {
        if matches!(stack.last(), Some(Frame::Template { .. })) {
            // Inside a template: read one chunk until its backtick or `${`.
            let template_start = stack.last().map_or(pos, Frame::start);
            let (chunk_end, next) = scan_template_chunk(bytes, pos, template_start)?;
            let Some(frame @ Frame::Template { .. }) = stack.last_mut() else {
                unreachable!("chunk mode is entered only under a template frame")
            };
            if let Frame::Template {
                chunk_start, parts, ..
            } = frame
                && chunk_end > *chunk_start
            {
                parts.push(TemplatePart::Chunk(Token {
                    text: &source[*chunk_start..chunk_end],
                    kind: TokenKind::TemplateChunk,
                    span: *chunk_start..chunk_end,
                }));
            }
            // `chunk_end` is where the chunk terminated (`${` or the closing
            // backtick); advance to it before consuming the terminator.
            pos = chunk_end;
            match next {
                ChunkEnd::Backtick => {
                    if let Some(Frame::Template { start, parts, .. }) = stack.pop() {
                        root_or_parent(&mut stack, &mut root).push(Tree::Template {
                            span: start..pos + 1,
                            parts,
                        });
                    }
                    regex_allowed = false;
                    pos += 1;
                }
                ChunkEnd::Dollar => {
                    stack.push(Frame::Interp {
                        start: pos,
                        trees: Vec::new(),
                    });
                    pos += 2;
                }
            }
            continue;
        }

        while pos < bytes.len() && bytes[pos].is_ascii_whitespace() {
            pos += 1;
        }
        if pos >= bytes.len() {
            break;
        }
        let byte = bytes[pos];

        match byte {
            b'/' if bytes.get(pos + 1) == Some(&b'/') => pos = skip_line(bytes, pos + 2),
            b'/' if bytes.get(pos + 1) == Some(&b'*') => {
                let Some(end) = find(bytes, pos + 2, b"*/") else {
                    return Err(TokenizeError {
                        kind: TokenizeErrorKind::UnterminatedComment,
                        span: pos..pos + 2,
                    });
                };
                pos = end + 2;
            }
            b'/' if regex_allowed => {
                let Some(body_end) = scan_regex(bytes, pos) else {
                    return Err(TokenizeError {
                        kind: TokenizeErrorKind::UnterminatedRegExp,
                        span: pos..pos + 1,
                    });
                };
                let mut flag_end = body_end + 1;
                while flag_end < bytes.len() && bytes[flag_end].is_ascii_alphabetic() {
                    flag_end += 1;
                }
                emit(
                    &mut stack,
                    &mut root,
                    Token {
                        text: &source[pos..flag_end],
                        kind: TokenKind::RegExp,
                        span: pos..flag_end,
                    },
                );
                regex_allowed = false;
                pos = flag_end;
            }
            b'\'' | b'"' => {
                let (end, closed) = scan_string(bytes, pos, byte);
                if !closed {
                    return Err(TokenizeError {
                        kind: TokenizeErrorKind::UnterminatedString,
                        span: pos..bytes.len().min(pos + 1),
                    });
                }
                emit(
                    &mut stack,
                    &mut root,
                    Token {
                        text: &source[pos..end],
                        kind: TokenKind::String,
                        span: pos..end,
                    },
                );
                regex_allowed = false;
                pos = end;
            }
            b'`' => {
                stack.push(Frame::Template {
                    start: pos,
                    chunk_start: pos + 1,
                    parts: Vec::new(),
                });
                pos += 1;
            }
            b'(' | b'[' | b'{' => {
                let delim = match byte {
                    b'(' => Delimiter::Paren,
                    b'[' => Delimiter::Bracket,
                    _ => Delimiter::Brace,
                };
                stack.push(Frame::Group {
                    delim,
                    start: pos,
                    trees: Vec::new(),
                });
                regex_allowed = true;
                pos += 1;
            }
            b')' | b']' | b'}' => {
                let found = match byte {
                    b')' => Delimiter::Paren,
                    b']' => Delimiter::Bracket,
                    _ => Delimiter::Brace,
                };
                // Decide before mutating the stack so the match below owns
                // its data: a closer either matches the innermost group, the
                // innermost interpolation (only `}` can), or nothing.
                let close = match stack.last() {
                    Some(Frame::Group { delim, .. }) if *delim == found => Close::Group,
                    Some(Frame::Interp { .. }) if found == Delimiter::Brace => Close::Interp,
                    _ => Close::Unexpected,
                };
                match close {
                    Close::Group => {
                        if let Some(Frame::Group { start, trees, .. }) = stack.pop() {
                            root_or_parent(&mut stack, &mut root).push(Tree::Group {
                                delim: found,
                                span: start..pos + 1,
                                trees,
                            });
                        }
                        // `}` reads as a block close: a regex may follow.
                        regex_allowed = found == Delimiter::Brace;
                    }
                    Close::Interp => {
                        if let Some(Frame::Interp { start, trees }) = stack.pop()
                            && let Some(Frame::Template {
                                chunk_start, parts, ..
                            }) = stack.last_mut()
                        {
                            parts.push(TemplatePart::Interpolation {
                                span: start..pos + 1,
                                trees,
                            });
                            *chunk_start = pos + 1;
                        }
                    }
                    Close::Unexpected => {
                        return Err(TokenizeError {
                            kind: TokenizeErrorKind::UnexpectedClosing(found),
                            span: pos..pos + 1,
                        });
                    }
                }
                pos += 1;
            }
            _ if byte.is_ascii_digit()
                || (byte == b'.' && bytes.get(pos + 1).is_some_and(u8::is_ascii_digit)) =>
            {
                let end = scan_number(bytes, pos);
                emit(
                    &mut stack,
                    &mut root,
                    Token {
                        text: &source[pos..end],
                        kind: TokenKind::Numeric,
                        span: pos..end,
                    },
                );
                regex_allowed = false;
                pos = end;
            }
            _ if is_ident_byte(byte)
                || (byte == b'#'
                    && bytes.get(pos + 1).is_some_and(|next| is_ident_byte(*next))) =>
            {
                let mut end = pos + if byte == b'#' { 2 } else { 1 };
                while end < bytes.len() && is_ident_byte(bytes[end]) {
                    end += 1;
                }
                let text = &source[pos..end];
                emit(
                    &mut stack,
                    &mut root,
                    Token {
                        text,
                        kind: TokenKind::Identifier,
                        span: pos..end,
                    },
                );
                regex_allowed = REGEX_CONTEXT_KEYWORDS.contains(&text);
                pos = end;
            }
            _ => {
                let matched = PUNCTUATORS
                    .iter()
                    .copied()
                    .find(|punct| source[pos..].starts_with(punct));
                // `?.` followed by a digit is a ternary over a decimal, not
                // optional chaining (ES resolves the same way).
                let matched = match matched {
                    Some("?.") if bytes.get(pos + 2).is_some_and(u8::is_ascii_digit) => Some("?"),
                    other => other,
                };
                let Some(punct) = matched else {
                    let ch = source[pos..].chars().next().unwrap_or('\u{fffd}');
                    return Err(TokenizeError {
                        kind: TokenizeErrorKind::UnexpectedChar(ch),
                        span: pos..pos + ch.len_utf8(),
                    });
                };
                let end = pos + punct.len();
                emit(
                    &mut stack,
                    &mut root,
                    Token {
                        text: &source[pos..end],
                        kind: TokenKind::Punctuator,
                        span: pos..end,
                    },
                );
                regex_allowed = punct != ")" && punct != "]";
                pos = end;
            }
        }
    }

    if let Some(frame) = stack.last() {
        // An unterminated template is the root cause of anything left open
        // inside it (its interpolations can never close), so the innermost
        // template frame outranks the innermost group at end of input.
        if let Some(start) = stack.iter().rev().find_map(|frame| match frame {
            Frame::Template { start, .. } => Some(*start),
            _ => None,
        }) {
            return Err(TokenizeError {
                kind: TokenizeErrorKind::UnterminatedTemplate,
                span: start..start + 1,
            });
        }
        let start = frame.start();
        return Err(TokenizeError {
            kind: match frame {
                Frame::Group { delim, .. } => TokenizeErrorKind::UnclosedGroup(*delim),
                Frame::Interp { .. } => TokenizeErrorKind::UnclosedGroup(Delimiter::Brace),
                Frame::Template { .. } => TokenizeErrorKind::UnterminatedTemplate,
            },
            span: start..start + 1,
        });
    }
    Ok(root)
}

enum ChunkEnd {
    Backtick,
    Dollar,
}

fn scan_template_chunk(
    bytes: &[u8],
    start: usize,
    template_start: usize,
) -> Result<(usize, ChunkEnd), TokenizeError> {
    let mut pos = start;
    while pos < bytes.len() {
        match bytes[pos] {
            b'\\' => pos += 2,
            b'`' => return Ok((pos, ChunkEnd::Backtick)),
            b'$' if bytes.get(pos + 1) == Some(&b'{') => return Ok((pos, ChunkEnd::Dollar)),
            _ => pos += 1,
        }
    }
    Err(TokenizeError {
        kind: TokenizeErrorKind::UnterminatedTemplate,
        span: template_start..template_start + 1,
    })
}

/// Scan a regex body from the opening `/`. Returns the byte of the closing
/// `/`, or `None` when the input ends or a line break appears first.
const fn scan_regex(bytes: &[u8], start: usize) -> Option<usize> {
    let mut pos = start + 1;
    let mut in_class = false;
    while pos < bytes.len() {
        match bytes[pos] {
            b'\\' => pos += 2,
            b'[' => {
                in_class = true;
                pos += 1;
            }
            b']' => {
                in_class = false;
                pos += 1;
            }
            b'/' if !in_class => return Some(pos),
            b'\n' | b'\r' => return None,
            _ => pos += 1,
        }
    }
    None
}

/// Scan a quoted string from its opening quote. Returns the byte after the
/// closing quote, and whether the string closed before the line or input
/// ended.
const fn scan_string(bytes: &[u8], start: usize, quote: u8) -> (usize, bool) {
    let mut pos = start + 1;
    while pos < bytes.len() {
        match bytes[pos] {
            b'\\' => pos += 2,
            b if b == quote => return (pos + 1, true),
            b'\n' | b'\r' => return (pos, false),
            _ => pos += 1,
        }
    }
    (bytes.len(), false)
}

fn scan_number(bytes: &[u8], start: usize) -> usize {
    let mut pos = start;
    // The caller admits a leading `.` before a digit (the `a?.3:b` ternary
    // split); consume it so the scan always advances past the start.
    if bytes[pos] == b'.' {
        pos += 1;
    }
    // A leading `0x`/`0o`/`0b` (either case) switches the digit vocabulary;
    // consuming every alphanumeric byte and `_` after the prefix covers hex
    // digits, binary digits, decimal exponents, separators, and BigInt `n`.
    let radix_prefixed = bytes[start] == b'0'
        && matches!(
            bytes.get(start + 1).copied(),
            Some(b'x' | b'X' | b'o' | b'O' | b'b' | b'B')
        );
    if radix_prefixed {
        pos += 2;
    }
    let mut last = b'\0';
    while pos < bytes.len() {
        let byte = bytes[pos];
        if byte.is_ascii_alphanumeric() || byte == b'_' {
            last = byte;
            pos += 1;
            continue;
        }
        // A decimal exponent may carry one sign after the `e`/`E`.
        if !radix_prefixed && last.eq_ignore_ascii_case(&b'e') && (byte == b'+' || byte == b'-') {
            last = byte;
            pos += 1;
            continue;
        }
        break;
    }
    pos
}

const fn skip_line(bytes: &[u8], start: usize) -> usize {
    let mut pos = start;
    while pos < bytes.len() && bytes[pos] != b'\n' {
        pos += 1;
    }
    if pos < bytes.len() { pos + 1 } else { pos }
}

fn find(bytes: &[u8], start: usize, needle: &[u8]) -> Option<usize> {
    (start..bytes.len().saturating_sub(needle.len() - 1))
        .find(|offset| &bytes[*offset..offset + needle.len()] == needle)
}

const fn is_ident_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'$' || byte >= 0x80
}

/// Where the next node lands: the innermost open frame's tree list, or the
/// root when every frame has closed.
fn root_or_parent<'src, 'a>(
    stack: &'a mut [Frame<'src>],
    root: &'a mut Vec<Tree<'src>>,
) -> &'a mut Vec<Tree<'src>> {
    match stack.last_mut() {
        Some(Frame::Group { trees, .. } | Frame::Interp { trees, .. }) => trees,
        Some(Frame::Template { .. }) => unreachable!("templates consume their own chunks"),
        None => root,
    }
}

fn emit<'src>(stack: &mut [Frame<'src>], root: &mut Vec<Tree<'src>>, token: Token<'src>) {
    root_or_parent(stack, root).push(Tree::Leaf(token));
}
