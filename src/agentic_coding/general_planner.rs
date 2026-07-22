//! Deterministic fallback planner for repository change requests (issue #654).
//!
//! Unlike the stored recipe fixtures, this planner derives its target and payload
//! from the formalized request.  The resulting plan is data: it is serialized to
//! Links Notation and written before execution, so the tool transcript is an
//! append-only record of the decision that caused the change.

use std::fmt::Write as _;

use crate::engine::stable_id;
use crate::intent_formalization::formalize_intent;
use crate::seed::{self, Slot};
use crate::self_ast_census::{self, CensusResolution};

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

/// Compose a safe file-creation plan from arbitrary wording in any supported
/// language (issue #680).
///
/// The universal intent formalizer supplies the stable impulse identity.  The
/// decomposition is deliberately bounded to requests that state both a relative
/// target file and literal content; ambiguous requests continue to the ordinary
/// solver instead of inventing a patch or shell command.
///
/// The target, the content, and the write *intent* itself are all recognised
/// from the seed lexicon (the `file_write_*` roles in
/// `data/seed/meanings-file-write.lino`) rather than from a hardcoded list of
/// English or Russian phrasings, so a file-creation request in en/ru/hi/zh — in
/// any phrasing — routes to the write tool (CONTRIBUTING §2).
#[must_use]
pub fn compose_general_change_plan(request: &str) -> Option<GeneralChangePlan> {
    let (target, content) = parse_write_request(request)?;
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
pub(super) fn has_file_write_intent(lower: &str) -> bool {
    parse_write_request(lower).is_some()
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
    let target_cues = bare_surfaces(seed::ROLE_FILE_WRITE_TARGET_CUE);
    let dest_cues = bare_surfaces(seed::ROLE_FILE_WRITE_DESTINATION_CUE);
    let action_cues = bare_surfaces(seed::ROLE_FILE_WRITE_ACTION_CUE);

    // The target file: the first safe, file-looking token that directly follows a
    // target cue, a destination cue, or an action cue. Requiring a cue keeps an
    // incidental dotted token (a version, an abbreviation) out of the write path.
    let (file_index, target) = toks.iter().enumerate().find_map(|(index, token)| {
        let cleaned = clean_path_token(token.text);
        let looks_like_file = cleaned.contains('.') && !cleaned.contains("://");
        if !looks_like_file || !safe_relative_path(cleaned) {
            return None;
        }
        let previous = index.checked_sub(1).map(|i| &toks[i])?;
        let previous_word = clean_cue_token(previous.text);
        (target_cues.contains(&previous_word)
            || dest_cues.contains(&previous_word)
            || action_cues.contains(&previous_word))
        .then(|| (index, cleaned.to_owned()))
    })?;

    let cue = &toks[file_index - 1];
    let clause_start = cue.start;
    let cue_is_destination = dest_cues.contains(&clean_cue_token(cue.text));

    if let Some((_, marker_end)) = first_content_lead_end(&lowered) {
        let marker_span = if marker_end <= clause_start {
            request.get(marker_end..clause_start)
        } else {
            request.get(marker_end..)
        };
        if let Some(content) = marker_span.and_then(clean_content) {
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
    if is_non_referential_content(&content) {
        return None;
    }
    Some((target, content))
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
        let looks_like_file = cleaned.contains('.') && !cleaned.contains("://");
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

/// One whitespace token together with its byte span in the original request.
struct Token<'a> {
    text: &'a str,
    start: usize,
    end: usize,
}

/// Split a request into whitespace tokens, recording each token's byte span.
fn tokens(request: &str) -> Vec<Token<'_>> {
    let mut cursor = 0;
    request
        .split_whitespace()
        .map(|word| {
            let start = request[cursor..]
                .find(word)
                .map_or(cursor, |offset| cursor + offset);
            let end = start + word.len();
            cursor = end;
            Token {
                text: word,
                start,
                end,
            }
        })
        .collect()
}

/// The bare (whole-word) surface forms for a role, lowercased for token matching.
fn bare_surfaces(role: &str) -> Vec<String> {
    seed::lexicon()
        .role_word_forms(role)
        .iter()
        .filter(|form| form.slot() == Slot::Bare)
        .map(|form| form.text.to_lowercase())
        .collect()
}

/// Trim the quoting/edge punctuation from a token that may be a file path,
/// preserving the interior dots that make it look like a file. Trailing sentence
/// punctuation is stripped too, so a plain word that merely *ends a sentence*
/// ("… add the plural to томат.") is not mistaken for a file whose only dot is the
/// terminal period — a real filename never ends in a bare `.`/`!`/`?`.
fn clean_path_token(word: &str) -> &str {
    word.trim_matches(|c: char| matches!(c, '`' | '"' | '\'' | ',' | ':' | ';'))
        .trim_end_matches(['.', '!', '?'])
}

/// Lowercase a token stripped of edge punctuation, for cue/action comparison.
fn clean_cue_token(word: &str) -> String {
    word.trim_matches(|c: char| matches!(c, '`' | '"' | '\'' | ',' | ':' | ';' | '.' | '!' | '?'))
        .to_lowercase()
}

/// The byte span just past the leftmost `file_write_content_lead` marker in the
/// lowercased request, honouring whole-word boundaries for space-delimited
/// scripts and substring matches for CJK (which has no inter-word spaces).
fn first_content_lead_end(lowered: &str) -> Option<(usize, usize)> {
    let markers: Vec<String> = seed::lexicon()
        .role_word_forms(seed::ROLE_FILE_WRITE_CONTENT_LEAD)
        .iter()
        .filter(|form| form.slot() == Slot::Prefix)
        .map(|form| form.before_slot().trim().to_lowercase())
        .filter(|marker| !marker.is_empty())
        .collect();
    let mut best: Option<(usize, usize)> = None;
    for marker in &markers {
        let mut from = 0;
        while let Some(relative) = lowered[from..].find(marker.as_str()) {
            let start = from + relative;
            let end = start + marker.len();
            let cjk = !marker.contains(' ') && !marker.is_ascii();
            let before_ok = cjk
                || start == 0
                || lowered[..start]
                    .chars()
                    .next_back()
                    .is_some_and(char::is_whitespace);
            let after_ok = cjk
                || end == lowered.len()
                || lowered[end..]
                    .chars()
                    .next()
                    .is_some_and(|c| c.is_whitespace() || c.is_ascii_punctuation());
            if before_ok && after_ok {
                if best.is_none_or(|(best_start, _)| start < best_start) {
                    best = Some((start, end));
                }
                break;
            }
            from = end;
        }
    }
    best
}

/// The byte offset just past the first `file_write_action_cue` token.
fn first_action_cue_end(toks: &[Token<'_>]) -> Option<usize> {
    let actions = bare_surfaces(seed::ROLE_FILE_WRITE_ACTION_CUE);
    toks.iter()
        .find(|token| actions.contains(&clean_cue_token(token.text)))
        .map(|token| token.end)
}

/// Trim a recovered content span down to its literal payload, dropping the
/// leading clause separator ("… the following: hello") and any surrounding
/// quoting. A delimiter is removed only when the entire payload has a matching
/// opening and closing delimiter. This matters for generated source and Links
/// Notation: a lone terminal quote is data, not presentation punctuation.
/// Returns [`None`] when nothing is left.
fn clean_content(raw: &str) -> Option<String> {
    let led = raw.trim().trim_start_matches([':', '-', '—', '–']).trim();
    let result = if led.len() >= 6 && led.starts_with("```") && led.ends_with("```") {
        led[3..led.len() - 3].trim()
    } else if led.len() >= 2 {
        let first = led.as_bytes()[0];
        let last = led.as_bytes()[led.len() - 1];
        if first == last && matches!(first, b'`' | b'"' | b'\'') {
            led[1..led.len() - 1].trim()
        } else {
            led
        }
    } else {
        led
    };
    (!result.is_empty()).then(|| result.to_owned())
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
