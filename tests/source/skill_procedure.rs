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
//!   step kind emitted here and dispatched on by a [`ProcedureHost`]. Teaching the
//!   compiler a new step is a seed edit plus a host capability, never a new match arm.
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

use crate::engine::{stable_id, KNOWLEDGE_SCHEMA_VERSION};
use crate::link_store::{DoubletLink, LinkRecord};
use crate::links_format::push_lino_node;
use crate::seed::{
    self, Meaning, ROLE_SKILL_PROCEDURE_CLAUSE_SEPARATOR, ROLE_SKILL_PROCEDURE_STEP_OBJECT,
    ROLE_SKILL_PROCEDURE_STEP_VERB, ROLE_SKILL_PROCEDURE_TRIGGER_LEAD, ROLE_TRANSLATION_LANGUAGE,
};

/// A procedure needs at least this many recognised steps before the compiler claims
/// the prompt at all.
///
/// The journey this serves (USER-JOURNEYS F2) is *multi-step* procedure statement. A
/// single imperative clause after a "when I …" lead is ordinary conversation and must
/// stay with the regular solver pipeline, so one recognised step is not a program.
const MINIMUM_STEPS: usize = 2;

/// One compiled step of a procedure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcedureStep {
    /// Stable content-addressed id of this step within its package.
    pub id: String,
    /// 1-based position in the program.
    pub index: usize,
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

