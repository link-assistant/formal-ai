//! Resume Hive Mind's existing working tree instead of reimplementing its issue.
use super::planner::{AgenticPlan, Capability, plan_one, tool_for};
use super::progress::Progress;
use crate::protocol::ChatMessage;
use serde_json::json;

/// Only the explicitly marked fenced porcelain listing is authoritative.
fn changed_paths(task: &str) -> Option<Vec<String>> {
    let (_, rest) = task.split_once("UNCOMMITTED CHANGES DETECTED")?;
    let (_, fenced) = rest.split_once("```")?;
    let (_, body) = fenced.split_once('\n')?;
    let (body, _) = body.split_once("```")?;
    let mut paths = Vec::new();
    for line in body.lines().filter(|line| !line.trim().is_empty()) {
        if line.len() < 4 || !line.is_char_boundary(3) {
            return None;
        }
        let status = &line[..2];
        if !status.bytes().all(|value| b" MADRCU?!".contains(&value)) {
            return None;
        }
        let path = &line[3..];
        // Quoted and rename records need a real porcelain parser; never guess.
        if path.is_empty()
            || path.starts_with('"')
            || path.contains(" -> ")
            || path.contains('\0')
            || std::path::Path::new(path).is_absolute()
            || std::path::Path::new(path)
                .components()
                .any(|part| matches!(part, std::path::Component::ParentDir))
        {
            return None;
        }
        if !paths.iter().any(|existing| existing == path) {
            paths.push(path.to_owned());
        }
    }
    (!paths.is_empty()).then_some(paths)
}

pub(super) fn feedback_needs_changes(output: &str) -> bool {
    let text =
        super::tool_result::shell_step(output).map_or_else(|| output.to_owned(), |step| step.text);
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) else {
        return true;
    };
    if !value
        .get("comments")
        .is_some_and(serde_json::Value::is_array)
        || !value
            .get("reviews")
            .is_some_and(serde_json::Value::is_array)
    {
        return true;
    }
    value
        .get("comments")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|items| {
            items.iter().any(|item| {
                item.get("body")
                    .and_then(serde_json::Value::as_str)
                    .is_some_and(|body| !body.trim().is_empty())
            })
        })
        || value
            .get("reviews")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|items| {
                items.iter().any(|item| {
                    item.get("state").and_then(serde_json::Value::as_str)
                        == Some("CHANGES_REQUESTED")
                })
            })
}

pub(super) fn plan_restart(
    task: &str,
    messages: &[ChatMessage],
    tools: &[&str],
) -> Option<AgenticPlan> {
    let paths = changed_paths(task)?;
    let reference = task
        .split_whitespace()
        .find(|word| word.starts_with("https://github.com/") && word.contains("/pull/"))
        .map(|word| word.trim_matches(|character| matches!(character, '`' | ')' | ',' | '.')))?;
    let branch = super::git_commit::named_branch(task).unwrap_or_else(|| "HEAD".to_owned());
    let quote = super::git_commit::shell_quote;
    let files = paths
        .iter()
        .map(|path| quote(path))
        .collect::<Vec<_>>()
        .join(" ");
    let target = quote(reference);
    let commit = super::work_item_steps::fill(
        "recipe_commit_command",
        &[
            (concat!("{", "files}"), &files),
            ("{subject}", &quote("fix: complete prepared work")),
            (
                concat!("{", "body}"),
                &quote(&super::git_commit::resolves_body(reference)),
            ),
            (concat!("{", "branch}"), &quote(&branch)),
        ],
    );
    let changed = paths
        .iter()
        .map(|path| format!("- `{path}`"))
        .collect::<Vec<_>>()
        .join("\n");
    let body = super::work_item_steps::fill(
        "prepared_work_pr_body",
        &[
            (concat!("{", "branch}"), branch.as_str()),
            (concat!("{", "reference}"), reference),
            (concat!("{", "paths}"), changed.as_str()),
        ],
    );
    let commands = [
        super::work_item_steps::fill("pr_comments_command", &[(concat!("{", "target}"), &target)]),
        "git diff --check && git diff --cached --check".to_owned(),
        commit,
        super::work_item_steps::fill(
            "pr_edit_command",
            &[
                (concat!("{", "target}"), &target),
                (concat!("{", "body}"), &quote(&body)),
            ],
        ),
        super::work_item_steps::fill("pr_ready_command", &[(concat!("{", "target}"), &target)]),
    ];
    let progress = Progress::scan(messages);
    let run = tool_for(tools, Capability::Run)?;
    if let Some(output) = progress.latest_successful_run_output_for(&commands[0])
        && feedback_needs_changes(output)
    {
        return Some(AgenticPlan::Final(super::work_item_steps::fill(
            "prepared_work_feedback_report",
            &[(concat!("{", "output}"), output)],
        )));
    }
    for command in &commands {
        if progress.successful_run_count_for(command) > 0 {
            continue;
        }
        if let Some(output) = progress.latest_run_output_for(command) {
            return Some(AgenticPlan::Final(super::work_item_steps::fill(
                "prepared_work_stopped_report",
                &[
                    (concat!("{", "command}"), command.as_str()),
                    (concat!("{", "output}"), output.as_str()),
                ],
            )));
        }
        return Some(plan_one(run, json!({"command": command}).to_string()));
    }
    Some(AgenticPlan::Final(super::work_item_steps::fill(
        "prepared_work_ready_report",
        &[
            (concat!("{", "branch}"), branch.as_str()),
            (concat!("{", "reference}"), reference),
        ],
    )))
}

#[cfg(test)]
mod tests {
    use super::changed_paths;
    #[test]
    fn only_the_marked_fenced_listing_is_consumed() {
        assert_eq!(
            changed_paths(
                "⚠️ UNCOMMITTED CHANGES DETECTED\n```text\n?? Main.kt\n M README.md\n```"
            ),
            Some(vec!["Main.kt".to_owned(), "README.md".to_owned()])
        );
        assert_eq!(
            changed_paths("UNCOMMITTED CHANGES DETECTED\n```\n?? ../outside\n```"),
            None
        );
        assert_eq!(changed_paths("?? Main.kt"), None);
    }
}

#[cfg(test)]
mod feedback_tests {
    use super::feedback_needs_changes;
    #[test]
    fn unreadable_or_requested_feedback_blocks_readiness() {
        assert!(!feedback_needs_changes(r#"{"comments":[],"reviews":[]}"#));
        assert!(feedback_needs_changes(
            r#"{"comments":[{"body":"Fix the output"}],"reviews":[]}"#
        ));
        assert!(feedback_needs_changes(
            r#"{"comments":[],"reviews":[{"state":"CHANGES_REQUESTED"}]}"#
        ));
        assert!(feedback_needs_changes("{}"));
        assert!(feedback_needs_changes("authentication failed"));
    }
}
