//! Compiler for arbitrary, freely-phrased natural-language procedures (issue #674).
//!
//! [`crate::skill_compiler`] compiles two *shapes*: quoted trigger/response prose and
//! the labeled `Skill`/`Step`/`Expected test` form. A user who simply states a
//! procedure — *"when I paste a link, fetch its title, translate it to Russian, save
//! both, and reply with the translation"* — matches neither, so before this module
//! such a prompt fell through to formalization with nothing compiled.
//!
//! This compiler decomposes the sentence into ordered clauses and maps each clause
//! onto the step vocabulary. Three properties are deliberate:
//!
//! * **The vocabulary is data.** Every step kind is a meaning in
//!   `data/seed/meanings-skill-procedure.lino` carrying
//!   [`seed::ROLE_SKILL_PROCEDURE_STEP_VERB`]; the meaning's *slug* is the canonical
//!   step kind emitted here and dispatched on by a [`ProcedureHost`]. Reviewed
//!   multilingual aliases can join the durable capability ledger without a parser
//!   branch; a genuinely new operation still needs explicit host semantics.
//! * **Nothing is silently dropped.** A clause that matches no step verb aborts the
//!   whole compilation with [`ProcedureCompileError::UncompilableStep`], which names
//!   the gap. Partial programs are never produced.
//! * **Identity is language-independent.** Step kinds and arguments are recorded as
//!   meaning slugs, so the same procedure stated in English, Russian, Hindi, or
//!   Chinese lowers to a byte-identical canonical program and therefore to identical
//!   content-addressed ids and links. Only [`CompiledProcedure::source_description`]
//!   and the per-step source spans remember the surface wording — which is what makes
//!   *"why did you do that?"* able to quote the sentence a step came from.

use std::error::Error;
use std::fmt;
use std::fmt::Write as _;
use std::sync::OnceLock;

use crate::engine::{stable_id, KNOWLEDGE_SCHEMA_VERSION};
use crate::intent_formalization::{
    formalize_intent, ordered_requirement_spans, OrderedRequirementSpan,
};
use crate::language::detect as detect_language;
use crate::link_store::{DoubletLink, LinkRecord};
use crate::links_format::push_lino_node;
use crate::seed::{
    self, Lexicon, Meaning, ROLE_SKILL_PROCEDURE_CLAUSE_SEPARATOR,
    ROLE_SKILL_PROCEDURE_STEP_OBJECT, ROLE_SKILL_PROCEDURE_STEP_VERB,
    ROLE_SKILL_PROCEDURE_TRIGGER_LEAD, ROLE_TRANSLATION_LANGUAGE,
};

#[path = "../../src/skill_procedure/artifact.rs"]
mod artifact;
#[path = "../../src/skill_procedure/learning.rs"]
mod learning;
pub use artifact::extract_compiled_procedure_artifact;
use learning::default_capability_ledger;
pub use learning::{
    ApprovedProcedureLesson, ProcedureCapabilityLedger, ProcedureCapabilityLesson,
    ProcedureLearnedSurface, ProcedureLearningApproval, ProcedureLearningError,
    ProcedureLearningGate, ProcedureLearningProposal,
};

/// A procedure needs at least this many recognised steps before the compiler claims
/// the prompt at all.
///
/// The journey this serves (USER-JOURNEYS F2) is *multi-step* procedure statement. A
/// single imperative clause after a "when I …" lead is ordinary conversation and must
/// stay with the regular solver pipeline, so one recognised step is not a program.
const MINIMUM_STEPS: usize = 2;

const PROCEDURE_MEANINGS_LINO: &str = include_str!("../../data/seed/meanings-skill-procedure.lino");

/// One ordered, source-grounded subrequirement produced by the shared
/// intent-formalization decomposer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcedureRequirement {
    pub id: String,
    pub index: usize,
    pub source_text: String,
    pub source_span: (usize, usize),
}

/// One compiled step of a procedure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcedureStep {
    /// Stable content-addressed id of this step within its package.
    pub id: String,
    /// 1-based position in the program.
    pub index: usize,
    /// The formalized subrequirement this executable leaf realizes.
    pub requirement_id: String,
    /// Canonical step kind — the slug of the step-verb meaning that matched.
    pub kind: String,
    /// Canonical arguments — slugs of the object meanings the clause mentions.
    pub objects: Vec<String>,
    /// Target-language meaning slug (`language_russian`, …) when the clause names one.
    pub target_language: Option<String>,
    /// The clause exactly as the user wrote it.
    pub source_text: String,
    /// Byte range of `source_text` inside the original description.
    pub source_span: (usize, usize),
}

