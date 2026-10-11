//! Deterministic fallback planner for repository change requests (issue #654).
//!
//! Unlike the stored recipe fixtures, this planner derives its target and payload
//! from the formalized request.  The resulting plan is data: it is serialized to
//! Links Notation and written before execution, so the tool transcript is an
//! append-only record of the decision that caused the change.
use super::planner::{Capability, trace_route};
use super::write_request::{
    bare_surfaces, clean_cue_token, clean_path_token, first_action_cue_end, first_prefix_lead_end,
    first_raw_content_lead_end, first_raw_prefix_lead_end, honouring_pinned_first_line,
    looks_like_file_path, safe_relative_path, tokens,
};
use crate::engine::stable_id;
use crate::intent_formalization::formalize_intent;
use crate::seed::{self, Slot};
use crate::self_ast_census::{self, CensusResolution};
use std::fmt::Write as _;
/// Workspace-relative event-log artifact written before a general plan executes.
pub const PLAN_PATH: &str = ".formal-ai/general-change-plan.lino";
const TARGET_PLACEHOLDER: &str = "{target}";

mod additive_scope;
pub(super) use additive_scope::owns_additive_scope;
mod content_shape;
mod literal_request;
mod owned_goals;
use content_shape::{describes_code_to_author, names_an_addition};
pub(super) use content_shape::{
    missing_implementation_contract, owned_semantic_authoring_lead, owns_literal_body,
};
use literal_request::parse_write_request;
pub(super) use owned_goals::{
    instruction_view_for_request, owns_complete_edit_request, pending_read_gap,
    plan_owned_goal_step,
};

