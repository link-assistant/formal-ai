//! Statement-level fusion for captured web research.
//!
//! Provider rows and fetched pages are observations, never answers by
//! themselves. This module formalizes both boundaries, joins only statements
//! with complete language-independent meaning links, applies the source tiers
//! from relative meta-logic, and projects the smallest useful ranked context
//! with reversible provenance.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use crate::event_log::EventLog;
use crate::formal_system::FormalSystem;
use crate::links_format::format_lino_record;
use crate::relative_meta_logic::{SourceTier, TruthValue};
use crate::source_fetch::{CachedSourceClient, FetchError, SourceTransport};
use crate::source_research::{execute_source_research, SourceResearchExecution};
use crate::summarization::{merge_into_formal_context, SourcedStatement, StatementSignature};
use crate::translation::{
    formalize_prompt, FormalizationAnchorKind, FormalizationCandidate, FormalizationRole,
};

const MAX_PRESENTED_MEANINGS: usize = 3;
const CONFLICT_SOURCE_DISAGREEMENT: &str = "source_disagreement";

/// Trust and language metadata supplied at the capture boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchSourceClassification {
    pub tier: SourceTier,
    pub language: String,
}

impl SearchSourceClassification {
    #[must_use]
    pub fn new(tier: SourceTier, language: impl Into<String>) -> Self {
        Self {
            tier,
            language: language.into(),
        }
    }

    /// Detect the language independently for each captured fragment.
    #[must_use]
    pub fn auto(tier: SourceTier) -> Self {
        Self::new(tier, "auto")
    }
}

/// Which exact search boundary supplied a formalized observation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchObservationOrigin {
    SearchHit,
    FetchedSource,
}

impl SearchObservationOrigin {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SearchHit => "search_hit",
            Self::FetchedSource => "fetched_source",
        }
    }
}

/// Trace from one exact source fragment into its meta-language statement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormalizedSearchObservation {
    pub source_url: String,
    pub origin: SearchObservationOrigin,
    pub original_text: String,
    pub rendered_text: String,
    pub language: String,
    pub formalization: String,
    pub semantic_terms: Vec<String>,
}

/// Normalized evidence card shared by CLI, HTTP, Telegram, and web rendering.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedSearchSource {
    pub url: String,
    pub title: String,
    pub quote: String,
    pub read_more: String,
    pub language: String,
    pub tier: SourceTier,
    pub providers: Vec<String>,
    pub capture_sha256: Option<String>,
}

/// One selected meaning with all source variants retained.
#[derive(Debug, Clone, PartialEq)]
pub struct FusedSearchStatement {
    pub id: String,
    pub text: String,
    pub semantic_links: String,
    pub source_count: usize,
    pub sources: Vec<NormalizedSearchSource>,
    pub posterior: TruthValue,
    pub weight: u8,
    pub conflict: Option<&'static str>,
}

/// Presentation-independent answer context.
#[derive(Debug, Clone, PartialEq)]
pub struct SearchFusionAnswer {
    pub query: String,
    pub target_language: String,
    pub sources: Vec<NormalizedSearchSource>,
    pub statements: Vec<FusedSearchStatement>,
}

/// Captures, formalizations, merge/rank result, and presentation projection.
#[derive(Debug, Clone)]
pub struct SearchFusionExecution {
    pub research: SourceResearchExecution,
    pub observations: Vec<FormalizedSearchObservation>,
    pub ignored_sources: Vec<String>,
    pub answer: SearchFusionAnswer,
    merge_links: Vec<(String, String, String)>,
}

