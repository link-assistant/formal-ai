//! Generic software-project request handler.
//!
//! The handler covers open-ended "build/write/create an extension/app/bot/tool"
//! prompts. It first projects the surface text into a small Links Notation
//! meaning record, then derives reasoning and plan steps from that meaning.
//! Code is returned only after the user approves the plan in a later turn.

use std::fmt::Write as _;

use nom::{branch::alt, bytes::complete::tag, combinator::value, IResult, Parser};

use crate::engine::{normalize_prompt, stable_id, SymbolicAnswer};
use crate::event_log::EventLog;
use crate::solver_handlers::finalize_simple;
use crate::solver_helpers::{last_assistant_turn, last_user_turn};

use super::software_project_code::implementation_code;

const FEATURE_MARKERS: &[&str] = &[
    "add",
    "admin",
    "audit",
    "backup",
    "calendar",
    "chart",
    "check",
    "conflict",
    "cooldown",
    "csv",
    "customer",
    "damage",
    "date",
    "email",
    "expense",
    "export",
    "file",
    "filter",
    "history",
    "hp",
    "import",
    "invoice",
    "log",
    "maintenance",
    "notification",
    "payment",
    "progress",
    "protection",
    "record",
    "reminder",
    "rename",
    "report",
    "resistance",
    "retry",
    "schedule",
    "scrape",
    "stack",
    "status",
    "sync",
    "track",
    "tracking",
    "upload",
    "validate",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ArtifactMatch {
    surface: &'static str,
    label: &'static str,
}

const ARTIFACT_MATCHES: &[(&str, &str)] = &[
    ("browser extension", "browser extension"),
    ("command line tool", "command-line tool"),
    ("github action", "action"),
    ("mobile app", "mobile app"),
    ("cli tool", "command-line tool"),
    ("web app", "web app"),
    ("application", "application"),
    ("extension", "extension"),
    ("dashboard", "dashboard"),
    ("scraper", "scraper"),
    ("library", "library"),
    ("website", "website"),
    ("plugin", "plugin"),
    ("add on", "extension"),
    ("addon", "extension"),
    ("service", "service"),
    ("bot", "bot"),
    ("app", "app"),
    ("api", "API"),
    ("sdk", "SDK"),
    ("tool", "tool"),
    ("mod", "mod"),
];

#[derive(Debug, Clone, PartialEq, Eq)]
struct SoftwareProjectMeaning {
    action: &'static str,
    artifact_surface: &'static str,
    artifact: &'static str,
    target: String,
    requirements: Vec<String>,
    subtasks: Vec<SoftwareSubtask>,
    delivery_mode: DeliveryMode,
    implementation_language: &'static str,
    approval_gates: Vec<&'static str>,
    game_tracker: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SoftwareSubtask {
    requirement_id: String,
    category: &'static str,
    title: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DeliveryMode {
    CodeGeneration,
    ManualInstructions,
    ScriptGeneration,
    ImmediateExecution,
}

impl DeliveryMode {
    const fn label(self) -> &'static str {
        match self {
            Self::CodeGeneration => "code_generation",
            Self::ManualInstructions => "manual_instructions",
            Self::ScriptGeneration => "script_generation",
            Self::ImmediateExecution => "immediate_execution",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ApprovalState {
    Proposed,
    Approved,
}

impl ApprovalState {
    const fn label(self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
        }
    }
}

impl SoftwareProjectMeaning {
    fn from_prompt(prompt: &str, normalized: &str) -> Option<Self> {
        if normalized.contains("hello") && normalized.contains("world") {
            return None;
        }

        let action = scan_parse(normalized, parse_action_word)?;
        let artifact = scan_parse(normalized, parse_artifact_phrase)?;
        let target = extract_target(prompt, artifact);
        let requirements = extract_requirements(prompt);
        let game_tracker = is_game_unit_tracker(normalized);
        let subtasks = derive_subtasks(&requirements, game_tracker);
        let delivery_mode = detect_delivery_mode(normalized);
        let implementation_language = detect_implementation_language(normalized);
        let approval_gates = approval_gates(normalized, delivery_mode);

        Some(Self {
            action,
            artifact_surface: artifact.surface,
            artifact: artifact.label,
            target,
            requirements,
            subtasks,
            delivery_mode,
            implementation_language,
            approval_gates,
            game_tracker,
        })
    }

    fn canonical_key(&self) -> String {
        let mut key = format!(
            "action={};artifact={};target={};game_tracker={}",
            self.action, self.artifact, self.target, self.game_tracker
        );
        let _ = write!(
            key,
            ";delivery_mode={};implementation_language={}",
            self.delivery_mode.label(),
            self.implementation_language
        );
        for requirement in &self.requirements {
            let _ = write!(key, ";requirement={requirement}");
        }
        for subtask in &self.subtasks {
            let _ = write!(key, ";subtask={}:{}", subtask.category, subtask.title);
        }
        key
    }

    fn meaning_id(&self) -> String {
        stable_id("software_project_request", &self.canonical_key())
    }

    const fn domain_label(&self) -> &'static str {
        if self.game_tracker {
            "tabletop_game_unit_tracker"
        } else {
            "software_project"
        }
    }

    fn meaning_lino(&self, approval_state: ApprovalState) -> String {
        let mut buffer = String::from("software_project_request");
        let _ = writeln!(buffer, "\n  action {}", lino_string(self.action));
        let _ = writeln!(buffer, "  artifact {}", lino_string(self.artifact));
        let _ = writeln!(
            buffer,
            "  artifact_surface {}",
            lino_string(self.artifact_surface)
        );
        let _ = writeln!(buffer, "  target {}", lino_string(&self.target));
        let _ = writeln!(buffer, "  domain {}", lino_string(self.domain_label()));
        let _ = writeln!(buffer, "  delivery_mode {}", self.delivery_mode.label());
        let _ = writeln!(
            buffer,
            "  implementation_language {}",
            lino_string(self.implementation_language)
        );
        let _ = writeln!(buffer, "  approval_state {}", approval_state.label());
        let _ = writeln!(buffer, "  approval_required true");
        for gate in &self.approval_gates {
            let _ = writeln!(buffer, "  approval_gate {}", lino_string(gate));
        }
        for requirement in &self.requirements {
            let _ = writeln!(buffer, "  requirement {}", lino_string(requirement));
            let _ = writeln!(
                buffer,
                "  requirement_category {}",
                lino_string(classify_requirement(requirement, self.game_tracker))
            );
        }
        for subtask in &self.subtasks {
            let _ = writeln!(
                buffer,
                "  subtask {}",
                lino_string(&format!(
                    "{} [{}] {}",
                    subtask.requirement_id, subtask.category, subtask.title
                ))
            );
        }
        if self.game_tracker {
            buffer.push_str("  state_model \"unit_state\"\n");
            buffer.push_str("  command \"apply_damage\"\n");
            buffer.push_str("  command \"set_stacks\"\n");
            buffer.push_str("  command \"tick_cooldowns\"\n");
            buffer.push_str("  validation \"damage_mitigation_floor_at_zero\"\n");
            buffer.push_str("  validation \"cooldowns_decrement_without_negative_rounds\"\n");
        } else {
            buffer.push_str("  state_model \"project_records\"\n");
            buffer.push_str("  command \"create_record\"\n");
            buffer.push_str("  command \"update_record\"\n");
            buffer.push_str("  command \"export_state\"\n");
            buffer.push_str("  validation \"pure_state_transitions_before_host_api\"\n");
        }
        buffer
    }
}

pub fn try_software_project_request(
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

    if is_approval_prompt(normalized) {
        if let Some(meaning) = prior_software_project_meaning(log) {
            record_meaning(log, &meaning, ApprovalState::Approved);
            let body = render_implementation_response(&meaning);
            return Some(finalize_simple(
                prompt,
                log,
                "software_project_implementation",
                "response:software_project_implementation",
                &body,
                0.82,
            ));
        }
    }

    let meaning = SoftwareProjectMeaning::from_prompt(prompt, normalized)?;
    record_meaning(log, &meaning, ApprovalState::Proposed);
    let body = render_plan_response(&meaning);
    Some(finalize_simple(
        prompt,
        log,
        "software_project_plan",
        "response:software_project_plan",
        &body,
        0.78,
    ))
}

/// Kinds of follow-up that exercise an already-designed artifact. The order in
/// [`detect_follow_up`] gives verification precedence over plain execution so a
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

/// Phrases that, while a software-project dialogue is active, signal a request
/// to exercise the just-designed artifact rather than start a fresh fact
/// lookup. Each marker is paired with the follow-up kind it implies. Markers
/// are matched against the lowercased, script-preserving normalized prompt, so
/// the table carries the supported languages (en, ru, hi, zh): a multilingual
/// user who designs in one language and then says "now test it" in another
/// (issue #341) still stays inside the project dialogue. Verification markers
/// precede execution and demonstration so a combined phrasing records the
/// stronger goal.
const FOLLOW_UP_MARKERS: &[(&str, FollowUpKind)] = &[
    // Verification — en
    ("test it", FollowUpKind::Verification),
    ("test the", FollowUpKind::Verification),
    ("test this", FollowUpKind::Verification),
    ("verify", FollowUpKind::Verification),
    ("check it", FollowUpKind::Verification),
    ("check that", FollowUpKind::Verification),
    ("run the tests", FollowUpKind::Verification),
    // Verification — ru / zh / hi
    ("протестируй", FollowUpKind::Verification),
    ("протестировать", FollowUpKind::Verification),
    ("проверь", FollowUpKind::Verification),
    ("тестируй", FollowUpKind::Verification),
    ("测试", FollowUpKind::Verification),
    ("检验", FollowUpKind::Verification),
    ("检查", FollowUpKind::Verification),
    ("परीक्षण", FollowUpKind::Verification),
    ("जाँच", FollowUpKind::Verification),
    ("जांच", FollowUpKind::Verification),
    // Execution — en
    ("run it", FollowUpKind::Execution),
    ("run this", FollowUpKind::Execution),
    ("run the", FollowUpKind::Execution),
    ("execute it", FollowUpKind::Execution),
    ("execute the", FollowUpKind::Execution),
    ("try it", FollowUpKind::Execution),
    // Execution — ru / zh / hi
    ("запусти", FollowUpKind::Execution),
    ("выполни", FollowUpKind::Execution),
    ("运行", FollowUpKind::Execution),
    ("执行", FollowUpKind::Execution),
    ("चलाओ", FollowUpKind::Execution),
    ("निष्पादित", FollowUpKind::Execution),
    // Demonstration — en
    ("demo it", FollowUpKind::Demonstration),
    ("show me", FollowUpKind::Demonstration),
    ("show the", FollowUpKind::Demonstration),
    ("print the", FollowUpKind::Demonstration),
    // Demonstration — ru / zh / hi
    ("покажи", FollowUpKind::Demonstration),
    ("显示", FollowUpKind::Demonstration),
    ("展示", FollowUpKind::Demonstration),
    ("दिखाओ", FollowUpKind::Demonstration),
];

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
    let kind = FOLLOW_UP_MARKERS
        .iter()
        .find(|(marker, _)| normalized.contains(marker))
        .map(|(_, kind)| *kind)?;
    Some(SoftwareProjectFollowUp {
        kind,
        target_site: extract_target_site(prompt),
        expected_output: extract_expected_output(prompt),
    })
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

/// Capture the clause after "show me"/"show" so the follow-up records what the
/// user wants surfaced (e.g. "the top 10 most frequent words").
fn extract_expected_output(prompt: &str) -> Option<String> {
    let lower = prompt.to_lowercase();
    for marker in ["show me ", "show ", "print ", "display "] {
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
    let _ = writeln!(body, "  delivery_mode {}", meaning.delivery_mode.label());
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

fn prior_software_project_meaning(log: &EventLog) -> Option<SoftwareProjectMeaning> {
    let assistant = last_assistant_turn(log)?;
    if !assistant.contains("software_project_request") || !assistant.contains("approve plan") {
        return None;
    }
    let prior_prompt = last_user_turn(log)?;
    let normalized = normalize_prompt(prior_prompt);
    SoftwareProjectMeaning::from_prompt(prior_prompt, &normalized)
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

fn record_meaning(
    log: &mut EventLog,
    meaning: &SoftwareProjectMeaning,
    approval_state: ApprovalState,
) {
    log.append("formalization", "text_to_links_notation".to_owned());
    log.append("meaning", meaning.meaning_id());
    log.append("software_project:action", meaning.action.to_owned());
    log.append("software_project:artifact", meaning.artifact.to_owned());
    log.append("software_project:target", meaning.target.clone());
    log.append("software_project:domain", meaning.domain_label().to_owned());
    log.append(
        "software_project:delivery_mode",
        meaning.delivery_mode.label().to_owned(),
    );
    log.append(
        "software_project:implementation_language",
        meaning.implementation_language.to_owned(),
    );
    log.append("approval_state", approval_state.label().to_owned());
    for gate in &meaning.approval_gates {
        log.append("approval_gate", (*gate).to_owned());
    }
    log.append(
        "software_project:strategy",
        if meaning.game_tracker {
            "game_unit_tracker"
        } else {
            "bounded_project_plan"
        }
        .to_owned(),
    );
    for requirement in &meaning.requirements {
        log.append("requirement", requirement.clone());
        log.append(
            "requirement_category",
            classify_requirement(requirement, meaning.game_tracker).to_owned(),
        );
    }
    for subtask in &meaning.subtasks {
        log.append(
            "software_project:subtask",
            format!(
                "{}:{}:{}",
                subtask.requirement_id, subtask.category, subtask.title
            ),
        );
    }
    for step in reasoning_steps(meaning) {
        log.append("reasoning_step", step);
    }
    for step in plan_steps(meaning) {
        log.append("plan_step", step);
    }
}

fn parse_action_word(input: &str) -> IResult<&str, &'static str> {
    alt((
        value("write", tag("write")),
        value("build", tag("build")),
        value("create", tag("create")),
        value("implement", tag("implement")),
        value("make", tag("make")),
        value("develop", tag("develop")),
        value("generate", tag("generate")),
        value("design", tag("design")),
        value("scaffold", tag("scaffold")),
    ))
    .parse(input)
}

fn parse_artifact_phrase(input: &str) -> IResult<&str, ArtifactMatch> {
    for &(surface, label) in ARTIFACT_MATCHES {
        if let Some(remaining) = input.strip_prefix(surface) {
            return Ok((remaining, ArtifactMatch { surface, label }));
        }
    }
    Err(nom::Err::Error(nom::error::Error::new(
        input,
        nom::error::ErrorKind::Tag,
    )))
}

fn scan_parse<T: Copy>(normalized: &str, parser: fn(&str) -> IResult<&str, T>) -> Option<T> {
    for (index, _) in normalized.char_indices() {
        if !is_start_boundary(normalized, index) {
            continue;
        }
        let input = &normalized[index..];
        if let Ok((remaining, value)) = parser(input) {
            let consumed = input.len().saturating_sub(remaining.len());
            if consumed > 0 && is_end_boundary(normalized, index + consumed) {
                return Some(value);
            }
        }
    }
    None
}

fn is_start_boundary(value: &str, index: usize) -> bool {
    if index == 0 {
        return true;
    }
    value[..index]
        .chars()
        .next_back()
        .is_some_and(|character| !is_word_character(character))
}

fn is_end_boundary(value: &str, index: usize) -> bool {
    if index >= value.len() {
        return true;
    }
    value[index..]
        .chars()
        .next()
        .is_some_and(|character| !is_word_character(character))
}

const fn is_word_character(character: char) -> bool {
    character.is_ascii_alphanumeric()
}

fn extract_target(prompt: &str, artifact: ArtifactMatch) -> String {
    let markers = [
        format!("{} for ", artifact.surface),
        format!("{} to ", artifact.surface),
        format!("{} for ", artifact.label),
        format!("{} to ", artifact.label),
        String::from(" for "),
        String::from(" to "),
    ];
    for marker in markers {
        if let Some(target) = extract_after_marker(prompt, &marker) {
            return target;
        }
    }
    String::from("the requested environment")
}

fn extract_after_marker(prompt: &str, marker: &str) -> Option<String> {
    let lower_prompt = prompt.to_lowercase();
    let lower_marker = marker.to_lowercase();
    let start = lower_prompt.find(&lower_marker)? + lower_marker.len();
    let tail = &prompt[start..];
    let stop = tail.find(['?', '.', ',', ';', '\n']).unwrap_or(tail.len());
    let raw = tail[..stop]
        .split(" with ")
        .next()
        .unwrap_or("")
        .split(" that ")
        .next()
        .unwrap_or("")
        .split(" and ")
        .next()
        .unwrap_or("")
        .trim();
    if raw.is_empty() {
        return None;
    }
    Some(capitalize_short_target(raw))
}

fn capitalize_short_target(raw: &str) -> String {
    let compact = raw.split_whitespace().take(5).collect::<Vec<_>>().join(" ");
    if compact.chars().any(char::is_uppercase) {
        return compact;
    }
    let mut chars = compact.chars();
    let Some(first) = chars.next() else {
        return compact;
    };
    format!("{}{}", first.to_uppercase(), chars.collect::<String>())
}

fn extract_requirements(prompt: &str) -> Vec<String> {
    let mut requirements = Vec::new();
    for segment in prompt.split(['.', ';', '\n']) {
        for clause in segment.split(',') {
            let cleaned = clause.trim();
            if cleaned.is_empty() {
                continue;
            }
            let lower = cleaned.to_lowercase();
            if contains_any(&lower, FEATURE_MARKERS) {
                push_unique(&mut requirements, sentence_case(cleaned));
            }
        }
    }
    if requirements.is_empty() {
        requirements.push(String::from(
            "Capture state, user commands, persistence, validation, and tests.",
        ));
    }
    requirements
}

fn derive_subtasks(requirements: &[String], game_tracker: bool) -> Vec<SoftwareSubtask> {
    requirements
        .iter()
        .enumerate()
        .map(|(index, requirement)| {
            let category = classify_requirement(requirement, game_tracker);
            SoftwareSubtask {
                requirement_id: format!("R{}", index + 1),
                category,
                title: subtask_title(category, requirement),
            }
        })
        .collect()
}

fn classify_requirement(requirement: &str, game_tracker: bool) -> &'static str {
    let lower = requirement.to_lowercase();
    if game_tracker || contains_any(&lower, &["track", "hp", "status", "damage", "cooldown"]) {
        "state_tracking"
    } else if contains_any(
        &lower,
        &["import", "export", "csv", "backup", "report", "calendar"],
    ) {
        "data_exchange"
    } else if contains_any(&lower, &["reminder", "notification", "schedule", "weekly"]) {
        "automation"
    } else if contains_any(&lower, &["validate", "check", "conflict", "audit"]) {
        "validation"
    } else if contains_any(&lower, &["api", "discord", "telegram", "github", "browser"]) {
        "integration"
    } else if contains_any(&lower, &["dashboard", "chart", "filter", "progress"]) {
        "user_interface"
    } else {
        "project_behavior"
    }
}

fn subtask_title(category: &str, requirement: &str) -> String {
    match category {
        "state_tracking" => format!("Model state fields and pure transitions for {requirement}"),
        "data_exchange" => {
            format!("Define parsers, serializers, and backup flow for {requirement}")
        }
        "automation" => {
            format!("Schedule deterministic jobs and delivery checks for {requirement}")
        }
        "validation" => format!("Encode validation rules and failure messages for {requirement}"),
        "integration" => format!("Isolate host API boundaries and mocks for {requirement}"),
        "user_interface" => format!("Design focused views and state updates for {requirement}"),
        _ => format!("Implement and test the smallest behavior for {requirement}"),
    }
}

fn detect_delivery_mode(normalized: &str) -> DeliveryMode {
    if contains_any(
        normalized,
        &["manual instruction", "instructions", "no code"],
    ) {
        DeliveryMode::ManualInstructions
    } else if contains_any(normalized, &["execute", "run command", "run it", "webvm"]) {
        DeliveryMode::ImmediateExecution
    } else if contains_any(normalized, &["bash", "shell"])
        || contains_word(normalized, &["script", "scripts", "commands"])
    {
        DeliveryMode::ScriptGeneration
    } else {
        DeliveryMode::CodeGeneration
    }
}

fn detect_implementation_language(normalized: &str) -> &'static str {
    if contains_any(normalized, &["python", "django", "fastapi"]) {
        "python"
    } else if contains_any(normalized, &["rust", "cargo"]) {
        "rust"
    } else if contains_any(normalized, &["javascript", "node.js", "node "]) {
        "javascript"
    } else {
        "typescript"
    }
}

fn approval_gates(normalized: &str, delivery_mode: DeliveryMode) -> Vec<&'static str> {
    let mut gates = vec!["task_formalization", "implementation_plan"];
    if normalized.contains("requirement") {
        gates.push("requirements");
    }
    if contains_any(normalized, &["each step", "step by step"]) {
        gates.push("each_step");
    }
    match delivery_mode {
        DeliveryMode::CodeGeneration => gates.push("generated_code"),
        DeliveryMode::ManualInstructions => gates.push("manual_instructions"),
        DeliveryMode::ScriptGeneration | DeliveryMode::ImmediateExecution => {
            gates.push("generated_script");
            gates.push("bash_command");
        }
    }
    if contains_any(normalized, &["shell", "bash", "command", "docker", "webvm"]) {
        gates.push("bash_command");
    }
    gates.sort_unstable();
    gates.dedup();
    gates
}

fn contains_any(haystack: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| haystack.contains(needle))
}

fn contains_word(haystack: &str, words: &[&str]) -> bool {
    haystack
        .split_whitespace()
        .any(|token| words.contains(&token))
}

fn push_unique(items: &mut Vec<String>, value: String) {
    if !items.iter().any(|item| item == &value) {
        items.push(value);
    }
}

fn sentence_case(raw: &str) -> String {
    let trimmed = raw.trim().trim_matches(['-', '*', ' ']);
    let mut chars = trimmed.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };
    format!("{}{}", first.to_uppercase(), chars.collect::<String>())
}

