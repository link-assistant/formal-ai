//! Source edits as substitutions over the file's links network (issue #1085, D2.1).
//!
//! Before this module the repository could turn Rust or JavaScript into a
//! links network and print the identical bytes back ([`super::self_ast`],
//! `crate::coding::cst`), but nothing edited the links and rendered changed
//! code: `structured_edit` moved bytes by offset. This is the first path in the
//! *links → code* direction that changes something.
//!
//! Three rule shapes cover every leaf of the Agent CLI ladder
//! (`experiments/issue_1028_agent_cli_ladder/run.sh`): add a member to a named
//! list, replace a literal inside string literals, rename an identifier as a
//! whole token. Which grammar node kinds each shape may touch is declared in
//! `data/meta/link-edit-rules.lino`; the code below reads that file and knows
//! no node kind of its own.
//!
//! The pipeline is parse → select links by kind and text → rewrite their byte
//! ranges through the network's incremental `apply_edit` (which reparses, so a
//! later selection sees the new tree) → `reconstruct_text`. Formatting,
//! compiling and testing the result stay with the caller: the ladder's
//! `verify-node.sh` runs `rustfmt` and `cargo check`, and the report returned
//! here says whether the network was still clean after the edit.

use std::fmt;

#[cfg(feature = "meta-language")]
use meta_language::{ByteRange, LinkNetwork, LinkType, NetworkProjection, ParseConfiguration};

const RULES: &str = include_str!("../../data/meta/link-edit-rules.lino");

/// An edit expressed as what to change, never where in the bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinkEditRule {
    /// Add `member` (unquoted) to the list the identifier `list` introduces.
    InsertMember { list: String, member: String },
    /// Replace every `old` inside string literals with `new`.
    ReplaceLiteral { old: String, new: String },
    /// Rename the identifier `old` to `new`, whole tokens only.
    RenameIdentifier { old: String, new: String },
}

impl LinkEditRule {
    /// The rule's name in `data/meta/link-edit-rules.lino`.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::InsertMember { .. } => "insert_member",
            Self::ReplaceLiteral { .. } => "replace_literal",
            Self::RenameIdentifier { .. } => "rename_identifier",
        }
    }
}

/// What the edit did to the network, for the caller's evidence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkEditReport {
    pub rule: String,
    pub language: String,
    pub links_before: usize,
    pub links_after: usize,
    /// Byte ranges rewritten; one per touched link occurrence.
    pub edits: usize,
    /// The parse reconstructed the input byte for byte before the edit.
    pub round_trip_before: bool,
    /// The network verified clean after the last edit.
    pub clean_after: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinkEditError {
    /// Built without the `meta-language` feature; there is no links network.
    EngineUnavailable,
    /// The rule file declares no node kinds for this rule.
    RuleUndeclared(String),
    /// The parse produced no syntax links for the language label.
    NotParsed(String),
    /// No link of an allowed kind carries the text the rule names.
    NoMatchingLink { rule: String, needle: String },
    /// The identifier was found but no list follows it in the same statement.
    NoListAfter(String),
    /// The member is already in the list; nothing to do.
    MemberPresent { list: String, member: String },
    /// An empty pattern would match everywhere.
    EmptyPattern,
    /// The network refused the byte-range edit.
    EditRejected { start: usize, end: usize },
}

impl fmt::Display for LinkEditError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EngineUnavailable => write!(f, "link_edit:engine_unavailable"),
            Self::RuleUndeclared(rule) => write!(f, "link_edit:rule_undeclared:{rule}"),
            Self::NotParsed(language) => write!(f, "link_edit:not_parsed:{language}"),
            Self::NoMatchingLink { rule, needle } => {
                write!(f, "link_edit:no_matching_link:{rule}:{needle}")
            }
            Self::NoListAfter(list) => write!(f, "link_edit:no_list_after:{list}"),
            Self::MemberPresent { list, member } => {
                write!(f, "link_edit:member_present:{list}:{member}")
            }
            Self::EmptyPattern => write!(f, "link_edit:empty_pattern"),
            Self::EditRejected { start, end } => {
                write!(f, "link_edit:edit_rejected:{start}:{end}")
            }
        }
    }
}