impl SearchFusionExecution {
    /// Deterministic trace that is identical for live capture and cache replay.
    #[must_use]
    pub fn trace(&self) -> String {
        let mut lines = Vec::new();
        for observation in &self.observations {
            lines.push(
                format_args!(
                    "search_fusion:formalization origin={} source={} language={} meaning={}",
                    observation.origin.as_str(),
                    observation.source_url,
                    observation.language,
                    semantic_key(&observation.semantic_terms)
                )
                .to_string(),
            );
        }
        for source in &self.ignored_sources {
            lines.push(
                format_args!("search_fusion:ignored source={source} tier=unoriginal").to_string(),
            );
        }
        for (representative, source, justification) in &self.merge_links {
            lines.push(format_args!(
                "search_fusion:merge representative={representative} source={source} justification={justification}"
            ).to_string());
        }
        for (rank, statement) in self.answer.statements.iter().enumerate() {
            lines.push(
                format_args!(
                    "search_fusion:rank rank={} id={} weight={} posterior={}",
                    rank + 1,
                    statement.id,
                    statement.weight,
                    statement.posterior
                )
                .to_string(),
            );
            if statement.conflict.is_some() {
                lines.push(
                    format_args!(
                        "conflict:{CONFLICT_SOURCE_DISAGREEMENT} statement={}",
                        statement.id
                    )
                    .to_string(),
                );
            }
        }
        lines.join("\n")
    }

    /// Append the exact captures and every derived statement operation.
    pub fn record(&self, log: &mut EventLog) {
        self.research.record(log);
        for observation in &self.observations {
            log.append(
                "search_fusion:formalization",
                format_args!(
                    "origin={} source={} language={} meaning={}",
                    observation.origin.as_str(),
                    observation.source_url,
                    observation.language,
                    semantic_key(&observation.semantic_terms)
                )
                .to_string(),
            );
        }
        for source in &self.ignored_sources {
            log.append(
                "search_fusion:ignored",
                format_args!("source={source} tier=unoriginal").to_string(),
            );
        }
        for (representative, source, justification) in &self.merge_links {
            log.append(
                "search_fusion:merge",
                format_args!(
                    "representative={representative} source={source} justification={justification}"
                )
                .to_string(),
            );
        }
        for (rank, statement) in self.answer.statements.iter().enumerate() {
            log.append(
                "search_fusion:rank",
                format_args!(
                    "rank={} id={} weight={} posterior={}",
                    rank + 1,
                    statement.id,
                    statement.weight,
                    statement.posterior
                )
                .to_string(),
            );
            if statement.conflict.is_some() {
                log.append(
                    "conflict:source_disagreement",
                    format!("statement={}", statement.id),
                );
            }
        }
    }

    /// Reviewable proposal containing captures and every formalize/merge/rank
    /// receipt. Cache-hit state is omitted so offline replay stays identical.
    #[must_use]
    pub fn learning_proposal(&self) -> String {
        let mut records = vec![self.research.learning_proposal()];
        for observation in &self.observations {
            records.push(format_lino_record(
                "search_statement_formalization",
                &[
                    ("source", observation.source_url.clone()),
                    ("origin", observation.origin.as_str().to_owned()),
                    ("language", observation.language.clone()),
                    ("original", observation.original_text.clone()),
                    ("rendered", observation.rendered_text.clone()),
                    ("meaning", semantic_key(&observation.semantic_terms)),
                ],
            ));
        }
        for (representative, source, justification) in &self.merge_links {
            records.push(format_lino_record(
                "search_statement_merge",
                &[
                    ("representative", representative.clone()),
                    ("source", source.clone()),
                    ("justification", justification.clone()),
                ],
            ));
        }
        for (rank, statement) in self.answer.statements.iter().enumerate() {
            records.push(format_lino_record(
                "search_statement_rank",
                &[
                    ("rank", (rank + 1).to_string()),
                    ("statement", statement.id.clone()),
                    ("weight", statement.weight.to_string()),
                    ("posterior", statement.posterior.to_string()),
                    (
                        "conflict",
                        statement.conflict.unwrap_or_default().to_owned(),
                    ),
                ],
            ));
        }
        records.join("\n")
    }