fn is_game_unit_tracker(normalized: &str) -> bool {
    let domain = normalized.contains("dnd")
        || normalized.contains("d&d")
        || normalized.contains("wargame")
        || normalized.contains("tabletop")
        || normalized.contains("unit")
        || normalized.contains("token")
        || normalized.contains("owlbear");
    let mechanics = normalized.contains("hp")
        || normalized.contains("damage")
        || normalized.contains("protection")
        || normalized.contains("resistance")
        || normalized.contains("cooldown");
    domain && mechanics
}

fn is_approval_prompt(normalized: &str) -> bool {
    let compact = normalized
        .trim()
        .trim_matches(|character: char| !character.is_ascii_alphanumeric())
        .replace(['.', '!', ','], "");
    matches!(
        compact.as_str(),
        "approve"
            | "approved"
            | "approve plan"
            | "yes"
            | "yes proceed"
            | "proceed"
            | "go ahead"
            | "looks good"
            | "do it"
            | "start implementation"
            | "generate code"
            | "convert to code"
    )
}

fn reasoning_steps(meaning: &SoftwareProjectMeaning) -> Vec<String> {
    let mut steps = vec![
        format!(
            "Classify the impulse as a request to {} a {} instead of a fact lookup.",
            meaning.action, meaning.artifact
        ),
        format!(
            "Bind the target environment to {} and keep the first response reviewable.",
            meaning.target
        ),
        format!(
            "Extract {} requirement(s) into the meaning record before planning.",
            meaning.requirements.len()
        ),
        format!(
            "Decompose the requirement graph into {} implementation subtask(s) with category labels.",
            meaning.subtasks.len()
        ),
        format!(
            "Select delivery mode {} and approval gates: {}.",
            meaning.delivery_mode.label(),
            meaning.approval_gates.join(", ")
        ),
    ];
    if meaning.game_tracker {
        steps.push(String::from(
            "Map HP, Protection, Resistance, damage, and cooldown phrases to a unit-state domain model.",
        ));
    }
    steps.push(String::from(
        "Ask for approval before producing code, scripts, manual instructions, or execution steps.",
    ));
    steps
}

