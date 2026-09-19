//! Ordered source text → [`ExtractedProcedure`] (issue #1138, plan 04 L9–L10).
//!
//! A step is `verified` only when an execution record says so; extraction never
//! sets it. An extracted procedure reaches the #919 ledger through
//! [`ExtractedProcedure::to_coding_procedure_source`], under the existing
//! execution and review gate rather than beside it.

use crate::procedure_text::ProcedureStepRecord;
use crate::seed::ROLE_STATEMENT_NEGATION_CUE;

const CODING_PROCEDURE_HEADER: &str = "Formal AI coding procedure";

/// An ordered, provenance-bearing procedure extracted from a trusted source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractedProcedure {
    pub id: String,
    pub goal: String,
    pub language: String,
    pub steps: Vec<ProcedureStep>,
    pub preconditions: Vec<String>,
    pub postconditions: Vec<String>,
    pub source_id: String,
    pub source_url: String,
    pub sha256: String,
    pub license_name: String,
    pub license_url: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcedureStep {
    pub position: usize,
    pub imperative: String,
    pub object: Option<String>,
    /// `"<doc_id>@<start>:<end>"`, the exact span the step was read from.
    pub source_span: String,
    /// Set only by an execution record, never by extraction.
    pub verified: bool,
}

/// Extract an ordered procedure from any captured ordered text: a how-to
/// guide's steps, a documentation page's numbered list, an answer's ordered
/// block.
#[must_use]
pub fn procedure_from_steps(
    goal: &str,
    steps: &[ProcedureStepRecord],
    language: &str,
) -> Option<ExtractedProcedure> {
    ExtractedProcedure::from_step_records(goal, steps, language)
}

impl ExtractedProcedure {
    /// The only constructor. `GuideStep::to_step_record()` in
    /// `src/how_to_guide.rs` is how a synthesised guide reaches it.
    #[must_use]
    pub fn from_step_records(
        goal: &str,
        steps: &[ProcedureStepRecord],
        language: &str,
    ) -> Option<Self> {
        let goal = goal.trim();
        let language = language.trim();
        if goal.is_empty()
            || language.is_empty()
            || steps.len() < crate::procedure_text::MIN_PROCEDURE_STEPS
        {
            return None;
        }

        let mut ordered = steps.iter().collect::<Vec<_>>();
        ordered.sort_by_key(|step| step.ordinal);
        let first = *ordered.first()?;
        if !valid_provenance(first)
            || ordered.iter().enumerate().any(|(index, step)| {
                step.ordinal != index + 1 || !same_source(first, step) || !valid_provenance(step)
            })
        {
            return None;
        }

        let extracted_steps = ordered
            .iter()
            .map(|record| step_from_record(record))
            .collect::<Option<Vec<_>>>()?;
        let mut procedure = Self {
            id: String::new(),
            goal: goal.to_owned(),
            language: language.to_owned(),
            steps: extracted_steps,
            preconditions: Vec::new(),
            postconditions: Vec::new(),
            source_id: first.source_id.clone(),
            source_url: first.source_url.clone(),
            sha256: first.sha256.clone(),
            license_name: first.license_name.clone(),
            license_url: first.license_url.clone(),
        };
        procedure.id = crate::engine::stable_id("extracted_procedure", &procedure.identity());
        Some(procedure)
    }

    /// Render into the versioned shape `coding_research_learning` already
    /// gates, so an extracted procedure enters the ledger through the existing
    /// execution and review boundary.
    #[must_use]
    pub fn to_coding_procedure_source(&self) -> String {
        let mut lines = vec![
            CODING_PROCEDURE_HEADER.to_owned(),
            field("SPDX-License-Identifier", &self.license_name),
            field("Task", &self.goal),
            field("Language", &self.language),
            field("Operation", "extracted_ordered_procedure"),
            field("Pattern", &self.goal),
            field(
                "Replacement",
                &self
                    .steps
                    .iter()
                    .map(|step| format!("{}. {}", step.position, step_text(step)))
                    .collect::<Vec<_>>()
                    .join(" "),
            ),
            field("Procedure-ID", &self.id),
            field("Source-ID", &self.source_id),
            field("Source-URL", &self.source_url),
            field("Source-SHA256", &self.sha256),
            field("License-URL", &self.license_url),
        ];
        for step in &self.steps {
            lines.push(field(&format!("Step-{}", step.position), &step_text(step)));
        }
        lines.join("\n") + "\n"
    }