    /// Shared Markdown projection used by every non-browser client.
    #[must_use]
    pub fn render_markdown(&self) -> String {
        let language = &self.answer.target_language;
        let mut output = crate::seed::localized_response("search_fusion_header", language)
            .unwrap_or_else(|| String::from("{query}\n\n{statements}"))
            .replace("{query}", &self.answer.query)
            .replace("{count}", &self.answer.statements.len().to_string())
            .replace("{sources}", &self.answer.sources.len().to_string());
        let read_more = crate::seed::localized_response("search_fusion_read_more", language)
            .unwrap_or_else(|| String::from("read_more"));
        let mut statements = String::new();
        for (index, statement) in self.answer.statements.iter().enumerate() {
            let tier_list = statement
                .sources
                .iter()
                .map(|source| source.tier.slug())
                .collect::<Vec<_>>()
                .join("|");
            let conflict = statement.conflict.map_or_else(String::new, |value| {
                let mut rendered = String::from(" conflict=");
                rendered.push_str(value);
                rendered
            });
            let _ = writeln!(statements, "{}. {}", index + 1, statement.text);
            let _ = writeln!(
                statements,
                "   `posterior={} source_count={} source_tier={tier_list}{conflict}`",
                statement.posterior, statement.source_count
            );
            for source in &statement.sources {
                let _ = writeln!(statements, "   - [{}]({})", source.title, source.url);
                let _ = writeln!(statements, "     > {}", source.quote);
                let _ = writeln!(statements, "     [{read_more}]({})", source.read_more);
            }
            statements.push('\n');
        }
        let statements_slot = ["{", "statements", "}"].concat();
        output = output.replace(&statements_slot, statements.trim_end());
        output
    }
}

/// Execute captured search research and fuse its rows/pages at statement level.
pub fn execute_search_fusion<T, C>(
    client: &CachedSourceClient<T>,
    query: &str,
    target_language: &str,
    page_limit: usize,
    classify: C,
) -> Result<SearchFusionExecution, FetchError>
where
    T: SourceTransport,
    C: Fn(&str) -> SearchSourceClassification,
{
    let research = execute_source_research(client, query, page_limit)?;
    let mut observations = Vec::new();
    let mut sources = Vec::new();
    let mut ignored_sources = Vec::new();
    let mut sourced_statements = Vec::new();

    for ranking in &research.search.fused {
        let classification = classify(&ranking.url);
        let page = research
            .pages
            .iter()
            .find(|page| page.ranking.url == ranking.url);
        let page_text = page.map(|page| visible_text(page.capture.bytes()));
        let (title, excerpt) = split_title_excerpt(&ranking.title, &ranking.excerpt);
        let quote = page_text
            .as_deref()
            .and_then(first_sentence)
            .unwrap_or_else(|| excerpt.clone());
        let detected_source_language = if classification.language == "auto" {
            crate::language::detect(&quote).slug().to_owned()
        } else {
            classification.language.clone()
        };
        let source = NormalizedSearchSource {
            url: ranking.url.clone(),
            title,
            quote,
            read_more: ranking.url.clone(),
            language: detected_source_language,
            tier: classification.tier,
            providers: ranking
                .providers
                .iter()
                .map(|(provider, rank)| format!("{provider}#{rank}"))
                .collect(),
            capture_sha256: page.map(|page| page.capture.sha256().to_owned()),
        };
        sources.push(source);

        let mut fragments = vec![(SearchObservationOrigin::SearchHit, excerpt)];
        if let Some(text) = page_text {
            fragments.push((SearchObservationOrigin::FetchedSource, text));
        }
        for (origin, fragment) in fragments {
            let fragment_language = if classification.language == "auto" {
                crate::language::detect(&fragment).slug()
            } else {
                classification.language.as_str()
            };
            for statement in crate::summarization::formalize(&fragment) {
                let observation = formalize_observation(
                    &ranking.url,
                    origin,
                    &statement.text,
                    fragment_language,
                    target_language,
                );
                if classification.tier != SourceTier::Unoriginal {
                    sourced_statements.push(
                        SourcedStatement::from_sentence(
                            &observation.rendered_text,
                            &ranking.url,
                            classification.tier,
                        )
                        .with_semantic_terms(observation.semantic_terms.clone()),
                    );
                }
                observations.push(observation);
            }
        }
        if classification.tier == SourceTier::Unoriginal {
            ignored_sources.push(ranking.url.clone());
        }
    }

    let merged = merge_into_formal_context(
        "search_fusion_context",
        FormalSystem::new("search_fusion_context")
            .with_universe("captured search statements")
            .with_interpretation("relative source provenance"),
        &sourced_statements,
    );
    let sources_by_url: BTreeMap<_, _> = sources
        .iter()
        .map(|source| (source.url.clone(), source.clone()))
        .collect();
    let quotes_by_statement: BTreeMap<_, _> = observations
        .iter()
        .map(|observation| {
            let signature = StatementSignature::with_semantic_terms(
                &observation.rendered_text,
                &observation.semantic_terms,
            );
            (
                (observation.source_url.clone(), signature.key()),
                observation.original_text.clone(),
            )
        })
        .collect();
    let statements = select_statements(&merged, &sources_by_url, &quotes_by_statement);
    let merge_links = merged
        .report
        .links
        .iter()
        .map(|link| {
            (
                link.representative.clone(),
                link.source.clone(),
                link.justification.clone(),
            )
        })
        .collect();
    Ok(SearchFusionExecution {
        research,
        observations,
        ignored_sources,
        answer: SearchFusionAnswer {
            query: query.to_owned(),
            target_language: target_language.to_owned(),
            sources,
            statements,
        },
        merge_links,
    })
}

