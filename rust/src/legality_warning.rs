//! Request-legality advisory (issue #836).
//!
//! Warn, don't police: when an incoming request matches one of the seeded
//! legality patterns (`data/seed/legality-patterns.lino`), the user gets a
//! clear, early caution — what appears illegal, which category fired, that
//! jurisdictions vary, and that the classification is not legal advice —
//! and then decides for themselves whether to rephrase, explain a
//! legitimate purpose, or withdraw. A narrow refuse set (child safety,
//! credible serious violence) is refused outright; everything else warns.
//!
//! Sensitive-but-legal framings — education, defensive security,
//! journalism, academic study — are exemption records in the same seed and
//! suppress the warning, which keeps the false-positive bias low. Every
//! decision is logged (`legality:category`, `legality:disposition`,
//! `legality:framing`) so behavior is auditable, and no warning text lives
//! in Rust: the templates come from
//! `data/seed/multilingual-responses-legality.lino`.

use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::seed::parser::parse_lino;
use crate::solver_handlers::finalize_simple;

const PATTERNS_PATH: &str = "data/seed/legality-patterns.lino";

/// What the seeded catalogue says to do with a matched category.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LegalityDisposition {
    /// Answer nothing further; the narrow prohibited set.
    Refuse,
    /// Answer with the caution template; the user decides.
    Warn,
}

impl LegalityDisposition {
    /// The seed's `disposition` field.
    fn from_seed(value: &str) -> Option<Self> {
        match value {
            "refuse" => Some(Self::Refuse),
            "warn" => Some(Self::Warn),
            _ => None,
        }
    }

    /// The word the evidence log carries.
    const fn as_str(self) -> &'static str {
        match self {
            Self::Refuse => "refuse",
            Self::Warn => "warn",
        }
    }
}

/// One `pattern` record from the seeded catalogue.
#[derive(Debug, Clone)]
pub struct LegalityPattern {
    /// The category slug (e.g. `fraud`).
    pub category: String,
    pub disposition: LegalityDisposition,
    /// Why a user can read; catalogue notation.
    pub reason: String,
    /// The jurisdiction-variance note interpolated into the warning.
    pub jurisdiction_note: String,
    phrases: Vec<String>,
}

/// The framing an `exemption` record names (e.g. `security_research`).
#[derive(Debug, Clone)]
pub struct LegalityExemption {
    pub framing: String,
    phrases: Vec<String>,
}

/// Look up embedded seed content by its registered path.
fn seed_text(path: &str) -> Option<&'static str> {
    crate::seed::seed_files()
        .into_iter()
        .find(|(registered, _)| *registered == path)
        .map(|(_, text)| text)
}

/// The seeded pattern catalogue, in file order.
#[must_use]
pub fn legality_patterns() -> Vec<LegalityPattern> {
    let Some(text) = seed_text(PATTERNS_PATH) else {
        return Vec::new();
    };
    let tree = parse_lino(text);
    let mut out = Vec::new();
    for record in tree
        .children
        .iter()
        .filter(|child| child.name == "legality_patterns")
        .flat_map(|root| root.children.iter())
        .filter(|record| record.name == "pattern")
    {
        let Some(disposition) =
            LegalityDisposition::from_seed(record.find_child_value("disposition"))
        else {
            continue;
        };
        let category = record.find_child_value("category").to_string();
        if category.is_empty() {
            continue;
        }
        let phrases = record
            .children
            .iter()
            .filter(|child| child.name == "phrase")
            .filter_map(|child| {
                let phrase = child.id.clone();
                (!phrase.is_empty()).then_some(phrase)
            })
            .collect();
        out.push(LegalityPattern {
            category,
            disposition,
            reason: record.find_child_value("reason").to_string(),
            jurisdiction_note: record.find_child_value("jurisdiction_note").to_string(),
            phrases,
        });
    }
    out
}

/// The seeded exemption framings, in file order.
#[must_use]
pub fn legality_exemptions() -> Vec<LegalityExemption> {
    let Some(text) = seed_text(PATTERNS_PATH) else {
        return Vec::new();
    };
    let tree = parse_lino(text);
    let mut out = Vec::new();
    for record in tree
        .children
        .iter()
        .filter(|child| child.name == "legality_exemptions")
        .flat_map(|root| root.children.iter())
        .filter(|record| record.name == "exemption")
    {
        let framing = record.find_child_value("framing").to_string();
        if framing.is_empty() {
            continue;
        }
        let phrases = record
            .children
            .iter()
            .filter(|child| child.name == "phrase")
            .filter_map(|child| {
                let phrase = child.id.clone();
                (!phrase.is_empty()).then_some(phrase)
            })
            .collect();
        out.push(LegalityExemption { framing, phrases });
    }
    out
}