pub use super::write_request::compose_edit_request;
pub(crate) use super::write_request::typed_write_target;
pub use owned_goals::{
    declared_addition_contract, has_additive_position, owned_additive_literal,
    owned_additive_literal_frame, owned_declared_create_frame,
};
/// What the bounded general planner can truthfully execute.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeneralPlanMode {
    /// Write literal request content to a workspace-relative file.
    LiteralFile,
    /// Capture the output of an explicitly quoted command in a file.
    CommandOutput,
    /// Persist a referenced repository work item without fabricating a patch.
    RepositoryWorkItem,
}
impl GeneralPlanMode {
    /// The mode's stable slug, which is also the vocabulary the obligation
    /// contract's `general_plan_mode` rule rows are keyed by (plan 05 leaf 6).
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::LiteralFile => "literal_file",
            Self::CommandOutput => "command_output",
            Self::RepositoryWorkItem => "repository_work_item",
        }
    }
}
/// Where a composed plan can honestly end (issue #904).
///
/// A plan whose steps all operate on the plan record itself changes nothing the
/// request named, and reading that record back verifies only that the run wrote
/// it. Such a plan reaches [`PlanTerminalState::PlannedNotExecuted`], never a
/// success state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanTerminalState {
    /// Every step operates on an artifact the request named.
    Executed,
    /// The plan was recorded; no artifact the request named was touched.
    PlannedNotExecuted,
}
impl PlanTerminalState {
    const fn slug(self) -> &'static str {
        match self {
            Self::Executed => "executed",
            Self::PlannedNotExecuted => "planned_not_executed",
        }
    }
}
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
    pub mode: GeneralPlanMode,
    pub goal: String,
    pub target: String,
    pub content: String,
    pub steps: Vec<GeneralPlanStep>,
    /// The command that observes the requested artifact after the change.
    /// Empty when the plan can name no such command, which is the only honest
    /// answer for a plan that reaches [`PlanTerminalState::PlannedNotExecuted`].
    pub verification_command: String,
    pub terminal_state: PlanTerminalState,
}
impl GeneralChangePlan {
    /// Render the plan shape consumed by the driver and documented by the meta fixture.
    #[must_use]
    pub fn links_notation(&self) -> String {
        let mut out = super::append_contract::general_change_plan_record_header()
            .expect("general plan record schema unavailable");
        field(&mut out, "id", &self.id);
        field(&mut out, "execution_mode", self.mode.slug());
        field(&mut out, "terminal_state", self.terminal_state.slug());
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
        if !self.verification_command.is_empty() {
            field(&mut out, "verification_command", &self.verification_command);
        }
        out
    }
    /// Render the terminal answer of a plan that touched nothing the request
    /// named: planned, not executed (issue #904).
    #[must_use]
    pub fn planned_not_executed_answer(&self) -> String {
        const BODY_PLACEHOLDER: &str = concat!("{", "plan", "}");
        let language = crate::language::detect(&self.goal).slug();
        seed::localized_response("general_plan_repository_planned", language)
            .unwrap_or_default()
            .replace(TARGET_PLACEHOLDER, &self.target)
            .replace("{plan_path}", PLAN_PATH)
            .replace(
                BODY_PLACEHOLDER,
                &crate::issue_report::fenced_block(
                    crate::issue_report::LINO_FENCE_LANGUAGE,
                    &self.links_notation(),
                ),
            )
    }
}
/// Return the request after its first line-anchored multilingual objective marker.
///
/// This removes an agent-harness preamble. Requests without a marker are unchanged.
#[must_use]
pub fn objective_text(request: &str) -> &str {
    let request =
        super::file_read::read_request_envelope(request).map_or(request, |(text, _, _)| text);
    first_raw_prefix_lead_end(request, seed::ROLE_REQUEST_OBJECTIVE_LEAD)
        .filter(|(start, _)| line_anchored(request, *start))
        .filter(|(start, _)| {
            !crate::normal_markov::quoted_segment_spans(request)
                .iter()
                .any(|segment| *start >= segment.start && *start < segment.end)
        })
        .and_then(|(_, end)| request.get(end..))
        .map_or(request, str::trim)
}
/// Whether a marker at `start` opens its own line, so a delimiter quoted inside
/// running prose ("write the words request: hello to notes.txt") does not
/// silently truncate the request.
/// A closed literal introduced by seeded creation or whole-content consent.
fn literal_payload(request: &str) -> Option<crate::normal_markov::QuotedSegment> {
    let quoted = crate::normal_markov::quoted_segment_spans(request);
    let literal = quoted.iter().find(|segment| {
        request[segment.end..].chars().all(|character| {
            character.is_whitespace()
                || matches!(character, '.' | '!' | '?' | '。' | '！' | '？' | '।')
        })
    })?;
    let mut prefix = request[..literal.start].to_owned();
    for segment in quoted
        .iter()
        .rev()
        .filter(|segment| segment.end <= literal.start)
    {
        prefix.replace_range(
            segment.start..segment.end,
            &" ".repeat(segment.end - segment.start),
        );
    }
    let normalized = crate::engine::normalize_prompt(&prefix);
    let lexicon = crate::seed::lexicon();
    let overwrite = lexicon.mentions_role("file_overwrite_consent", &normalized);
    let lead = first_raw_content_lead_end(&prefix);
    if lead.is_none() && !overwrite {
        return None;
    }
    if lead.is_some_and(|(_, end)| {
        !(prefix[end..]
            .chars()
            .all(|character| character.is_whitespace() || character == ':')
            || (overwrite && !prefix[end..].trim_end().contains('\n')))
    }) {
        return None;
    }
    let words = tokens(&prefix);
    let action_start = super::write_request::first_action_cue_start(&words)?;
    let action_end = first_action_cue_end(&words)?;
    let action = crate::engine::normalize_prompt(&prefix[action_start..action_end]);
    (overwrite || lexicon.mentions_role("file_whole_write_action", &action))
        .then(|| literal.clone())
}

