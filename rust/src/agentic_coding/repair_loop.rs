//! Error-driven repair loop for the agentic executor (issue #1185, E149).
//!
//! The architect's vision is the traditional way of coding by hand: build or
//! run, *read* the compiler/runtime error, search the error text, read the
//! fix, apply it, retry. The executor today detects a failed step honestly
//! ([`super::tool_result`]) and reports it ([`super::command_reroute`]'s
//! `StepFailure`), but the recipe stops at the first failed step — nothing
//! closes the loop back to a fix.
//!
//! This module is that loop, as a deterministic planner step like
//! [`super::web_research`]: formalize the failed output into structured
//! diagnostics (R1), search the diagnostic (R2), retain a fetched fragment
//! that addresses the exact error code or message as a candidate fix (R2),
//! express the fix as a Links Notation `repair_edit` record rather than a raw
//! text patch (R3), retry the failed command on the next ladder rung (R4),
//! and record every attempt as an evidence chain for the answer's derivation
//! (R5). A diagnostic no fetched source addresses is reported as an
//! unresolved need, never as a fabricated fix (R6).
//!
//! The loop is uniform over every language Formal AI emits code in (R7): what
//! varies per language — where the error code sits in a diagnostic line, and
//! where the file/line location is spelled — lives in
//! `data/seed/diagnostic-code-shapes.lino` as `{slot}` templates, and the
//! matcher below interprets those templates generically, the same
//! data-over-code shape as the cue loaders. No per-language fix table exists
//! anywhere: the fix always comes from a fetched, matched source (R2/R3).
//!
//! Neural inference stays a NON-GOAL: matching is symbolic token overlap, the
//! same non-neural measure [`super::web_research`] ranks pages with.
//!
//! ## Integration seam (wired by the main session)
//!
//! `command_reroute.rs`'s failure branch (lines 67-84 today) finalizes a
//! failed step immediately. The one-line integration consults this loop
//! first, before constructing [`AgenticPlan::Final`]:
//!
//! ```ignore
//! if let Some(failure) = &progress.failure {
//!     let step_data = super::repair_loop::FailedStep::new(&recipe.language, &failure.reported)
//!         .with_exit_code(failure.exit_code)
//!         .with_failed_command(/* the `step` command_reroute already computes */);
//!     if let Some(plan) = super::repair_loop::plan_repair(
//!         messages, tool_names, &step_data, progress.repair_rung,
//!         super::repair_loop::MAX_REPAIR_RUNGS,
//!     ) {
//!         return Some(plan);
//!     }
//!     // exhausted or unmatched: today's honest report, plus the ladder note
//! }
//! ```
//!
//! `RecipeProgress` gains a `repair_rung: u8` field the caller increments
//! when a retry fails again, mirroring plan 06's `ExecutionBox` iteration
//! ladder rather than inventing a second ladder (R4).

use std::collections::BTreeSet;
use std::fmt::Write as _;

use serde_json::json;

use super::planner::{
    AgenticPlan, Capability, Progress, fetch_arguments, plan_one, tool_for, write_arguments,
};
use crate::protocol::ChatMessage;
use crate::seed::parser::parse_lino;

/// How many repair rungs one failed step may climb (R4).
///
/// Each rung is one formalize → search → fetch → record → retry pass. The
/// bound exists because the loop's own stopping rule — a fetched source that
/// addresses the diagnostic — can be unreachable when the fix simply is not
/// on the open web, and a repair loop must terminate either way (R6). Three
/// rungs mirror the search→fetch research budget in [`super::web_research`].
pub const MAX_REPAIR_RUNGS: u8 = 3;

/// How many sources one rung's search may read (mirrors the research round's
/// breadth bound: enough to triangulate a fix without an unbounded crawl).
const MAX_REPAIR_SOURCES: usize = 3;

/// Longest message fragment carried into a search query, in tokens.
const QUERY_MESSAGE_TOKENS: usize = 12;

const SHAPES_PATH: &str = "data/seed/diagnostic-code-shapes.lino";

/// Look up embedded seed content by its registered path (same loader the
/// cue handlers use). `None` until the seed is registered in
/// `data/meta/seed-registry.lino` — the loop is inert, not wrong, without it.
fn seed_text(path: &str) -> Option<&'static str> {
    crate::seed::seed_files()
        .into_iter()
        .find(|(registered, _)| *registered == path)
        .map(|(_, text)| text)
}

