//! Per-statement verification planning for the document-verification class.
//!
//! Issue #535 asks us to *"use our web search to check for each statement in the
//! text"* and to weigh those statements with
//! [`relative_meta_logic`](crate::relative_meta_logic): assume a statement true,
//! raise its probability with trusted original-first evidence, lower it with
//! contradicting evidence, and ignore reposts.
//!
//! This module turns a raw text sample into a deterministic, inspectable plan:
//! it splits the sample into statements across scripts, builds a grounding
//! web-search query for each, and produces an assumed-true
//! [`StatementAssessment`](crate::relative_meta_logic::StatementAssessment) plus
//! the trusted-source tier policy that governs how live evidence would move each
//! statement. The solver runs offline and deterministically, so no network call
//! is made here; instead the plan records exactly what would be checked and how
//! the resulting evidence would be weighed, which the handler replays into the
//! append-only event log.

use crate::relative_meta_logic::{
    RelativeEvidence, SourceTier, Stance, StatementAssessment, TruthValue, ASSUMED_TRUE_PRIOR,
};

/// Sentence terminators across the scripts the solver recognises: ASCII stops,
/// CJK full stop / exclamation / question, the Devanagari danda and double
/// danda, and the Arabic question mark.
const SENTENCE_TERMINATORS: &[char] = &['.', '!', '?', '。', '！', '？', '।', '॥', '؟', '।', '\n'];

/// Minimum number of words a fragment must contain to count as a checkable
/// statement. Below this it is treated as a heading or fragment and skipped.
const MIN_STATEMENT_WORDS: usize = 3;

/// Minimum number of non-whitespace characters an otherwise word-sparse
/// fragment must contain to count as a statement. This is the fallback gate for
/// scripts that do not separate words with spaces (Chinese, Japanese), where a
/// whole sentence is a single whitespace token.
const MIN_STATEMENT_CHARS: usize = 6;

/// The trusted-source tiers, in descending trust order.
///
/// These govern how live evidence for a statement would be weighed. Original
/// first-party and original journalism sources are trusted most; unoriginal
/// reposts are ignored.
pub const TRUSTED_SOURCE_POLICY: &[SourceTier] = &[
    SourceTier::OriginalFirstParty,
    SourceTier::OriginalJournalism,
    SourceTier::IndependentCorroboration,
    SourceTier::Unoriginal,
];

const MARKET_PRICE_CLAIM_STATUS_CONTRADICTED: &str = "contradicted";
const MARKET_PRICE_CLAIM_STATUS_WITHIN_RANGE: &str = "within_recorded_range";

const ETH_ALIASES: &[&str] = &[
    "$eth",
    "eth",
    "ethereum",
    "etherium",
    "эфириум",
    "эфир",
    "以太坊",
    "एथेरियम",
];

const MARKET_PRICE_REFERENCES: &[MarketPriceReference] = &[MarketPriceReference {
    asset: "ETH",
    asset_label: "Ethereum",
    aliases: ETH_ALIASES,
    quote_currency: "USDT",
    period: "2024",
    source_id: "binance_ethusdt_1d_2024",
    source_label: "Binance ETHUSDT daily klines",
    source_url: "https://api.binance.com/api/v3/klines?symbol=ETHUSDT&interval=1d&startTime=1704067200000&endTime=1735689599999&limit=1000",
    observed_min_price: 2100.0,
    observed_min_date: "2024-01-03",
    observed_max_price: 4107.8,
    observed_max_date: "2024-12-16",
}];

/// A single checkable statement with its grounding query and assumed-true
/// assessment.
#[derive(Debug, Clone, PartialEq)]
pub struct StatementPlan {
    /// The statement text as extracted from the sample.
    pub statement: String,
    /// The web-search query that would ground this statement.
    pub query: String,
    /// The relative-meta-logic assessment given the evidence weighed so far.
    pub assessment: StatementAssessment,
}

/// A structured market-price claim extracted from OCR or attached text.
#[derive(Debug, Clone, PartialEq)]
pub struct MarketPriceClaim {
    /// Canonical asset ticker, e.g. `ETH`.
    pub asset: String,
    /// Human label for the asset, e.g. `Ethereum`.
    pub asset_label: String,
    /// The year or period the claim talks about.
    pub period: String,
    /// Claimed spot price.
    pub claimed_price: f64,
    /// Claimed quote currency. `$` is normalized to `USD`.
    pub currency: String,
    /// Original statement fragment.
    pub statement: String,
}

