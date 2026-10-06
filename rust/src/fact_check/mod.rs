//! Fact checking over code, Git, GitHub, and the internet (issue #1179).
//!
//! The statement audit weighs repository statements against captured evidence,
//! but its evidence was caller-supplied only, and its structured extractor
//! turned every `key: value` line into an exclusive claim, so a YAML workflow
//! with N steps yielded one contradiction per repeated step key — 8,179
//! contradictions on this repository, every one a key collision. This module
//! owns the two halves of the fix:
//!
//! 1. **Claims, not keys.** [`structured_claim`] is the drop-in replacement
//!    for the unconditional `Claim::exclusive` in
//!    `statement_audit::extract::extract_structured`. A structured line
//!    becomes a claim only when it is a top-level (zero-indent) field whose
//!    key is a declared fact field of its document type
//!    (`data/seed/statement-audit-exclusions.lino`); sequence-item lines and
//!    nested keys stay structure. [`predicate_is_excluded`] additionally
//!    filters contradictions whose predicate carries a YAML list-item key, so
//!    audits already recorded in the legacy format read clean.
//! 2. **Evidence from every context.** The [`code_source`], [`git_source`],
//!    and [`web_source`] collectors answer bounded queries about a claim from
//!    the repository's own AST engine, its Git history, and classified
//!    internet captures, each weighed through `relative_meta_logic` with the
//!    trust tiers and priors declared in `data/seed/fact-check-sources.lino`
//!    (code above tests above generated status above docs above notes).
//!
//! Everything here is pure over caller-supplied inputs: filesystem and
//! network access stay at the collector boundaries, and no collector mutates
//! the audit corpus it reads.

pub mod code_source;
pub mod git_source;
pub mod web_source;

use crate::relative_meta_logic::{
    ASSUMED_TRUE_PRIOR, RelativeEvidence, SourceTier, StatementAssessment, TruthValue,
};
use crate::seed::parser::parse_lino;
use crate::statement_audit::Claim;

const EXCLUSIONS_LINO: &str =
    include_str!("../../embedded/data/seed/statement-audit-exclusions.lino");
const SOURCES_LINO: &str = include_str!("../../embedded/data/seed/fact-check-sources.lino");

/// The document family a structured path belongs to, matching the
/// `document_type` values of `data/seed/statement-audit-exclusions.lino`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentType {
    /// `Cargo.toml`-style manifests (`.toml`, `.json`).
    Manifest,
    /// YAML workflows and action metadata (`.yaml`, `.yml`).
    Workflow,
    /// Any other structured document; only `any` claim fields apply.
    Other,
}

impl DocumentType {
    /// Classify a repository path by extension, the same set
    /// `statement_audit::extract` treats as structured.
    #[must_use]
    pub fn of_path(path: &str) -> Self {
        match path.rsplit('.').next().unwrap_or_default() {
            "toml" | "json" => Self::Manifest,
            "yaml" | "yml" => Self::Workflow,
            _ => Self::Other,
        }
    }

    const fn slug(self) -> &'static str {
        match self {
            Self::Manifest => "manifest",
            Self::Workflow => "workflow",
            Self::Other => "any",
        }
    }
}

/// Whether a raw structured line is a YAML sequence item (`- key: value`).
///
/// These lines are list structure: sibling items repeat the same keys by
/// design, so they never form exclusive claims (issue #1179 R1).
#[must_use]
pub fn is_sequence_item_key(raw_line: &str) -> bool {
    raw_line.trim_start().starts_with("- ")
}

/// Whether `predicate` carries a YAML list-item key (`- name`, `- run`, …),
/// the shape of every one of the 8,179 false-positive contradictions.
#[must_use]
pub fn predicate_is_excluded(predicate: &str) -> bool {
    let Some(key) = predicate.strip_prefix("- ") else {
        return false;
    };
    let key = key.trim();
    sequence_item_keys()
        .iter()
        .any(|candidate| candidate == key)
}

/// The repeated step-level keys declared as sequence structure in the seed.
#[must_use]
pub fn sequence_item_keys() -> Vec<String> {
    registry_root(EXCLUSIONS_LINO, "statement_audit_exclusions")
        .iter()
        .flat_map(|exclusions| exclusions.children.iter())
        .filter(|record| record.name == "sequence_item_key")
        .flat_map(|record| {
            record
                .children
                .iter()
                .filter(|child| child.name == "key")
                .map(|child| child.id.clone())
                .collect::<Vec<_>>()
        })
        .collect()
}

