//! Follow-up handler for an active software-project dialogue (issue #341).
//!
//! Once a software-project plan is on the table, a decomposed agent step such as
//! "test it by scraping wikipedia.org and show me the top 10 most frequent
//! words" should stay bound to that project rather than spawning a fresh fact
//! lookup. This module recovers the active dialogue from the conversation log
//! and formalizes the verification/execution/demonstration request into its own
//! Links Notation meaning record, behind code-execution and network gates.

use std::fmt::Write as _;

use crate::engine::{SymbolicAnswer, normalize_prompt, stable_id};
use crate::event_log::EventLog;
use crate::language::detect as detect_language;
use crate::rule_interpreter::{handler_policy, handler_table_value};
use crate::seed::{self, Slot, fill_template_once, response_for};
use crate::solver_handlers::finalize_simple;
use crate::solver_helpers::{last_assistant_turn, last_user_turn};

use super::software_project::{SoftwareProjectMeaning, is_approval_prompt, lino_string};

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

    /// The seed role whose surfaces recognise the kind.
    const fn role(self) -> &'static str {
        match self {
            Self::Verification => seed::ROLE_SOFTWARE_FOLLOWUP_VERIFICATION,
            Self::Execution => seed::ROLE_SOFTWARE_FOLLOWUP_EXECUTION,
            Self::Demonstration => seed::ROLE_SOFTWARE_FOLLOWUP_DEMONSTRATION,
        }
    }

    /// The verb the kind renders: the row of the
    /// `software_project_followup_action` table keyed by its role (issue #918).
    fn action(self) -> &'static str {
        handler_table_value("software_project_followup_action", self.role()).unwrap_or_default()
    }
}

/// A value of the `software_project_followup` policy of
/// `data/seed/handler-rules.lino` (issue #918).
fn follow_up_policy(key: &str) -> String {
    handler_policy("software_project_followup", key).unwrap_or_default()
}

