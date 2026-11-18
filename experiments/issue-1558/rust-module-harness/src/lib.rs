//! Issue #1558: a harness for `work_item_references.rs`, the pure half of the fix.
//! The module and its test file are included unchanged; only the three sibling
//! modules it names through `super::` are replaced. `source_read_path` and
//! `command_argument` are copied from the crate; the rest are minimal shims.

pub mod planner {
    /// The capabilities `work_item_references` names.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Capability {
        Read,
        Run,
        Glob,
    }

    /// Shim: map the Agent CLI tool names onto a capability.
    pub fn tool_for<'a>(tools: &[&'a str], capability: Capability) -> Option<&'a str> {
        tools.iter().copied().find(|name| match capability {
            Capability::Read => *name == "read",
            Capability::Run => *name == "bash",
            Capability::Glob => *name == "glob",
        })
    }
}

pub mod tool_result {
    use serde_json::Value;

    /// The fields `read_outcome` inspects.
    pub struct SourceReadObservation {
        pub complete: bool,
        pub absent: bool,
        pub source: Option<String>,
        pub error: Option<String>,
    }

    /// Copied from `rust/src/agentic_coding/tool_result.rs`.
    pub fn command_argument(arguments: &str) -> Option<String> {
        let value: Value = serde_json::from_str(arguments).ok()?;
        ["command", "cmd", "script"]
            .iter()
            .find_map(|key| match value.get(*key) {
                Some(Value::String(command)) => Some(command.clone()),
                Some(Value::Array(words)) => {
                    let joined = words
                        .iter()
                        .filter_map(Value::as_str)
                        .collect::<Vec<_>>()
                        .join(" ");
                    (!joined.is_empty()).then_some(joined)
                }
                _ => None,
            })
    }
}

pub mod progress {
    use super::planner::Capability;
    use super::tool_result::SourceReadObservation;

    /// Shim of the transcript attempt: only the fields the module reads.
    pub struct ToolAttempt {
        pub capability: Capability,
        pub source_read: Option<SourceReadObservation>,
        pub succeeded: bool,
        pub detail: String,
        pub arguments: Option<String>,
    }

    impl ToolAttempt {
        pub const fn source_observation(&self) -> Option<&SourceReadObservation> {
            self.source_read.as_ref()
        }
    }

    /// Shim of the transcript scan: the attempts in order.
    pub struct Progress {
        pub attempts: Vec<ToolAttempt>,
    }

    impl Progress {
        pub fn research_attempts(&self) -> &[ToolAttempt] {
            &self.attempts
        }
    }

    /// Copied from `rust/src/agentic_coding/progress.rs`.
    pub fn source_read_path(arguments: &str) -> Option<String> {
        let value: serde_json::Value = serde_json::from_str(arguments).ok()?;
        let object = value.as_object()?;
        let fields = ["path", "filePath", "file_path"]
            .iter()
            .filter_map(|key| object.get(*key))
            .collect::<Vec<_>>();
        let first = fields.first()?.as_str()?;
        (!first.trim().is_empty() && fields.iter().all(|field| field.as_str() == Some(first)))
            .then(|| first.to_owned())
    }
}

#[path = "../../../../rust/src/agentic_coding/work_item_references.rs"]
pub mod work_item_references;

/// Root causes A and B of #1558 against the real functions: a planned relative
/// read matches the read the client bound to the workspace, and a `File not found`
/// answer is a failed read.
#[cfg(test)]
mod dedup {
    use super::planner::Capability;
    use super::progress::{Progress, ToolAttempt};
    use super::tool_result::SourceReadObservation;
    use super::work_item_references::{
        ReadOutcome, WorkItemReference, attempt_for, read_attempt_for, read_outcome,
        search_actions, work_item_references, workspace_roots,
    };

    const WORKSPACE: &str = "/tmp/gh-issue-solver-1";