impl ProcedureStep {
    /// The canonical arguments of this step, in canonical order.
    #[must_use]
    pub fn arguments(&self) -> Vec<String> {
        let mut arguments = self.objects.clone();
        if let Some(language) = &self.target_language {
            arguments.push(language.clone());
        }
        arguments
    }
}

/// The situation that starts a compiled procedure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcedureTrigger {
    /// The formalized subrequirement this trigger realizes.
    pub requirement_id: String,
    /// Canonical arguments — slugs of the object meanings the trigger clause mentions.
    pub objects: Vec<String>,
    /// The trigger clause exactly as the user wrote it.
    pub source_text: String,
    /// Byte range of `source_text` inside the original description.
    pub source_span: (usize, usize),
}

/// A reviewable program compiled from one freely-phrased procedure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledProcedure {
    /// Stable package id, derived from [`CompiledProcedure::canonical_program`] alone.
    pub id: String,
    /// The sentence the user actually wrote.
    pub source_description: String,
    /// The same stable impulse identity used by the universal solver.
    pub impulse_id: String,
    /// Trigger and steps as ordered, source-grounded subrequirements.
    pub requirements: Vec<ProcedureRequirement>,
    /// The situation the procedure reacts to.
    pub trigger: ProcedureTrigger,
    /// The ordered steps, all of which compiled.
    pub steps: Vec<ProcedureStep>,
    /// Language-independent program text the ids are computed from.
    pub canonical_program: String,
}

/// Why a prose procedure did not compile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcedureCompileError {
    /// The prompt is not a multi-step procedure statement; other handlers may claim it.
    NotAProcedure,
    /// One clause named an operation with no entry in the step vocabulary.
    ///
    /// The whole compilation fails: a procedure missing a step is not the procedure
    /// the user asked for, so no partial program is produced.
    UncompilableStep {
        /// The clause, as written, that could not be compiled.
        step: String,
        /// Byte range of `step` inside the original description.
        span: (usize, usize),
        /// Honest, quotable gap name.
        gap: String,
    },
}

impl fmt::Display for ProcedureCompileError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotAProcedure => formatter.write_str("prompt is not a multi-step procedure"),
            Self::UncompilableStep { gap, .. } => formatter.write_str(gap),
        }
    }
}

impl Error for ProcedureCompileError {}

/// Name the missing capability in seeded wording (R379).
///
/// The gap name is part of the compiler's identity — it travels in
/// [`ProcedureCompileError::UncompilableStep`] and in the `skill_gap` event — so it is
/// looked up in English (`data/seed/multilingual-responses-procedure.lino`, intent
/// `skill_gap_name`) regardless of the language the procedure was stated in; the reply
/// the user reads is localized separately in `solver_handlers::procedure_rules`.
#[allow(clippy::literal_string_with_formatting_args)]
fn gap_name(step: &str) -> String {
    seed::response_for("skill_gap_name", "en")
        .unwrap_or_default()
        .replace("{step}", step)
}

/// Compile a freely-phrased procedure into an executable program.
///
/// # Errors
///
/// Returns [`ProcedureCompileError::NotAProcedure`] when the prompt carries no
/// procedure trigger lead or fewer than two recognised steps, and
/// [`ProcedureCompileError::UncompilableStep`] when a clause names an operation the
/// step vocabulary does not cover.
pub fn compile_procedure(description: &str) -> Result<CompiledProcedure, ProcedureCompileError> {
    compile_procedure_with_ledger(description, default_capability_ledger())
}

