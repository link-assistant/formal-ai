//! The js/ts ↔ meta pivot legs of plan 16 L2 (issue #1138).
//!
//! A token tree rendered as a lino document, that document parsed back, and
//! the projection renderers that turn a tree into js or ts source under the
//! rules the seed — not this file — declares.
//!
//! Engine/data split, per the founding doctrine: everything here is a
//! generic function of the [`crate::es_tokenizer`] tree and of
//! `data/seed/language-projection.lino`. Which token class carries where,
//! and which TypeScript-only construct signature is refused to js, lives in
//! that seed; a rule change re-points every renderer at once, and
//! [`projection_rules_from`] exists so a test can prove the behavior moves
//! with the data.
//!
//! The round-trip contract is token-tree equality with spans ignored: spans
//! name bytes of one text, and a rendered target is a different text. The
//! renderers emit canonical spacing — every leaf separated by one space,
//! group delimiters spaced, template chunks verbatim — and because the
//! tokenizer's regex-versus-division decision is a pure function of the
//! previous significant token, re-tokenizing canonically spaced output
//! repeats every decision the original made. Rendered source is not
//! promised to be runnable (automatic-semicolon insertion is not a
//! token-tree fact); it is promised to re-tokenize to the same tree.

extern crate alloc;

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt::Write as _;

use crate::es_tokenizer::{Delimiter, TemplatePart, TokenKind, TokenizeError, Tree, tokenize};
use crate::seed::LANGUAGE_PROJECTION_LINO;
use crate::seed::parser::{LinoNode, find_closing_quote, parse_lino, unescape_value};

/// The two ES dialects the pivot serves; TS is a syntactic superset of the
/// committed `./js` corpus, so one tokenizer serves both roots and only the
/// document label differs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SourceLanguage {
    /// The `./js` root.
    JavaScript,
    /// The `./ts` root.
    TypeScript,
}

impl SourceLanguage {
    /// The canonical document spelling.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::JavaScript => "javascript",
            Self::TypeScript => "typescript",
        }
    }

    /// Parse the document spelling.
    #[must_use]
    pub fn parse(name: &str) -> Option<Self> {
        match name {
            "javascript" => Some(Self::JavaScript),
            "typescript" => Some(Self::TypeScript),
            _ => None,
        }
    }
}

/// A token detached from the source it came from — the pivot's own value.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OwnedToken {
    /// The token's text.
    pub text: String,
    /// The token's class.
    pub kind: TokenKind,
}

/// The pivot's tree: the tokenizer's tree with every borrow owned, spans
/// dropped (they name one text's bytes; the pivot outlives it), and parts
/// reduced to what projection needs.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum OwnedTree {
    /// One token.
    Leaf(OwnedToken),
    /// A balanced group.
    Group {
        /// Which delimiter pair.
        delim: Delimiter,
        /// The subtree inside the delimiters.
        trees: Vec<Self>,
    },
    /// A template literal.
    Template {
        /// Chunks and interpolations in source order.
        parts: Vec<OwnedPart>,
    },
}

/// One part of a template in the pivot tree.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum OwnedPart {
    /// Literal text between interpolations; escapes kept verbatim.
    Chunk(String),
    /// A `${ ... }` interpolation with its subtree.
    Interpolation {
        /// The tokens and nested groups inside.
        trees: Vec<OwnedTree>,
    },
}

/// The pivot document: a token tree plus the header that names where it
/// came from and what produced it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PivotDocument {
    /// The display path of the source the tree was extracted from.
    pub target: String,
    /// Which ES dialect the source was.
    pub language: SourceLanguage,
    /// Leaves and chunks the tree carries.
    pub token_count: usize,
    /// The tree itself.
    pub trees: Vec<OwnedTree>,
}

/// Extract one source into a pivot document. Fails exactly when the
/// tokenizer refuses the source, passing its error through untouched.
///
/// # Errors
///
/// Returns the tokenizer's error for any source it cannot close honestly.
pub fn extract(
    display_path: &str,
    language: SourceLanguage,
    source: &str,
) -> Result<PivotDocument, TokenizeError> {
    let trees = tokenize(source)?;
    let owned = trees.iter().map(own_tree).collect::<Vec<_>>();
    let token_count = count_tokens(&owned);
    Ok(PivotDocument {
        target: display_path.to_string(),
        language,
        token_count,
        trees: owned,
    })
}