    fn attempt(
        capability: Capability,
        arguments: &str,
        detail: &str,
        observed: Option<SourceReadObservation>,
    ) -> ToolAttempt {
        ToolAttempt {
            capability,
            source_read: observed,
            succeeded: true,
            detail: detail.to_owned(),
            arguments: Some(arguments.to_owned()),
        }
    }

    fn observation(source: Option<&str>) -> SourceReadObservation {
        SourceReadObservation {
            complete: source.is_some(),
            absent: false,
            source: source.map(str::to_owned),
            error: None,
        }
    }

    fn roots() -> Vec<String> {
        workspace_roots(Some(WORKSPACE.to_owned()), Some("/home/box".to_owned()))
    }

    #[test]
    fn a_read_bound_to_the_workspace_answers_the_planned_relative_read() {
        let progress = Progress {
            attempts: vec![attempt(
                Capability::Read,
                &format!(r#"{{"filePath":"{WORKSPACE}/scripts/hive-delivery-attempt.mjs"}}"#),
                "1: x",
                Some(observation(Some("x"))),
            )],
        };
        assert!(
            read_attempt_for(&progress, "scripts/hive-delivery-attempt.mjs", &roots()).is_some()
        );
        assert!(read_attempt_for(&progress, "hive-delivery-attempt.mjs", &roots()).is_none());
        let dotted = Progress {
            attempts: vec![attempt(
                Capability::Read,
                r#"{"path":"./a/b.mjs"}"#,
                "",
                Some(observation(Some("x"))),
            )],
        };
        assert!(read_attempt_for(&dotted, "a/b.mjs", &roots()).is_some());
    }

    #[test]
    fn file_not_found_is_a_failed_read_and_source_is_complete() {
        let missing = attempt(
            Capability::Read,
            r#"{"filePath":"/w/Node.js"}"#,
            "Error: File not found: /tmp/gh-issue-solver-1/Node.js",
            Some(observation(None)),
        );
        assert_eq!(read_outcome(&missing), ReadOutcome::Failed);
        let unobserved = attempt(Capability::Read, "{}", "", None);
        assert_eq!(read_outcome(&unobserved), ReadOutcome::Failed);
        let read = attempt(Capability::Read, "{}", "1: x", Some(observation(Some("x"))));
        assert_eq!(read_outcome(&read), ReadOutcome::Complete);
    }

    #[test]
    fn a_search_already_answered_is_found_by_pattern_or_command() {
        let reference: WorkItemReference = work_item_references(
            "Change `hive-delivery-attempt.mjs` so it retries.",
            "https://github.com/o/r/issues/1",
        )
        .into_iter()
        .next()
        .expect("one reference");
        let actions = search_actions(&reference, &["read", "glob", "bash"]);
        assert_eq!(
            actions
                .iter()
                .map(|action| action.capability)
                .collect::<Vec<_>>(),
            [Capability::Glob, Capability::Run]
        );
        assert!(
            actions[1]
                .arguments
                .contains("-name 'hive-delivery-attempt.mjs'")
        );
        let progress = Progress { attempts: vec![
            attempt(Capability::Glob, r#"{"path":"/w","pattern":"**/hive-delivery-attempt.mjs"}"#, "/w/scripts/hive-delivery-attempt.mjs", None),
            attempt(Capability::Run, &serde_json::json!({"command": serde_json::from_str::<serde_json::Value>(&actions[1].arguments).unwrap()["command"], "description": "find"}).to_string(), "", None),
        ] };
        assert!(
            attempt_for(&progress, &actions[0])
                .is_some_and(|found| found.capability == Capability::Glob)
        );
        assert!(
            attempt_for(&progress, &actions[1])
                .is_some_and(|found| found.capability == Capability::Run)
        );
        assert!(
            attempt_for(
                &Progress {
                    attempts: Vec::new()
                },
                &actions[0]
            )
            .is_none()
        );
    }
}