fn plan_steps(meaning: &SoftwareProjectMeaning) -> Vec<String> {
    let mut steps = Vec::new();
    steps.push(String::from(
        "Review the formalized task, requirement graph, approval gates, and delivery mode with the user.",
    ));
    if meaning.game_tracker {
        steps.extend([
            format!("Confirm the {} storage and selected-token API boundaries.", meaning.target),
            String::from(
                "Define `UnitState` with HP, max HP, Protection, Resistance, and cooldowns.",
            ),
            String::from(
                "Write pure transition functions for damage mitigation, stack edits, and round ticks.",
            ),
            String::from(
                "Add tests for zero damage, overkill damage, stack changes, and cooldown expiry.",
            ),
            String::from("Wire the tested core into the extension panel and host persistence."),
        ]);
        return steps;
    }

    steps.extend([
        format!(
            "Confirm the host API and data boundaries for {}.",
            meaning.target
        ),
        String::from("Define the smallest serializable state records for the requirements."),
    ]);
    for subtask in &meaning.subtasks {
        steps.push(format!(
            "Implement {}: {}.",
            subtask.category, subtask.title
        ));
    }
    steps.push(format!(
        "Generate a {} starter core plus language-appropriate repository initialization and checks.",
        meaning.implementation_language
    ));
    steps.push(String::from(
        "Keep shell, Docker, or WebVM commands behind the configured approval gates.",
    ));
    steps
}

