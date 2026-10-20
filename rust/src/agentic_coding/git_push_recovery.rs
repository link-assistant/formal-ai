//! Local refspec errors require a current-branch observation, never a web search.
use super::planner::{AgenticPlan, Capability, plan_one, tool_for};
use super::progress::Progress;
use crate::protocol::ChatMessage;
use serde_json::json;
pub(super) const CURRENT_BRANCH_PUSH: &str = r#"branch=$(git branch --show-current) && test -n "$branch" && git check-ref-format --branch "$branch" >/dev/null && git push -q origin "HEAD:refs/heads/$branch" && git rev-parse HEAD"#;
pub(super) fn recover(
    messages: &[ChatMessage],
    tools: &[&str],
    failure: &super::repair_loop::FailedStep,
) -> Option<AgenticPlan> {
    let markers: Vec<String> = serde_json::from_str(&super::work_item_steps::fill(
        "refspec-failure-markers",
        &[],
    ))
    .expect("valid refspec observation markers");
    let command = failure.failed_command.as_deref().unwrap_or_default();
    let push_pattern = regex::Regex::new(&super::work_item_steps::fill(
        "git-push-command-pattern",
        &[],
    ))
    .expect("valid Git push observation pattern");
    if !markers
        .iter()
        .all(|marker| failure.reported.contains(marker.as_str()))
        || !push_pattern.is_match(command)
    {
        return None;
    }
    let origin_pattern = regex::Regex::new(&super::work_item_steps::fill(
        "origin-push-command-pattern",
        &[],
    ))
    .expect("valid origin observation pattern");
    if !origin_pattern.is_match(command) {
        return Some(AgenticPlan::Final("The push failed with an invalid local refspec. Inspect the current branch and the originally requested remote before retrying; no alternative remote is authorized.".to_owned()));
    }
    if messages.iter().any(|message| {
        message.role == "user"
            && !matches!(
                super::git_commit::request_target_binding(
                    &message.content.plain_text(),
                    "branch-binding-patterns",
                    super::git_commit::valid_branch_target
                ),
                super::git_commit::RequestTargetBinding::Absent
            )
    }) {
        return Some(AgenticPlan::Final("The refspec failed while an explicit branch binding exists. Inspect that local ref and preserve the requested destination; the current branch was not pushed automatically.".to_owned()));
    }
    if Progress::scan(messages).has_run(CURRENT_BRANCH_PUSH) {
        return Some(AgenticPlan::Final("The current-branch push recovery was attempted. Inspect its Git receipt; any remaining task requirements are incomplete.".to_owned()));
    }
    Some(tool_for(tools, Capability::Run).map_or_else(
        || AgenticPlan::Final("The push failed with an invalid local refspec. Read git branch --show-current in the client workspace before retrying; no shell tool is available.".to_owned()),
        |tool| plan_one(tool, json!({"command": CURRENT_BRANCH_PUSH}).to_string())))
}
