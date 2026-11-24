//! Source-bound work-item receipt payloads and owned trailing status.

/// The exit status a read command printed about itself, when it printed one.
fn exit_sentinel(raw: &str, command: &str) -> Option<i32> {
    command.contains("__formal_ai_exit=%s").then_some(())?;
    raw.lines()
        .rev()
        .find(|line| !line.trim().is_empty())?
        .trim()
        .strip_prefix("__formal_ai_exit=")?
        .parse()
        .ok()
}

/// The output of a read without the sentinel lines it printed about itself —
/// the issue text is the payload, not the plumbing.
/// Only the owned trailing status is removed; literal examples remain bytes.
pub(super) fn without_exit_sentinel(text: &str, command: &str) -> String {
    let mut lines = text.lines().collect::<Vec<_>>();
    if exit_sentinel(text, command).is_some()
        && let Some(index) = lines.iter().rposition(|line| !line.trim().is_empty())
    {
        lines.remove(index);
    }
    lines.join("\n")
}

/// Why a work-item read's output is not page evidence, when it is not.
pub(super) fn work_item_read_failure_reason(
    command: &str,
    raw: &str,
    echoed_failure: Option<&str>,
) -> Option<String> {
    if let Some(failure) = echoed_failure {
        return Some(failure.to_owned());
    }
    let payload = super::super::tool_result::observed_payload(raw).unwrap_or_default();
    let status = exit_sentinel(&payload, command);
    if let Some(status) = status
        && status != 0
    {
        return Some(super::super::work_item_steps::fill(
            "read_failure_exit_status",
            &[(concat!("{", "status}"), status.to_string().as_str())],
        ));
    }
    let text = without_exit_sentinel(&payload, command);
    let shaped = if command.starts_with("curl ") {
        !text.trim().is_empty() && !text.trim_start().starts_with('{')
    } else {
        text.contains("\n\n")
    };
    (!shaped).then(|| {
        if command.starts_with("curl ") {
            "the REST answer was not issue text".to_owned()
        } else {
            "the output did not have the title-and-body shape of a work item".to_owned()
        }
    })
}

fn is_issue_view_command(command: &str) -> bool {
    let mut words = command.split_whitespace();
    words.next() == Some("gh")
        && matches!(words.next(), Some("issue" | "pr"))
        && words.next() == Some("view")
}

/// Whether a shell command is one of the reads this planner plans for a work
/// item: the `gh` view read, or a REST fallback of issue #1155.
pub(super) fn is_work_item_read_command(command: &str) -> bool {
    is_issue_view_command(command)
        || (command.starts_with("gh api ") || command.starts_with("curl "))
            && rest_read_url(command).is_some()
}

/// The GitHub work-item URL a shell read command targets, normalized to its `github.com` form.
pub(super) fn work_item_read_url(command: &str) -> Option<String> {
    issue_view_command_url(command).or_else(|| rest_read_url(command))
}

/// The work-item URL of a `gh issue|pr view <url> …` command, when it is one.
pub(super) fn issue_view_command_url(command: &str) -> Option<String> {
    let mut words = command.split_whitespace();
    if words.next()? != "gh" {
        return None;
    }
    if !matches!(words.next()?, "issue" | "pr") || words.next()? != "view" {
        return None;
    }
    words
        .next()
        .and_then(super::super::general_planner::repository_work_reference)
}

/// The work-item URL a `repos/{owner}/{repo}/{kind}/{number}` API path reads.
pub(super) fn rest_read_url(command: &str) -> Option<String> {
    let start = command.find("repos/")? + "repos/".len();
    let mut segments = command[start..].split('/');
    let owner = segments.next().filter(|owner| !owner.is_empty())?;
    let repo = segments.next().filter(|repo| !repo.is_empty())?;
    let kind = match segments.next()? {
        "issues" => "issues",
        "pulls" => "pulls",
        _ => return None,
    };
    // The number segment runs into the `; printf` sentinel the read commands
    // carry, so only its leading digits are the number.
    let number: String = segments
        .next()?
        .chars()
        .take_while(char::is_ascii_digit)
        .collect();
    if number.is_empty() {
        return None;
    }
    // GitHub prints an issue as `/issues/{n}` and a pull request as
    // `/pull/{n}`; folding `issues` onto a singular `issue` named a page no
    // plan targets, so the REST reads were never matched to their work item.
    let kind = if kind == "pulls" { "pull" } else { "issues" };
    Some(format!("https://github.com/{owner}/{repo}/{kind}/{number}"))
}