// ---------------------------------------------------------------------------
// R1 — formalize the failed output into structured diagnostics
// ---------------------------------------------------------------------------

/// One structured diagnostic read out of a failed step's raw output: the
/// file, line, error code and message the compiler or interpreter stated,
/// plus the raw line it was read from.
///
/// `code` is `None` for languages whose diagnostics carry no machine code
/// (Kotlin, Go, Scala, …) — absent, never fabricated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub file: Option<String>,
    pub line: Option<u32>,
    pub code: Option<String>,
    pub message: String,
    pub raw: String,
}

/// The per-language shape data from `data/seed/diagnostic-code-shapes.lino`:
/// line templates for diagnostic entries and locations, plus the known
/// diagnostic head words for languages that spell errors as `Head: message`.
#[derive(Default)]
struct DiagnosticShapes {
    patterns: Vec<String>,
    locations: Vec<String>,
    heads: Vec<String>,
}

impl DiagnosticShapes {
    fn is_empty(&self) -> bool {
        self.patterns.is_empty()
    }
}

/// Normalize a recipe language slug to the seed's language ids.
fn shape_language(language: &str) -> String {
    match language.trim().to_ascii_lowercase().as_str() {
        "c++" | "cxx" => "cpp".to_owned(),
        "c#" => "csharp".to_owned(),
        "js" => "javascript".to_owned(),
        "ts" => "typescript".to_owned(),
        other => other.to_owned(),
    }
}

/// Load the shape templates for `language`. Fields sit one wrapper level
/// below the `language` record, the nesting every seed file uses.
fn shapes_for(language: &str) -> DiagnosticShapes {
    let Some(text) = seed_text(SHAPES_PATH) else {
        return DiagnosticShapes::default();
    };
    let tree = parse_lino(text);
    let slug = shape_language(language);
    let mut shapes = DiagnosticShapes::default();
    for record in tree
        .children
        .iter()
        .filter(|child| child.name == "language" && child.id == slug)
    {
        for field in &record.children {
            match field.name.as_str() {
                "pattern" if !field.id.is_empty() => shapes.patterns.push(field.id.clone()),
                "location" if !field.id.is_empty() => shapes.locations.push(field.id.clone()),
                "head" if !field.id.is_empty() => shapes.heads.push(field.id.clone()),
                _ => {}
            }
        }
    }
    shapes
}

/// One captured `{slot}` of a matched template.
struct Capture {
    slot: String,
    value: String,
}

/// Match `line` against a seed `{slot}` template, anchored at the line start.
///
/// The template alternates literals and slots (`error[{code}]: {message}` is
/// literal `error[`, slot `code`, literal `]: `, slot `message`). Each
/// literal must be present in order; each slot captures the *minimal* run of
/// text up to the next literal. Slots carry their own validation, so a
/// template cannot swallow prose as a file or line number: `{file}` must
/// contain a `.` or `/` (a path), `{line}` and `{column}` must be numbers,
/// `{head}` must be one of the language's declared head words, `{code}` must
/// be a code-shaped token (letter-prefixed, digit-carrying), and `{message}`
/// is free text.
fn match_template(template: &str, line: &str, heads: &[String]) -> Option<Vec<Capture>> {
    let mut literals: Vec<&str> = Vec::new();
    let mut slots: Vec<String> = Vec::new();
    let mut rest = template;
    while let Some(open) = rest.find('{') {
        let (literal, after) = rest.split_at(open);
        literals.push(literal);
        let close = after.find('}')?;
        slots.push(after[1..close].to_owned());
        rest = &after[close + 1..];
    }
    literals.push(rest);
    let mut captures = Vec::new();
    let mut cursor = line;
    for (index, literal) in literals.iter().enumerate() {
        if !literal.is_empty() {
            cursor = cursor.strip_prefix(literal)?;
        }
        let Some(slot) = slots.get(index) else {
            continue;
        };
        // The slot runs to the next literal; a trailing slot runs to the end.
        let next_literal = literals.get(index + 1).copied().unwrap_or("");
        let split = if next_literal.is_empty() {
            cursor.len()
        } else {
            cursor.find(next_literal)?
        };
        let (slot_text, next) = cursor.split_at(split);
        cursor = next;
        validate_slot(slot, slot_text, heads)?;
        captures.push(Capture {
            slot: slot.clone(),
            value: slot_text.to_owned(),
        });
    }
    Some(captures)
}