/// Compile with an additional, durable set of human-approved vocabulary
/// lessons. Seed vocabulary and learned vocabulary enter the same classifier;
/// no operation-specific Rust match arm is generated.
pub fn compile_procedure_with_ledger(
    description: &str,
    ledger: &ProcedureCapabilityLedger,
) -> Result<CompiledProcedure, ProcedureCompileError> {
    let separator_surfaces = procedure_role_surfaces(ROLE_SKILL_PROCEDURE_CLAUSE_SEPARATOR);
    let separator_refs = separator_surfaces
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let source_requirements = ordered_requirement_spans(description, &separator_refs);
    let trigger_position = source_requirements
        .iter()
        .position(|requirement| {
            first_procedure_match(
                &requirement.source_text.to_lowercase(),
                ROLE_SKILL_PROCEDURE_TRIGGER_LEAD,
            )
            .is_some()
        })
        .ok_or(ProcedureCompileError::NotAProcedure)?;

    let step_requirements = &source_requirements[trigger_position + 1..];
    if step_requirements.len() < MINIMUM_STEPS {
        return Err(ProcedureCompileError::NotAProcedure);
    }

    // Classify every clause first. A gap is only reported once the prompt has proven
    // itself a procedure; otherwise any unrecognised sentence starting with "when I"
    // would be reported as a missing capability.
    let classified: Vec<Option<Found>> = step_requirements
        .iter()
        .map(|requirement| first_step_match(&requirement.source_text.to_lowercase(), ledger))
        .collect();
    if classified.iter().filter(|found| found.is_some()).count() < MINIMUM_STEPS {
        return Err(ProcedureCompileError::NotAProcedure);
    }

    for (index, found) in classified.iter().enumerate() {
        if found.is_none() {
            let requirement = &step_requirements[index];
            let span = requirement.source_span;
            let step = requirement.source_text.clone();
            return Err(ProcedureCompileError::UncompilableStep {
                gap: gap_name(&step),
                step,
                span,
            });
        }
    }

    // Most agent turns are not procedures. Defer the comparatively expensive
    // intent graph until the cheap shape/vocabulary checks prove that this route
    // owns the prompt, so unrelated tool routes do not pay compilation cost.
    let formalization = formalize_intent(description, detect_language(description).slug(), None);
    let requirements = source_requirements
        .iter()
        .enumerate()
        .map(|(index, requirement)| ProcedureRequirement {
            id: requirement_id(&formalization.impulse_id, index + 1, requirement),
            index: index + 1,
            source_text: requirement.source_text.clone(),
            source_span: requirement.source_span,
        })
        .collect::<Vec<_>>();
    let trigger_source = &source_requirements[trigger_position];
    let trigger_requirement = &requirements[trigger_position];
    let trigger_lower = trigger_source.source_text.to_lowercase();
    let trigger = ProcedureTrigger {
        requirement_id: trigger_requirement.id.clone(),
        objects: objects_in(&trigger_lower, None),
        source_text: trigger_source.source_text.clone(),
        source_span: trigger_source.source_span,
    };

    let mut steps = Vec::with_capacity(step_requirements.len());
    for (index, (found, requirement)) in classified.into_iter().zip(step_requirements).enumerate() {
        let verb = found.expect("every clause classified above");
        let clause = requirement.source_text.to_lowercase();
        let requirement_record = &requirements[trigger_position + index + 1];
        steps.push(ProcedureStep {
            id: String::new(),
            index: index + 1,
            requirement_id: requirement_record.id.clone(),
            objects: objects_in(&clause, Some((verb.start, verb.end))),
            target_language: first_match(&clause, ROLE_TRANSLATION_LANGUAGE)
                .map(|found| found.slug),
            kind: verb.slug,
            source_text: requirement.source_text.clone(),
            source_span: requirement.source_span,
        });
    }

    let canonical_program = canonical_program(&trigger, &steps);
    let id = stable_id("compiled_procedure", &canonical_program);
    for step in &mut steps {
        step.id = stable_id(
            "compiled_procedure_step",
            &format!(
                "{id}:{}:{}:{}",
                step.index,
                step.kind,
                step.arguments().join("+")
            ),
        );
    }

    Ok(CompiledProcedure {
        id,
        source_description: description.to_owned(),
        impulse_id: formalization.impulse_id,
        requirements,
        trigger,
        steps,
        canonical_program,
    })
}

/// What one compiled step produced when executed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepOutcome {
    /// Id of the step that ran.
    pub step_id: String,
    /// Canonical kind of the step that ran.
    pub kind: String,
    /// Value the host produced.
    pub output: String,
}

/// The result of running a compiled procedure once.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcedureRun {
    /// Id of the package that ran.
    pub package_id: String,
    /// One outcome per step, in program order.
    pub outcomes: Vec<StepOutcome>,
}

impl ProcedureRun {
    /// Value produced by the final step — what the user sees.
    #[must_use]
    pub fn answer(&self) -> &str {
        self.outcomes.last().map_or("", |last| last.output.as_str())
    }
}

/// A step failed while executing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcedureRunError {
    /// Id of the step that failed.
    pub step_id: String,
    /// Canonical kind of the step that failed.
    pub kind: String,
    /// Host-supplied reason.
    pub reason: String,
}