/// Compile a freely-phrased procedure into an executable program.
///
/// # Errors
///
/// Returns [`ProcedureCompileError::NotAProcedure`] when the prompt carries no
/// procedure trigger lead or fewer than two recognised steps, and
/// [`ProcedureCompileError::UncompilableStep`] when a clause names an operation the
/// step vocabulary does not cover.
pub fn compile_procedure(description: &str) -> Result<CompiledProcedure, ProcedureCompileError> {
    let (lower, offsets) = lower_with_offsets(description);
    let lead = first_match(&lower, ROLE_SKILL_PROCEDURE_TRIGGER_LEAD)
        .ok_or(ProcedureCompileError::NotAProcedure)?;

    let clauses = split_clauses(&lower);
    let trigger_position = clauses
        .iter()
        .position(|(start, end)| lead.start >= *start && lead.start < *end)
        .ok_or(ProcedureCompileError::NotAProcedure)?;

    let step_clauses = &clauses[trigger_position + 1..];
    if step_clauses.len() < MINIMUM_STEPS {
        return Err(ProcedureCompileError::NotAProcedure);
    }

    // Classify every clause first. A gap is only reported once the prompt has proven
    // itself a procedure; otherwise any unrecognised sentence starting with "when I"
    // would be reported as a missing capability.
    let classified: Vec<Option<Found>> = step_clauses
        .iter()
        .map(|(start, end)| first_match(&lower[*start..*end], ROLE_SKILL_PROCEDURE_STEP_VERB))
        .collect();
    if classified.iter().filter(|found| found.is_some()).count() < MINIMUM_STEPS {
        return Err(ProcedureCompileError::NotAProcedure);
    }

    for (index, found) in classified.iter().enumerate() {
        if found.is_none() {
            let (start, end) = step_clauses[index];
            let span = (offsets[start], offsets[end]);
            let step = description[span.0..span.1].to_owned();
            return Err(ProcedureCompileError::UncompilableStep {
                gap: format!("no compiled capability for \"{step}\""),
                step,
                span,
            });
        }
    }

    let (trigger_start, trigger_end) = clauses[trigger_position];
    let trigger_span = (offsets[trigger_start], offsets[trigger_end]);
    let trigger = ProcedureTrigger {
        objects: objects_in(&lower[trigger_start..trigger_end], None),
        source_text: description[trigger_span.0..trigger_span.1].to_owned(),
        source_span: trigger_span,
    };

    let mut steps = Vec::with_capacity(step_clauses.len());
    for (index, (found, (start, end))) in classified
        .into_iter()
        .zip(step_clauses.iter().copied())
        .enumerate()
    {
        let verb = found.expect("every clause classified above");
        let clause = &lower[start..end];
        let span = (offsets[start], offsets[end]);
        steps.push(ProcedureStep {
            id: String::new(),
            index: index + 1,
            objects: objects_in(clause, Some((verb.start, verb.end))),
            target_language: first_match(clause, ROLE_TRANSLATION_LANGUAGE).map(|found| found.slug),
            kind: verb.slug,
            source_text: description[span.0..span.1].to_owned(),
            source_span: span,
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

/// The meaning of `role` whose surface appears earliest in `hay`.
///
/// Earliest wins because clauses are imperative — the verb leads — so a later mention
/// of another vocabulary word ("reply with the translation") cannot outrank it. Ties
/// on position are broken by the longer surface, which prefers the more specific
/// reading.
fn first_match(hay: &str, role: &str) -> Option<Found> {
    let mut best: Option<Found> = None;
    for meaning in seed::lexicon().meanings_with_role(role) {
        for word in surfaces(meaning) {
            let Some(start) = hay.find(word) else {
                continue;
            };
            let candidate = Found {
                slug: meaning.slug.clone(),
                start,
                end: start + word.len(),
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
    best
}

/// Every object meaning mentioned in `clause`, in mention order, without repeats.
///
/// Occurrences overlapping `skip` (the span already consumed by the step verb) are
/// ignored so a verb surface cannot be re-read as its own object.
fn objects_in(clause: &str, skip: Option<(usize, usize)>) -> Vec<String> {
    let mut hits: Vec<(usize, String)> = Vec::new();
    for meaning in seed::lexicon().meanings_with_role(ROLE_SKILL_PROCEDURE_STEP_OBJECT) {
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

/// Lower-case `text` while recording, for every byte of the result, the byte offset it
/// came from in the original.
///
/// Case folding is not length-preserving in every script, so span reporting cannot
/// assume the two strings share offsets. The trailing sentinel makes the map total for
/// exclusive end offsets.
fn lower_with_offsets(text: &str) -> (String, Vec<usize>) {
    let mut lower = String::with_capacity(text.len());
    let mut offsets = Vec::with_capacity(text.len() + 1);
    for (index, character) in text.char_indices() {
        let before = lower.len();
        for folded in character.to_lowercase() {
            lower.push(folded);
        }
        for _ in before..lower.len() {
            offsets.push(index);
        }
    }
    offsets.push(text.len());
    (lower, offsets)
}

/// Sentence punctuation that always ends a clause.
///
/// These are marks, not words: they carry no language-specific meaning worth seeding,
/// unlike the connectives, which live in `data/seed/meanings-skill-procedure.lino`.
const CLAUSE_PUNCTUATION: &[char] = &[
    ',', ';', ':', '.', '!', '?', '，', '、', '；', '。', '！', '？', '।',
];

/// Split `lower` into clause spans on punctuation and on seeded connectives.
fn split_clauses(lower: &str) -> Vec<(usize, usize)> {
    let mut cuts: Vec<(usize, usize)> = Vec::new();
    for (index, character) in lower.char_indices() {
        if CLAUSE_PUNCTUATION.contains(&character) {
            cuts.push((index, index + character.len_utf8()));
        }
    }
    for meaning in seed::lexicon().meanings_with_role(ROLE_SKILL_PROCEDURE_CLAUSE_SEPARATOR) {
        for word in surfaces(meaning) {
            for (start, _) in lower.match_indices(word) {
                let end = start + word.len();
                if is_standalone(lower, start, end) {
                    cuts.push((start, end));
                }
            }
        }
    }
    cuts.sort_by(|left, right| left.0.cmp(&right.0).then(right.1.cmp(&left.1)));

    let mut spans = Vec::new();
    let mut cursor = 0usize;
    for (start, end) in cuts {
        if start < cursor {
            continue;
        }
        push_trimmed(lower, cursor, start, &mut spans);
        cursor = end;
    }
    push_trimmed(lower, cursor, lower.len(), &mut spans);
    spans
}

/// Is `lower[start..end]` a free-standing word rather than part of a longer one?
///
/// Ideographic connectives ("然后") are written without spaces, so for them the
/// neighbour test is skipped: in a script with no word separators, adjacency to
/// another ideograph is not evidence of being embedded in a larger word.
fn is_standalone(lower: &str, start: usize, end: usize) -> bool {
    if lower[start..end].chars().any(is_ideographic) {
        return true;
    }
    let before = lower[..start].chars().next_back();
    let after = lower[end..].chars().next();
    !before.is_some_and(char::is_alphanumeric) && !after.is_some_and(char::is_alphanumeric)
}

/// Does `character` belong to a script written without word separators?
const fn is_ideographic(character: char) -> bool {
    matches!(character, '\u{3400}'..='\u{9fff}' | '\u{f900}'..='\u{faff}')
}

/// Record `lower[start..end]` as a clause once its surrounding whitespace is dropped.
fn push_trimmed(lower: &str, start: usize, end: usize, spans: &mut Vec<(usize, usize)>) {
    let slice = &lower[start..end];
    let trimmed = slice.trim();
    if trimmed.is_empty() {
        return;
    }
    let offset = start + (slice.len() - slice.trim_start().len());
    spans.push((offset, offset + trimmed.len()));
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