fn line_anchored(text: &str, start: usize) -> bool {
    text[..start]
        .chars()
        .rev()
        .take_while(|character| *character != '\n')
        .all(char::is_whitespace)
}
#[must_use]
pub fn compose_general_change_plan(full_request: &str) -> Option<GeneralChangePlan> {
    let request = objective_text(full_request);
    // An additive edit (append, prepend) never rewrites the whole file: the
    // workspace-change arm owns it, and a request it cannot ground is declined.
    let literal = literal_payload(request);
    let instruction = literal.as_ref().map_or_else(
        || request.to_owned(),
        |segment| format!("{}{}", &request[..segment.start], &request[segment.end..]),
    );
    let lowered = instruction.to_lowercase();
    let lexicon = crate::seed::lexicon();
    if lexicon.mentions_role("file_edit_position_end", &lowered)
        || lexicon.mentions_role("file_edit_position_start", &lowered)
    {
        return None;
    }
    let command_output = parse_command_output_request(request);
    let file_request = command_output.as_ref().map_or_else(
        || parse_write_request(request),
        |(target, _)| Some((target.clone(), String::new())),
    );
    let Some((target, content)) = file_request else {
        return compose_repository_work_plan(request);
    };
    // Issue #906: "…containing Hello World, in JavaScript." names the bytes and,
    // separately, the language to write them with. Only the bytes are content.
    let content = if literal.is_some() {
        content
    } else {
        crate::implementation_language::without_trailing_known_modifier(&content).unwrap_or(content)
    };
    // Issue #1066: the same request can state the bytes and, separately,
    // constrain the line the file has to open with. The repair belongs here
    // rather than at the call sites, because every step of the plan quotes the
    // content it was composed from -- the verification step's expected evidence
    // most of all -- and a plan whose steps disagree with its own bytes is not
    // one a reader can check.
    let content = honouring_pinned_first_line(&instruction, &content).map_or(content, |repaired| {
        // Whether the repair applied is not recoverable from the finished plan,
        // and the two outcomes it separates -- bytes taken from the prose, and
        // bytes corrected to match a constraint stated elsewhere in the same
        // request -- are exactly what a reader checking the plan needs to tell
        // apart (default off; `FORMAL_AI_TRACE_REQUESTS=1`).
        trace_route("general_change_plan", "repaired_pinned_first_line");
        repaired
    });
    if !safe_relative_path(&target) {
        return None;
    }
    if command_output.is_none()
        && (describes_code_to_author(request, &content)
            || names_an_addition(request, &content, &target))
    {
        return None;
    }
    let response_language = language(request);
    let intent = formalize_intent(request, response_language, None);
    let verification_command = if command_output.is_some() {
        super::work_item_steps::fill(
            "command-capture-readback",
            &[(concat!("{", "target", "}"), &shell_quote(&target))],
        )
    } else {
        format!("cat {target}")
    };
    let mut steps = vec![GeneralPlanStep {
        capability: Capability::Write,
        action: crate::seed::render_response(
            "general-plan-append-action",
            "en",
            &[("plan_path", PLAN_PATH)],
        )
        .unwrap_or_default(),
        expected_evidence: format!("written plan event {}", intent.impulse_id),
        command: None,
    }];
    if let Some((_, command)) = &command_output {
        let setup = super::work_item_steps::fill(
            "command-capture-setup",
            &[(concat!("{", "target", "}"), &shell_quote(&target))],
        );
        let end = super::work_item_steps::fill("command-capture-end", &[]);
        let generation_command = format!("{setup}{command}{end}");
        steps.push(GeneralPlanStep {
            capability: Capability::Run,
            action: command_plan_text(
                "general_plan_command_capture_action",
                response_language,
                &target,
            ),
            expected_evidence: command_plan_text(
                "general_plan_command_output_evidence",
                response_language,
                &target,
            ),
            command: Some(generation_command),
        });
    } else {
        steps.push(GeneralPlanStep {
            capability: Capability::Write,
            action: format!("write the requested content to {target}"),
            expected_evidence: format!("workspace file {target}"),
            command: None,
        });
    }
    steps.push(GeneralPlanStep {
        capability: Capability::Run,
        action: String::from("run the request-derived verification command"),
        expected_evidence: if command_output.is_some() {
            command_plan_text(
                "general_plan_command_verification_evidence",
                response_language,
                &target,
            )
        } else {
            content.clone()
        },
        command: Some(verification_command.clone()),
    });
    Some(GeneralChangePlan {
        id: stable_id(
            "general_change_plan",
            &format!(
                "{}:{target}:{content}:{}",
                intent.impulse_id,
                command_output
                    .as_ref()
                    .map_or("", |(_, command)| command.as_str())
            ),
        ),
        mode: if command_output.is_some() {
            GeneralPlanMode::CommandOutput
        } else {
            GeneralPlanMode::LiteralFile
        },
        goal: intent.source_text,
        target,
        content,
        steps,
        verification_command,
        // The verification command observes the file the request named, not the
        // plan record this run wrote, so the plan really is executed.
        terminal_state: PlanTerminalState::Executed,
    })
}
fn compose_repository_work_plan(request: &str) -> Option<GeneralChangePlan> {
    let target = repository_work_reference(request)?;
    // Hive Mind's objective field (`Issue to solve: <url>`, then the prepared
    // branch and "Proceed.") carries its verb in the delimiter, which
    // `objective_text` has already stripped. An objective that opens with the
    // work-item reference itself is therefore that field: the reference is
    // the request (issues #1154 and #1155 replay this exact prompt).
    let opens_with_reference = request
        .split_whitespace()
        .next()
        .and_then(repository_work_reference)
        .is_some();
    if !opens_with_reference && !mentions_bare_role(request, seed::ROLE_SOFTWARE_AUTHORING_ACTION) {
        return None;
    }
    let response_language = language(request);
    let intent = formalize_intent(request, response_language, None);
    Some(GeneralChangePlan {
        id: stable_id(
            "repository_work_item_plan",
            &format!("{}:{target}", intent.impulse_id),
        ),
        mode: GeneralPlanMode::RepositoryWorkItem,
        goal: intent.source_text,
        target: target.clone(),
        content: String::new(),
        // A work item names an issue, not an artifact, so step one reads the
        // issue — that text is where the artifact is named, and planning
        // without it would fabricate one (issue #904, follow-up). Recording the
        // reference stays step two, and the plan still names no verification
        // command: reading back the record this run wrote observes only its own
        // write.
        steps: vec![
            work_item_step(Capability::Fetch, "read", response_language, &target),
            work_item_step(Capability::Write, "action", response_language, PLAN_PATH),
        ],
        verification_command: String::new(),
        terminal_state: PlanTerminalState::PlannedNotExecuted,
    })
}
/// One step of a repository work-item plan, with its seeded action and
/// evidence. `slug` is `read` (the fetch of the work item) or `action` (the
/// record written afterwards).
fn work_item_step(capability: Capability, slug: &str, lang: &str, target: &str) -> GeneralPlanStep {
    let evidence = if slug == "read" {
        "general_plan_repository_read_evidence"
    } else {
        "general_plan_repository_evidence"
    };
    GeneralPlanStep {
        capability,
        action: command_plan_text(&format!("general_plan_repository_{slug}"), lang, target),
        expected_evidence: command_plan_text(evidence, lang, target),
        command: None,
    }
}
/// Extract a concrete GitHub issue or pull-request URL structurally.
///
/// The software action itself comes from the multilingual seed. URL host/path
/// segments are protocol identifiers, not natural-language routing phrases.
pub(super) fn repository_work_reference(request: &str) -> Option<String> {
    request.split_whitespace().find_map(|token| {
        // Sentence punctuation in any registered script: a URL that ends a
        // Chinese sentence carries `。` the way an English one carries `.`.
        let url = token.trim_matches(|character: char| {
            matches!(
                character,
                '<' | '>'
                    | '('
                    | ')'
                    | '['
                    | ']'
                    | '{'
                    | '}'
                    | ','
                    | ';'
                    | '.'
                    | '"'
                    | '\''
                    | '。'
                    | '，'
                    | '、'
                    | '；'
                    | '：'
                    | '（'
                    | '）'
                    | '「'
                    | '」'
                    | '«'
                    | '»'
                    | '।'
            )
        });
        let path = url
            .strip_prefix("https://github.com/")
            .or_else(|| url.strip_prefix("http://github.com/"))?;
        let segments: Vec<&str> = path.split('/').collect();
        (segments.len() == 4
            && !segments[0].is_empty()
            && !segments[1].is_empty()
            && matches!(segments[2], "issues" | "pull")
            && segments[3]
                .chars()
                .all(|character| character.is_ascii_digit()))
        .then(|| url.to_owned())
    })
}
/// Recover a command-output file request from a structural, seed-backed frame.
///
/// The command must immediately follow a seed-defined run verb and be enclosed
/// in single quotes, double quotes, or backticks. The suffix must name a
/// seed-defined command-output reference, a file-write action, and a safe target
/// introduced by a write target/destination cue. Requiring every element keeps
/// an incidental quoted phrase or filename from becoming executable.
fn parse_command_output_request(request: &str) -> Option<(String, String)> {
    let toks = tokens(request);
    let run_verbs = seed::terminal_command_vocabulary().run_verbs;
    let actions = bare_surfaces(seed::ROLE_FILE_WRITE_ACTION_CUE);
    let targets = bare_surfaces(seed::ROLE_FILE_WRITE_TARGET_CUE);
    let destinations = bare_surfaces(seed::ROLE_FILE_WRITE_DESTINATION_CUE);
    for run in toks
        .iter()
        .filter(|token| run_verbs.contains(&clean_cue_token(token.text)))
    {
        let tail = request.get(run.end..)?;
        let leading = tail.len() - tail.trim_start().len();
        let quoted = tail.get(leading..)?;
        let quote = quoted.chars().next()?;
        if !matches!(quote, '\'' | '"' | '`') {
            continue;
        }
        let body = quoted.get(quote.len_utf8()..)?;
        let Some(close) = body.find(quote) else {
            continue;
        };
        let command = body.get(..close)?.trim();
        if command.is_empty() || command.contains(['\n', '\r', '\0']) {
            continue;
        }
        let suffix_offset = run.end + leading + quote.len_utf8() + close + quote.len_utf8();
        let suffix = request.get(suffix_offset..)?;
        if !mentions_bare_role(suffix, seed::ROLE_FILE_WRITE_COMMAND_OUTPUT_REFERENCE) {
            continue;
        }
        let suffix_tokens = tokens(suffix);
        let has_write_action = suffix_tokens
            .iter()
            .any(|token| actions.contains(&clean_cue_token(token.text)));
        if !has_write_action {
            continue;
        }
        let target = suffix_tokens.iter().enumerate().find_map(|(index, token)| {
            let cleaned = clean_path_token(token.text);
            let looks_like_file = looks_like_file_path(cleaned);
            let previous = index
                .checked_sub(1)
                .map(|position| &suffix_tokens[position])?;
            let cue = clean_cue_token(previous.text);
            (looks_like_file
                && safe_relative_path(cleaned)
                && (targets.contains(&cue) || destinations.contains(&cue)))
            .then(|| cleaned.to_owned())
        });
        if let Some(target) = target {
            return Some((target, command.to_owned()));
        }
    }
    None
}
pub(super) fn mentions_bare_role(text: &str, role: &str) -> bool {
    let lower = text.to_lowercase();
    seed::lexicon()
        .role_word_forms(role)
        .iter()
        .filter(|form| form.slot() == Slot::Bare)
        .any(|form| {
            let needle = form.text.to_lowercase();
            let Some(start) = lower.find(&needle) else {
                return false;
            };
            if !needle.is_ascii() {
                return true;
            }
            let end = start + needle.len();
            let before_ok = start == 0
                || lower[..start]
                    .chars()
                    .next_back()
                    .is_some_and(|character| !character.is_alphanumeric());
            let after_ok = end == lower.len()
                || lower[end..]
                    .chars()
                    .next()
                    .is_some_and(|character| !character.is_alphanumeric());
            before_ok && after_ok
        })
}
pub(super) fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}
fn command_plan_text(intent: &str, language: &str, target: &str) -> String {
    seed::localized_response(intent, language)
        .unwrap_or_else(|| intent.to_owned())
        .replace(TARGET_PLACEHOLDER, target)
}
/// Whether `lower` (an already-lowercased request) is a file **write / create**
/// intent — a write verb applied to something file-shaped. This is the single
/// signal the router uses to keep a file-creation request from ever being
/// misrouted to the file-read recipe (issue #681): a request to *produce* a file
/// is a write, never a read of the not-yet-existing target.
///
/// This is intentionally the same structural parse used to compose the eventual
/// write plan. A request is classified as a write only when the seed-defined
/// action/target/content roles yield a safe target and non-empty payload. One
/// parser for classification and composition cannot drift into claiming an
/// operation that the planner is unable to execute.
#[must_use]
pub(crate) fn has_file_write_intent(lower: &str) -> bool {
    parse_write_request(lower).is_some()
}
/// Whether a request explicitly marks its recovered payload as authoritative
/// literal bytes rather than a description of another workspace operation.
///
/// The marker is seed-backed and multilingual. Requiring both the narrow marker
/// role and a successfully composed literal-file plan prevents arbitrary prose
/// containing "exactly" from changing planner precedence.
pub(super) fn has_authoritative_literal_write(request: &str) -> bool {
    let normalized = request.to_lowercase();
    let authoritative = first_prefix_lead_end(
        &normalized,
        seed::ROLE_FILE_WRITE_AUTHORITATIVE_CONTENT_LEAD,
    )
    .is_some()
        || literal_payload(request).is_some();
    authoritative
        && compose_general_change_plan(request)
            .is_some_and(|plan| plan.mode == GeneralPlanMode::LiteralFile)
}
/// Resolve an edit target named in a request through the workspace self-AST
/// census (issue #673).
///
/// Before the census existed, the planner could only edit a file the request spelt
/// out in full, and its own self-inspection was pinned to a single hardcoded module
/// (`src/agentic_coding/planner.rs`). Now any `path`, `path:symbol`, unambiguous
/// module suffix, or uniquely-declared item name resolves to the real module path
/// through [`crate::self_ast_census`], so the planner can address every module of
/// the workspace by the same mechanism.
///
/// The token must *address* the workspace to be resolved: it has to carry a
/// directory component (`agentic_coding/source_links.rs`) or a `path:symbol`
/// pair (`self_ast_census.rs:resolve_census_target`). A bare file name such as
/// `main.rs` is left exactly as the request spelt it, because the request may be
/// about the *client's* working directory rather than this workspace, and an
/// ordinary word that happens to match an item name is never mistaken for an edit
/// target. The census itself fails closed on anything ambiguous.
#[must_use]
pub fn resolve_census_target(reference: &str) -> Option<CensusResolution> {
    let addresses_workspace =
        reference.contains('/') || (reference.contains(':') && !reference.contains("://"));
    if !addresses_workspace {
        return None;
    }
    self_ast_census::workspace().resolve(reference)
}
/// Whether `text` carries a software-authoring verb (implement, resolve,
/// develop, …) in any seeded language.
#[must_use]
pub fn mentions_software_authoring(text: &str) -> bool {
    mentions_bare_role(text, seed::ROLE_SOFTWARE_AUTHORING_ACTION)
}
const fn capability_slug(capability: Capability) -> &'static str {
    match capability {
        Capability::Search => "Search",
        Capability::Fetch => "Fetch",
        Capability::Read => "Read",
        Capability::Write => "Write",
        Capability::Edit => "Edit",
        Capability::Run => "Run",
        Capability::Grep => "Grep",
        Capability::Glob => "Glob",
        Capability::ListDir => "ListDir",
        Capability::Todo => "Todo",
        Capability::Subagent => "Subagent",
        Capability::ReadMany => "ReadMany",
        Capability::MultiEdit => "MultiEdit",
        Capability::AskUser => "AskUser",
    }
}
fn language(request: &str) -> &'static str {
    crate::language::detect(request).slug()
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