impl fmt::Display for ProcedureRunError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "step {} ({}): {}",
            self.step_id, self.kind, self.reason
        )
    }
}

impl Error for ProcedureRunError {}

/// A persisted procedure artifact was missing data or failed integrity checks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcedureArtifactError {
    pub reason: String,
}

impl ProcedureArtifactError {
    fn new(reason: impl Into<String>) -> Self {
        Self {
            reason: reason.into(),
        }
    }
}

impl fmt::Display for ProcedureArtifactError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.reason)
    }
}

impl Error for ProcedureArtifactError {}

/// The environment a compiled procedure runs in.
///
/// The compiler stays free of capability knowledge: it only records *which* step kind
/// each clause names. A host decides what `skill_procedure_fetch` actually does, which
/// is what lets tests execute a procedure deterministically while a production wiring
/// gates the same step behind a network permission.
pub trait ProcedureHost {
    /// Run `step` on `input` (the previous step's output, or the trigger value for the
    /// first step) and return its output, or an honest failure reason.
    ///
    /// # Errors
    ///
    /// Returns the host's reason string when the step cannot be performed.
    fn perform(&mut self, step: &ProcedureStep, input: &str) -> Result<String, String>;
}

impl CompiledProcedure {
    /// Execute every step in order, threading each output into the next step.
    ///
    /// # Errors
    ///
    /// Returns [`ProcedureRunError`] naming the first step the host refused.
    pub fn execute(
        &self,
        trigger_value: &str,
        host: &mut dyn ProcedureHost,
    ) -> Result<ProcedureRun, ProcedureRunError> {
        let mut input = trigger_value.to_owned();
        let mut outcomes = Vec::with_capacity(self.steps.len());
        for step in &self.steps {
            let output = host
                .perform(step, &input)
                .map_err(|reason| ProcedureRunError {
                    step_id: step.id.clone(),
                    kind: step.kind.clone(),
                    reason,
                })?;
            outcomes.push(StepOutcome {
                step_id: step.id.clone(),
                kind: step.kind.clone(),
                output: output.clone(),
            });
            input = output;
        }
        Ok(ProcedureRun {
            package_id: self.id.clone(),
            outcomes,
        })
    }

    /// Re-state the compiled steps, each quoting the sentence span it came from.
    ///
    /// This is what *"why did you do that?"* answers with: every line names the
    /// canonical step kind, its canonical arguments, the exact words that produced it,
    /// and the byte range those words occupy in the original request.
    #[must_use]
    pub fn restate_steps(&self) -> String {
        let mut out = String::new();
        for step in &self.steps {
            let arguments = step.arguments();
            let _ = write!(out, "{}. {}", step.index, step.kind);
            if !arguments.is_empty() {
                let _ = write!(out, "({})", arguments.join(", "));
            }
            let _ = writeln!(
                out,
                " — \"{}\" [{}..{}]",
                step.source_text, step.source_span.0, step.source_span.1
            );
        }
        out
    }

    /// Export the compiled program as reviewable Links Notation.
    ///
    /// Only canonical, language-independent facts are projected, so two phrasings of
    /// the same procedure export byte-identical notation.
    #[must_use]
    pub fn links_notation(&self) -> String {
        let mut out = String::new();
        push_lino_node(&mut out, 0, &self.id, None);
        push_lino_node(&mut out, 2, "type", Some("compiled_procedure"));
        push_lino_node(
            &mut out,
            2,
            "schema_version",
            Some(KNOWLEDGE_SCHEMA_VERSION),
        );
        push_lino_node(&mut out, 2, "package_kind", Some("associative_package"));
        push_lino_node(&mut out, 2, "source", Some("natural_language_procedure"));
        for object in &self.trigger.objects {
            push_lino_node(&mut out, 2, "trigger_object", Some(object));
        }
        for step in &self.steps {
            push_lino_node(&mut out, 2, "step", Some(&step.id));
            push_lino_node(&mut out, 4, "index", Some(&step.index.to_string()));
            push_lino_node(&mut out, 4, "kind", Some(&step.kind));
            for argument in step.arguments() {
                push_lino_node(&mut out, 4, "argument", Some(&argument));
            }
        }
        out
    }

