//! Source-owned Rust AST alpha-renaming candidates; no effect/native authority.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RustConstantRenameCandidate {
    pub kind: &'static str,
    pub content: String,
    pub previous: String,
    pub replacement: String,
    pub spans: Vec<[usize; 2]>,
    pub unit: &'static str,
    pub scope: &'static str,
    pub source_syntax: &'static str,
    pub compilation: &'static str,
    pub write_authority: bool,
    pub runtime_effects: &'static str,
    pub unresolved_needs: Vec<&'static str>,
}
/// Parse the actual supplied bytes through the maintained engine before reasoning.
/// An unavailable parser refuses; public trees/JSON cannot assert source ownership.
#[must_use = "the candidate records source edits and unresolved obligations"]
pub fn rust_constant_rename_candidate(
    source: &str,
    previous: &str,
    replacement: &str,
) -> Option<RustConstantRenameCandidate> {
    candidate_from_source(source, previous, replacement)
}
#[cfg(not(feature = "meta-language"))]
fn candidate_from_source(
    _source: &str,
    _previous: &str,
    _replacement: &str,
) -> Option<RustConstantRenameCandidate> {
    None
}

#[cfg(feature = "meta-language")]
fn candidate_from_source(
    source: &str,
    previous: &str,
    replacement: &str,
) -> Option<RustConstantRenameCandidate> {
    use super::code_task::valid_identifier;
    use meta_language::{LinkId, LinkNetwork, LinkType, ParseConfiguration};
    if !valid_identifier(previous) || !valid_identifier(replacement) || previous == replacement {
        return None;
    }
    let network = LinkNetwork::parse(source, "rust", ParseConfiguration::default());
    if !network.verify_full_match(None).is_clean() || network.reconstruct_text() != source {
        return None;
    }
    let nodes: Vec<_> = network
        .links()
        .filter(|link| {
            link.metadata().link_type() == Some(LinkType::Syntax) && link.metadata().is_named()
        })
        .collect();
    let parent = |identifier: LinkId| {
        network
            .link(identifier)?
            .references()
            .first()
            .and_then(|identifier| network.link(*identifier))
    };
    let text = |identifier: LinkId| {
        let range = network.link(identifier)?.metadata().span()?.byte_range();
        source.get(range.start()..range.end())
    };
    let field = |identifier: LinkId, name: &str| {
        let fields: Vec<_> = network
            .links()
            .filter(|link| link.metadata().link_type() == Some(LinkType::Field))
            .filter_map(|link| {
                let [owner, label, value] = link.references() else {
                    return None;
                };
                (*owner == identifier && network.link(*label)?.metadata().term() == Some(name))
                    .then_some(*value)
            })
            .collect();
        (fields.len() == 1).then(|| fields[0])
    };
    if nodes.iter().any(|node| {
        matches!(
            node.metadata().term(),
            Some("mod_item" | "macro_definition")
        )
    }) {
        return None;
    }
    let declarations: Vec<_> = nodes
        .iter()
        .filter(|node| {
            node.metadata().term() == Some("const_item")
                && field(node.id(), "name").and_then(text) == Some(previous)
        })
        .collect();
    if declarations.len() != 1 {
        return None;
    }
    let declaration = declarations[0];
    if parent(declaration.id())?.metadata().term() != Some("source_file")
        || nodes.iter().any(|node| {
            node.metadata().term() == Some("visibility_modifier")
                && parent(node.id()).is_some_and(|owner| owner.id() == declaration.id())
        })
    {
        return None;
    }
    if nodes.iter().any(|node| {
        node.metadata().term() == Some("identifier") && text(node.id()) == Some(replacement)
    }) {
        return None;
    }
    let name = field(declaration.id(), "name")?;
    let occurrences: Vec<_> = nodes
        .iter()
        .filter(|node| {
            node.metadata().term() == Some("identifier") && text(node.id()) == Some(previous)
        })
        .collect();
    if occurrences.is_empty() || !occurrences.iter().any(|node| node.id() == name) {
        return None;
    }
    let allowed = [
        "binary_expression",
        "arguments",
        "return_expression",
        "reference_expression",
        "array_expression",
        "tuple_expression",
        "expression_statement",
        "parenthesized_expression",
    ];
    let mut bytes = Vec::new();
    for node in occurrences {
        let owner = parent(node.id())?;
        let tail = owner.metadata().term() == Some("block")
            && nodes
                .iter()
                .filter(|child| {
                    parent(child.id()).is_some_and(|candidate| candidate.id() == owner.id())
                })
                .max_by_key(|child| {
                    child
                        .metadata()
                        .span()
                        .map(|span| span.byte_range().start())
                })
                .is_some_and(|last| last.id() == node.id());
        if node.id() != name && !allowed.contains(&owner.metadata().term()?) && !tail {
            return None;
        }
        let range = node.metadata().span()?.byte_range();
        if source.get(range.start()..range.end())? != previous {
            return None;
        }
        bytes.push([range.start(), range.end()]);
    }
    let word = regex::Regex::new(&format!(
        "(?:^|[^A-Za-z0-9_]){}(?:$|[^A-Za-z0-9_])",
        regex::escape(previous)
    ))
    .ok()?;
    if nodes.iter().any(|node| {
        node.metadata().term() == Some("macro_invocation")
            && text(node.id()).is_some_and(|value| word.is_match(value))
    }) {
        return None;
    }
    bytes.sort_unstable();
    if bytes.windows(2).any(|pair| pair[0][1] > pair[1][0]) {
        return None;
    }
    let spans: Vec<_> = bytes
        .iter()
        .map(|span| {
            Some([
                source.get(..span[0])?.encode_utf16().count(),
                source.get(..span[1])?.encode_utf16().count(),
            ])
        })
        .collect::<Option<_>>()?;
    let mut content = source.to_owned();
    for span in bytes.iter().rev() {
        content.replace_range(span[0]..span[1], replacement);
    }
    // The platform JS wrapper reparses its rendered candidate; do so here too.
    let after = LinkNetwork::parse(&content, "rust", ParseConfiguration::default());
    if !after.verify_full_match(None).is_clean() || after.reconstruct_text() != content {
        return None;
    }
    Some(RustConstantRenameCandidate {
        kind: "private-constant-alpha-renaming",
        content,
        previous: previous.to_owned(),
        replacement: replacement.to_owned(),
        spans,
        unit: "utf16",
        scope: "one-source-file-private-constant",
        source_syntax: "tree-sitter-clean",
        compilation: "pending",
        write_authority: false,
        runtime_effects: "not-executed",
        unresolved_needs: vec!["trusted-source-read", "owned-write", "native-verification"],
    })
}

#[cfg(test)]
#[path = "../../tests/fixtures/rust_constant_rename-tests.rs"]
mod tests;
#[cfg(test)]
#[path = "../../tests/fixtures/rust_constant_rename-unavailable_tests.rs"]
mod unavailable_tests;