/// A versioned external market-data range used to check extracted claims.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MarketPriceReference {
    /// Canonical asset ticker.
    pub asset: &'static str,
    /// Human label for the asset.
    pub asset_label: &'static str,
    /// Natural-language and ticker aliases accepted for this asset.
    pub aliases: &'static [&'static str],
    /// Quote currency used by the source.
    pub quote_currency: &'static str,
    /// Covered period.
    pub period: &'static str,
    /// Stable source slug for trace logs.
    pub source_id: &'static str,
    /// Human-readable source label.
    pub source_label: &'static str,
    /// Source URL used for the captured raw data.
    pub source_url: &'static str,
    /// Minimum observed price in the covered period.
    pub observed_min_price: f64,
    /// Date of the minimum observed price.
    pub observed_min_date: &'static str,
    /// Maximum observed price in the covered period.
    pub observed_max_price: f64,
    /// Date of the maximum observed price.
    pub observed_max_date: &'static str,
}

/// A market-price claim assessed with relative-meta-logic evidence.
#[derive(Debug, Clone, PartialEq)]
pub struct MarketPriceAssessment {
    /// The extracted claim.
    pub claim: MarketPriceClaim,
    /// Stable status slug: `contradicted` or `within_recorded_range`.
    pub status: &'static str,
    /// Stable source slug.
    pub source_id: &'static str,
    /// Human-readable source label.
    pub source_label: &'static str,
    /// Source URL used for the captured raw data.
    pub source_url: &'static str,
    /// Minimum observed price in the period.
    pub observed_min_price: f64,
    /// Date of the minimum observed price.
    pub observed_min_date: &'static str,
    /// Maximum observed price in the period.
    pub observed_max_price: f64,
    /// Date of the maximum observed price.
    pub observed_max_date: &'static str,
    /// The relative-meta-logic statement plan after source evidence is weighed.
    pub statement_plan: StatementPlan,
}

impl MarketPriceAssessment {
    /// Stable one-line trace payload for append-only evidence logs.
    #[must_use]
    pub fn trace_payload(&self) -> String {
        format!(
            "asset={} period={} claimed={:.2} status={} source={} min={:.2} \
             min_date={} max={:.2} max_date={} posterior={}",
            self.claim.asset,
            self.claim.period,
            self.claim.claimed_price,
            self.status,
            self.source_id,
            self.observed_min_price,
            self.observed_min_date,
            self.observed_max_price,
            self.observed_max_date,
            self.statement_plan.assessment.posterior,
        )
    }

    /// Human-readable summary for the chat answer.
    #[must_use]
    pub fn summary_sentence(&self) -> String {
        if self.status == MARKET_PRICE_CLAIM_STATUS_CONTRADICTED {
            format!(
                "{} is contradicted: {} reports {} {} daily candles in {} \
                 stayed between ${:.2} on {} and ${:.2} on {}.",
                self.claim.statement,
                self.source_label,
                self.claim.asset,
                self.source_quote_currency(),
                self.claim.period,
                self.observed_min_price,
                self.observed_min_date,
                self.observed_max_price,
                self.observed_max_date,
            )
        } else {
            format!(
                "{} is within the recorded {} {} daily candle range for {} \
                 (${:.2} to ${:.2}).",
                self.claim.statement,
                self.claim.asset,
                self.source_quote_currency(),
                self.claim.period,
                self.observed_min_price,
                self.observed_max_price,
            )
        }
    }

    fn source_quote_currency(&self) -> &'static str {
        MARKET_PRICE_REFERENCES
            .iter()
            .find(|reference| reference.source_id == self.source_id)
            .map_or("USD", |reference| reference.quote_currency)
    }
}

impl StatementPlan {
    /// Build a plan for `statement`, weighing any already-collected `evidence`
    /// (empty in the deterministic offline path, non-empty when a caller has
    /// gathered grounding results).
    #[must_use]
    pub fn new(statement: impl Into<String>, evidence: &[RelativeEvidence]) -> Self {
        let statement = statement.into();
        let query = grounding_query(&statement);
        let assessment = StatementAssessment::assess(
            statement.clone(),
            TruthValue::new(ASSUMED_TRUE_PRIOR),
            evidence,
        );
        Self {
            statement,
            query,
            assessment,
        }
    }
}

/// A verification plan over every statement extracted from a text sample.
#[derive(Debug, Clone, PartialEq)]
pub struct StatementVerificationPlan {
    /// One plan per extracted statement, in source order.
    pub statements: Vec<StatementPlan>,
}

impl StatementVerificationPlan {
    /// Extract statements from `sample` and plan grounding for each, with no
    /// evidence collected yet (the deterministic offline path).
    #[must_use]
    pub fn from_sample(sample: &str) -> Self {
        let statements = extract_statements(sample)
            .into_iter()
            .map(|statement| StatementPlan::new(statement, &[]))
            .collect();
        Self { statements }
    }

    /// Whether any statement was extracted.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.statements.is_empty()
    }

    /// The number of statements planned.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.statements.len()
    }
}