/// Reject a slot value that is not what its slot names: this is what keeps a
/// generic template from reading prose as a path or a line number.
fn validate_slot(slot: &str, value: &str, heads: &[String]) -> Option<()> {
    match slot {
        "file" => value.contains('.') || value.contains('/'),
        "line" | "column" => value.parse::<u32>().is_ok(),
        "head" => heads.iter().any(|head| head == value),
        "code" => {
            value.chars().next().is_some_and(char::is_alphabetic)
                && value.chars().any(char::is_numeric)
        }
        _ => Some(()),
    }
}

fn capture<'a>(captures: &'a [Capture], slot: &str) -> Option<&'a str> {
    captures
        .iter()
        .find(|capture| capture.slot == slot)
        .map(|capture| capture.value.as_str())
        .filter(|value| !value.is_empty())
}

/// Formalize a failed step's raw output into structured diagnostics (R1).
///
/// Every line is tried against the language's location templates first (a
/// location binds to the nearest diagnostic without one — rustc states the
/// error first and the ` -->` line under it, gcc the reverse), then against
/// its diagnostic patterns. Output with no recognizable shape yields no
/// diagnostics, which the caller reads as "nothing to search" — never as a
/// fabricated one.
///
/// The line-splitting here is the same shape a generic document formalizer
/// sees in a list of prefixed entries; when the E128 (#1163) page-and-text
/// formalizer lands, the per-line loop below is the seam it replaces, and
/// the seed templates keep deciding where the code and the location sit.
pub fn formalize_diagnostic(language: &str, raw_output: &str) -> Vec<Diagnostic> {
    let shapes = shapes_for(language);
    if shapes.is_empty() {
        return Vec::new();
    }
    let mut diagnostics: Vec<Diagnostic> = Vec::new();
    let mut pending_location: Option<(String, u32)> = None;
    for line in raw_output.lines() {
        let trimmed = line.trim_start();
        if trimmed.is_empty() {
            continue;
        }
        if let Some(found) = shapes
            .locations
            .iter()
            .find_map(|template| match_template(template, trimmed, &shapes.heads))
        {
            if let (Some(file), Some(line_number)) = (
                capture(&found, "file"),
                capture(&found, "line").and_then(|value| value.parse::<u32>().ok()),
            ) {
                // Bind to the previous diagnostic first (rustc order), else
                // hold for the next one (gcc/python order).
                match diagnostics.last_mut() {
                    Some(previous) if previous.file.is_none() => {
                        previous.file = Some(file.to_owned());
                        previous.line = Some(line_number);
                    }
                    _ => pending_location = Some((file.to_owned(), line_number)),
                }
            }
            continue;
        }
        if let Some(found) = shapes
            .patterns
            .iter()
            .find_map(|template| match_template(template, trimmed, &shapes.heads))
        {
            let message = capture(&found, "message").unwrap_or_default().to_owned();
            if message.is_empty() {
                continue;
            }
            let (file, line_number) = pending_location
                .take()
                .map_or((None, None), |(file, line)| (Some(file), Some(line)));
            diagnostics.push(Diagnostic {
                file,
                line: line_number,
                code: capture(&found, "code").map(str::to_owned),
                message,
                raw: line.trim_end().to_owned(),
            });
        }
    }
    diagnostics
}

// ---------------------------------------------------------------------------
// R2 — the diagnostic becomes a search query; fetched sources are matched
// ---------------------------------------------------------------------------

/// The search query a diagnostic becomes: the language, the error code when
/// the language has one, and the head of the message, punctuation-stripped
/// and bounded so the query stays what a source can actually answer.
#[must_use]
pub fn search_query(language: &str, diagnostic: &Diagnostic) -> String {
    let mut tokens: Vec<String> = vec![shape_language(language)];
    if let Some(code) = &diagnostic.code {
        tokens.push(code.clone());
    }
    tokens.extend(
        diagnostic
            .message
            .split(|character: char| !character.is_alphanumeric())
            .filter(|token| token.chars().count() >= 2)
            .take(QUERY_MESSAGE_TOKENS)
            .map(str::to_owned),
    );
    tokens.join(" ")
}

