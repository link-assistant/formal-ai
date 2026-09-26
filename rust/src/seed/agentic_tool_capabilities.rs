//! Data-driven agentic capability registry for issue #758.

use super::AGENTIC_TOOL_CAPABILITIES_LINO;
use super::parser::{parse_lino, split_pipe_list};

#[derive(Debug, Clone, Default)]
pub struct AgenticToolCapability {
    pub id: String,
    pub aliases: Vec<String>,
    /// Aliases whose input schema accepts a shell command string.
    ///
    /// Some broad execution tools share the `shell` capability without
    /// accepting `{ "command": ... }`.  Command planning therefore uses this
    /// narrower, data-declared subset instead of guessing from a tool name.
    pub command_aliases: Vec<String>,
    pub cues: Vec<String>,
}

#[must_use]
pub fn agentic_tool_capabilities() -> Vec<AgenticToolCapability> {
    let tree = parse_lino(AGENTIC_TOOL_CAPABILITIES_LINO);
    let Some(root) = tree.children.first() else {
        return Vec::new();
    };
    root.children
        .iter()
        .filter(|node| node.name == "capability")
        .map(|node| {
            let cues = node
                .children
                .iter()
                .find(|child| child.name == "cues")
                .into_iter()
                .flat_map(|group| &group.children)
                .flat_map(|language| split_pipe_list(&language.id))
                .map(|cue| cue.to_lowercase())
                .collect();
            AgenticToolCapability {
                id: node.id.clone(),
                aliases: split_pipe_list(node.find_child_value("aliases")),
                command_aliases: split_pipe_list(node.find_child_value("command_aliases")),
                cues,
            }
        })
        .collect()
}