fn own_tree(tree: &Tree<'_>) -> OwnedTree {
    match tree {
        Tree::Leaf(token) => OwnedTree::Leaf(OwnedToken {
            text: token.text.to_string(),
            kind: token.kind,
        }),
        Tree::Group { delim, trees, .. } => OwnedTree::Group {
            delim: *delim,
            trees: trees.iter().map(own_tree).collect(),
        },
        Tree::Template { parts, .. } => OwnedTree::Template {
            parts: parts
                .iter()
                .map(|part| match part {
                    TemplatePart::Chunk(token) => OwnedPart::Chunk(token.text.to_string()),
                    TemplatePart::Interpolation { trees, .. } => OwnedPart::Interpolation {
                        trees: trees.iter().map(own_tree).collect(),
                    },
                })
                .collect(),
        },
    }
}

fn count_tokens(trees: &[OwnedTree]) -> usize {
    let mut total = 0;
    for tree in trees {
        match tree {
            OwnedTree::Leaf(_) => total += 1,
            OwnedTree::Group { trees, .. } => total += count_tokens(trees),
            OwnedTree::Template { parts } => {
                for part in parts {
                    match part {
                        OwnedPart::Chunk(_) => total += 1,
                        OwnedPart::Interpolation { trees } => total += count_tokens(trees),
                    }
                }
            }
        }
    }
    total
}

/// The seed class slug for a token class — the vocabulary of
/// `data/seed/language-projection.lino`.
const fn class_slug(kind: TokenKind) -> &'static str {
    match kind {
        TokenKind::Identifier => "identifier",
        TokenKind::Numeric => "numeric",
        TokenKind::String => "string",
        TokenKind::TemplateChunk => "template_chunk",
        TokenKind::RegExp => "regexp",
        TokenKind::Punctuator => "punctuator",
    }
}

const fn delimiter_slug(delim: Delimiter) -> &'static str {
    match delim {
        Delimiter::Paren => "paren",
        Delimiter::Bracket => "bracket",
        Delimiter::Brace => "brace",
    }
}

fn delimiter_from_slug(slug: &str) -> Option<Delimiter> {
    match slug {
        "paren" => Some(Delimiter::Paren),
        "bracket" => Some(Delimiter::Bracket),
        "brace" => Some(Delimiter::Brace),
        _ => None,
    }
}

/// Render the pivot document as lino. Deterministic, and exactly what
/// [`parse_document`] reads back: leaf values are double-quoted with the
/// backslash escapes the shared seed parser decodes.
#[must_use]
pub fn render_document(document: &PivotDocument) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "token_tree");
    let _ = writeln!(out, "  target {}", quoted(&document.target));
    let _ = writeln!(out, "  language {}", document.language.name());
    let _ = writeln!(out, "  engine es_tokenizer");
    let _ = writeln!(out, "  fidelity token_tree");
    let _ = writeln!(out, "  token_count {}", document.token_count);
    let _ = writeln!(out, "  trees");
    write_trees(&mut out, 2, &document.trees);
    format!("{}\n", out.trim_end())
}

fn write_trees(out: &mut String, indent: usize, trees: &[OwnedTree]) {
    let pad = "  ".repeat(indent);
    for tree in trees {
        match tree {
            OwnedTree::Leaf(token) => {
                let _ = writeln!(
                    out,
                    "{pad}leaf {} {}",
                    class_slug(token.kind),
                    quoted(&token.text)
                );
            }
            OwnedTree::Group { delim, trees } => {
                let _ = writeln!(out, "{pad}group {}", delimiter_slug(*delim));
                write_trees(out, indent + 1, trees);
            }
            OwnedTree::Template { parts } => {
                let _ = writeln!(out, "{pad}template");
                write_parts(out, indent + 1, parts);
            }
        }
    }
}