fn formalize_observation(
    source_url: &str,
    origin: SearchObservationOrigin,
    text: &str,
    language: &str,
    target_language: &str,
) -> FormalizedSearchObservation {
    let candidate = formalize_prompt(text.trim_end_matches(sentence_terminator), language);
    let semantic_terms = complete_semantic_terms(&candidate);
    let rendered_text = if language == target_language || semantic_terms.is_empty() {
        normalized_sentence(text)
    } else {
        render_candidate(&candidate, target_language).map_or_else(
            || normalized_sentence(text),
            |value| normalized_sentence(&value),
        )
    };
    FormalizedSearchObservation {
        source_url: source_url.to_owned(),
        origin,
        original_text: normalized_sentence(text),
        rendered_text,
        language: language.to_owned(),
        formalization: candidate.to_links_notation(),
        semantic_terms,
    }
}

fn complete_semantic_terms(candidate: &FormalizationCandidate) -> Vec<String> {
    let mut terms = Vec::new();
    for role in [
        FormalizationRole::Subject,
        FormalizationRole::Predicate,
        FormalizationRole::Object,
    ] {
        let Some(slot) = candidate.slot(role) else {
            return Vec::new();
        };
        if !matches!(
            slot.anchor.kind,
            FormalizationAnchorKind::WikidataItem | FormalizationAnchorKind::WikidataProperty
        ) {
            return Vec::new();
        }
        terms.push(format!("{}={}", role.slug(), slot.anchor.id));
    }
    terms
}

fn render_candidate(candidate: &FormalizationCandidate, target_language: &str) -> Option<String> {
    let mut surfaces = Vec::new();
    for role in [
        FormalizationRole::Subject,
        FormalizationRole::Predicate,
        FormalizationRole::Object,
    ] {
        let slot = candidate.slot(role)?;
        let wikidata = slot.anchor.id.strip_prefix("wikidata:")?;
        let meaning = crate::seed::lexicon().meaning_by_wikidata(wikidata)?;
        let surface = if role == FormalizationRole::Predicate {
            meaning
                .lexemes
                .iter()
                .find(|lexeme| lexeme.language == target_language)
                .and_then(|lexeme| {
                    lexeme
                        .words
                        .iter()
                        .find(|word| !word.action.is_empty())
                        .or_else(|| lexeme.words.first())
                })
                .map(|word| word.text.as_str())?
        } else {
            meaning.word_in(target_language)?
        };
        surfaces.push(surface);
    }
    let joined = surfaces.join(" ");
    Some(capitalize_first(&joined))
}