    #[must_use]
    pub fn to_links_notation(&self) -> String {
        let mut out = String::new();
        crate::links_format::push_lino_node(&mut out, 0, "procedure", Some(&self.id));
        crate::links_format::push_lino_node(&mut out, 2, "goal", Some(&self.goal));
        crate::links_format::push_lino_node(&mut out, 2, "language", Some(&self.language));
        crate::links_format::push_lino_node(&mut out, 2, "source_id", Some(&self.source_id));
        crate::links_format::push_lino_node(&mut out, 2, "source_url", Some(&self.source_url));
        crate::links_format::push_lino_node(&mut out, 2, "sha256", Some(&self.sha256));
        crate::links_format::push_lino_node(&mut out, 2, "license_name", Some(&self.license_name));
        crate::links_format::push_lino_node(&mut out, 2, "license_url", Some(&self.license_url));
        for precondition in &self.preconditions {
            crate::links_format::push_lino_node(&mut out, 2, "precondition", Some(precondition));
        }
        for step in &self.steps {
            crate::links_format::push_lino_node(
                &mut out,
                2,
                "step",
                Some(&step.position.to_string()),
            );
            crate::links_format::push_lino_node(&mut out, 4, "imperative", Some(&step.imperative));
            if let Some(object) = &step.object {
                crate::links_format::push_lino_node(&mut out, 4, "object", Some(object));
            }
            crate::links_format::push_lino_node(
                &mut out,
                4,
                "source_span",
                Some(&step.source_span),
            );
            crate::links_format::push_lino_node(
                &mut out,
                4,
                "verified",
                Some(if step.verified { "true" } else { "false" }),
            );
        }
        for postcondition in &self.postconditions {
            crate::links_format::push_lino_node(&mut out, 2, "postcondition", Some(postcondition));
        }
        out
    }

    fn identity(&self) -> String {
        let mut values = [
            self.goal.as_str(),
            self.language.as_str(),
            self.source_id.as_str(),
            self.source_url.as_str(),
            self.sha256.as_str(),
            self.license_name.as_str(),
            self.license_url.as_str(),
        ]
        .join("\u{1f}");
        for step in &self.steps {
            values.push('\u{1e}');
            values.push_str(&step.position.to_string());
            values.push('\u{1f}');
            values.push_str(&step.imperative);
            values.push('\u{1f}');
            values.push_str(step.object.as_deref().unwrap_or_default());
            values.push('\u{1f}');
            values.push_str(&step.source_span);
        }
        values
    }
}

fn step_from_record(record: &ProcedureStepRecord) -> Option<ProcedureStep> {
    let text = record.text.trim();
    let imperative_end = text
        .char_indices()
        .find_map(|(index, character)| character.is_whitespace().then_some(index))
        .unwrap_or(text.len());
    let imperative = text[..imperative_end]
        .trim_matches(|character: char| !character.is_alphanumeric())
        .to_owned();
    if imperative.is_empty() {
        return None;
    }

    let remainder = text[imperative_end..]
        .trim()
        .trim_end_matches(|character: char| !character.is_alphanumeric())
        .trim();
    let object_end = first_role_surface(remainder, ROLE_STATEMENT_NEGATION_CUE)
        .map_or(remainder.len(), |(start, _)| start);
    let object = remainder[..object_end].trim();
    let source_span = format!(
        "{}#step={}@0:{}",
        record.source_url,
        record.ordinal,
        record.text.len()
    );
    Some(ProcedureStep {
        position: record.ordinal,
        imperative,
        object: (!object.is_empty()).then(|| object.to_owned()),
        source_span,
        verified: false,
    })
}

fn first_role_surface(text: &str, role: &str) -> Option<(usize, usize)> {
    let lower = text.to_lowercase();
    crate::seed::lexicon()
        .meanings_with_role(role)
        .flat_map(|meaning| meaning.lexemes.iter())
        .flat_map(|lexeme| lexeme.words.iter())
        .filter_map(|word| {
            let surface = word.text.to_lowercase();
            lower.match_indices(&surface).find_map(|(start, _)| {
                let end = start + surface.len();
                standalone(&lower, start, end).then_some((start, end))
            })
        })
        .min_by_key(|(start, end)| (*start, usize::MAX - (*end - *start)))
}

fn standalone(text: &str, start: usize, end: usize) -> bool {
    text[..start]
        .chars()
        .next_back()
        .is_none_or(|character| !character.is_alphanumeric())
        && text[end..]
            .chars()
            .next()
            .is_none_or(|character| !character.is_alphanumeric())
}

fn valid_provenance(step: &ProcedureStepRecord) -> bool {
    !step.text.trim().is_empty()
        && !step.source_id.trim().is_empty()
        && step
            .source_url
            .split_once("://")
            .is_some_and(|(scheme, rest)| matches!(scheme, "http" | "https") && !rest.is_empty())
        && step.sha256.len() == 64
        && step.sha256.bytes().all(|byte| byte.is_ascii_hexdigit())
        && !step.fetched_at.trim().is_empty()
        && !step.license_name.trim().is_empty()
        && !step.license_url.trim().is_empty()
}

fn same_source(first: &ProcedureStepRecord, candidate: &ProcedureStepRecord) -> bool {
    first.source_id == candidate.source_id
        && first.source_url == candidate.source_url
        && first.sha256 == candidate.sha256
        && first.fetched_at == candidate.fetched_at
        && first.license_name == candidate.license_name
        && first.license_url == candidate.license_url
        && first.depth == candidate.depth
}

fn step_text(step: &ProcedureStep) -> String {
    step.object.as_ref().map_or_else(
        || step.imperative.clone(),
        |object| format!("{} {object}", step.imperative),
    )
}

fn field(name: &str, value: &str) -> String {
    format!("{name}: {}", value.replace(['\r', '\n'], " ").trim())
}
