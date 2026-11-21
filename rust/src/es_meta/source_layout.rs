//! Preserve and emit the whitespace and comments around ES token trees.

use super::{OwnedPart, OwnedTree, String, TemplatePart, ToString, Tree, Vec};

/// The layout of one source: the text before every slot of its tree.
///
/// Slots come in the order the renderer visits them, then one more holds
/// the text after the last token. A slot is a leaf, a group's opening and
/// closing delimiter, a template's opening backtick, and an interpolation's
/// closing brace; everything else in a template is its verbatim chunks. The
/// text is what the tokenizer skips: whitespace, comments, a leading
/// shebang line.
#[must_use]
pub fn layout_of(source: &str, trees: &[Tree<'_>]) -> Vec<String> {
    let mut layout = Vec::new();
    let mut cursor = 0;
    collect_layout(source, trees, &mut cursor, &mut layout);
    push_gap(source, cursor, source.len(), &mut layout);
    layout
}

fn collect_layout(source: &str, trees: &[Tree<'_>], cursor: &mut usize, layout: &mut Vec<String>) {
    for tree in trees {
        match tree {
            Tree::Leaf(token) => {
                push_gap(source, *cursor, token.span.start, layout);
                *cursor = token.span.end;
            }
            Tree::Group { span, trees, .. } => {
                push_gap(source, *cursor, span.start, layout);
                *cursor = span.start + 1;
                collect_layout(source, trees, cursor, layout);
                push_gap(source, *cursor, span.end - 1, layout);
                *cursor = span.end;
            }
            Tree::Template { span, parts } => {
                push_gap(source, *cursor, span.start, layout);
                for part in parts {
                    if let TemplatePart::Interpolation {
                        span: interpolation,
                        trees,
                    } = part
                    {
                        *cursor = interpolation.start + 2;
                        collect_layout(source, trees, cursor, layout);
                        push_gap(source, *cursor, interpolation.end - 1, layout);
                    }
                }
                *cursor = span.end;
            }
        }
    }
}

fn push_gap(source: &str, start: usize, end: usize, layout: &mut Vec<String>) {
    layout.push(source.get(start..end).unwrap_or_default().to_string());
}

/// How many layout slots a tree has, the trailing one not counted.
pub(super) fn slot_count(trees: &[OwnedTree]) -> usize {
    let mut total = 0;
    for tree in trees {
        match tree {
            OwnedTree::Leaf(_) => total += 1,
            OwnedTree::Group { trees, .. } => total += 2 + slot_count(trees),
            OwnedTree::Template { parts } => {
                total += 1;
                for part in parts {
                    if let OwnedPart::Interpolation { trees } = part {
                        total += 1 + slot_count(trees);
                    }
                }
            }
        }
    }
    total
}

/// Emit target source, each token after its slot's layout text.
///
/// A slot ([`layout_of`]) takes the layout's own text when the document
/// carries one, and canonical spacing otherwise — every leaf separated by
/// one space, group delimiters spaced around their subtree, template chunks
/// verbatim between backticks. An empty interpolation cannot round-trip
/// (`${}` tokenizes as `$` then an empty group, never an interpolation) and
/// cannot arise from extraction, so it renders as-is.
pub(super) fn write_source(out: &mut String, trees: &[OwnedTree], layout: &[String]) {
    let mut slot = 0;
    write_level(out, trees, layout, &mut slot, "");
    out.push_str(slot_text(layout, &mut slot, ""));
}

/// One tree level; `lead` is the canonical text before its first item.
fn write_level(
    out: &mut String,
    trees: &[OwnedTree],
    layout: &[String],
    slot: &mut usize,
    lead: &str,
) {
    for (index, tree) in trees.iter().enumerate() {
        out.push_str(slot_text(layout, slot, if index == 0 { lead } else { " " }));
        match tree {
            OwnedTree::Leaf(token) => out.push_str(&token.text),
            OwnedTree::Group { delim, trees } => {
                out.push(delim.opens() as char);
                write_level(out, trees, layout, slot, " ");
                out.push_str(slot_text(
                    layout,
                    slot,
                    if trees.is_empty() { "" } else { " " },
                ));
                out.push(delim.closes() as char);
            }
            OwnedTree::Template { parts } => {
                out.push('`');
                for part in parts {
                    match part {
                        OwnedPart::Chunk(text) => out.push_str(text),
                        OwnedPart::Interpolation { trees } => {
                            out.push_str("${");
                            write_level(out, trees, layout, slot, "");
                            out.push_str(slot_text(layout, slot, ""));
                            out.push('}');
                        }
                    }
                }
                out.push('`');
            }
        }
    }
}

/// The next slot's text: the layout's own, or the canonical spacing given.
fn slot_text<'text>(
    layout: &'text [String],
    slot: &mut usize,
    canonical: &'text str,
) -> &'text str {
    let text = layout.get(*slot).map_or(canonical, String::as_str);
    *slot += 1;
    text
}