/// Split `sample` into checkable statements across scripts, trimming
/// whitespace and dropping fragments shorter than [`MIN_STATEMENT_WORDS`].
#[must_use]
pub fn extract_statements(sample: &str) -> Vec<String> {
    let mut statements = Vec::new();
    let mut current = String::new();
    for character in sample.chars() {
        if SENTENCE_TERMINATORS.contains(&character) {
            push_statement(&mut statements, &current);
            current.clear();
        } else {
            current.push(character);
        }
    }
    push_statement(&mut statements, &current);
    statements
}

/// Extract market-price claims such as `ETH in 2024: $1,700` from OCR or text.
///
/// The extractor is pattern-based rather than example-based: it looks for a
/// known asset alias, a four-digit period year, and a currency-marked amount in
/// the same fragment. Additional assets can be added by extending the market
/// reference registry without changing the parser.
#[must_use]
pub fn extract_market_price_claims(sample: &str) -> Vec<MarketPriceClaim> {
    let mut claims: Vec<MarketPriceClaim> = Vec::new();
    for fragment in market_price_fragments(sample) {
        if let Some(claim) = parse_market_price_claim(&fragment) {
            if !claims.iter().any(|existing| {
                existing.asset == claim.asset
                    && existing.period == claim.period
                    && (existing.claimed_price - claim.claimed_price).abs() < f64::EPSILON
                    && existing.statement == claim.statement
            }) {
                claims.push(claim);
            }
        }
    }
    claims
}

/// Assess extracted market-price claims against the built-in market-data facts.
#[must_use]
pub fn assess_market_price_claims(claims: &[MarketPriceClaim]) -> Vec<MarketPriceAssessment> {
    claims
        .iter()
        .filter_map(assess_market_price_claim)
        .collect()
}

fn assess_market_price_claim(claim: &MarketPriceClaim) -> Option<MarketPriceAssessment> {
    let reference = MARKET_PRICE_REFERENCES
        .iter()
        .find(|reference| reference.asset == claim.asset && reference.period == claim.period)?;
    let status = if claim.claimed_price < reference.observed_min_price
        || claim.claimed_price > reference.observed_max_price
    {
        MARKET_PRICE_CLAIM_STATUS_CONTRADICTED
    } else {
        MARKET_PRICE_CLAIM_STATUS_WITHIN_RANGE
    };
    let stance = if status == MARKET_PRICE_CLAIM_STATUS_CONTRADICTED {
        Stance::Contradicts
    } else {
        Stance::Supports
    };
    let evidence = [RelativeEvidence::new(
        reference.source_label,
        SourceTier::OriginalFirstParty,
        stance,
        0.95,
    )];
    let statement_plan = StatementPlan::new(claim.statement.clone(), &evidence);
    Some(MarketPriceAssessment {
        claim: claim.clone(),
        status,
        source_id: reference.source_id,
        source_label: reference.source_label,
        source_url: reference.source_url,
        observed_min_price: reference.observed_min_price,
        observed_min_date: reference.observed_min_date,
        observed_max_price: reference.observed_max_price,
        observed_max_date: reference.observed_max_date,
        statement_plan,
    })
}

fn market_price_fragments(sample: &str) -> Vec<String> {
    let mut fragments = Vec::new();
    for line in sample.lines() {
        let line = line.split_whitespace().collect::<Vec<_>>().join(" ");
        if line.is_empty() {
            continue;
        }
        let positions = ascii_asset_positions(&line);
        if positions.len() <= 1 {
            fragments.push(line);
            continue;
        }
        for (index, start) in positions.iter().copied().enumerate() {
            let end = positions.get(index + 1).copied().unwrap_or(line.len());
            let fragment = line[start..end].trim();
            if !fragment.is_empty() {
                fragments.push(fragment.to_owned());
            }
        }
    }
    fragments
}

fn ascii_asset_positions(line: &str) -> Vec<usize> {
    let lower = line.to_ascii_lowercase();
    let mut positions = Vec::new();
    for alias in ["$eth", "ethereum", "etherium", "eth"] {
        for (position, _) in lower.match_indices(alias) {
            if alias_occurs_at(&lower, alias, position) {
                positions.push(position);
            }
        }
    }
    positions.sort_unstable();
    positions.dedup();
    positions
}

fn parse_market_price_claim(fragment: &str) -> Option<MarketPriceClaim> {
    let reference = MARKET_PRICE_REFERENCES.iter().find(|reference| {
        reference
            .aliases
            .iter()
            .any(|alias| alias_occurs(fragment, alias))
    })?;
    let period = extract_year(fragment)?;
    let claimed_price = extract_currency_amount(fragment, &period)?;
    Some(MarketPriceClaim {
        asset: reference.asset.to_owned(),
        asset_label: reference.asset_label.to_owned(),
        period,
        claimed_price,
        currency: "USD".to_owned(),
        statement: fragment.trim().to_owned(),
    })
}

