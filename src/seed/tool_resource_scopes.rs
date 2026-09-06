//! Where an advertised tool's effect lands, read from `data/seed/tool-resource-scopes.lino`.
//!
//! Issue #1075: a router that reads only the name substring `create_file`
//! cannot tell a client-workspace writer from a remote repository connector,
//! so `github.create_file` was selected to write a file into the checkout.

use super::TOOL_RESOURCE_SCOPES_LINO;
use super::parser::{parse_lino, split_pipe_list};

/// The vocabulary that decides scope, in the order it is consulted.
#[derive(Debug, Clone, Default)]
pub struct ToolResourceScopeVocabulary {
    /// Required argument names that prove the effect lands on a remote service.
    pub remote_arguments: Vec<String>,
    /// Namespaces whose tools address a remote service.
    pub remote_namespaces: Vec<String>,
    /// Required argument names that address a live process.
    pub process_arguments: Vec<String>,
    /// Exact tool names that write to a live process.
    pub process_names: Vec<String>,
    /// Arguments that name a resource instead of describing one, and therefore
    /// may never be fabricated to satisfy a schema.
    pub identity_arguments: Vec<String>,
}

#[must_use]
pub fn tool_resource_scope_vocabulary() -> ToolResourceScopeVocabulary {
    parse_tool_resource_scopes(TOOL_RESOURCE_SCOPES_LINO)
}

#[must_use]
pub fn parse_tool_resource_scopes(text: &str) -> ToolResourceScopeVocabulary {
    let tree = parse_lino(text);
    let Some(root) = tree.children.first() else {
        return ToolResourceScopeVocabulary::default();
    };
    let mut vocabulary = ToolResourceScopeVocabulary::default();
    for node in &root.children {
        match (node.name.as_str(), node.id.as_str()) {
            ("scope", "remote_service") => {
                vocabulary.remote_arguments = lowercase_list(node.find_child_value("arguments"));
                vocabulary.remote_namespaces = lowercase_list(node.find_child_value("namespaces"));
            }
            ("scope", "process_input") => {
                vocabulary.process_arguments = lowercase_list(node.find_child_value("arguments"));
                vocabulary.process_names = lowercase_list(node.find_child_value("names"));
            }
            ("identity", _) => {
                vocabulary.identity_arguments = lowercase_list(node.find_child_value("arguments"));
            }
            _ => {}
        }
    }
    vocabulary
}

fn lowercase_list(raw: &str) -> Vec<String> {
    split_pipe_list(raw)
        .iter()
        .map(|entry| entry.to_lowercase())
        .collect()
}
