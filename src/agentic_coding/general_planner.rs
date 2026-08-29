//! Deterministic fallback planner for repository change requests (issue #654).
//!
//! Unlike the stored recipe fixtures, this planner derives its target and payload
//! from the formalized request.  The resulting plan is data: it is serialized to
//! Links Notation and written before execution, so the tool transcript is an
//! append-only record of the decision that caused the change.
use super::planner::{Capability, trace_route};
use super::shell_command_policy::sentences;
use super::write_request::{
    bare_surfaces, clean_cue_token, clean_content, clean_path_token, cued_write_target,
    first_action_cue_end, first_content_lead_end, first_prefix_lead_end,
    honouring_pinned_first_line, looks_like_file_path, safe_relative_path, tokens,
};
use crate::engine::stable_id;
use crate::intent_formalization::formalize_intent;
use crate::seed::{self, Slot};
use crate::self_ast_census::{self, CensusResolution};
use std::fmt::Write as _;
/// Workspace-relative event-log artifact written before a general plan executes.
pub const PLAN_PATH: &str = ".formal-ai/general-change-plan.lino";
const TARGET_PLACEHOLDER: &str = "{target}";
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
    const fn slug(self) -> &'static str {
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
        let mut out = String::from("general_change_plan\n");
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
    let lowered = request.to_lowercase();
    first_prefix_lead_end(&lowered, seed::ROLE_REQUEST_OBJECTIVE_LEAD)
        .filter(|(start, _)| line_anchored(&lowered, *start))
        .and_then(|(_, end)| request.get(end..))
        .map_or(request, str::trim)
}
/// Whether a marker at `start` opens its own line, so a delimiter quoted inside
/// running prose ("write the words request: hello to notes.txt") does not
/// silently truncate the request.
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
    let content = crate::implementation_language::without_trailing_known_modifier(&content)
        .unwrap_or(content);
    // Issue #1066: the same request can state the bytes and, separately,
    // constrain the line the file has to open with. The repair belongs here
    // rather than at the call sites, because every step of the plan quotes the
    // content it was composed from -- the verification step's expected evidence
    // most of all -- and a plan whose steps disagree with its own bytes is not
    // one a reader can check.
    let content = honouring_pinned_first_line(full_request, &content).map_or(content, |repaired| {
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
    let response_language = language(request);
    let intent = formalize_intent(request, response_language, None);
    let verification_command = format!("cat {target}");
    let mut steps = vec![GeneralPlanStep {
        capability: Capability::Write,
        action: format!("append the composed plan to {PLAN_PATH}"),
        expected_evidence: format!("written plan event {}", intent.impulse_id),
        command: None,
    }];
    if let Some((_, command)) = &command_output {
        let generation_command = format!("{command} > {}", shell_quote(&target));
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
    if !mentions_bare_role(request, seed::ROLE_SOFTWARE_AUTHORING_ACTION) {
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
fn repository_work_reference(request: &str) -> Option<String> {
    request.split_whitespace().find_map(|token| {
        let url = token.trim_matches(|character: char| {
            matches!(
                character,
                '<' | '>' | '(' | ')' | '[' | ']' | '{' | '}' | ',' | ';' | '.' | '"' | '\''
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
fn mentions_bare_role(text: &str, role: &str) -> bool {
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
fn shell_quote(value: &str) -> String {
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
    first_prefix_lead_end(
        &normalized,
        seed::ROLE_FILE_WRITE_AUTHORITATIVE_CONTENT_LEAD,
    )
    .is_some()
        && compose_general_change_plan(request)
            .is_some_and(|plan| plan.mode == GeneralPlanMode::LiteralFile)
}
/// Recover the `(target, content)` of a write request from its wording.
///
/// The recogniser is entirely seed-driven (issue #680). It locates the target
/// file by a `file_write_target_cue`/`file_write_destination_cue` that directly
/// precedes a file-looking, safe relative path, then recovers the content two
/// ways:
///
/// * **Marker-led** — a `file_write_content_lead` phrase ("containing", "with
///   the following", …) introduces the payload. The content is the span after
///   the marker (when the file precedes it) or the span between the marker and
///   the file clause (when the content precedes the file).
/// * **Destination-led** — the "write CONTENT to FILE" shape, where a
///   `file_write_action_cue` opens the request and a *destination* cue (not a
///   positional target cue) routes the preceding span into the file.
/// * **Action-led** — the "write FILE saying CONTENT" shape, where a
///   `file_write_action_cue` ("write"/"create"/"save"/…) directly names the file
///   and a content-lead marker introduces the payload after it. An action cue
///   licenses the file just like a target cue, but only marker-led content is
///   accepted for it, so a bare "create app.rs" (no content) still falls through
///   to the ordinary solver rather than fabricating an empty file.
///
/// Both byte offsets index the lowercased copy, which is byte-length preserving
/// for en/ru/hi/zh, so the same offsets slice the original request and the
/// recovered content keeps its case and punctuation.
fn parse_write_request(request: &str) -> Option<(String, String)> {
    let lowered = request.to_lowercase();
    let toks = tokens(request);
    let dest_cues = bare_surfaces(seed::ROLE_FILE_WRITE_DESTINATION_CUE);
    let (file_index, target) = cued_write_target(&toks)?;
    let cue = &toks[file_index - 1];
    let clause_start = cue.start;
    let cue_is_destination = dest_cues.contains(&clean_cue_token(cue.text));
    // Marker-led content. The payload sits after the marker, bounded by the file
    // clause when the marker comes first ("write the following: hello to x.txt")
    // and running to the end when the clause comes first ("store file x.txt
    // containing hello").
    //
    // A marker that *precedes* the clause additionally needs a write verb, which
    // is the same rule the destination-led and assignment-shaped branches below
    // already apply — without it a read request whose object happens to be a
    // content-lead surface claims a write. The issue-#671 matrix caught
    // `show me the contents of the file beta.md` planning
    // `write(beta.md, "of the")`, destroying the fixture it was asked to read.
    if let Some((_, marker_end)) = first_content_lead_end(&lowered) {
        let marker_leads = marker_end <= clause_start;
        let marker_span = if marker_leads {
            request.get(marker_end..end_of_statement(request, marker_end, clause_start))
        } else {
            request.get(marker_end..end_of_statement(request, marker_end, request.len()))
        };
        if (!marker_leads || first_action_cue_end(&toks).is_some())
            && let Some(content) = marker_span
                .and_then(clean_content)
                .filter(|content| is_literal_content(content))
            {
                return Some((target, content));
            }
    }
    let content_span = if cue_is_destination {
        let action_end = first_action_cue_end(&toks)?;
        (action_end <= clause_start).then(|| request.get(action_end..clause_start))?
    } else if let Some(value_lead) = toks
        .iter()
        .skip(file_index + 1)
        .find(|token| dest_cues.contains(&clean_cue_token(token.text)))
    {
        // Assignment shape: "set the contents of FILE to VALUE". The target
        // cue identifies the file object and a following destination cue
        // introduces its literal value. Requiring a write action before the
        // file keeps an unrelated "contents of FILE" read request out.
        let action_end = first_action_cue_end(&toks)?;
        (action_end <= clause_start).then(|| request.get(value_lead.end..))?
    } else {
        None
    };
    let content = clean_content(content_span?)?;
    // A recovered payload that is *only* a non-referential subject ("save it to
    // FILE", "write this to FILE") names no literal content — the pronoun points
    // back at content the request expects the recipe to still compose. Treating
    // it as a literal write both fabricates the wrong file (the string "it") and
    // steals the request from the keyword recipe that would author the real
    // artifact, so fall through instead (issue #663).
    //
    // The same is true of a payload that names the *work product* rather than
    // supplying it: "save the answer to FILE" states where an answer goes, not
    // what it says (issue #1066).
    if is_non_referential_content(&content)
        || names_deferred_work_product(&content)
        || !is_literal_content(&content)
    {
        return None;
    }
    Some((target, content))
}
/// Where the statement that begins at `from` ends, never past `limit`.
///
/// A literal payload is something the request *states*, and a statement ends
/// where its sentence does. Bounding the span by the file clause alone reads
/// across every sentence in between: "Draft a handover memo containing the
/// migration status, the outstanding blockers, and the on-call owner. Leave the
/// memo in `handover/2026-q3.md`" put the marker in the first sentence and the
/// clause in the second, so the recovered payload ended with the words *Leave
/// the memo* and the caller's memo opened by instructing them to leave it
/// (issue #1066).
///
/// The clause bound still applies inside the sentence, because "write the
/// following: hello to `x.txt`" states marker, payload and clause in one
/// breath. This only refuses to look further than the sentence the marker is in.
///
/// A marker that says nothing more on its own line is the exception, because
/// there the payload is a *block* rather than a phrase: "Create file
/// `rules.lino` containing\n<three lines of lino>" leaves the marker with an
/// empty tail, and a newline ends a sentence, so the sentence bound would
/// recover nothing at all. When the marker's own line has no word left on it,
/// the statement is the block that follows and runs to `limit`, which is what
/// this route always did for block payloads.
fn end_of_statement(request: &str, from: usize, limit: usize) -> usize {
    let Some(sentence) = sentences(request)
        .into_iter()
        .find(|sentence| sentence.span.contains(&from))
    else {
        return limit;
    };
    let says_more = request
        .get(from..sentence.span.end)
        .is_some_and(|tail| tail.chars().any(char::is_alphanumeric));
    if says_more {
        sentence.span.end.min(limit)
    } else {
        limit
    }
}
/// Whether a recovered payload says anything at all. A span of nothing but
/// punctuation is what a mis-parse leaves behind — the `opencode` leg of the
/// issue-#671 matrix recovered a single `"`, the tail of a quoted prompt after
/// its trailing content-lead marker — and writing it would replace real file
/// bytes with a stray delimiter.
fn is_literal_content(content: &str) -> bool {
    content.chars().any(char::is_alphanumeric)
}
/// Whether a recovered write payload is nothing but a non-referential subject —
/// a bare pronoun/function word ("it", "this", "that", …) that refers back to
/// context rather than naming literal content. The surfaces carry the
/// [`seed::ROLE_NON_REFERENTIAL_SUBJECT`] role; only whole-word
/// ([`Slot::Bare`]) forms are rejected, so legitimate content that merely
/// *begins* with such a word ("to be or not to be") is still accepted.
fn is_non_referential_content(content: &str) -> bool {
    let lower = content.to_lowercase();
    seed::lexicon()
        .role_word_forms(seed::ROLE_NON_REFERENTIAL_SUBJECT)
        .iter()
        .any(|form| form.slot() == Slot::Bare && lower == form.text)
}
/// Whether a recovered write payload names the result of work the same request
/// asks for, instead of supplying bytes (issue #1066).
///
/// "Save the answer to `out/e.md`" and "leave observable evidence to
/// `out/e.md`" have the destination-led shape of a literal write -- a write
/// verb, a span, a destination cue, a path -- and supply no literal. Taking the
/// span at face value wrote the words *the answer* into the file and, worse,
/// claimed the request as finished, so the investigation that would have
/// produced the real bytes never ran.
///
/// The surfaces carry [`seed::ROLE_FILE_WRITE_DEFERRED_CONTENT_REFERENCE`] as
/// [`Slot::Suffix`] forms, because the head noun is what defers and the
/// modifiers in front of it ("observable", "final") are the caller's, not the
/// lexicon's.
///
/// Deliberately not applied to marker-led content. "Create `a.txt` containing 42
/// is the answer" states, with the marker, that the span *is* the payload; only
/// the shapes that infer a payload from position need the check.
fn names_deferred_work_product(content: &str) -> bool {
    let lower = content.to_lowercase();
    seed::lexicon()
        .role_word_forms(seed::ROLE_FILE_WRITE_DEFERRED_CONTENT_REFERENCE)
        .iter()
        .any(|form| match form.slot() {
            Slot::Bare => lower == form.text,
            Slot::Suffix => ends_with_head_noun(&lower, form.after_slot().trim_start()),
            Slot::Prefix | Slot::Circumfix => false,
        })
}
/// Whether `content` ends with `noun` standing as its own word.
///
/// English and Russian separate a head noun from its modifiers with a space, so
/// the character before the match settles it. Chinese writes the same phrase
/// with no separator at all, which is why the boundary is stated as "not an
/// ASCII alphanumeric" rather than "whitespace": demanding a space would never
/// match 结论, and demanding nothing would match the tail of an unrelated
/// English word.
fn ends_with_head_noun(content: &str, noun: &str) -> bool {
    !noun.is_empty()
        && content.strip_suffix(noun).is_some_and(|before| {
            before
                .chars()
                .next_back()
                .is_none_or(|character| !character.is_ascii_alphanumeric())
        })
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
/// Recover the `(target, old, new)` of a file-edit request from its wording
/// (issue #680).
///
/// The recogniser is entirely seed-driven. It locates the target file by a
/// [`ROLE_FILE_EDIT_TARGET_CUE`](seed::ROLE_FILE_EDIT_TARGET_CUE) ("in", "within",
/// "of", "file", …) that directly precedes a file-looking, safe relative path,
/// finds the leftmost
/// [`ROLE_FILE_EDIT_ACTION_CUE`](seed::ROLE_FILE_EDIT_ACTION_CUE) ("change",
/// "replace", "edit", …), then the first
/// [`ROLE_FILE_EDIT_NEW_LEAD_CUE`](seed::ROLE_FILE_EDIT_NEW_LEAD_CUE) ("to",
/// "with", "into", …) after it. The *old* text is the span between the action and
/// the new-lead; the *new* text is the span after the new-lead, bounded by the
/// file clause when the file follows the replacement (the "replace OLD with NEW in
/// FILE" shape) or running to the end when the file was named first (the "in FILE,
/// change OLD to NEW" shape).
///
/// Returns [`None`] unless a target file, an action cue, a new-lead, and non-empty
/// old and new spans are all present — and unless the file clause sits *outside*
/// the replaced span — so ambiguous or non-edit requests fall through to the
/// ordinary solver rather than fabricating an edit.
///
/// Byte offsets index the original request directly (the cue matching lowercases
/// per token, which is byte-length preserving for en/ru/hi/zh), so the recovered
/// old/new text keeps its original case and punctuation.
#[must_use]
pub fn compose_edit_request(request: &str) -> Option<(String, String, String)> {
    let toks = tokens(request);
    let action_cues = bare_surfaces(seed::ROLE_FILE_EDIT_ACTION_CUE);
    let new_leads = bare_surfaces(seed::ROLE_FILE_EDIT_NEW_LEAD_CUE);
    let target_cues = bare_surfaces(seed::ROLE_FILE_EDIT_TARGET_CUE);
    // The target file: the first safe, file-looking token that sits directly beside
    // a target cue — before it in prepositional languages ("in notes.txt") or after
    // it in postpositional ones ("doc.txt में", "the report.md file"). Requiring the
    // cue keeps an incidental dotted token out of the edit path, exactly as the
    // write recogniser does.
    let is_target_cue = |index: usize| target_cues.contains(&clean_cue_token(toks[index].text));
    let is_action_cue = |index: usize| action_cues.contains(&clean_cue_token(toks[index].text));
    let (file_index, target) = toks.iter().enumerate().find_map(|(index, token)| {
        let cleaned = clean_path_token(token.text);
        // A repository target may be named as a bare module or item — `source_links.rs`,
        // `src/agentic_coding/source_links.rs:render_document`, or just
        // `is_source_links_task` — in which case the workspace self-AST census
        // (issue #673) resolves it to the module that actually declares it.
        let resolved = resolve_census_target(cleaned);
        let looks_like_file = looks_like_file_path(cleaned);
        if resolved.is_none() && (!looks_like_file || !safe_relative_path(cleaned)) {
            return None;
        }
        let prev_is_cue = index
            .checked_sub(1)
            .is_some_and(|previous| is_target_cue(previous) || is_action_cue(previous));
        let next_is_cue =
            (index + 1 < toks.len()) && (is_target_cue(index + 1) || is_action_cue(index + 1));
        let target = resolved.map_or_else(|| cleaned.to_owned(), |census| census.module_path);
        (prev_is_cue || next_is_cue).then_some((index, target))
    })?;
    // Extend the clause boundary left over any run of target cues so a multi-word
    // file clause ("в файле notes.txt") is excluded from the replacement text in
    // full, not just its innermost word.
    let mut clause_start_index = file_index;
    while clause_start_index > 0 && is_target_cue(clause_start_index - 1) {
        clause_start_index -= 1;
    }
    let file_clause_start = toks[clause_start_index].start;
    // The edit action opens the replacement clause; the new-lead separates the old
    // text from the new text. The new-lead must follow the action so a "to"/"with"
    // belonging to an earlier clause is never mistaken for the replacement lead.
    // When a leading edit action names the target ("update FILE and change A to
    // B"), prefer the later action that introduces the replacement itself.
    let action = toks
        .iter()
        .filter(|token| action_cues.contains(&clean_cue_token(token.text)))
        .find(|token| token.start > toks[file_index].end)
        .or_else(|| {
            toks.iter()
                .find(|token| action_cues.contains(&clean_cue_token(token.text)))
        })?;
    let action_end = action.end;
    let new_lead = toks.iter().find(|token| {
        token.start >= action_end && new_leads.contains(&clean_cue_token(token.text))
    })?;
    // A well-formed edit names the file before the action ("in F, change A to B")
    // or after the replacement ("replace A with B in F") — never between the action
    // and the new-lead, which would fold the filename into the replaced text.
    if file_clause_start >= action_end && file_clause_start < new_lead.start {
        return None;
    }
    let old_span = request.get(action_end..new_lead.start)?;
    let new_end = if file_clause_start > new_lead.end {
        file_clause_start
    } else {
        request.len()
    };
    let new_span = request.get(new_lead.end..new_end)?;
    let old = clean_content(old_span)?;
    let new = clean_content(new_span)?;
    Some((target, old, new))
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