fn write_parts(out: &mut String, indent: usize, parts: &[OwnedPart]) {
    let pad = "  ".repeat(indent);
    for part in parts {
        match part {
            OwnedPart::Chunk(text) => {
                let _ = writeln!(out, "{pad}chunk {}", quoted(text));
            }
            OwnedPart::Interpolation { trees } => {
                let _ = writeln!(out, "{pad}interpolation");
                write_trees(out, indent + 1, trees);
            }
        }
    }
}

/// A double-quoted lino value with the escape set the shared parser decodes
/// (`\` `"` newline tab CR); everything else passes through both ways.
fn quoted(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    out.push('"');
    for character in text.chars() {
        match character {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            other => out.push(other),
        }
    }
    out.push('"');
    out
}

/// Why a text is not a pivot document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PivotParseError {
    /// The `token_tree` record itself is missing.
    MissingRecord,
    /// A header field is missing or empty.
    MissingField(&'static str),
    /// A header field has a value this reader does not know.
    UnknownValue(String),
    /// A tree line names a token class or delimiter the reader does not know.
    UnknownKind(String),
}

impl core::fmt::Display for PivotParseError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::MissingRecord => write!(f, "not a token_tree document"),
            Self::MissingField(name) => write!(f, "token_tree document is missing {name}"),
            Self::UnknownValue(value) => write!(f, "unknown token_tree value {value:?}"),
            Self::UnknownKind(kind) => write!(f, "unknown token_tree kind {kind:?}"),
        }
    }
}

/// Parse a rendered pivot document back. The inverse of [`extract`]
/// followed by [`render_document`]: for any document this module renders,
/// the parse reproduces the tree it was rendered from.
///
/// # Errors
///
/// Returns [`PivotParseError`] for any text that is not a document
/// [`render_document`] could have produced (modulo the tree contents).
pub fn parse_document(text: &str) -> Result<PivotDocument, PivotParseError> {
    let parsed = parse_lino(text);
    let record = parsed
        .children
        .iter()
        .find(|node| node.name == "token_tree")
        .ok_or(PivotParseError::MissingRecord)?;
    let target = record.find_child_value("target");
    if target.is_empty() {
        return Err(PivotParseError::MissingField("target"));
    }
    let language = SourceLanguage::parse(record.find_child_value("language"))
        .ok_or(PivotParseError::MissingField("language"))?;
    let token_count = record
        .find_child_value("token_count")
        .parse::<usize>()
        .map_err(|_| PivotParseError::MissingField("token_count"))?;
    let trees_node = record
        .children
        .iter()
        .find(|node| node.name == "trees")
        .ok_or(PivotParseError::MissingField("trees"))?;
    let trees = trees_node
        .children
        .iter()
        .map(parse_tree)
        .collect::<Result<Vec<_>, _>>()?;
    let document = PivotDocument {
        target: target.to_string(),
        language,
        token_count,
        trees,
    };
    if document.token_count != count_tokens(&document.trees) {
        return Err(PivotParseError::MissingField("token_count"));
    }
    Ok(document)
}

fn parse_tree(node: &LinoNode) -> Result<OwnedTree, PivotParseError> {
    match node.name.as_str() {
        "leaf" => {
            let (kind, value) = split_kind_and_value(&node.id)?;
            let kind = kind_from_slug(kind)
                .ok_or_else(|| PivotParseError::UnknownKind(kind.to_string()))?;
            Ok(OwnedTree::Leaf(OwnedToken { text: value, kind }))
        }
        "group" => Ok(OwnedTree::Group {
            delim: delimiter_from_slug(&node.id)
                .ok_or_else(|| PivotParseError::UnknownKind(node.id.clone()))?,
            trees: node
                .children
                .iter()
                .map(parse_tree)
                .collect::<Result<Vec<_>, _>>()?,
        }),
        "template" => Ok(OwnedTree::Template {
            parts: node
                .children
                .iter()
                .map(parse_part)
                .collect::<Result<Vec<_>, _>>()?,
        }),
        other => Err(PivotParseError::UnknownKind(other.to_string())),
    }
}