impl std::error::Error for LinkEditError {}

/// The node kinds `data/meta/link-edit-rules.lino` allows a rule to touch.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct RuleShape {
    pub rule: String,
    pub node_kinds: Vec<String>,
    pub anchor_kind: Option<String>,
    pub list_kind: Option<String>,
    pub element_kind: Option<String>,
}

/// Every declared rule shape, in file order.
#[must_use]
pub fn rule_shapes() -> Vec<RuleShape> {
    parse_rule_shapes(RULES)
}

fn parse_rule_shapes(text: &str) -> Vec<RuleShape> {
    let mut shapes: Vec<RuleShape> = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(name) = trimmed.strip_prefix("rule ") {
            shapes.push(RuleShape {
                rule: name.trim().to_owned(),
                ..RuleShape::default()
            });
            continue;
        }
        let Some(shape) = shapes.last_mut() else {
            continue;
        };
        if let Some(kind) = trimmed.strip_prefix("node_kind ") {
            shape.node_kinds.push(kind.trim().to_owned());
        } else if let Some(kind) = trimmed.strip_prefix("anchor_kind ") {
            shape.anchor_kind = Some(kind.trim().to_owned());
        } else if let Some(kind) = trimmed.strip_prefix("list_kind ") {
            shape.list_kind = Some(kind.trim().to_owned());
        } else if let Some(kind) = trimmed.strip_prefix("element_kind ") {
            shape.element_kind = Some(kind.trim().to_owned());
        }
    }
    shapes
}

#[cfg_attr(not(feature = "meta-language"), allow(dead_code))]
fn shape_for(rule: &LinkEditRule) -> Result<RuleShape, LinkEditError> {
    rule_shapes()
        .into_iter()
        .find(|shape| shape.rule == rule.name())
        .ok_or_else(|| LinkEditError::RuleUndeclared(rule.name().to_owned()))
}

/// Apply `rule` to `source` written in `language` (a meta-language grammar
/// label such as `rust` or `javascript`) and return the rewritten source with
/// its report.
///
/// # Errors
///
/// Returns a [`LinkEditError`] when the rule is not declared in the rule file,
/// the source did not parse, no link carries the named text, the list cannot be
/// located, the member is already present, or the network rejects an edit.
#[cfg(feature = "meta-language")]
pub fn apply_link_edit(
    source: &str,
    language: &str,
    rule: &LinkEditRule,
) -> Result<(String, LinkEditReport), LinkEditError> {
    let shape = shape_for(rule)?;
    let mut network = LinkNetwork::parse(source, language, ParseConfiguration::default());
    let links_before = network.len();
    let round_trip_before = network.reconstruct_text() == source;
    if !has_syntax_links(&network) {
        return Err(LinkEditError::NotParsed(language.to_owned()));
    }

    let edits = match rule {
        LinkEditRule::RenameIdentifier { old, new } => {
            if old.is_empty() {
                return Err(LinkEditError::EmptyPattern);
            }
            let text = network.reconstruct_text();
            let ranges = spans_of_kinds(&network, &shape.node_kinds)
                .into_iter()
                .filter(|range| text.get(range.start()..range.end()) == Some(old.as_str()))
                .map(|range| (range, new.clone()))
                .collect::<Vec<_>>();
            if ranges.is_empty() {
                return Err(LinkEditError::NoMatchingLink {
                    rule: rule.name().to_owned(),
                    needle: old.clone(),
                });
            }
            ranges
        }
        LinkEditRule::ReplaceLiteral { old, new } => {
            if old.is_empty() {
                return Err(LinkEditError::EmptyPattern);
            }
            let text = network.reconstruct_text();
            let mut ranges = Vec::new();
            for range in spans_of_kinds(&network, &shape.node_kinds) {
                let Some(slice) = text.get(range.start()..range.end()) else {
                    continue;
                };
                let mut from = 0;
                while let Some(found) = slice[from..].find(old.as_str()) {
                    let start = range.start() + from + found;
                    ranges.push((ByteRange::new(start, start + old.len()), new.clone()));
                    from += found + old.len();
                }
            }
            if ranges.is_empty() {
                return Err(LinkEditError::NoMatchingLink {
                    rule: rule.name().to_owned(),
                    needle: old.clone(),
                });
            }
            ranges
        }
        LinkEditRule::InsertMember { list, member } => {
            vec![member_insertion_edit(&network, &shape, list, member)?]
        }
    };

    // Rewrite from the end of the file towards the start: every earlier range
    // keeps its offsets while the network reparses after each edit.
    let mut ordered = edits;
    ordered.sort_by_key(|(range, _)| std::cmp::Reverse(range.start()));
    ordered.dedup_by(|later, earlier| later.0.start() == earlier.0.start());
    let count = ordered.len();
    for (range, replacement) in ordered {
        if !network.apply_edit(range, &replacement) {
            return Err(LinkEditError::EditRejected {
                start: range.start(),
                end: range.end(),
            });
        }
    }
    let clean_after = network.verify_full_match(None).is_clean();
    let text = network.reconstruct_text();
    Ok((
        text,
        LinkEditReport {
            rule: rule.name().to_owned(),
            language: language.to_owned(),
            links_before,
            links_after: network.len(),
            edits: count,
            round_trip_before,
            clean_after,
        },
    ))
}

