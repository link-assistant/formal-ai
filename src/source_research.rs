//! Executed source research over the replayable capture boundary.
//!
//! Search results are observations, not facts. This module keeps that boundary
//! explicit: a provider response is captured before its rankings are fused,
//! result pages are captured before they can update an option network, and a
//! statement page affects probability only when a caller classifies its exact
//! bytes. Every observation can be projected to Links Notation for review; no
//! seed or durable learning ledger is mutated without a later human gate.

use crate::event_log::EventLog;
use crate::links_format::format_lino_record;
use crate::option_evidence::{candidate_from_page, with_offer_from_page};
use crate::option_network::{OptionNetwork, Tier};
use crate::relative_meta_logic::RelativeEvidence;
use crate::source_fetch::{CachedSourceClient, FetchError, SourceCapture, SourceTransport};
use crate::statement_verification::{
    extract_statements, CapturedStatementEvidence, StatementPlan, StatementVerificationExecution,
    StatementVerificationPlan,
};
use crate::web_search_core::{
    execute_duckduckgo_search, FusedEntry, SearchExecution, WEB_SEARCH_PROVIDER_LIMIT,
    WEB_SEARCH_RRF_K,
};

/// One fused result page captured from the exact URL returned by the provider.
#[derive(Debug, Clone, PartialEq)]
pub struct ResearchPage {
    pub ranking: FusedEntry,
    pub capture: SourceCapture,
}

/// A ranked URL that could not be captured. It is diagnostic state, never
/// evidence and never an option-network observation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResearchFailure {
    pub url: String,
    pub error: FetchError,
}

/// Provider capture, fused rankings, and the result pages successfully read.
#[derive(Debug, Clone, PartialEq)]
pub struct SourceResearchExecution {
    pub query: String,
    pub search: SearchExecution,
    pub pages: Vec<ResearchPage>,
    pub failures: Vec<ResearchFailure>,
}

impl SourceResearchExecution {
    /// Record only operations backed by completed captures.
    pub fn record(&self, log: &mut EventLog) {
        for capture in &self.search.captures {
            capture.record(log);
        }
        for boundary in &self.search.component_boundaries {
            log.append("web_search:component", boundary.clone());
        }
        for diagnostic in &self.search.component_diagnostics {
            log.append("web_search:component_error", diagnostic.clone());
        }
        log.append("web_search:provider", String::from("duckduckgo"));
        for ranking in &self.search.rankings {
            log.append(
                "web_search:rank",
                format!(
                    "provider={} rank={} url={}",
                    ranking.provider_id, ranking.rank, ranking.url
                ),
            );
        }
        log.append(
            "web_search:combined",
            format!(
                "rrf:k={WEB_SEARCH_RRF_K} results={}",
                self.search.fused.len()
            ),
        );
        for page in &self.pages {
            page.capture.record(log);
        }
        for failure in &self.failures {
            log.append(
                "error:fetch",
                format!("url={} error={}", failure.url, failure.error),
            );
        }
    }

    /// A deterministic, proposal-only learning projection.
    ///
    /// Cache-hit state is intentionally absent: live capture and offline replay
    /// represent the same learned observation when URL, time, bytes, and ranking
    /// are the same.
    #[must_use]
    pub fn learning_proposal(&self) -> String {
        let mut records = vec![format_lino_record(
            "source_research",
            &[
                ("query", self.query.clone()),
                ("provider", String::from("duckduckgo")),
                ("results", self.search.fused.len().to_string()),
                ("captured_pages", self.pages.len().to_string()),
            ],
        )];
        for boundary in &self.search.component_boundaries {
            records.push(format_lino_record(
                "search_component_boundary",
                &[("component", boundary.clone())],
            ));
        }
        for capture in self
            .search
            .captures
            .iter()
            .chain(self.pages.iter().map(|page| &page.capture))
        {
            records.push(capture_learning_record(capture));
        }
        for ranking in &self.search.rankings {
            records.push(format_lino_record(
                "search_observation",
                &[
                    ("provider", ranking.provider_id.clone()),
                    ("rank", ranking.rank.to_string()),
                    ("url", ranking.url.clone()),
                    ("title", ranking.title.clone()),
                ],
            ));
        }
        records.join("\n")
    }
}