/// The fraction of the diagnostic's message content tokens that `text`
/// carries — the same symbolic, non-neural aboutness [`super::web_research`]
/// scores sentences with, applied to a whole page.
fn message_coverage(text: &str, diagnostic: &Diagnostic) -> f32 {
    let page = text.to_lowercase();
    let tokens: Vec<String> = diagnostic
        .message
        .split(|character: char| !character.is_alphanumeric())
        .filter(|token| token.chars().count() >= 2)
        .map(|token| token.to_lowercase())
        .collect();
    if tokens.is_empty() {
        return 0.0;
    }
    let carried = tokens
        .iter()
        .filter(|token| page.contains(token.as_str()))
        .count();
    #[expect(
        clippy::cast_precision_loss,
        reason = "token counts are far below f32's exact-integer range"
    )]
    {
        carried as f32 / tokens.len() as f32
    }
}

/// Whether a fetched page addresses the diagnostic (R2's retention test,
/// R6's unresolved-need test): the page names the exact error code when the
/// language has one, or carries at least half the diagnostic's message
/// tokens.
fn page_addresses(text: &str, diagnostic: &Diagnostic) -> bool {
    if let Some(code) = &diagnostic.code
        && text.contains(code.as_str())
    {
        return true;
    }
    message_coverage(text, diagnostic) >= 0.5
}

/// The candidate-fix fragment a matched page retains: the fenced code block
/// whose surrounding window addresses the diagnostic, else the sentence with
/// the highest token overlap that still addresses it. `None` when the page
/// matches only in passing — a fragment is retained, never invented (R6).
fn fix_fragment(page: &str, diagnostic: &Diagnostic) -> Option<String> {
    let mut best: Option<(f32, String)> = None;
    let bytes = page.as_bytes();
    let mut cursor = 0usize;
    while let Some(open) = page[cursor..].find("```") {
        let open_at = cursor + open;
        let body_start = page[open_at + 3..]
            .find('\n')
            .map_or(open_at + 3, |newline| open_at + 3 + newline + 1);
        let Some(close) = page[body_start..].find("```") else {
            break;
        };
        let close_at = body_start + close;
        let window_start = page[..open_at]
            .char_indices()
            .rev()
            .take(400)
            .last()
            .map_or(0, |(index, _)| index);
        let window = &page[window_start..open_at];
        let code = diagnostic.code.as_deref().unwrap_or_default();
        let window_addresses = (!code.is_empty() && window.contains(code))
            || message_coverage(window, diagnostic) >= 0.5;
        if window_addresses {
            let score = message_coverage(window, diagnostic);
            let candidate = page[body_start..close_at].trim().to_owned();
            let better = best
                .as_ref()
                .is_none_or(|(best_score, _)| score > *best_score);
            if better && !candidate.is_empty() {
                best = Some((score, candidate));
            }
        }
        cursor = (close_at + 3).min(bytes.len());
        if cursor >= bytes.len() {
            break;
        }
    }
    if let Some((_, fragment)) = best {
        return Some(fragment);
    }
    // Pages without fences: the best single addressing sentence.
    let mut best_sentence: Option<(f32, String)> = None;
    for sentence in page.split(|character: char| matches!(character, '.' | '\n')) {
        let trimmed = sentence.trim();
        if trimmed.is_empty() || !page_addresses(trimmed, diagnostic) {
            continue;
        }
        let score = message_coverage(trimmed, diagnostic);
        if best_sentence
            .as_ref()
            .is_none_or(|(best_score, _)| score > *best_score)
        {
            best_sentence = Some((score, trimmed.to_owned()));
        }
    }
    best_sentence.map(|(_, sentence)| sentence)
}

// ---------------------------------------------------------------------------
// R3/R5 — the fix as a meta-language record; the attempts as an evidence chain
// ---------------------------------------------------------------------------

/// Escape a value for a quoted Links Notation field.
fn lino_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

/// The sidecar path the `repair_edit` record is written next to the artifact.
#[must_use]
pub fn repair_document_path(artifact_path: &str, diagnostic: &Diagnostic) -> String {
    let stem = artifact_path
        .rsplit_once('.')
        .map_or(artifact_path, |(stem, _)| stem);
    let stem = if stem.is_empty() {
        diagnostic.file.as_deref().unwrap_or("artifact")
    } else {
        stem
    };
    format!("{stem}.repair.lino")
}