/// The seeded `software_project_followup_<name>` sentence with its slots filled.
///
/// The sentence is read in `language`, in English when the seed has no
/// translation. Twin of `softwareFollowUpText` in
/// `js/worker/formal_ai_worker_installation_and_software_followups.js`.
fn follow_up_text(name: &str, language: &str, values: &[(&str, &str)]) -> String {
    let intent = format!("software_project_followup_{name}");
    let template = response_for(&intent, language)
        .or_else(|| response_for(&intent, "en"))
        .unwrap_or_default();
    fill_template_once(&template, values)
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
    let language = detect_language(prompt).slug();
    let body = render_follow_up_response(&meaning, &follow_up, approved, language);
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
    for kind in [
        FollowUpKind::Verification,
        FollowUpKind::Execution,
        FollowUpKind::Demonstration,
    ] {
        let role = kind.role();
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
/// The recognized openers are the prefix forms of the
/// [`seed::ROLE_OUTPUT_DISPLAY_REQUEST`] role; each text before the `…` slot is
/// a marker, tried in declaration order so the longer "show me " wins over the
/// bare "show ". A marker is matched anywhere in the prompt, and the clause that
/// follows — read from the original-case prompt, stopped at the first
/// sentence-ending punctuation and capped at the policy `output-word-limit` —
/// is returned. Issue #918: bare and circumfix forms are not openers (a bare
/// "print" matched inside "printing", the Hindi circumfix lead "जो " anywhere),
/// and the browser twin reads the same prefix forms instead of its own list.
fn extract_expected_output(prompt: &str) -> Option<String> {
    let lower = prompt.to_lowercase();
    let limit = follow_up_policy("output-word-limit").parse().unwrap_or(0);
    let forms = seed::lexicon().role_word_forms(seed::ROLE_OUTPUT_DISPLAY_REQUEST);
    for form in forms.iter().filter(|form| form.slot() == Slot::Prefix) {
        let marker = form.before_slot();
        let Some(start) = lower.find(marker).map(|index| index + marker.len()) else {
            continue;
        };
        let tail = &prompt[start..];
        let stop = tail.find(['.', '?', '\n', ';']).unwrap_or(tail.len());
        let clause = tail[..stop]
            .split_whitespace()
            .take(limit)
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
        log.append("approval_gate", gate);
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

fn follow_up_gates() -> Vec<String> {
    follow_up_policy("gates")
        .split_whitespace()
        .map(str::to_owned)
        .collect()
}

fn follow_up_reasoning_steps(
    meaning: &SoftwareProjectMeaning,
    follow_up: &SoftwareProjectFollowUp,
    language: &str,
) -> Vec<String> {
    let mut steps = vec![follow_up_text(
        "step_recognize",
        language,
        &[
            ("action", follow_up.kind.action()),
            ("kind", follow_up.kind.label()),
            ("artifact", meaning.artifact),
        ],
    )];
    if let Some(site) = &follow_up.target_site {
        steps.push(follow_up_text(
            "step_bind_site",
            language,
            &[("site", site.as_str())],
        ));
    }
    if let Some(output) = &follow_up.expected_output {
        steps.push(follow_up_text(
            "step_record_output",
            language,
            &[("output", output.as_str())],
        ));
    }
    steps.push(follow_up_text("step_fixture", language, &[]));
    steps.push(follow_up_text("step_gates", language, &[]));
    steps
}

fn follow_up_plan_steps(
    meaning: &SoftwareProjectMeaning,
    follow_up: &SoftwareProjectFollowUp,
    language: &str,
) -> Vec<String> {
    let site = follow_up
        .target_site
        .clone()
        .unwrap_or_else(|| follow_up_text("default_target", language, &[]));
    let mut steps = vec![
        follow_up_text(
            "plan_generate",
            language,
            &[("artifact", meaning.artifact), ("site", site.as_str())],
        ),
        follow_up_text("plan_assert", language, &[]),
    ];
    if let Some(output) = &follow_up.expected_output {
        steps.push(follow_up_text(
            "plan_surface",
            language,
            &[("output", output.as_str())],
        ));
    }
    steps.push(follow_up_text(
        "plan_run",
        language,
        &[("language", meaning.implementation_language)],
    ));
    steps.push(follow_up_text(
        "plan_promote",
        language,
        &[("site", site.as_str())],
    ));
    steps
}

fn render_follow_up_response(
    meaning: &SoftwareProjectMeaning,
    follow_up: &SoftwareProjectFollowUp,
    approved: bool,
    language: &str,
) -> String {
    let mut body = follow_up_text(
        "recorded",
        language,
        &[
            ("kind", follow_up.kind.label()),
            ("artifact", meaning.artifact),
        ],
    );
    body.push_str("\n\n");
    body.push_str(&follow_up_text("heading_meaning", language, &[]));
    body.push_str("\n```lino\n");
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
        let _ = writeln!(body, "  approval_gate {}", lino_string(&gate));
    }
    let _ = writeln!(
        body,
        "```\n\n{}",
        follow_up_text("heading_reasoning", language, &[])
    );
    for (index, step) in follow_up_reasoning_steps(meaning, follow_up, language)
        .iter()
        .enumerate()
    {
        let _ = writeln!(body, "{}. {step}", index + 1);
    }
    let _ = writeln!(body, "\n{}", follow_up_text("heading_plan", language, &[]));
    for (index, step) in follow_up_plan_steps(meaning, follow_up, language)
        .iter()
        .enumerate()
    {
        let _ = writeln!(body, "{}. {step}", index + 1);
    }
    body.push('\n');
    body.push_str(&follow_up_text(
        if approved { "approved" } else { "proposed" },
        language,
        &[],
    ));
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

/// Whether the earlier reply formalized a software project request.
///
/// The `prior_software_project` claim evidence of issue #1175 R3.
#[must_use]
pub fn continues_software_project(log: &EventLog) -> bool {
    prior_software_project_dialogue(log).is_some()
}