/// The category's human-readable name: the slug with underscores read as
/// spaces. Catalogue slugs are notation; the readable form is derived, not
/// stored twice.
fn category_readable(category: &str) -> String {
    category.replace('_', " ")
}

/// The first pattern whose phrase occurs in the normalized or lowered
/// prompt. File order is the precedence.
fn matched_pattern<'a>(
    prompt: &str,
    normalized: &str,
    patterns: &'a [LegalityPattern],
) -> Option<&'a LegalityPattern> {
    let lower = prompt.to_lowercase();
    patterns.iter().find(|pattern| {
        pattern
            .phrases
            .iter()
            .any(|phrase| normalized.contains(phrase.as_str()) || lower.contains(phrase.as_str()))
    })
}

/// The first exemption whose phrase occurs in the prompt.
fn matched_framing<'a>(
    prompt: &str,
    normalized: &str,
    exemptions: &'a [LegalityExemption],
) -> Option<&'a str> {
    let lower = prompt.to_lowercase();
    exemptions
        .iter()
        .find(|exemption| {
            exemption.phrases.iter().any(|phrase| {
                normalized.contains(phrase.as_str()) || lower.contains(phrase.as_str())
            })
        })
        .map(|exemption| exemption.framing.as_str())
}

/// The outcome of assessing one request against the seeded catalogue.
#[derive(Debug, Clone)]
pub struct LegalityAssessment {
    pub pattern: LegalityPattern,
    /// The exemption framing that matched, if any.
    pub exempt_framing: Option<String>,
    /// What the agent should do after exemptions are applied.
    pub effective: LegalityDisposition,
}

/// Assess a request. `None` means the seeded catalogue has nothing to say
/// and the request proceeds unflagged.
pub fn assess(prompt: &str, normalized: &str) -> Option<LegalityAssessment> {
    let patterns = legality_patterns();
    let pattern = matched_pattern(prompt, normalized, &patterns)?;
    let exemptions = legality_exemptions();
    let exempt_framing = matched_framing(prompt, normalized, &exemptions);
    Some(LegalityAssessment {
        effective: pattern.disposition,
        pattern: pattern.clone(),
        exempt_framing: exempt_framing.map(str::to_owned),
    })
}

impl LegalityAssessment {
    /// A warn category with a matched exemption framing: answer normally,
    /// no warning.
    #[must_use]
    pub fn warning_suppressed(&self) -> bool {
        self.pattern.disposition == LegalityDisposition::Warn && self.exempt_framing.is_some()
    }
}

/// Fill a localized legality template's `{placeholder}` slots.
fn template(intent: &str, values: &[(&str, &str)]) -> String {
    let mut out = crate::seed::localized_response(intent, "en").unwrap_or_default();
    for (key, value) in values {
        out = out.replace(&format!("{{{key}}}"), value);
    }
    out
}

/// Assess the request and, when the catalogue says warn or refuse, return
/// the advisory answer.
///
/// `None` when the request is unflagged or its warn
/// was suppressed by a legitimate framing — the caller then proceeds to
/// its normal handling.
pub fn handle_legality_warning(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let assessment = assess(prompt, normalized)?;
    log.append("legality:category", assessment.pattern.category.clone());
    log.append(
        "legality:disposition",
        assessment.pattern.disposition.as_str().to_owned(),
    );
    if let Some(framing) = &assessment.exempt_framing {
        log.append("legality:framing", framing.clone());
    }
    if assessment.warning_suppressed() {
        log.append(
            "legality:outcome",
            "warn_suppressed_by_legitimate_framing".to_owned(),
        );
        return None;
    }

    let values = [
        ("category", assessment.pattern.category.as_str()),
        (
            "category_readable",
            &category_readable(&assessment.pattern.category),
        ),
        ("reason", assessment.pattern.reason.as_str()),
        (
            "jurisdiction_note",
            assessment.pattern.jurisdiction_note.as_str(),
        ),
    ];
    let (intent, response_link, confidence) = match assessment.pattern.disposition {
        LegalityDisposition::Refuse => ("legality_refuse", "response:legality_refuse", 0.9),
        LegalityDisposition::Warn => ("legality_warn", "response:legality_warn", 0.7),
    };
    let body = template(intent, &values);
    Some(finalize_simple(
        prompt,
        log,
        intent,
        response_link,
        &body,
        confidence,
    ))
}