/// Search through the common capture client and optionally read the first
/// `page_limit` fused URLs. Individual page failures are retained as diagnostics
/// while other real results continue.
pub fn execute_source_research<T: SourceTransport>(
    client: &CachedSourceClient<T>,
    query: &str,
    page_limit: usize,
) -> Result<SourceResearchExecution, FetchError> {
    let search = execute_duckduckgo_search(client, query)?;
    let mut pages = Vec::new();
    let mut failures = Vec::new();
    let limit = page_limit.min(WEB_SEARCH_PROVIDER_LIMIT as usize);
    for ranking in search.fused.iter().take(limit) {
        match client.fetch(&ranking.url) {
            Ok(capture) => pages.push(ResearchPage {
                ranking: ranking.clone(),
                capture,
            }),
            Err(error) => failures.push(ResearchFailure {
                url: ranking.url.clone(),
                error,
            }),
        }
    }
    Ok(SourceResearchExecution {
        query: query.to_owned(),
        search,
        pages,
        failures,
    })
}

/// Source research that has already updated an associative option network.
#[derive(Debug, Clone, PartialEq)]
pub struct OptionResearchExecution {
    pub research: SourceResearchExecution,
    pub observed_ids: Vec<String>,
}

impl OptionResearchExecution {
    /// Combine capture observations and the derived option network into one
    /// reviewable proposal. Promotion remains the caller's human-gated step.
    #[must_use]
    pub fn learning_proposal(&self, network: &OptionNetwork) -> String {
        format!(
            "{}\n{}",
            self.research.learning_proposal(),
            network.links_notation()
        )
    }
}

/// Execute general option research and learn only attributes stated in captured
/// result pages. Missing quantities, nominal values, or prices remain unknown.
pub fn execute_option_research<T: SourceTransport>(
    network: &mut OptionNetwork,
    client: &CachedSourceClient<T>,
    query: &str,
    tier: Tier,
    currency: &str,
    page_limit: usize,
) -> Result<OptionResearchExecution, FetchError> {
    let research = execute_source_research(client, query, page_limit)?;
    let mut observed_ids = Vec::new();
    for page in &research.pages {
        let id = page.capture.source_url().to_owned();
        let text = String::from_utf8_lossy(page.capture.bytes());
        let candidate = candidate_from_page(&id, tier, &text, network.constraints());
        let candidate = with_offer_from_page(
            candidate,
            &text,
            currency,
            source_label(&id),
            page.capture.source_url(),
        );
        network.observe(candidate);
        observed_ids.push(id);
    }
    Ok(OptionResearchExecution {
        research,
        observed_ids,
    })
}

/// Search execution paired with its per-statement assessments.
#[derive(Debug, Clone, PartialEq)]
pub struct StatementResearchExecution {
    pub searches: Vec<SourceResearchExecution>,
    pub verification: StatementVerificationExecution,
}

/// Search each extracted statement, read bounded result pages, and admit only
/// evidence explicitly classified from the exact capture.
pub fn execute_statement_research<T, C>(
    sample: &str,
    client: &CachedSourceClient<T>,
    page_limit: usize,
    classify: C,
) -> Result<StatementResearchExecution, FetchError>
where
    T: SourceTransport,
    C: Fn(&str, &SourceCapture) -> Option<RelativeEvidence>,
{
    let mut searches = Vec::new();
    let mut captures = Vec::new();
    let mut classified = Vec::new();
    let mut statements = Vec::new();
    for statement in extract_statements(sample) {
        let initial = StatementPlan::new(statement.clone(), &[]);
        let research = execute_source_research(client, &initial.query, page_limit)?;
        let mut evidence = Vec::new();
        for page in &research.pages {
            let capture = page.capture.clone();
            if let Some(item) = classify(&statement, &capture) {
                evidence.push(item.clone());
                classified.push(CapturedStatementEvidence {
                    statement: statement.clone(),
                    capture: capture.clone(),
                    evidence: item,
                });
            }
            captures.push(capture);
        }
        statements.push(StatementPlan::new(statement, &evidence));
        searches.push(research);
    }
    Ok(StatementResearchExecution {
        searches,
        verification: StatementVerificationExecution {
            plan: StatementVerificationPlan { statements },
            captures,
            classified,
        },
    })
}

fn capture_learning_record(capture: &SourceCapture) -> String {
    format_lino_record(
        "source_observation",
        &[
            ("url", capture.source_url().to_owned()),
            ("fetched_at", capture.fetched_at().to_owned()),
            ("sha256", capture.sha256().to_owned()),
        ],
    )
}

fn source_label(url: &str) -> &str {
    url.split_once("://")
        .map_or(url, |(_, rest)| rest.split('/').next().unwrap_or(rest))
}
