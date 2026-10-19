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
    crate::portable_string_contracts::join_failure_invitation(body, invitation.trim())
}