fn render_plan_response(meaning: &SoftwareProjectMeaning) -> String {
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

fn render_implementation_response(meaning: &SoftwareProjectMeaning) -> String {
    let mut body = String::new();
    let _ = writeln!(
        body,
        "Approved implementation starter for a {} targeting {}.",
        meaning.artifact, meaning.target
    );
    body.push('\n');
    body.push_str("Formalized meaning:\n```lino\n");
    body.push_str(&meaning.meaning_lino(ApprovalState::Approved));
    body.push_str("```\n\nImplementation steps:\n");
    for (index, step) in plan_steps(meaning).iter().enumerate() {
        let _ = writeln!(body, "{}. {step}", index + 1);
    }
    let code = implementation_code(meaning.game_tracker, meaning.implementation_language);
    let _ = write!(
        body,
        "\nStarter {} core:\n\n```{}\n",
        code.label, code.fence
    );
    body.push_str(code.body);
    body.push_str("\n```\n");
    body.push_str("\nGenerated code checks:\n");
    let _ = writeln!(
        body,
        "1. Initialize a {} project in an isolated workspace.",
        code.label
    );
    let _ = writeln!(
        body,
        "2. Run the language-native syntax/type check before host integration."
    );
    body
}

fn lino_string(value: &str) -> String {
    let escaped = value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r");
    format!("\"{escaped}\"")
}