    /// Project the compiled program as link records for the associative store.
    ///
    /// Like [`CompiledProcedure::links_notation`] this carries canonical slugs only.
    #[must_use]
    pub fn link_records(&self) -> Vec<LinkRecord> {
        let mut records = vec![link_record(
            &self.id,
            "CompiledProcedure",
            "associative_package",
            &stable_id("natural_language_procedure", &self.canonical_program),
            &[
                ("step_count", self.steps.len().to_string().as_str()),
                ("trigger_objects", self.trigger.objects.join("+").as_str()),
            ],
        )];
        for step in &self.steps {
            records.push(link_record(
                &step.id,
                "CompiledProcedureStep",
                "procedure_step",
                &self.id,
                &[
                    ("index", step.index.to_string().as_str()),
                    ("kind", step.kind.as_str()),
                    ("arguments", step.arguments().join("+").as_str()),
                ],
            ));
        }
        records
    }
}

/// The language-independent program text every id is derived from.
fn canonical_program(trigger: &ProcedureTrigger, steps: &[ProcedureStep]) -> String {
    let mut out = String::from("procedure\n");
    out.push_str("  trigger\n");
    for object in &trigger.objects {
        let _ = writeln!(out, "    object {object}");
    }
    for step in steps {
        out.push_str("  step\n");
        let _ = writeln!(out, "    index {}", step.index);
        let _ = writeln!(out, "    kind {}", step.kind);
        for argument in step.arguments() {
            let _ = writeln!(out, "    argument {argument}");
        }
    }
    out
}

/// A seed surface located inside a haystack, tagged with the meaning it belongs to.
#[derive(Debug, Clone)]
struct Found {
    slug: String,
    start: usize,
    end: usize,
}

fn requirement_id(impulse_id: &str, index: usize, requirement: &OrderedRequirementSpan) -> String {
    stable_id(
        "procedure_requirement",
        &format!(
            "{impulse_id}:{index}:{}..{}:{}",
            requirement.source_span.0, requirement.source_span.1, requirement.source_text
        ),
    )
}

fn procedure_lexicon() -> &'static Lexicon {
    static CACHE: OnceLock<Lexicon> = OnceLock::new();
    CACHE.get_or_init(|| seed::parse_lexicon_text(PROCEDURE_MEANINGS_LINO))
}

fn procedure_role_surfaces(role: &str) -> Vec<String> {
    procedure_lexicon()
        .meanings_with_role(role)
        .flat_map(surfaces)
        .map(str::to_owned)
        .collect()
}

fn meaning_has_role(slug: &str, role: &str) -> bool {
    seed::lexicon()
        .meanings_with_role(role)
        .any(|meaning| meaning.slug == slug)
}

fn first_step_match(hay: &str, ledger: &ProcedureCapabilityLedger) -> Option<Found> {
    let mut best = first_procedure_match(hay, ROLE_SKILL_PROCEDURE_STEP_VERB);
    for entry in &ledger.lessons {
        for surface in &entry.lesson.surfaces {
            for (start, _) in hay.match_indices(&surface.text) {
                let end = start + surface.text.len();
                if !is_standalone(hay, start, end) {
                    continue;
                }
                let candidate = Found {
                    slug: entry.lesson.canonical_kind.clone(),
                    start,
                    end,
                };
                let better = best.as_ref().is_none_or(|current| {
                    candidate.start < current.start
                        || (candidate.start == current.start
                            && candidate.end - candidate.start > current.end - current.start)
                });
                if better {
                    best = Some(candidate);
                }
            }
        }
    }
    best
}

/// The meaning of `role` whose surface appears earliest in `hay`.
///
/// Earliest wins because clauses are imperative — the verb leads — so a later mention
/// of another vocabulary word ("reply with the translation") cannot outrank it. Ties
/// on position are broken by the longer surface, which prefers the more specific
/// reading.
fn first_match(hay: &str, role: &str) -> Option<Found> {
    first_match_in(seed::lexicon(), hay, role)
}

fn first_procedure_match(hay: &str, role: &str) -> Option<Found> {
    first_match_in(procedure_lexicon(), hay, role)
}

fn first_match_in(lexicon: &Lexicon, hay: &str, role: &str) -> Option<Found> {
    let mut best: Option<Found> = None;
    for meaning in lexicon.meanings_with_role(role) {
        for word in surfaces(meaning) {
            for (start, _) in hay.match_indices(word) {
                let end = start + word.len();
                let candidate = Found {
                    slug: meaning.slug.clone(),
                    start,
                    end,
                };
                let better = match &best {
                    None => true,
                    Some(current) => {
                        candidate.start < current.start
                            || (candidate.start == current.start
                                && candidate.end - candidate.start > current.end - current.start)
                    }
                };
                if better {
                    best = Some(candidate);
                }
            }
        }
    }
    best
}