fn alias_occurs(fragment: &str, alias: &str) -> bool {
    if alias.is_ascii() {
        let lower = fragment.to_ascii_lowercase();
        let alias = alias.to_ascii_lowercase();
        return lower
            .match_indices(&alias)
            .any(|(position, _)| alias_occurs_at(&lower, &alias, position));
    }
    fragment.contains(alias)
}

fn alias_occurs_at(lower: &str, alias: &str, position: usize) -> bool {
    let before = position
        .checked_sub(1)
        .and_then(|index| lower.as_bytes().get(index))
        .copied();
    let after = lower.as_bytes().get(position + alias.len()).copied();
    let alias_starts_with_word = alias
        .as_bytes()
        .first()
        .is_some_and(u8::is_ascii_alphanumeric);
    let alias_ends_with_word = alias
        .as_bytes()
        .last()
        .is_some_and(u8::is_ascii_alphanumeric);
    (!alias_starts_with_word || before.is_none_or(|byte| !byte.is_ascii_alphanumeric()))
        && (!alias_ends_with_word || after.is_none_or(|byte| !byte.is_ascii_alphanumeric()))
}

fn extract_year(fragment: &str) -> Option<String> {
    let bytes = fragment.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if !bytes[index].is_ascii_digit() {
            index += 1;
            continue;
        }
        let start = index;
        while index < bytes.len() && bytes[index].is_ascii_digit() {
            index += 1;
        }
        if index - start == 4 {
            let candidate = &fragment[start..index];
            if (1900..=2100).contains(&candidate.parse::<u16>().ok()?) {
                return Some(candidate.to_owned());
            }
        }
    }
    None
}

fn extract_currency_amount(fragment: &str, period: &str) -> Option<f64> {
    let mut search_start = 0;
    while let Some(relative_dollar) = fragment[search_start..].find('$') {
        let dollar = search_start + relative_dollar;
        if let Some(price) = parse_number_after(&fragment[dollar + 1..]).filter(|price| {
            (*price - period.parse::<f64>().unwrap_or_default()).abs() > f64::EPSILON
        }) {
            return Some(price);
        }
        search_start = dollar + 1;
    }
    let lower = fragment.to_lowercase();
    for marker in ["usd", "usdt", "доллар", "美元", "डॉलर"] {
        if let Some(position) = lower.find(marker) {
            if let Some(price) = parse_number_after(&fragment[position + marker.len()..]) {
                return Some(price);
            }
            if let Some(price) = parse_number_before(&fragment[..position]) {
                return Some(price);
            }
        }
    }
    None
}

fn parse_number_after(value: &str) -> Option<f64> {
    parse_number(value.trim_start().chars())
}

fn parse_number_before(value: &str) -> Option<f64> {
    let reversed = value
        .trim_end()
        .chars()
        .rev()
        .take_while(|character| {
            character.is_ascii_digit() || matches!(character, ',' | '.' | '_' | ' ' | '\u{00a0}')
        })
        .collect::<String>();
    let number = reversed.chars().rev().collect::<String>();
    parse_number(number.chars())
}

fn parse_number(characters: impl IntoIterator<Item = char>) -> Option<f64> {
    let mut normalized = String::new();
    let mut saw_digit = false;
    for character in characters {
        if character.is_ascii_digit() {
            saw_digit = true;
            normalized.push(character);
        } else if character == '.' {
            normalized.push(character);
        } else if !matches!(character, ',' | '_' | ' ' | '\u{00a0}') {
            break;
        }
    }
    if !saw_digit {
        return None;
    }
    normalized.parse::<f64>().ok()
}

fn push_statement(statements: &mut Vec<String>, candidate: &str) {
    let trimmed = candidate.trim();
    if trimmed.is_empty() {
        return;
    }
    let word_count = trimmed.split_whitespace().count();
    let char_count = trimmed
        .chars()
        .filter(|character| !character.is_whitespace())
        .count();
    if word_count < MIN_STATEMENT_WORDS && char_count < MIN_STATEMENT_CHARS {
        return;
    }
    statements.push(trimmed.to_owned());
}

/// Build the web-search query that grounds `statement`: the quoted statement
/// paired with fact-check intent terms so the fusion layer surfaces original
/// first sources for or against it.
#[must_use]
pub fn grounding_query(statement: &str) -> String {
    let condensed = statement.split_whitespace().collect::<Vec<_>>().join(" ");
    format!("\"{condensed}\" fact check source")
}

/// Whether an evidence stance would raise (`Supports`) or lower (`Contradicts`)
/// a statement's probability, exposed for callers that translate grounding
/// results into [`RelativeEvidence`].
#[must_use]
pub const fn stance_for_agreement(agrees: bool) -> Stance {
    if agrees {
        Stance::Supports
    } else {
        Stance::Contradicts
    }
}