fn parse_part(node: &LinoNode) -> Result<OwnedPart, PivotParseError> {
    match node.name.as_str() {
        // `parse_lino` already unquoted the value; re-decoding here would
        // strip a chunk that genuinely starts and ends with a quote.
        "chunk" => Ok(OwnedPart::Chunk(node.id.clone())),
        "interpolation" => Ok(OwnedPart::Interpolation {
            trees: node
                .children
                .iter()
                .map(parse_tree)
                .collect::<Result<Vec<_>, _>>()?,
        }),
        other => Err(PivotParseError::UnknownKind(other.to_string())),
    }
}

fn kind_from_slug(slug: &str) -> Option<TokenKind> {
    match slug {
        "identifier" => Some(TokenKind::Identifier),
        "numeric" => Some(TokenKind::Numeric),
        "string" => Some(TokenKind::String),
        "template_chunk" => Some(TokenKind::TemplateChunk),
        "regexp" => Some(TokenKind::RegExp),
        "punctuator" => Some(TokenKind::Punctuator),
        _ => None,
    }
}

/// Split a `leaf <kind> "<text>"` id into its kind and text. The id tail
/// carries the value in quoted form (the record name owns the first token,
/// so the shared parser could not decode it), decoded here.
fn split_kind_and_value(id: &str) -> Result<(&str, String), PivotParseError> {
    let (kind, rest) = id
        .split_once(char::is_whitespace)
        .ok_or(PivotParseError::MissingField("leaf value"))?;
    Ok((kind, quoted_tail(rest)))
}

/// Decode a `"<text>"` id tail; an unquoted tail passes through as-is (the
/// seed's punctuation operands use that form).
fn quoted_tail(raw: &str) -> String {
    let Some(rest) = raw.strip_prefix('"') else {
        return raw.to_string();
    };
    match find_closing_quote(rest) {
        Some(close) if rest[close + 1..].trim().is_empty() => unescape_value(&rest[..close]),
        _ => raw.to_string(),
    }
}

/// Which root a projection renders into.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProjectionTarget {
    /// The `./js` root.
    JavaScript,
    /// The `./ts` root.
    TypeScript,
}

impl ProjectionTarget {
    /// The seed's spelling of this root.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::JavaScript => "js",
            Self::TypeScript => "ts",
        }
    }
}

/// One item of a construct signature: what must appear in the tree for the
/// signature to match.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignatureItem {
    /// A leaf with exactly this text (any class).
    Leaf(String),
    /// A leaf of the identifier class, any text.
    Identifier,
    /// A group with this delimiter, any subtree.
    Group(Delimiter),
}

/// A token-class rule from the seed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassRule {
    /// The class slug ([`class_slug`] vocabulary, plus `group`, `template`,
    /// `interpolation` for the structural nodes).
    pub class: String,
    /// Roots this class carries to.
    pub carries_to: Vec<String>,
}

/// A construct rule from the seed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstructRule {
    /// The construct's name, as the report refuses it.
    pub construct: String,
    /// The signature that identifies the construct.
    pub signature: Vec<SignatureItem>,
    /// Roots this construct is refused to.
    pub refuses_to: Vec<String>,
}

/// The parsed projection seed.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ProjectionRules {
    /// Token-class rules.
    pub classes: Vec<ClassRule>,
    /// Construct rules.
    pub constructs: Vec<ConstructRule>,
}

/// The projection rules from the committed seed.
#[must_use]
pub fn projection_rules() -> ProjectionRules {
    projection_rules_from(LANGUAGE_PROJECTION_LINO)
}

/// The projection rules from any lino text — exposed so a test can prove
/// the renderers follow the data, not a copy of it.
#[must_use]
pub fn projection_rules_from(lino_text: &str) -> ProjectionRules {
    let parsed = parse_lino(lino_text);
    let Some(record) = parsed
        .children
        .iter()
        .find(|node| node.name == "language_projection")
    else {
        return ProjectionRules::default();
    };
    let mut rules = ProjectionRules::default();
    for node in &record.children {
        match node.name.as_str() {
            "token_class" => rules.classes.push(ClassRule {
                class: node.id.clone(),
                carries_to: flag_values(node, "carries_to"),
            }),
            "ts_construct" => rules.constructs.push(ConstructRule {
                construct: node.id.clone(),
                signature: node
                    .children
                    .iter()
                    .filter(|child| child.name == "signature")
                    .map(|child| signature_item(&child.id))
                    .collect(),
                refuses_to: flag_values(node, "refuses_to"),
            }),
            _ => {}
        }
    }
    rules
}