/// Every object meaning mentioned in `clause`, in mention order, without repeats.
///
/// Occurrences overlapping `skip` (the span already consumed by the step verb) are
/// ignored so a verb surface cannot be re-read as its own object.
fn objects_in(clause: &str, skip: Option<(usize, usize)>) -> Vec<String> {
    let mut hits: Vec<(usize, String)> = Vec::new();
    for meaning in procedure_lexicon().meanings_with_role(ROLE_SKILL_PROCEDURE_STEP_OBJECT) {
        let mut earliest: Option<usize> = None;
        for word in surfaces(meaning) {
            for (start, _) in clause.match_indices(word) {
                let end = start + word.len();
                if let Some((skip_start, skip_end)) = skip {
                    if start < skip_end && skip_start < end {
                        continue;
                    }
                }
                earliest = Some(earliest.map_or(start, |current: usize| current.min(start)));
            }
        }
        if let Some(start) = earliest {
            hits.push((start, meaning.slug.clone()));
        }
    }
    hits.sort_by_key(|left| left.0);
    hits.into_iter().map(|(_, slug)| slug).collect()
}

/// Every non-empty surface text of `meaning`, slot markers and all.
fn surfaces(meaning: &Meaning) -> impl Iterator<Item = &str> {
    meaning
        .lexemes
        .iter()
        .flat_map(|lexeme| lexeme.words.iter())
        .map(|word| word.text.as_str())
        .filter(|text| !text.is_empty())
}

/// Is a learned alias at `lower[start..end]` a free-standing word rather than part
/// of a longer one?
///
/// Ideographic connectives ("然后") are written without spaces, so for them the
/// neighbour test is skipped: in a script with no word separators, adjacency to
/// another ideograph is not evidence of being embedded in a larger word. Seed
/// lexemes do not use this check because some reviewed entries are deliberate
/// inflection stems (for example Russian `перевед`).
fn is_standalone(lower: &str, start: usize, end: usize) -> bool {
    let matched = &lower[start..end];
    if matched.chars().any(is_ideographic) {
        return true;
    }
    let first = matched.chars().next();
    let last = matched.chars().next_back();
    let before = lower[..start].chars().next_back();
    let after = lower[end..].chars().next();
    (!first.is_some_and(char::is_alphanumeric) || !before.is_some_and(char::is_alphanumeric))
        && (!last.is_some_and(char::is_alphanumeric) || !after.is_some_and(char::is_alphanumeric))
}

/// Does `character` belong to a script written without word separators?
const fn is_ideographic(character: char) -> bool {
    matches!(character, '\u{3400}'..='\u{9fff}' | '\u{f900}'..='\u{faff}')
}

fn link_record(
    record_id: &str,
    record_type: &str,
    subtype: &str,
    source_id: &str,
    fields: &[(&str, &str)],
) -> LinkRecord {
    let mut links = Vec::new();
    push_doublet(&mut links, record_id, "Type");
    push_doublet(&mut links, "Type", record_type);
    push_doublet(&mut links, record_type, "SubType");
    push_doublet(&mut links, "SubType", subtype);
    push_doublet(&mut links, subtype, "Value");
    push_doublet(&mut links, record_id, source_id);
    push_field(
        &mut links,
        record_id,
        "schema_version",
        KNOWLEDGE_SCHEMA_VERSION,
    );
    for (key, value) in fields {
        push_field(&mut links, record_id, key, value);
    }
    LinkRecord {
        stable_id: record_id.to_owned(),
        schema_version: String::from(KNOWLEDGE_SCHEMA_VERSION),
        record_type: record_type.to_owned(),
        source_id: source_id.to_owned(),
        links,
    }
}

fn push_field(links: &mut Vec<DoubletLink>, record_id: &str, key: &str, value: &str) {
    if value.is_empty() {
        return;
    }
    let field = format!("field:{key}");
    let field_value = format!("value:{value}");
    push_doublet(links, record_id, &field);
    push_doublet(links, &field, &field_value);
}

fn push_doublet(links: &mut Vec<DoubletLink>, from: &str, to: &str) {
    links.push(DoubletLink {
        index: stable_id("doublet", &format!("{from}->{to}")),
        from: from.to_owned(),
        to: to.to_owned(),
    });
}