/// The claim-field allow-list of fact-bearing top-level keys for one document
/// type, read from the seed. `document_type any` rows apply to every type.
#[must_use]
pub fn claim_fields(document_type: DocumentType) -> Vec<String> {
    let wanted = document_type.slug();
    registry_root(EXCLUSIONS_LINO, "statement_audit_exclusions")
        .iter()
        .flat_map(|exclusions| exclusions.children.iter())
        .filter(|record| {
            record.name == "claim_field"
                && (record.find_child_value("document_type") == wanted
                    || record.find_child_value("document_type") == "any")
        })
        .flat_map(|record| {
            record
                .children
                .iter()
                .filter(|child| child.name == "key")
                .map(|child| child.id.clone())
                .collect::<Vec<_>>()
        })
        .collect()
}

/// The drop-in claim decision for one raw structured line (issue #1179 R1).
///
/// Mirrors the key/value parsing of `extract_structured` but returns a claim
/// only when the line is a stated fact field: zero indentation (a top-level
/// document field, not a nested key), not a sequence item, and a key in the
/// claim-field allow-list of the document type. Everything else stays a
/// `SourceKind::Structured` statement with no claim.
#[must_use]
pub fn structured_claim(path: &str, raw_line: &str) -> Option<Claim> {
    if raw_line.starts_with(' ') || raw_line.starts_with('\t') {
        return None;
    }
    if is_sequence_item_key(raw_line) {
        return None;
    }
    let trimmed = raw_line.trim().trim_end_matches(',');
    if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with('[') {
        return None;
    }
    let pair = trimmed
        .split_once('=')
        .or_else(|| trimmed.split_once(':'))?;
    let (raw_key, raw_value) = pair;
    let key = raw_key.trim().trim_matches('"');
    let value = raw_value
        .trim()
        .trim_matches('"')
        .trim()
        .trim_end_matches(['.', ',', ';', ':'])
        .trim();
    if key.is_empty() || value.is_empty() || matches!(value, "{" | "[") {
        return None;
    }
    claim_fields(DocumentType::of_path(path))
        .iter()
        .any(|candidate| candidate == key)
        .then(|| Claim::exclusive(path, key, value))
}

/// The assumed-true prior declared for one evidence context in the seed,
/// falling back to the module default when the context is not registered.
#[must_use]
pub fn prior_for_context(context: &str) -> f64 {
    source_registry_row(context)
        .and_then(|row| row.find_child_value("prior").parse::<f64>().ok())
        .unwrap_or(ASSUMED_TRUE_PRIOR)
}

/// The trust tier declared for one evidence context in the seed, falling back
/// to independent corroboration when the context is not registered.
#[must_use]
pub fn tier_for_context(context: &str) -> SourceTier {
    let tier = source_registry_row(context)
        .map(|row| row.find_child_value("tier").to_owned())
        .unwrap_or_else(|| "independent_corroboration".to_owned());
    match tier.as_str() {
        "original_first_party" => SourceTier::OriginalFirstParty,
        "original_journalism" => SourceTier::OriginalJournalism,
        "unoriginal" => SourceTier::Unoriginal,
        _ => SourceTier::IndependentCorroboration,
    }
}

/// The reposting hosts that carry no weight, declared in the seed.
#[must_use]
pub fn web_aggregator_hosts() -> Vec<String> {
    registry_root(SOURCES_LINO, "fact_check_sources")
        .iter()
        .flat_map(|sources| sources.children.iter())
        .filter(|record| record.name == "web_aggregator_host")
        .flat_map(|record| {
            record
                .children
                .iter()
                .filter(|child| child.name == "host")
                .map(|child| child.id.clone())
                .collect::<Vec<_>>()
        })
        .collect()
}

/// One fact-check verdict: a statement, its context's declared prior, and the
/// relative-meta-logic assessment of the collected evidence.
#[derive(Debug, Clone, PartialEq)]
pub struct FactCheckVerdict {
    /// The statement text assessed.
    pub statement: String,
    /// The evidence context the statement came from (`code`, `git`, `docs`, …).
    pub context: String,
    /// The assessment, computed from the context's declared prior.
    pub assessment: StatementAssessment,
}

/// Assess one statement from its context's declared prior against collected
/// evidence (issue #1179 "What to build" item 3).
#[must_use]
pub fn fact_check(
    statement: impl Into<String>,
    context: &str,
    evidence: &[RelativeEvidence],
) -> FactCheckVerdict {
    let statement = statement.into();
    let prior = TruthValue::new(prior_for_context(context));
    let assessment = StatementAssessment::assess(statement.as_str(), prior, evidence);
    FactCheckVerdict {
        statement,
        context: context.to_owned(),
        assessment,
    }
}

fn source_registry_row(context: &str) -> Option<crate::seed::parser::LinoNode> {
    registry_root(SOURCES_LINO, "fact_check_sources")
        .into_iter()
        .flat_map(|sources| sources.children.into_iter())
        .find(|record| {
            record.name == "source_tier" && record.find_child_value("context") == context
        })
}

fn registry_root(text: &str, head: &str) -> Vec<crate::seed::parser::LinoNode> {
    parse_lino(text)
        .children
        .into_iter()
        .filter(|node| node.name == head)
        .collect()
}
