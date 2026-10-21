//! Source-bound work-item receipt payloads and owned trailing status.

/// The exit status a read command printed about itself, when it printed one.
fn exit_sentinel(raw: &str) -> Option<i32> {
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
    if command.contains("__formal_ai_exit=%s")
        && exit_sentinel(text).is_some()
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
    let status = command
        .contains("__formal_ai_exit=%s")
        .then(|| exit_sentinel(&payload))
        .flatten();
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