/// Render the meta-language `repair_edit` record (R3): the diagnostic, the
/// matched source, and the retained fix fragment — a Links Notation document
/// the renderer (E132/#1167) lowers into the target language. It is never a
/// raw text patch pasted over the previous source.
#[must_use]
pub fn repair_edit_document(
    language: &str,
    diagnostic: &Diagnostic,
    fix: Option<&str>,
    source_url: &str,
) -> String {
    let mut out = String::from("repair_edit\n");
    let _ = writeln!(out, "  language {}", shape_language(language));
    if let Some(file) = &diagnostic.file {
        let _ = writeln!(out, "  file \"{}\"", lino_escape(file));
    }
    if let Some(line) = diagnostic.line {
        let _ = writeln!(out, "  line {line}");
    }
    if let Some(code) = &diagnostic.code {
        let _ = writeln!(out, "  error_code \"{}\"", lino_escape(code));
    }
    let _ = writeln!(out, "  message \"{}\"", lino_escape(&diagnostic.message));
    let _ = writeln!(out, "  source \"{}\"", lino_escape(source_url));
    match fix {
        Some(fix) => {
            out.push_str("  fix\n");
            let _ = writeln!(out, "    retained \"{}\"", lino_escape(fix));
        }
        None => out.push_str("  fix\n    retained none\n"),
    }
    out.push_str("  rendering meta_language\n");
    out.push_str("  applied false\n");
    format!("{}\n", out.trim_end())
}

/// One repair attempt, kept for the answer's derivation (R5): the diagnostic
/// that started it, the query that searched for it, the candidate fix a
/// fetched source supplied (when one did), and whether it was applied and
/// resolved the failure.
#[derive(Debug, Clone)]
pub struct RepairAttempt {
    pub diagnostic: Diagnostic,
    pub search_query: String,
    pub candidate_fix: Option<String>,
    pub applied: bool,
    pub resolved: bool,
}

impl RepairAttempt {
    /// The attempt as a Links Notation record, so the derivation chain is a
    /// document the `formal-ai explain` surface (#1184) can render, not a
    /// Rust struct printout.
    #[must_use]
    pub fn links_notation(&self) -> String {
        let mut out = String::from("repair_attempt\n");
        let _ = writeln!(out, "  query \"{}\"", lino_escape(&self.search_query));
        if let Some(code) = &self.diagnostic.code {
            let _ = writeln!(out, "  error_code \"{}\"", lino_escape(code));
        }
        let _ = writeln!(
            out,
            "  candidate_fix {}",
            if self.candidate_fix.is_some() {
                "true"
            } else {
                "false"
            }
        );
        let _ = writeln!(out, "  applied {}", self.applied);
        let _ = writeln!(out, "  resolved {}", self.resolved);
        format!("{}\n", out.trim_end())
    }
}

/// Reconstruct the repair-attempt chain from the transcript, so a repair
/// that took three tries is auditable rather than silently absorbed into a
/// single "succeeded" report (R5). Statelessly derivable: the executor calls
/// this when composing the derivation record, with the same failure data it
/// passed to [`plan_repair`].
#[must_use]
pub fn attempts_from(messages: &[ChatMessage], failure: &FailedStep) -> Vec<RepairAttempt> {
    let diagnostics = formalize_diagnostic(&failure.language, &failure.reported);
    let Some(primary) = diagnostics.first() else {
        return Vec::new();
    };
    let progress = Progress::scan(messages);
    let applied = progress
        .fetched_pages
        .iter()
        .any(|(url, text)| page_addresses(text, primary))
        && progress.successful_write_for(&repair_document_path(&failure.artifact_path, primary));
    let resolved = failure
        .failed_command
        .as_deref()
        .is_some_and(|command| progress.successful_run_count_for(command) > 0);
    let candidate = progress
        .fetched_pages
        .iter()
        .find(|(_, text)| page_addresses(text, primary))
        .and_then(|(_, text)| fix_fragment(text, primary));
    vec![RepairAttempt {
        diagnostic: primary.clone(),
        search_query: search_query(&failure.language, primary),
        candidate_fix: candidate,
        applied,
        resolved,
    }]
}