fn flag_values(node: &LinoNode, flag: &str) -> Vec<String> {
    node.children
        .iter()
        .filter(|child| child.name == flag)
        .map(|child| child.id.clone())
        .collect()
}

fn signature_item(id: &str) -> SignatureItem {
    let Some((kind, operand)) = id.split_once(char::is_whitespace) else {
        // `signature identifier` names the class with no operand; any other
        // bare word is a leaf matched by its own text.
        return if id == "identifier" {
            SignatureItem::Identifier
        } else {
            SignatureItem::Leaf(id.to_string())
        };
    };
    match kind {
        "identifier" => SignatureItem::Identifier,
        "group" => delimiter_from_slug(operand)
            .map_or_else(|| SignatureItem::Leaf(id.to_string()), SignatureItem::Group),
        _ => SignatureItem::Leaf(quoted_tail(operand)),
    }
}

/// One refusal: the construct (or token class) the seed refuses, and the
/// text that identified it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refusal {
    /// The refused construct's name from the seed.
    pub construct: String,
    /// The tree text the signature or class matched on.
    pub detail: String,
}

/// What a projection carried and refused.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RenderReport {
    /// Leaves and chunks carried into the target.
    pub carried: usize,
    /// Every refusal the seed demanded; empty is the only success.
    pub refused: Vec<Refusal>,
}

/// A projection's output: the rendered source when nothing was refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderedSource {
    /// Canonically spaced target source; `None` when anything was refused.
    pub output: Option<String>,
    /// What the projection carried and refused.
    pub report: RenderReport,
}

/// Render a pivot document into a target root under the seed's rules.
///
/// Refusal is total: one refused token class or construct signature means
/// no output at all and the report names everything refused, because
/// dropping only the refused pieces would silently change the tree.
#[must_use]
pub fn render_source(document: &PivotDocument, target: ProjectionTarget) -> RenderedSource {
    let rules = projection_rules();
    let mut report = RenderReport::default();
    refuse_classes(&document.trees, target, &rules, &mut report);
    refuse_constructs(&document.trees, target, &rules, &mut report);
    if !report.refused.is_empty() {
        return RenderedSource {
            output: None,
            report,
        };
    }
    let mut out = String::new();
    write_source(&mut out, &document.trees);
    report.carried = document.token_count;
    RenderedSource {
        output: Some(out),
        report,
    }
}

/// Fail-closed class check: every leaf, group, template, and interpolation
/// must have a seed rule that carries it to the target.
fn refuse_classes(
    trees: &[OwnedTree],
    target: ProjectionTarget,
    rules: &ProjectionRules,
    report: &mut RenderReport,
) {
    for tree in trees {
        match tree {
            OwnedTree::Leaf(token) => {
                check_class(class_slug(token.kind), target, rules, report, &token.text);
            }
            OwnedTree::Group { delim, trees } => {
                check_class("group", target, rules, report, delimiter_slug(*delim));
                refuse_classes(trees, target, rules, report);
            }
            OwnedTree::Template { parts } => {
                check_class("template", target, rules, report, "");
                for part in parts {
                    match part {
                        OwnedPart::Chunk(text) => {
                            check_class("template_chunk", target, rules, report, text);
                        }
                        OwnedPart::Interpolation { trees } => {
                            check_class("interpolation", target, rules, report, "");
                            refuse_classes(trees, target, rules, report);
                        }
                    }
                }
            }
        }
    }
}

fn check_class(
    class: &str,
    target: ProjectionTarget,
    rules: &ProjectionRules,
    report: &mut RenderReport,
    detail: &str,
) {
    let carries = rules.classes.iter().any(|rule| {
        rule.class == class && rule.carries_to.iter().any(|root| root == target.name())
    });
    if !carries {
        report.refused.push(Refusal {
            construct: format!("token_class {class}"),
            detail: detail.to_string(),
        });
    }
}

