//! Deterministic fallback planner for repository change requests (issue #654).
//!
//! Unlike the stored recipe fixtures, this planner derives its target and payload
//! from the formalized request.  The resulting plan is data: it is serialized to
//! Links Notation and written before execution, so the tool transcript is an
//! append-only record of the decision that caused the change.

use std::fmt::Write as _;

use crate::engine::stable_id;
use crate::intent_formalization::formalize_intent;

use super::planner::Capability;

/// Workspace-relative event-log artifact written before a general plan executes.
pub const PLAN_PATH: &str = ".formal-ai/general-change-plan.lino";

/// One ordered, capability-tagged operation in a general change plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneralPlanStep {
    pub capability: Capability,
    pub action: String,
    pub expected_evidence: String,
    pub command: Option<String>,
}

/// A deterministic plan composed from a formalized, previously unrecognised request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneralChangePlan {
    pub id: String,
    pub goal: String,
    pub target: String,
    pub content: String,
    pub steps: Vec<GeneralPlanStep>,
    pub verification_command: String,
}

impl GeneralChangePlan {
    /// Render the plan shape consumed by the driver and documented by the meta fixture.
    #[must_use]
    pub fn links_notation(&self) -> String {
        let mut out = String::from("general_change_plan\n");
        field(&mut out, "id", &self.id);
        field(&mut out, "goal", &self.goal);
        field(&mut out, "target", &self.target);
        for (index, step) in self.steps.iter().enumerate() {
            let _ = writeln!(out, "  step {}", index + 1);
            field_nested(&mut out, "capability", capability_slug(step.capability));
            field_nested(&mut out, "action", &step.action);
            field_nested(&mut out, "expected_evidence", &step.expected_evidence);
            if let Some(command) = &step.command {
                field_nested(&mut out, "command", command);
            }
        }
        field(&mut out, "verification_command", &self.verification_command);
        out
    }
}

/// Compose a safe file-creation plan from arbitrary English or Russian wording.
///
/// The universal intent formalizer supplies the stable impulse identity.  The
/// decomposition is deliberately bounded to requests that state both a relative
/// target file and literal content; ambiguous requests continue to the ordinary
/// solver instead of inventing a patch or shell command.
#[must_use]
pub fn compose_general_change_plan(request: &str) -> Option<GeneralChangePlan> {
    let target = extract_target(request)?;
    let content = extract_content(request)?;
    if !safe_relative_path(&target) {
        return None;
    }
    let intent = formalize_intent(request, language(request), None);
    let verification_command = format!("cat {target}");
    let steps = vec![
        GeneralPlanStep {
            capability: Capability::Write,
            action: format!("append the composed plan to {PLAN_PATH}"),
            expected_evidence: format!("written plan event {}", intent.impulse_id),
            command: None,
        },
        GeneralPlanStep {
            capability: Capability::Write,
            action: format!("write the requested content to {target}"),
            expected_evidence: format!("workspace file {target}"),
            command: None,
        },
        GeneralPlanStep {
            capability: Capability::Run,
            action: String::from("run the request-derived verification command"),
            expected_evidence: content.clone(),
            command: Some(verification_command.clone()),
        },
    ];
    Some(GeneralChangePlan {
        id: stable_id(
            "general_change_plan",
            &format!("{}:{target}:{content}", intent.impulse_id),
        ),
        goal: intent.source_text,
        target,
        content,
        steps,
        verification_command,
    })
}

fn extract_target(request: &str) -> Option<String> {
    let words: Vec<&str> = request.split_whitespace().collect();
    words.iter().enumerate().find_map(|(index, word)| {
        let cleaned = word.trim_matches(|c: char| matches!(c, '`' | '"' | '\'' | ',' | ':' | ';'));
        let previous = index
            .checked_sub(1)
            .and_then(|i| words.get(i))
            .map(|w| w.to_lowercase());
        let looks_like_file = cleaned.contains('.') && !cleaned.contains("://");
        (looks_like_file
            && previous
                .as_deref()
                .is_some_and(|p| ["file", "файл", "in", "в", "create", "создай"].contains(&p)))
        .then(|| cleaned.to_owned())
    })
}

fn extract_content(request: &str) -> Option<String> {
    let lower = request.to_lowercase();
    [
        " containing ",
        " with content ",
        " with text ",
        " содержанием ",
        " текстом ",
    ]
    .iter()
    .find_map(|marker| lower.find(marker).map(|at| (marker, at)))
    .map(|(marker, at)| {
        request[at + marker.len()..]
            .trim()
            .trim_matches(|c: char| matches!(c, '`' | '"' | '\'' | '.' | '。'))
            .to_owned()
    })
    .filter(|content| !content.is_empty())
}

fn safe_relative_path(path: &str) -> bool {
    !path.starts_with('/')
        && !path.starts_with('-')
        && !path.split('/').any(|part| part == ".." || part.is_empty())
        && path
            .chars()
            .all(|c| c.is_alphanumeric() || matches!(c, '/' | '.' | '_' | '-'))
}

const fn capability_slug(capability: Capability) -> &'static str {
    match capability {
        Capability::Search => "Search",
        Capability::Fetch => "Fetch",
        Capability::Read => "Read",
        Capability::Write => "Write",
        Capability::Run => "Run",
    }
}

fn language(request: &str) -> &'static str {
    if request
        .chars()
        .any(|c| ('\u{0400}'..='\u{04ff}').contains(&c))
    {
        "ru"
    } else {
        "en"
    }
}

fn escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

fn field(out: &mut String, name: &str, value: &str) {
    let _ = writeln!(out, "  {name} \"{}\"", escape(value));
}

fn field_nested(out: &mut String, name: &str, value: &str) {
    let _ = writeln!(out, "    {name} \"{}\"", escape(value));
}
