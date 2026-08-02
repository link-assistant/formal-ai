use std::error::Error;
use std::fmt;

use serde::Deserialize;

use crate::relative_meta_logic::{SourceTier, Stance};
use crate::seed::response_for;

use super::model::{EvidenceCapture, EvidenceSelector};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct EvidenceDocument {
    captures: Vec<RawCapture>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawCapture {
    statement: Option<String>,
    subject: Option<String>,
    predicate: Option<String>,
    value: Option<String>,
    source_label: String,
    source_url: String,
    tier: String,
    stance: String,
    strength: f64,
    captured_at: String,
    sha256: String,
}

/// A structural or provenance error in a replayable evidence document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidenceParseError {
    message: String,
}

impl EvidenceParseError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for EvidenceParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for EvidenceParseError {}

/// Parse externally captured evidence without performing network access.
///
/// Each capture selects either an exact statement string or a symbolic claim.
/// Provenance tier and stance use the stable slugs exposed by relative
/// meta-logic, while timestamps and content hashes make later replay possible.
pub fn parse_evidence_json(input: &str) -> Result<Vec<EvidenceCapture>, EvidenceParseError> {
    let document: EvidenceDocument = serde_json::from_str(input).map_err(|error| {
        EvidenceParseError::new(render_error(
            "statement_audit_invalid_json",
            &[("error", &error.to_string())],
        ))
    })?;
    document
        .captures
        .into_iter()
        .enumerate()
        .map(|(index, raw)| parse_capture(index, raw))
        .collect()
}

fn parse_capture(index: usize, raw: RawCapture) -> Result<EvidenceCapture, EvidenceParseError> {
    let selector = parse_selector(index, &raw)?;
    require_text(index, "source_label", &raw.source_label)?;
    require_text(index, "source_url", &raw.source_url)?;
    require_text(index, "captured_at", &raw.captured_at)?;
    require_text(index, "sha256", &raw.sha256)?;
    if !(raw.strength.is_finite() && (0.0..=1.0).contains(&raw.strength)) {
        return Err(field_error(
            index,
            "strength",
            "statement_audit_strength_range",
        ));
    }
    let tier = parse_tier(index, &raw.tier)?;
    let stance = parse_stance(index, &raw.stance)?;
    Ok(EvidenceCapture {
        selector,
        source_label: raw.source_label,
        source_url: raw.source_url,
        tier,
        stance,
        strength: raw.strength,
        captured_at: raw.captured_at,
        sha256: raw.sha256,
    })
}

fn parse_selector(index: usize, raw: &RawCapture) -> Result<EvidenceSelector, EvidenceParseError> {
    let has_claim_field = raw.subject.is_some() || raw.predicate.is_some() || raw.value.is_some();
    match (&raw.statement, has_claim_field) {
        (Some(_), true) => Err(field_error(
            index,
            "selector",
            "statement_audit_selector_exclusive",
        )),
        (Some(statement), false) => {
            require_text(index, "statement", statement)?;
            Ok(EvidenceSelector::StatementText(statement.clone()))
        }
        (None, true) => {
            let subject = raw
                .subject
                .as_deref()
                .ok_or_else(|| field_error(index, "selector", "statement_audit_claim_subject"))?;
            let predicate = raw
                .predicate
                .as_deref()
                .ok_or_else(|| field_error(index, "selector", "statement_audit_claim_predicate"))?;
            require_text(index, "subject", subject)?;
            require_text(index, "predicate", predicate)?;
            Ok(EvidenceSelector::Claim {
                subject: subject.to_owned(),
                predicate: predicate.to_owned(),
                value: raw.value.clone(),
            })
        }
        (None, false) => Err(field_error(
            index,
            "selector",
            "statement_audit_selector_required",
        )),
    }
}

fn parse_tier(index: usize, value: &str) -> Result<SourceTier, EvidenceParseError> {
    match value {
        "original_first_party" => Ok(SourceTier::OriginalFirstParty),
        "original_journalism" => Ok(SourceTier::OriginalJournalism),
        "independent_corroboration" => Ok(SourceTier::IndependentCorroboration),
        "unoriginal" => Ok(SourceTier::Unoriginal),
        _ => Err(field_error(index, "tier", "statement_audit_unknown_tier")),
    }
}

fn parse_stance(index: usize, value: &str) -> Result<Stance, EvidenceParseError> {
    match value {
        "supports" => Ok(Stance::Supports),
        "contradicts" => Ok(Stance::Contradicts),
        "neutral" => Ok(Stance::Neutral),
        _ => Err(field_error(
            index,
            "stance",
            "statement_audit_unknown_stance",
        )),
    }
}

fn require_text(index: usize, field: &str, value: &str) -> Result<(), EvidenceParseError> {
    if value.trim().is_empty() {
        Err(field_error(index, field, "statement_audit_nonempty"))
    } else {
        Ok(())
    }
}

fn field_error(index: usize, field: &str, detail_intent: &str) -> EvidenceParseError {
    let index = index.to_string();
    let detail = render_error(detail_intent, &[]);
    EvidenceParseError::new(render_error(
        "statement_audit_capture_field",
        &[("index", &index), ("field", field), ("detail", &detail)],
    ))
}

fn render_error(intent: &str, values: &[(&str, &str)]) -> String {
    let mut rendered = response_for(intent, "en").unwrap_or_else(|| intent.to_owned());
    for (name, value) in values {
        rendered = rendered.replace(&format!("{{{name}}}"), value);
    }
    rendered
}