/// The evidence document for the whole chain (R5): one Links Notation
/// record per attempt, headed with the count.
#[must_use]
pub fn evidence_document(attempts: &[RepairAttempt]) -> String {
    let mut out = String::from("repair_attempts\n");
    let _ = writeln!(out, "  attempt_count {}", attempts.len());
    for attempt in attempts {
        for line in attempt.links_notation().lines() {
            out.push_str("  ");
            out.push_str(line);
            out.push('\n');
        }
    }
    format!("{}\n", out.trim_end())
}

// ---------------------------------------------------------------------------
// R4/R6 — bounded retries, honest stops
// ---------------------------------------------------------------------------

/// Why [`plan_repair`] declined to plan the next rung. Each stop is reported,
/// never silently absorbed: `Exhausted` and `NoMatch` are the two the
/// executor names in its answer through the seed templates below.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepairStop {
    /// The raw output carried no diagnostic the shape table recognizes, so
    /// there is nothing to search — today's failure report stands.
    NoDiagnostic,
    /// The client advertises none of the tools a rung needs.
    NoTools,
    /// The ladder is spent (R4): reaching the top rung is reported as such.
    Exhausted,
    /// Every fetched source was read and none addresses the diagnostic
    /// (R6): an unresolved need, never a fabricated fix.
    NoMatch,
}

/// The next repair step, with the reason a step was declined.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RepairOutcome {
    /// Search the diagnostic's query (R2, rung opening).
    Search(AgenticPlan),
    /// Read the next source the search returned (R2).
    Fetch(AgenticPlan),
    /// Write the `repair_edit` record for a matched fix (R3).
    RecordFix(AgenticPlan),
    /// Re-run the failed command (R4's retry).
    Retry(AgenticPlan),
    /// Declined, with the honest stop reason.
    Stop(RepairStop),
}

impl RepairOutcome {
    /// The plan, when there is one.
    #[must_use]
    pub fn plan(self) -> Option<AgenticPlan> {
        match self {
            Self::Search(plan) | Self::Fetch(plan) | Self::RecordFix(plan) | Self::Retry(plan) => {
                Some(plan)
            }
            Self::Stop(_) => None,
        }
    }
}

/// The failed step as plain data, mirroring `command_reroute::StepFailure`
/// plus what the repair loop needs that the report does not: the language
/// the artifact was emitted in, the command that failed, and the artifact's
/// path. Built by the executor at the failure branch.
#[derive(Debug, Clone)]
pub struct FailedStep {
    pub language: String,
    pub reported: String,
    pub exit_code: Option<i32>,
    pub failed_command: Option<String>,
    pub artifact_path: String,
}

impl FailedStep {
    #[must_use]
    pub fn new(language: &str, reported: &str) -> Self {
        Self {
            language: language.to_owned(),
            reported: reported.to_owned(),
            exit_code: None,
            failed_command: None,
            artifact_path: String::new(),
        }
    }

    #[must_use]
    pub const fn with_exit_code(mut self, exit_code: Option<i32>) -> Self {
        self.exit_code = exit_code;
        self
    }

    #[must_use]
    pub fn with_failed_command(mut self, command: Option<String>) -> Self {
        self.failed_command = command;
        self
    }

    #[must_use]
    pub fn with_artifact_path(mut self, path: &str) -> Self {
        self.artifact_path = path.to_owned();
        self
    }
}

/// Rank a search result's URLs: deduped, capped, first read next — the same
/// shape [`super::web_research`] ranks with, kept local because its copy is
/// private to that module.
fn source_urls(text: &str) -> Vec<String> {
    let mut urls: Vec<String> = text
        .split_whitespace()
        .filter(|token| token.starts_with("http://") || token.starts_with("https://"))
        .map(|token| {
            token
                .trim_end_matches(['.', ',', ';', ')', ']', '"', '\''])
                .to_owned()
        })
        .collect();
    let mut seen = BTreeSet::new();
    urls.retain(|url| seen.insert(url.clone()));
    urls.truncate(MAX_REPAIR_SOURCES);
    urls
}

