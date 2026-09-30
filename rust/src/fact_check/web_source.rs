//! Internet evidence for fact checking (issue #1179 R6/R7).
//!
//! A thin adapter over the existing internet-facing machinery rather than a
//! new fetch path: external-claim research reuses
//! `source_research::execute_statement_research` (which already extracts
//! statements, reads bounded result pages through the capture boundary, and
//! classifies them into `RelativeEvidence`), and this module only re-tiers
//! the classified captures by URL host using the aggregator policy declared
//! in `data/seed/fact-check-sources.lino`. GitHub-shaped evidence (issues,
//! PRs, reviews, Actions runs) reuses the `github_logs` collector
//! configuration rather than re-implementing `gh` invocation.

use std::path::PathBuf;

use crate::github_logs::GithubLogCollectorConfig;
use crate::relative_meta_logic::{RelativeEvidence, SourceTier, Stance, TruthValue};
use crate::statement_audit::{EvidenceCapture, EvidenceSelector};
use crate::statement_verification::CapturedStatementEvidence;

use super::web_aggregator_hosts;

/// The trust tier a captured page deserves by its host (issue #1179 R7).
///
/// A host on the seed's aggregator list is a repost and carries no weight;
/// every other host is independent corroboration — the first-party case is
/// decided by the caller, who knows whether the host is the claim's subject.
#[must_use]
pub fn tier_for_url(url: &str) -> SourceTier {
    let host = url_host(url);
    if web_aggregator_hosts().iter().any(|aggregator| {
        host == aggregator || host.strip_prefix("www.").is_some_and(|stripped| stripped == *aggregator)
    }) {
        return SourceTier::Unoriginal;
    }
    SourceTier::IndependentCorroboration
}

/// The host of `url`, or the whole string when it carries no scheme.
#[must_use]
pub fn url_host(url: &str) -> &str {
    let rest = url.split_once("://").map_or(url, |(_, rest)| rest);
    rest.split('/').next().unwrap_or(rest)
}

/// Whether a statement is about something outside the repository and so can
/// be researched on the internet: it names no repository path and no
/// backticked code symbol.
#[must_use]
pub fn is_external_statement(text: &str) -> bool {
    !text.contains('`')
        && !text.split_whitespace().any(|token| {
            token.contains('/') && token.contains('.') && !token.contains("://")
        })
}

/// The search query for one external statement: its longest sentence-shaped
/// span, quoted so the provider matches the phrase rather than the words.
#[must_use]
pub fn web_query_for(statement: &str) -> String {
    let trimmed = statement.trim().trim_end_matches(['.', '!', '?']);
    let span = trimmed
        .split(". ")
        .map(str::trim)
        .filter(|sentence| !sentence.is_empty())
        .max_by_key(|sentence| sentence.len())
        .unwrap_or(trimmed);
    format!("\"{span}\"")
}

/// Whether a statement is worth one internet query at all: external and not
/// so short that a phrase search would only echo the claim.
#[must_use]
pub fn researchable(statement: &str) -> bool {
    is_external_statement(statement) && statement.split_whitespace().count() >= 4
}

/// Re-tier classified internet captures into replayable statement-audit
/// evidence without letting the timestamp, URL, or content hash drift from
/// the underlying retrieval (issue #1179 R7).
#[must_use]
pub fn internet_evidence_captures(
    classified: &[CapturedStatementEvidence],
) -> Vec<EvidenceCapture> {
    classified
        .iter()
        .map(|item| {
            EvidenceCapture::for_statement(
                item.statement.clone(),
                item.evidence.source_label.clone(),
                item.capture.source_url().to_owned(),
                tier_for_url(item.capture.source_url()),
                item.evidence.stance,
                item.evidence.strength.get(),
            )
            .with_capture(item.capture.fetched_at(), item.capture.sha256())
        })
        .collect()
}

/// The same classified captures as relative evidence for direct assessment.
#[must_use]
pub fn internet_evidence(classified: &[CapturedStatementEvidence]) -> Vec<RelativeEvidence> {
    classified
        .iter()
        .map(|item| {
            RelativeEvidence::new(
                item.evidence.source_label.clone(),
                tier_for_url(item.capture.source_url()),
                item.evidence.stance,
                item.evidence.strength,
            )
        })
        .collect()
}

/// A GitHub-evidence capture plan for one claim subject (issue #1179 R6).
///
/// Reuses `github_logs`' own configuration and command planning — the
/// collector shells out to `gh` exactly as `collect_github_logs` does, so
/// there is no second `gh` implementation here. The plan names the issue, PR,
/// review, and Actions-run captures relevant to a claim whose subject is a
/// repository path or symbol.
#[must_use]
pub fn github_capture_plan(repo: &str, output_dir: PathBuf) -> GithubLogCollectorConfig {
    GithubLogCollectorConfig {
        repo: repo.to_owned(),
        output_dir,
        issues: Vec::new(),
        pulls: Vec::new(),
        runs: Vec::new(),
        // One page of each recent stream is enough to answer "does any issue,
        // PR, review, or run speak to this claim" without unbounded capture.
        recent_issues: 30,
        recent_pulls: 30,
        recent_runs: 10,
        branch: None,
    }
}

/// Relative evidence from one GitHub capture file, classified by the caller.
///
/// GitHub's own issues, PRs, and Actions runs are original first-party
/// sources about the repository they belong to, so the tier is fixed here and
/// only the stance and strength are caller-supplied.
#[must_use]
pub fn github_evidence(kind: &str, stance: Stance, strength: f64) -> RelativeEvidence {
    RelativeEvidence::new(
        format!("github:{kind}"),
        SourceTier::OriginalFirstParty,
        stance,
        TruthValue::new(strength),
    )
}

/// The statement-audit selector for one GitHub-shaped claim.
#[must_use]
pub fn github_claim_selector(subject: &str) -> EvidenceSelector {
    EvidenceSelector::Claim {
        subject: subject.to_owned(),
        predicate: "github_state".to_owned(),
        value: None,
    }
}
