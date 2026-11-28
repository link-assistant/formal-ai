use super::*;
use std::fmt::Write as _;

pub(super) fn render_plan_response(meaning: &SoftwareProjectMeaning) -> String {
    let mut body = String::new();
    let _ = writeln!(
        body,
        "Implementation plan pending approval for a {} targeting {}.",
        meaning.artifact, meaning.target
    );
    body.push('\n');
    body.push_str("Formalized meaning:\n```lino\n");
    body.push_str(&meaning.meaning_lino(ApprovalState::Proposed));
    body.push_str("```\n\nReasoning steps:\n");
    for (index, step) in reasoning_steps(meaning).iter().enumerate() {
        let _ = writeln!(body, "{}. {step}", index + 1);
    }
    body.push_str("\nRequirement model:\n");
    for (index, requirement) in meaning.requirements.iter().enumerate() {
        let category = classify_requirement(requirement, meaning.game_tracker);
        let _ = writeln!(body, "{}. [{category}] {requirement}", index + 1);
    }
    body.push_str("\nSubtasks:\n");
    for (index, subtask) in meaning.subtasks.iter().enumerate() {
        let _ = writeln!(
            body,
            "{}. {} -> {}",
            index + 1,
            subtask.requirement_id,
            subtask.title
        );
    }
    body.push_str("\nApproval gates:\n");
    for gate in &meaning.approval_gates {
        let _ = writeln!(body, "- {gate}");
    }
    body.push_str("\nProposed plan:\n");
    for (index, step) in plan_steps(meaning).iter().enumerate() {
        let _ = writeln!(body, "{}. {step}", index + 1);
    }
    body.push_str(
        "\nReply `approve plan` to generate the starter implementation, or describe what to change.",
    );
    body
}