/// Plan the next rung of the repair loop (R2-R4).
///
/// One rung reads: search the diagnostic's query → fetch the sources the
/// search returned → write the `repair_edit` record for the first source
/// that addresses the diagnostic → re-run the failed command. The caller
/// owns the rung counter; a retry that fails again comes back here one rung
/// higher, and a rung past [`MAX_REPAIR_RUNGS`] stops honestly (R4). A
/// diagnostic no fetched source addresses stops as an unresolved need (R6).
pub fn repair_step(
    messages: &[ChatMessage],
    tool_names: &[&str],
    failure: &FailedStep,
    ladder_rung: u8,
    max_rungs: u8,
) -> RepairOutcome {
    if ladder_rung >= max_rungs {
        return RepairOutcome::Stop(RepairStop::Exhausted);
    }
    let diagnostics = formalize_diagnostic(&failure.language, &failure.reported);
    let Some(primary) = diagnostics.first() else {
        return RepairOutcome::Stop(RepairStop::NoDiagnostic);
    };
    let progress = Progress::scan(messages);
    if progress.search_output.is_none() {
        let Some(tool) = tool_for(tool_names, Capability::Search) else {
            return RepairOutcome::Stop(RepairStop::NoTools);
        };
        let query = search_query(&failure.language, primary);
        return RepairOutcome::Search(plan_one(tool, json!({ "query": query }).to_string()));
    }
    if let Some(tool) = tool_for(tool_names, Capability::Fetch) {
        let already: BTreeSet<&str> = progress
            .attempted_fetches
            .iter()
            .map(String::as_str)
            .collect();
        let output = progress.search_output.as_deref().unwrap_or_default();
        if let Some(url) = source_urls(output)
            .into_iter()
            .find(|url| !already.contains(url.as_str()))
        {
            return RepairOutcome::Fetch(plan_one(tool, fetch_arguments(&url)));
        }
    }
    let Some((url, page)) = progress
        .fetched_pages
        .iter()
        .find(|(_, text)| page_addresses(text, primary))
    else {
        return RepairOutcome::Stop(RepairStop::NoMatch);
    };
    let document_path = repair_document_path(&failure.artifact_path, primary);
    if !progress.successful_write_for(&document_path) {
        let Some(tool) = tool_for(tool_names, Capability::Write) else {
            return RepairOutcome::Stop(RepairStop::NoTools);
        };
        let document = repair_edit_document(
            &failure.language,
            primary,
            fix_fragment(page, primary).as_deref(),
            url,
        );
        return RepairOutcome::RecordFix(plan_one(
            tool,
            write_arguments(&document_path, &document),
        ));
    }
    let Some(command) = failure.failed_command.clone() else {
        // The step that failed was not a command (a file write, say), so
        // there is nothing to re-run: the recorded fix is the rung's result.
        return RepairOutcome::Stop(RepairStop::NoMatch);
    };
    let Some(tool) = tool_for(tool_names, Capability::Run) else {
        return RepairOutcome::Stop(RepairStop::NoTools);
    };
    RepairOutcome::Retry(plan_one(tool, json!({ "command": command }).to_string()))
}

/// [`repair_step`] as the issue's signature: the plan when a rung has one,
/// `None` for the caller's honest fallback to today's failure report.
#[must_use]
pub fn plan_repair(
    messages: &[ChatMessage],
    tool_names: &[&str],
    failure: &FailedStep,
    ladder_rung: u8,
    max_rungs: u8,
) -> Option<AgenticPlan> {
    repair_step(messages, tool_names, failure, ladder_rung, max_rungs).plan()
}

/// Fill a localized repair template's `{placeholder}` slots.
fn template(intent: &str, language: &str, values: &[(&str, &str)]) -> String {
    let mut out = crate::seed::localized_response(intent, language).unwrap_or_default();
    for (key, value) in values {
        out = out.replace(&format!("{{{key}}}"), value);
    }
    out
}

/// The note appended to the failure report when the ladder is spent (R4):
/// the rung reached is named, so the bound is reported honestly rather than
/// silently truncating. Empty when the response seed is not registered, in
/// which case today's report already stands on its own.
#[must_use]
pub fn ladder_note(language: &str, rung: u8, max_rungs: u8) -> String {
    template(
        "repair_ladder_exhausted",
        language,
        &[
            ("rung", &rung.to_string()),
            ("max_rungs", &max_rungs.to_string()),
        ],
    )
}

/// The note appended when no fetched source addresses the diagnostic (R6):
/// an unresolved need in the seed's own words, never a fabricated fix.
#[must_use]
pub fn unresolved_note(language: &str, query: &str) -> String {
    template("repair_unresolved_need", language, &[("query", query)])
}
