//! The consented source-prelearning half of issue #705's anticipation.
//!
//! Extracted from `src/anticipation.rs` so the parent stays under the
//! file-size warn band the whole tree is held to (plan 07 leaf 17). Nothing
//! here changed in the move: the consent gate, the standard source-research
//! cache and the retained URL, fetch time, digest, cache status and TTL are
//! exactly the ones PR #887 wrote.

use crate::engine::stable_id;
use crate::links_format::format_lino_record;
use crate::source_fetch::{CachedSourceClient, SourceTransport};
use crate::source_research::execute_source_research;

use super::{AnticipationConfig, AnticipationPlan, ProbeStatus, source_record};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnticipationConsent {
    Denied,
    Granted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrelearningStatus {
    ConsentRequired,
    Captured,
    NoSource,
    FetchFailed,
}

impl PrelearningStatus {
    const fn slug(self) -> &'static str {
        match self {
            Self::ConsentRequired => "consent_required",
            Self::Captured => "captured",
            Self::NoSource => "no_source",
            Self::FetchFailed => "fetch_failed",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrelearningAttempt {
    pub prediction_id: String,
    pub query: String,
    pub status: PrelearningStatus,
    pub diagnostic: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrelearnedSource {
    pub id: String,
    pub prediction_id: String,
    pub class_id: String,
    pub base_event_id: String,
    pub query: String,
    pub aliases: Vec<String>,
    pub answer: String,
    pub result_url: String,
    pub source_url: String,
    pub fetched_at: String,
    pub sha256: String,
    pub cached: bool,
    pub expires_at: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PrelearningRun {
    pub attempts: Vec<PrelearningAttempt>,
    pub sources: Vec<PrelearnedSource>,
}

impl PrelearningRun {
    #[must_use]
    pub fn links_notation(&self) -> String {
        let mut records = Vec::new();
        for attempt in &self.attempts {
            records.push(format_lino_record(
                "anticipation_prelearning_attempt",
                &[
                    ("prediction", attempt.prediction_id.clone()),
                    ("query", attempt.query.clone()),
                    ("status", attempt.status.slug().to_owned()),
                    ("diagnostic", attempt.diagnostic.clone()),
                ],
            ));
        }
        for source in &self.sources {
            records.push(source_record(source, None));
        }
        records.join("\n")
    }
}

pub fn prelearn_predictions<T: SourceTransport>(
    plan: &AnticipationPlan,
    client: &CachedSourceClient<T>,
    consent: AnticipationConsent,
    config: &AnticipationConfig,
) -> PrelearningRun {
    let candidates = plan
        .predictions
        .iter()
        .filter_map(|prediction| {
            plan.probes
                .iter()
                .find(|probe| {
                    probe.prediction_id == prediction.id && probe.status != ProbeStatus::Passed
                })
                .map(|probe| (prediction, probe))
        })
        .collect::<Vec<_>>();
    let mut run = PrelearningRun::default();
    for (prediction, probe) in candidates {
        if consent == AnticipationConsent::Denied {
            run.attempts.push(PrelearningAttempt {
                prediction_id: prediction.id.clone(),
                query: probe.prompt.clone(),
                status: PrelearningStatus::ConsentRequired,
                diagnostic: String::from("fetch_consent_required"),
            });
            continue;
        }
        match execute_source_research(client, &probe.prompt, config.source_page_limit) {
            Err(error) => run.attempts.push(PrelearningAttempt {
                prediction_id: prediction.id.clone(),
                query: probe.prompt.clone(),
                status: PrelearningStatus::FetchFailed,
                diagnostic: error.to_string(),
            }),
            Ok(research) => {
                let Some(result) = research.search.fused.first() else {
                    run.attempts.push(PrelearningAttempt {
                        prediction_id: prediction.id.clone(),
                        query: probe.prompt.clone(),
                        status: PrelearningStatus::NoSource,
                        diagnostic: String::from("search_result_absent"),
                    });
                    continue;
                };
                let answer = if result.excerpt.trim().is_empty() {
                    result.title.trim()
                } else {
                    result.excerpt.trim()
                };
                if answer.is_empty() {
                    run.attempts.push(PrelearningAttempt {
                        prediction_id: prediction.id.clone(),
                        query: probe.prompt.clone(),
                        status: PrelearningStatus::NoSource,
                        diagnostic: String::from("search_excerpt_absent"),
                    });
                    continue;
                }
                let capture = research
                    .pages
                    .iter()
                    .find(|page| page.ranking.url == result.url)
                    .map(|page| &page.capture)
                    .or_else(|| research.search.captures.first());
                let Some(capture) = capture else {
                    continue;
                };
                let fetched_at = capture.fetched_at().parse::<u64>().unwrap_or_default();
                let aliases = prediction
                    .variants
                    .iter()
                    .filter(|variant| variant.base_event_id == probe.base_event_id)
                    .map(|variant| variant.prompt.clone())
                    .collect::<Vec<_>>();
                let id = stable_id(
                    "anticipation_source",
                    &format!("{}:{}:{}", prediction.id, result.url, capture.sha256()),
                );
                run.sources.push(PrelearnedSource {
                    id,
                    prediction_id: prediction.id.clone(),
                    class_id: prediction.class.id.clone(),
                    base_event_id: probe.base_event_id.clone(),
                    query: probe.prompt.clone(),
                    aliases,
                    answer: answer.to_owned(),
                    result_url: result.url.clone(),
                    source_url: capture.source_url().to_owned(),
                    fetched_at: capture.fetched_at().to_owned(),
                    sha256: capture.sha256().to_owned(),
                    cached: capture.cached(),
                    expires_at: fetched_at.saturating_add(config.ttl_seconds),
                });
                run.attempts.push(PrelearningAttempt {
                    prediction_id: prediction.id.clone(),
                    query: probe.prompt.clone(),
                    status: PrelearningStatus::Captured,
                    diagnostic: result.url.clone(),
                });
            }
        }
    }
    run
}
