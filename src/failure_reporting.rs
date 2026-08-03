//! Seed-backed proactive reporting prompts for failures detected by formal-ai.

/// Append the localized opt-in reporting prompt without changing the original
/// diagnostic text. The existing `report_issue` intent still owns confirmation
/// and report construction; this helper only invites the user to start it.
pub fn append_invitation(body: &str, language: &str) -> String {
    let body = body.trim_end();
    let Some(invitation) =
        crate::seed::localized_response("detected_failure_report_invitation", language)
    else {
        return body.to_owned();
    };
    let invitation = invitation.trim();
    if invitation.is_empty() {
        return body.to_owned();
    }
    if body.is_empty() {
        invitation.to_owned()
    } else {
        let mut answer = String::with_capacity(body.len() + invitation.len() + 2);
        answer.push_str(body);
        answer.push_str("\n\n");
        answer.push_str(invitation);
        answer
    }
}
