//! Experimental seed-tree adapter for the installed links-notation 0.16.1 (#1182).
//!
//! The crate's public `parse_lino` flattens indentation into relation paths;
//! seed loaders require the original hierarchy. Its public `parser::parse_document`
//! retains that hierarchy, so this adapter uses that API instead. Nested inline
//! groups are reported as unsupported rather than silently discarded. Corpus
//! adoption remains gated on the explicit conformance audit; existing loaders
//! keep their current parser until all semantic differences are resolved.

pub use crate::seed::parser::LinoNode;
use links_notation::LiNo;
pub use links_notation::ParseError;
use links_notation::format_config::FormatConfig;
use links_notation::parser::Link;

pub fn parse_lino(text: &str) -> Result<LinoNode, ParseError> {
    // Only whole-line seed comments are stripped here. Inline comments and the
    // historical backslash dialect remain visible conformance gaps.
    let prepared = text
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");
    let (_, links) = links_notation::parser::parse_document(&prepared)
        .map_err(|error| ParseError::SyntaxError(format!("{error:?}")))?;
    let children = links.iter().map(convert).collect::<Result<Vec<_>, _>>()?;
    Ok(LinoNode {
        children,
        ..LinoNode::default()
    })
}

/// Preserve the parser's unflattened indentation tree. A scalar is one
/// reference; composite inline links require a different representation.
pub fn convert(link: &Link) -> Result<LinoNode, ParseError> {
    if link.nested.is_some() {
        return Err(ParseError::SyntaxError(String::from(
            "unsupported_nested_group",
        )));
    }
    let scalar = |value: &Link| -> Result<String, ParseError> {
        if !value.values.is_empty() || !value.children.is_empty() || value.nested.is_some() {
            return Err(ParseError::SyntaxError(String::from(
                "unsupported_inline_link",
            )));
        }
        value
            .id
            .clone()
            .ok_or_else(|| ParseError::SyntaxError(String::from("missing_scalar")))
    };
    let (name, values) = match &link.id {
        Some(name) => (name.clone(), link.values.as_slice()),
        None => {
            let (head, tail) = link
                .values
                .split_first()
                .ok_or_else(|| ParseError::SyntaxError(String::from("missing_head")))?;
            (scalar(head)?, tail)
        }
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

/// Render indentation with the crate's scalar encoder. Calling LiNo's Display
/// would omit escaping, and flattening children would erase the seed hierarchy.
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn indentation_and_quoted_values_survive() {
        let doc = "# seed comment\nstrategy\n  value \"quoted value\"\n  step\n    name first\n";
        let parsed = parse_lino(doc).unwrap();
        assert_eq!(parsed.children[0].name, "strategy");
        assert_eq!(parsed.children[0].find_child_value("value"), "quoted value");
        assert_eq!(parsed.children[0].children[1].children[0].id, "first");
        let reparsed = parse_lino(&format_lino(&parsed)).unwrap();
        assert_eq!(
            reparsed.children[0].find_child_value("value"),
            "quoted value"
        );
    }
    #[test]
    fn malformed_document_is_rejected() {
        assert!(parse_lino("value: (\n").is_err());
    }
}