/// Structural items of one tree level, in order: leaves by text and class,
/// groups by delimiter. Templates are opaque to signatures — a construct
/// signature the seed can write never spans one.
enum LevelItem<'a> {
    Leaf(&'a OwnedToken),
    Group(Delimiter),
}

fn level_items(trees: &[OwnedTree]) -> Vec<LevelItem<'_>> {
    trees
        .iter()
        .filter_map(|tree| match tree {
            OwnedTree::Leaf(token) => Some(LevelItem::Leaf(token)),
            OwnedTree::Group { delim, .. } => Some(LevelItem::Group(*delim)),
            OwnedTree::Template { .. } => None,
        })
        .collect()
}

/// Match every construct signature over every level of the tree, in tree
/// order — a signature names a leaf/group sequence that cannot be valid
/// plain JavaScript, so any match anywhere refuses the document, and each
/// matching site is refused by name.
fn refuse_constructs(
    trees: &[OwnedTree],
    target: ProjectionTarget,
    rules: &ProjectionRules,
    report: &mut RenderReport,
) {
    let items = level_items(trees);
    for rule in &rules.constructs {
        if rule.signature.is_empty()
            || !rule.refuses_to.iter().any(|root| root == target.name())
            || items.len() < rule.signature.len()
        {
            continue;
        }
        for window in items.windows(rule.signature.len()) {
            if window_matches(window, &rule.signature) {
                report.refused.push(Refusal {
                    construct: rule.construct.clone(),
                    detail: match window.first() {
                        Some(LevelItem::Leaf(token)) => token.text.clone(),
                        Some(LevelItem::Group(delim)) => delimiter_slug(*delim).to_string(),
                        None => String::new(),
                    },
                });
            }
        }
    }
    for tree in trees {
        match tree {
            OwnedTree::Group { trees, .. } => refuse_constructs(trees, target, rules, report),
            OwnedTree::Template { parts } => {
                for part in parts {
                    if let OwnedPart::Interpolation { trees } = part {
                        refuse_constructs(trees, target, rules, report);
                    }
                }
            }
            OwnedTree::Leaf(_) => {}
        }
    }
}

fn window_matches(window: &[LevelItem<'_>], signature: &[SignatureItem]) -> bool {
    window.len() == signature.len()
        && window
            .iter()
            .zip(signature)
            .all(|(item, expected)| match (item, expected) {
                (LevelItem::Leaf(token), SignatureItem::Leaf(text)) => &token.text == text,
                (LevelItem::Leaf(token), SignatureItem::Identifier) => {
                    token.kind == TokenKind::Identifier
                }
                (LevelItem::Group(delim), SignatureItem::Group(want)) => delim == want,
                _ => false,
            })
}

/// Emit canonically spaced source: every leaf separated by one space, group
/// delimiters spaced around their subtree, template chunks verbatim between
/// backticks. An empty interpolation cannot round-trip (`${}` tokenizes as
/// `$` then an empty group, never an interpolation) and cannot arise from
/// extraction, so it renders as-is.
fn write_source(out: &mut String, trees: &[OwnedTree]) {
    let mut first = true;
    for tree in trees {
        if !first {
            out.push(' ');
        }
        first = false;
        match tree {
            OwnedTree::Leaf(token) => out.push_str(&token.text),
            OwnedTree::Group { delim, trees } => {
                out.push(delim.opens() as char);
                if !trees.is_empty() {
                    out.push(' ');
                    write_source(out, trees);
                    out.push(' ');
                }
                out.push(delim.closes() as char);
            }
            OwnedTree::Template { parts } => {
                out.push('`');
                for part in parts {
                    match part {
                        OwnedPart::Chunk(text) => out.push_str(text),
                        OwnedPart::Interpolation { trees } => {
                            out.push_str("${");
                            write_source(out, trees);
                            out.push('}');
                        }
                    }
                }
                out.push('`');
            }
        }
    }
}
