//! Seed-tree adapter for links-notation 0.23 (#1182).
//!
//! The crate's public `parse_lino` flattens indentation into relation paths;
//! seed loaders require the original hierarchy. Its public
//! `parser::parse_document_with_diagnostics` retains that hierarchy, so this
//! adapter uses that API instead. Nested inline groups are reported as
//! unsupported rather than silently discarded. Corpus adoption remains gated on
//! the explicit conformance audit; existing loaders keep their current parser
//! until all semantic differences are resolved.

pub use crate::seed::parser::LinoNode;
use links_notation::LiNo;
pub use links_notation::ParseError;
use links_notation::comments::strip_comments;
use links_notation::format_config::FormatConfig;
use links_notation::parser::Link;

pub fn parse_lino(text: &str) -> Result<LinoNode, ParseError> {
    // Comments follow the crate's own rule (0.23): a `#` that opens a line or
    // follows whitespace, outside a quoted reference, runs to the end of the
    // line. They are blanked rather than removed, so positions still match.
    // The concise lexeme form expands first, as in `seed::parser` (R1188-U7).
    let expanded = crate::seed::expand_concise_lexemes(text);
    let prepared = strip_comments(&expanded);
    let links = links_notation::parser::parse_document_with_diagnostics(&prepared)
        .map_err(|_| located_error(text))?;
    let children = links.iter().map(convert).collect::<Result<Vec<_>, _>>()?;
    Ok(LinoNode {
        children,
        ..LinoNode::default()
    })
}

/// The unflattened parser reports a bare byte offset. The crate's `parse_lino`
/// reads the same comment-stripped document with the same depth limit and turns
/// that failure into a located [`ParseError`] (line, column, expectation), so
/// the error path asks it rather than re-deriving positions here.
fn located_error(text: &str) -> ParseError {
    links_notation::parse_lino(text).err().unwrap_or_else(|| {
        ParseError::InternalError(String::from(
            "parse_document rejected a document parse_lino accepts",
        ))
    })
}

/// A construct the grammar accepts but the seed tree cannot hold. The crate's
/// `SyntaxError` carries a position the unflattened [`Link`] does not record,
/// so these are reported by name.
fn unsupported(what: &str) -> ParseError {
    ParseError::InternalError(what.to_owned())
}

/// Preserve the parser's unflattened indentation tree. A scalar is one
/// reference; composite inline links require a different representation.
pub fn convert(link: &Link) -> Result<LinoNode, ParseError> {
    if link.nested.is_some() {
        return Err(unsupported("unsupported_nested_group"));
    }
    let scalar = |value: &Link| -> Result<String, ParseError> {
        if !value.values.is_empty() || !value.children.is_empty() || value.nested.is_some() {
            return Err(unsupported("unsupported_inline_link"));
        }
        value
            .id
            .clone()
            .ok_or_else(|| unsupported("missing_scalar"))
    };
    let (name, values) = if let Some(name) = &link.id {
        (name.clone(), link.values.as_slice())
    } else {
        let (head, tail) = link
            .values
            .split_first()
            .ok_or_else(|| unsupported("missing_head"))?;
        (scalar(head)?, tail)
    };
    let id = values
        .iter()
        .map(scalar)
        .collect::<Result<Vec<_>, _>>()?
        .join(" ");
    let children = link
        .children
        .iter()
        .map(convert)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(LinoNode { name, id, children })
}

/// Render indentation with the crate's scalar encoder. Calling `LiNo`'s Display
/// would omit escaping, and flattening children would erase the seed hierarchy.
#[must_use]
pub fn format_lino(root: &LinoNode) -> String {
    fn emit(node: &LinoNode, depth: usize, config: &FormatConfig, out: &mut String) {
        out.push_str(&"  ".repeat(depth));
        out.push_str(&LiNo::reference(node.name.clone()).format_with_config(config));
        if !node.id.is_empty() {
            out.push_str(": ");
            out.push_str(&LiNo::reference(node.id.clone()).format_with_config(config));
        }
        out.push('\n');
        for child in &node.children {
            emit(child, depth + 1, config, out);
        }
    }
    let config = FormatConfig::builder()
        .less_parentheses(true)
        .prefer_inline(true)
        .build();
    let mut out = String::new();
    if root.name.is_empty() {
        for child in &root.children {
            emit(child, 0, &config, &mut out);
        }
    } else {
        emit(root, 0, &config, &mut out);
    }
    out
}