fn select_statements(
    merged: &crate::summarization::MergedContext,
    sources: &BTreeMap<String, NormalizedSearchSource>,
    quotes_by_statement: &BTreeMap<(String, String), String>,
) -> Vec<FusedSearchStatement> {
    let mut selected_meanings: Vec<Vec<String>> = Vec::new();
    let mut output = Vec::new();
    for ranked in &merged.ranked {
        let meaning = ranked.statement.signature.terms.clone();
        let seen = selected_meanings.contains(&meaning);
        if !seen && selected_meanings.len() >= MAX_PRESENTED_MEANINGS {
            continue;
        }
        if !seen {
            selected_meanings.push(meaning);
        }
        let signature = ranked.statement.signature.key();
        let mut statement_sources: Vec<NormalizedSearchSource> = ranked
            .statement
            .sources()
            .into_iter()
            .filter_map(|url| {
                let mut source = sources.get(url).cloned()?;
                if let Some(quote) = quotes_by_statement.get(&(url.to_owned(), signature.clone())) {
                    source.quote.clone_from(quote);
                }
                Some(source)
            })
            .collect();
        statement_sources.sort_by(|left, right| {
            right
                .tier
                .weight_percent()
                .cmp(&left.tier.weight_percent())
                .then_with(|| left.url.cmp(&right.url))
        });
        output.push(FusedSearchStatement {
            id: ranked.statement.id.clone(),
            text: normalized_sentence(&ranked.statement.representative.text),
            semantic_links: ranked.statement.signature.terms.join(" "),
            source_count: ranked.statement.source_count(),
            sources: statement_sources,
            posterior: ranked.probability,
            weight: ranked.score.weight,
            conflict: ranked
                .is_contested()
                .then_some(CONFLICT_SOURCE_DISAGREEMENT),
        });
    }
    output
}

fn split_title_excerpt(title: &str, excerpt: &str) -> (String, String) {
    if title != excerpt && !title.trim().is_empty() {
        return (title.trim().to_owned(), excerpt.trim().to_owned());
    }
    if let Some((prefix, remainder)) = excerpt.split_once(" - ") {
        if !prefix.trim().is_empty() && !remainder.trim().is_empty() {
            return (prefix.trim().to_owned(), remainder.trim().to_owned());
        }
    }
    let fallback = title.trim();
    (fallback.to_owned(), excerpt.trim().to_owned())
}

fn visible_text(bytes: &[u8]) -> String {
    let input = String::from_utf8_lossy(bytes);
    let mut output = String::new();
    let mut inside_tag = false;
    for character in input.chars() {
        match character {
            '<' => inside_tag = true,
            '>' => {
                inside_tag = false;
                output.push(' ');
            }
            _ if !inside_tag => output.push(character),
            _ => {}
        }
    }
    output.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn first_sentence(text: &str) -> Option<String> {
    crate::summarization::formalize(text)
        .first()
        .map(|statement| normalized_sentence(&statement.text))
}

fn normalized_sentence(text: &str) -> String {
    let trimmed = text.trim();
    if trimmed.is_empty() || trimmed.ends_with(sentence_terminator) {
        trimmed.to_owned()
    } else {
        format!("{trimmed}.")
    }
}

const fn sentence_terminator(character: char) -> bool {
    matches!(character, '.' | '!' | '?' | '。' | '…' | '।' | '॥')
}

fn capitalize_first(text: &str) -> String {
    let mut characters = text.chars();
    let Some(first) = characters.next() else {
        return String::new();
    };
    first.to_uppercase().chain(characters).collect()
}

fn semantic_key(terms: &[String]) -> String {
    if terms.is_empty() {
        String::from("lexical_fallback")
    } else {
        terms.join("|")
    }
}
