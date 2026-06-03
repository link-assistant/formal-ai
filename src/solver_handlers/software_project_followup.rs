//! Follow-up handler for an active software-project dialogue (issue #341).
//!
//! Once a software-project plan is on the table, a decomposed agent step such as
//! "test it by scraping wikipedia.org and show me the top 10 most frequent
//! words" should stay bound to that project rather than spawning a fresh fact
//! lookup. This module recovers the active dialogue from the conversation log
//! and formalizes the verification/execution/demonstration request into its own
//! Links Notation meaning record, behind code-execution and network gates.

use std::fmt::Write as _;

use crate::engine::{normalize_prompt, stable_id, SymbolicAnswer};
use crate::event_log::EventLog;
use crate::seed;
use crate::solver_handlers::finalize_simple;
use crate::solver_helpers::{last_assistant_turn, last_user_turn};

use super::software_project::{is_approval_prompt, lino_string, SoftwareProjectMeaning};

/// Kinds of follow-up that exercise an already-designed artifact. The order in
/// [`follow_up_kind`] gives verification precedence over plain execution so a
/// "test it and run it" phrasing is recorded as the stronger verification goal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FollowUpKind {
    Verification,
    Execution,
    Demonstration,
}

impl FollowUpKind {
    const fn label(self) -> &'static str {
        match self {
            Self::Verification => "verification",
            Self::Execution => "execution",
            Self::Demonstration => "demonstration",
        }
    }

    const fn action(self) -> &'static str {
        match self {
            Self::Verification => "test",
            Self::Execution => "run",
            Self::Demonstration => "show",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SoftwareProjectFollowUp {
    kind: FollowUpKind,
    target_site: Option<String>,
    expected_output: Option<String>,
}

/// Follow-up handler for an active software-project dialogue (issue #341).
///
/// Runs before `concept_lookup` so a step like "test it by scraping
/// wikipedia.org and show me the top 10 most frequent words" stays bound to the
/// project instead of resolving the `wikipedia` concept. It only fires when the
/// previous assistant turn already formalized a `software_project_request`, so
/// unrelated prompts are untouched.
pub fn try_software_project_followup(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let canonical = normalize_prompt(prompt);
    let normalized = if canonical.is_empty() {
        normalized
    } else {
        canonical.as_str()
    };

    // Approval prompts ("approve plan", "yes", ...) stay with the main request
    // handler, which advances the dialogue to the implementation starter.
    if is_approval_prompt(normalized) {
        return None;
    }

    let (meaning, approved) = prior_software_project_dialogue(log)?;
    let follow_up = detect_follow_up(prompt, normalized)?;
    record_follow_up(log, &meaning, &follow_up, approved);
    let body = render_follow_up_response(&meaning, &follow_up, approved);
    Some(finalize_simple(
        prompt,
        log,
        "software_project_followup",
        "response:software_project_followup",
        &body,
        0.74,
    ))
}

fn detect_follow_up(prompt: &str, normalized: &str) -> Option<SoftwareProjectFollowUp> {
    let kind = follow_up_kind(normalized)?;
    Some(SoftwareProjectFollowUp {
        kind,
        target_site: extract_target_site(prompt),
        expected_output: extract_expected_output(prompt),
    })
}

/// Recognise which follow-up a prompt evidences by *meaning*, not a hardcoded
/// per-language marker table (issue #386).
///
/// Each follow-up kind is a self-describing meaning in
/// `data/seed/meanings-software-project.lino`; its surface words — in every
/// supported language — live there, while this code knows only the concepts and
/// their precedence. Verification outranks execution, which outranks
/// demonstration, so a combined "test it and run it" records the stronger goal
/// (preserving the former marker-table ordering). A multilingual user who
/// designs in one language and then says "now test it" in another (issue #341)
/// still stays inside the project dialogue.
///
/// Surface words are matched as raw substrings of the normalized prompt — not
/// whole whitespace tokens — because many are multi-word phrases ("run the
/// tests", "show me"); a token-boundary match would never find them.
fn follow_up_kind(normalized: &str) -> Option<FollowUpKind> {
    for (role, kind) in [
        (
            seed::ROLE_SOFTWARE_FOLLOWUP_VERIFICATION,
            FollowUpKind::Verification,
        ),
        (
            seed::ROLE_SOFTWARE_FOLLOWUP_EXECUTION,
            FollowUpKind::Execution,
        ),
        (
            seed::ROLE_SOFTWARE_FOLLOWUP_DEMONSTRATION,
            FollowUpKind::Demonstration,
        ),
    ] {
        let mentioned = seed::lexicon().meanings_with_role(role).any(|meaning| {
            meaning
                .words()
                .any(|word| !word.is_empty() && normalized.contains(word))
        });
        if mentioned {
            return Some(kind);
        }
    }
    None
}

/// Pull the first domain-like token (e.g. `wikipedia.org`) out of the prompt so
/// the follow-up records the concrete test target instead of guessing.
fn extract_target_site(prompt: &str) -> Option<String> {
    for raw in prompt.split_whitespace() {
        let token = raw.trim_matches(|character: char| !character.is_ascii_alphanumeric());
        if !token.contains('.') {
            continue;
        }
        let mut parts = token.rsplitn(2, '.');
        let tld = parts.next().unwrap_or("");
        let host = parts.next().unwrap_or("");
        if tld.len() >= 2
            && tld.chars().all(|character| character.is_ascii_alphabetic())
            && host
                .chars()
                .any(|character| character.is_ascii_alphabetic())
        {
            return Some(token.to_lowercase());
        }
    }
    None
}

/// Capture the clause after a show-me/print/display opener so the follow-up
/// records what the user wants surfaced (e.g. "the top 10 most frequent words").
///
/// The recognized openers carry the [`seed::ROLE_OUTPUT_DISPLAY_REQUEST`] role;
/// each is a prefix whose text before the `…` slot is the marker, tried in
/// declaration order so the longer "show me " wins over the bare "show ". A
/// marker is matched anywhere in the prompt, and the clause that follows — read
/// from the original-case prompt, stopped at the first sentence-ending
/// punctuation and capped at twelve words — is returned. No per-language marker
/// list lives here; the surfaces come from
/// `data/seed/meanings-software-project.lino`.
fn extract_expected_output(prompt: &str) -> Option<String> {
    let lower = prompt.to_lowercase();
    let forms = seed::lexicon().role_word_forms(seed::ROLE_OUTPUT_DISPLAY_REQUEST);
    for form in &forms {
        let marker = form.before_slot();
        let Some(start) = lower.find(marker).map(|index| index + marker.len()) else {
            continue;
        };
        let tail = &prompt[start..];
        let stop = tail.find(['.', '?', '\n', ';']).unwrap_or(tail.len());
        let clause = tail[..stop]
            .split_whitespace()
            .take(12)
            .collect::<Vec<_>>()
            .join(" ");
        if !clause.is_empty() {
            return Some(clause);
        }
    }
    None
}

fn record_follow_up(
    log: &mut EventLog,
    meaning: &SoftwareProjectMeaning,
    follow_up: &SoftwareProjectFollowUp,
    approved: bool,
) {
    log.append("formalization", "text_to_links_notation".to_owned());
    log.append("meaning", follow_up_meaning_id(meaning, follow_up));
    log.append("software_project:parent", meaning.meaning_id());
    log.append(
        "software_project:follow_up_kind",
        follow_up.kind.label().to_owned(),
    );
    if let Some(site) = &follow_up.target_site {
        log.append("software_project:target_site", site.clone());
    }
    if let Some(output) = &follow_up.expected_output {
        log.append("software_project:expected_output", output.clone());
    }
    log.append(
        "approval_state",
        if approved { "approved" } else { "proposed" }.to_owned(),
    );
    for gate in follow_up_gates() {
        log.append("approval_gate", gate.to_owned());
    }
}

fn follow_up_meaning_id(
    meaning: &SoftwareProjectMeaning,
    follow_up: &SoftwareProjectFollowUp,
) -> String {
    let key = format!(
        "parent={};kind={};site={};output={}",
        meaning.meaning_id(),
        follow_up.kind.label(),
        follow_up.target_site.as_deref().unwrap_or(""),
        follow_up.expected_output.as_deref().unwrap_or(""),
    );
    stable_id("software_project_followup", &key)
}

const fn follow_up_gates() -> [&'static str; 3] {
    ["generated_code", "test_execution", "network_access"]
}

fn follow_up_reasoning_steps(
    meaning: &SoftwareProjectMeaning,
    follow_up: &SoftwareProjectFollowUp,
) -> Vec<String> {
    let mut steps = vec![format!(
        "Recognize \"{}\" as a {} request that exercises the {} from the active plan, not a fact lookup.",
        follow_up.kind.action(),
        follow_up.kind.label(),
        meaning.artifact
    )];
    if let Some(site) = &follow_up.target_site {
        steps.push(format!(
            "Bind the test target to {site} and keep live fetches behind the network_access gate.",
        ));
    }
    if let Some(output) = &follow_up.expected_output {
        steps.push(format!(
            "Record the expected output as \"{output}\" so the test harness can assert it.",
        ));
    }
    steps.push(String::from(
        "Drive the artifact through a deterministic fixture before any host API or network call.",
    ));
    steps.push(String::from(
        "Keep code execution behind approval gates because the sandbox cannot run untrusted code.",
    ));
    steps
}

fn follow_up_plan_steps(
    meaning: &SoftwareProjectMeaning,
    follow_up: &SoftwareProjectFollowUp,
) -> Vec<String> {
    let site = follow_up
        .target_site
        .clone()
        .unwrap_or_else(|| String::from("the requested target"));
    let mut steps = vec![
        format!(
            "Generate the {} core plus a deterministic test harness with a captured {site} fixture.",
            meaning.artifact
        ),
        String::from(
            "Assert each requirement (parsing, extraction, counting, summary) against the fixture.",
        ),
    ];
    if let Some(output) = &follow_up.expected_output {
        steps.push(format!("Surface {output} from the fixture run."));
    }
    steps.push(format!(
        "Run the {} test command once the generated_code gate is approved.",
        meaning.implementation_language
    ));
    steps.push(format!(
        "Promote the run to live {site} only after the test_execution and network_access gates pass.",
    ));
    steps
}

fn render_follow_up_response(
    meaning: &SoftwareProjectMeaning,
    follow_up: &SoftwareProjectFollowUp,
    approved: bool,
) -> String {
    let mut body = String::new();
    let _ = writeln!(
        body,
        "Recorded a {} follow-up for the {} from the active plan.",
        follow_up.kind.label(),
        meaning.artifact
    );
    body.push('\n');
    body.push_str("Formalized meaning:\n```lino\n");
    body.push_str("software_project_followup\n");
    let _ = writeln!(
        body,
        "  parent_request {}",
        lino_string(&meaning.meaning_id())
    );
    let _ = writeln!(body, "  parent_artifact {}", lino_string(meaning.artifact));
    let _ = writeln!(body, "  action {}", lino_string(follow_up.kind.action()));
    let _ = writeln!(body, "  follow_up_kind {}", follow_up.kind.label());
    if let Some(site) = &follow_up.target_site {
        let _ = writeln!(body, "  target_site {}", lino_string(site));
    }
    if let Some(output) = &follow_up.expected_output {
        let _ = writeln!(body, "  expected_output {}", lino_string(output));
    }
    let _ = writeln!(body, "  delivery_mode {}", meaning.delivery_mode_label());
    let _ = writeln!(
        body,
        "  implementation_language {}",
        lino_string(meaning.implementation_language)
    );
    let _ = writeln!(
        body,
        "  approval_state {}",
        if approved { "approved" } else { "proposed" }
    );
    body.push_str("  approval_required true\n");
    for gate in follow_up_gates() {
        let _ = writeln!(body, "  approval_gate {}", lino_string(gate));
    }
    body.push_str("```\n\nReasoning steps:\n");
    for (index, step) in follow_up_reasoning_steps(meaning, follow_up)
        .iter()
        .enumerate()
    {
        let _ = writeln!(body, "{}. {step}", index + 1);
    }
    body.push_str("\nVerification plan:\n");
    for (index, step) in follow_up_plan_steps(meaning, follow_up).iter().enumerate() {
        let _ = writeln!(body, "{}. {step}", index + 1);
    }
    body.push('\n');
    if approved {
        body.push_str(
            "The plan is approved, so the generated starter already includes this test harness. \
             Running it live needs the test_execution and network_access gates.",
        );
    } else {
        body.push_str(
            "Reply `approve plan` to generate the artifact plus this test harness. Running it live \
             against the target needs the test_execution and network_access gates.",
        );
    }
    body
}

/// Recover the active software-project dialogue from the conversation log,
/// regardless of whether the plan has been approved yet. Returns the recovered
/// meaning together with a flag describing whether the prior assistant turn
/// already produced an approved implementation starter.
///
/// Issue #341: a decomposed agent step such as "test it by scraping
/// wikipedia.org and show me the top 10 most frequent words" arrives while a
/// software-project plan is still on the table. Without this recovery the step
/// was misrouted to a `wikipedia` concept lookup (online) or the unknown opener
/// (offline) instead of staying inside the project dialogue.
fn prior_software_project_dialogue(log: &EventLog) -> Option<(SoftwareProjectMeaning, bool)> {
    let assistant = last_assistant_turn(log)?;
    if !assistant.contains("software_project_request") {
        return None;
    }
    let approved = assistant.contains("approval_state approved");
    let prior_prompt = last_user_turn(log)?;
    let normalized = normalize_prompt(prior_prompt);
    let meaning = SoftwareProjectMeaning::from_prompt(prior_prompt, &normalized)?;
    Some((meaning, approved))
}