/// Without the parsing engine there is no links network to edit.
///
/// # Errors
///
/// Always [`LinkEditError::EngineUnavailable`].
#[cfg(not(feature = "meta-language"))]
pub fn apply_link_edit(
    _source: &str,
    _language: &str,
    _rule: &LinkEditRule,
) -> Result<(String, LinkEditReport), LinkEditError> {
    Err(LinkEditError::EngineUnavailable)
}

/// Insert every value of `values` into the list `list` introduces.
///
/// One rule application per value. `None` when the list cannot be reached
/// through the links network, so the caller keeps its byte-level path for those
/// shapes; a value already present is left in place rather than duplicated.
#[must_use]
pub fn insert_members_via_links(
    source: &str,
    list: &str,
    values: &[String],
) -> Option<(String, Vec<String>)> {
    let mut text = source.to_owned();
    let mut inserted = Vec::new();
    for value in values {
        let rule = LinkEditRule::InsertMember {
            list: list.to_owned(),
            member: value.clone(),
        };
        match apply_link_edit(&text, "rust", &rule) {
            Ok((next, _)) => {
                text = next;
                inserted.push(value.clone());
            }
            Err(LinkEditError::MemberPresent { .. }) => {}
            Err(_) => return None,
        }
    }
    Some((text, inserted))
}

#[cfg(feature = "meta-language")]
fn has_syntax_links(network: &LinkNetwork) -> bool {
    network
        .projected_links(NetworkProjection::ConcreteSyntax)
        .any(|link| link.metadata().link_type() == Some(LinkType::Syntax))
}

/// Byte ranges of every named syntax link whose grammar kind is in `kinds`,
/// in source order.
#[cfg(feature = "meta-language")]
fn spans_of_kinds(network: &LinkNetwork, kinds: &[String]) -> Vec<ByteRange> {
    let mut ranges = network
        .projected_links(NetworkProjection::ConcreteSyntax)
        .filter_map(|link| {
            let metadata = link.metadata();
            if metadata.link_type() != Some(LinkType::Syntax) {
                return None;
            }
            let kind = metadata.term()?;
            if !kinds.iter().any(|allowed| allowed == kind) {
                return None;
            }
            Some(metadata.span()?.byte_range())
        })
        .collect::<Vec<_>>();
    ranges.sort_by_key(|range| (range.start(), range.end()));
    ranges.dedup_by(|later, earlier| {
        later.start() == earlier.start() && later.end() == earlier.end()
    });
    ranges
}

