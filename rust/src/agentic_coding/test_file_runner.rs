//! Running a named test file (PR #1188 T91, gap G11).
//!
//! "Run the tests in m.test.mjs." ran the workspace's whole-suite command,
//! chosen by whichever marker file the server's own directory held
//! (`bun test`). A request that names a test file runs that file with the
//! runtime its extension needs, read from the seeded `test_file_runners`
//! group of `data/seed/shell-intents.lino`. Twin of
//! `js/agentic/test_file_runner.mjs`.

use std::sync::OnceLock;

use crate::seed::SHELL_INTENTS_LINO;
use crate::seed::parser::parse_lino;

const WORKSPACE_TEST_COMMAND: &str = "formal-ai:workspace-test";
const FILE_SLOT: &str = concat!("{", "file", "}");

/// The seeded `(extension, command)` runners, in declaration order.
fn runners() -> &'static [(String, String)] {
    static RUNNERS: OnceLock<Vec<(String, String)>> = OnceLock::new();
    RUNNERS.get_or_init(|| {
        let tree = parse_lino(SHELL_INTENTS_LINO);
        tree.children
            .first()
            .and_then(|root| {
                root.children
                    .iter()
                    .find(|group| group.name == "test_file_runners")
            })
            .map(|group| {
                group
                    .children
                    .iter()
                    .filter(|node| node.name == "runner")
                    .map(|node| {
                        (
                            node.find_child_value("extension").to_owned(),
                            node.find_child_value("command").to_owned(),
                        )
                    })
                    .collect()
            })
            .unwrap_or_default()
    })
}

/// The command that runs the named test file for the workspace-test intent.
///
/// `None` when the intent is another one, no file is named, or no runner is
/// seeded for the file's extension.
pub(super) fn test_file_command(intent_command: &str, path: Option<&str>) -> Option<String> {
    if intent_command != WORKSPACE_TEST_COMMAND {
        return None;
    }
    let path = path?;
    let (_, extension) = path.rsplit_once('.')?;
    let extension = extension.to_lowercase();
    runners()
        .iter()
        .find(|(candidate, _)| *candidate == extension)
        .map(|(_, command)| command.replace(FILE_SLOT, path))
}