/// Locate the list the identifier `list` introduces and compute the one edit
/// that adds `member` to it, formatted the way its last member is.
#[cfg(feature = "meta-language")]
fn member_insertion_edit(
    network: &LinkNetwork,
    shape: &RuleShape,
    list: &str,
    member: &str,
) -> Result<(ByteRange, String), LinkEditError> {
    let text = network.reconstruct_text();
    let anchor_kind = shape
        .anchor_kind
        .clone()
        .unwrap_or_else(|| "identifier".to_owned());
    let list_kind = shape
        .list_kind
        .clone()
        .unwrap_or_else(|| "array_expression".to_owned());
    let element_kind = shape
        .element_kind
        .clone()
        .unwrap_or_else(|| "string_literal".to_owned());

    let anchors = spans_of_kinds(network, std::slice::from_ref(&anchor_kind))
        .into_iter()
        .filter(|range| text.get(range.start()..range.end()) == Some(list))
        .collect::<Vec<_>>();
    let anchor = anchors
        .first()
        .ok_or_else(|| LinkEditError::NoMatchingLink {
            rule: "insert_member".to_owned(),
            needle: list.to_owned(),
        })?;

    // The nearest list after the identifier, inside the same statement.
    let lists = spans_of_kinds(network, std::slice::from_ref(&list_kind));
    let array = lists
        .into_iter()
        .filter(|range| range.start() >= anchor.end())
        .filter(|range| {
            text.get(anchor.end()..range.start())
                .is_some_and(|between| !between.contains(';'))
        })
        .min_by_key(|range| range.start())
        .ok_or_else(|| LinkEditError::NoListAfter(list.to_owned()))?;

    let quoted = if member.starts_with('"') {
        member.to_owned()
    } else {
        format!("\"{member}\"")
    };
    let elements = spans_of_kinds(network, std::slice::from_ref(&element_kind))
        .into_iter()
        .filter(|range| range.start() >= array.start() && range.end() <= array.end())
        .collect::<Vec<_>>();
    if elements
        .iter()
        .any(|range| text.get(range.start()..range.end()) == Some(quoted.as_str()))
    {
        return Err(LinkEditError::MemberPresent {
            list: list.to_owned(),
            member: member.to_owned(),
        });
    }

    let Some(final_member) = elements.last() else {
        // An empty list: insert right after its opening bracket.
        let slice = text.get(array.start()..array.end()).unwrap_or("");
        let open = slice.find('[').map_or(0, |index| index + 1);
        let at = array.start() + open;
        return Ok((ByteRange::new(at, at), quoted));
    };
    // Repeat the separator the list already uses between its last two members
    // (`", "` on one line, `",\n    "` across lines); a single member gets `, `.
    let separator = elements
        .windows(2)
        .next_back()
        .and_then(|pair| text.get(pair[0].end()..pair[1].start()))
        .filter(|gap| !gap.is_empty())
        .map_or_else(|| ", ".to_owned(), str::to_owned);
    Ok((
        ByteRange::new(final_member.end(), final_member.end()),
        format!("{separator}{quoted}"),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_rule_file_declares_the_three_ladder_shapes() {
        let shapes = rule_shapes();
        let names = shapes
            .iter()
            .map(|shape| shape.rule.as_str())
            .collect::<Vec<_>>();
        assert_eq!(
            names,
            ["insert_member", "replace_literal", "rename_identifier"]
        );
        assert!(shapes[2].node_kinds.contains(&"identifier".to_owned()));
        assert_eq!(shapes[0].list_kind.as_deref(), Some("array_expression"));
    }

    #[test]
    fn rule_shapes_parse_only_indented_declarations() {
        let parsed =
            parse_rule_shapes("x\n  rule a\n    node_kind k\n  rule b\n    anchor_kind i\n");
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].node_kinds, vec!["k".to_owned()]);
        assert_eq!(parsed[1].anchor_kind.as_deref(), Some("i"));
    }
}
