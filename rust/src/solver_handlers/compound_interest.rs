//! Compound-interest finance word problems.
//!
//! The generic calculator can evaluate symbolic expressions once extracted,
//! but prompts such as "invest $1000 at 8% annual interest compounded monthly
//! for 10 years" need slot extraction before there is an arithmetic
//! expression to evaluate.
//!
//! Issue #918: the handler holds no vocabulary, prose or rate. The cue words
//! are the finance and currency meanings of the seed lexicon; the report and
//! conversion wording are the seeded `compound_interest_*` responses; the
//! periods per compounding meaning, the frequency labels and the
//! calculation markers are the `policy compound_interest`
//! block of `data/seed/handler-rules.lino`. The browser twin
//! (`tryCompoundInterest` in `js/worker/formal_ai_worker_translation_recall_and_compound_interest.js`) reads the same
//! records, so both runtimes answer alike.

use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::seed;

use super::calculator_rate::read_quote;
use super::finalize_simple;
use crate::calculation::currency::{currency_code, currency_mentions, stated_quote_date};

const POLICY: &str = "compound_interest";

#[derive(Debug, Clone)]
pub struct CompoundInterestRequest {
    principal: f64,
    principal_currency: String,
    annual_rate_percent: f64,
    compounds_per_year: u32,
    years: f64,
    target_currency: Option<String>,
}

pub fn try_compound_interest(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    try_with_online(
        prompt,
        normalized,
        log,
        crate::solver_config::read_network_allowed(),
    )
}

pub(crate) fn try_with_online(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
    online: bool,
) -> Option<SymbolicAnswer> {
    if let Some(request) = parse_compound_interest_request(prompt, normalized) {
        return Some(answer_compound_interest(prompt, log, request, online));
    }
    if let Some((amount, source_currency, target_currency)) =
        parse_final_amount_conversion_request(prompt, normalized, log)
    {
        return Some(answer_final_amount_conversion(
            prompt,
            log,
            amount,
            &source_currency,
            &target_currency,
            online,
        ));
    }
    None
}

fn policy(key: &str) -> String {
    crate::rule_interpreter::handler_policy(POLICY, key).unwrap_or_default()
}

fn response(intent: &str, values: &[(&str, &str)]) -> String {
    seed::render_response(intent, "en", values).unwrap_or_default()
}

fn answer_compound_interest(
    prompt: &str,
    log: &mut EventLog,
    request: CompoundInterestRequest,
    online: bool,
) -> SymbolicAnswer {
    let principal = request.principal;
    let annual_rate = request.annual_rate_percent / 100.0;
    let compounds = f64::from(request.compounds_per_year);
    let periodic_rate = annual_rate / compounds;
    let periods = compounds * request.years;
    let final_amount = principal * (1.0 + periodic_rate).powf(periods);

    log.append(
        "calculation:compound_interest",
        format!(
            "P={};r={};n={};t={}",
            format_number(principal),
            format_rate(annual_rate),
            request.compounds_per_year,
            format_number(request.years),
        ),
    );
    log.append("calculation:formula", "A=P(1+r/n)^(n*t)");

    let periods_per_year = request.compounds_per_year.to_string();
    let mut body = response(
        "compound_interest_report",
        &[
            ("principal", &format_number(principal)),
            ("principal_currency", &request.principal_currency),
            ("rate", &format_rate(annual_rate)),
            ("percent", &format_number(request.annual_rate_percent)),
            ("periods_per_year", &periods_per_year),
            ("frequency", &compounding_label(request.compounds_per_year)),
            ("years", &format_number(request.years)),
            ("periodic_rate", &format_rate(periodic_rate)),
            ("periods", &format_number(periods)),
            ("final_amount", &format_money(final_amount)),
        ],
    );

    if let Some(target_currency) = request.target_currency {
        append_conversion(
            log,
            &mut body,
            final_amount,
            &request.principal_currency,
            &target_currency,
            stated_quote_date(prompt),
            online,
        );
    }

    log.append("calculation", body.clone());
    finalize_simple(
        prompt,
        log,
        "calculation",
        "response:calculation",
        &body,
        1.0,
    )
}

fn answer_final_amount_conversion(
    prompt: &str,
    log: &mut EventLog,
    amount: f64,
    source_currency: &str,
    target_currency: &str,
    online: bool,
) -> SymbolicAnswer {
    let mut body = response(
        "compound_interest_final_amount_conversion",
        &[
            ("amount", &format_money(amount)),
            ("source", source_currency),
        ],
    );
    append_conversion(
        log,
        &mut body,
        amount,
        source_currency,
        target_currency,
        stated_quote_date(prompt),
        online,
    );
    log.append("calculation", body.clone());
    finalize_simple(
        prompt,
        log,
        "calculation",
        "response:calculation",
        &body,
        1.0,
    )
}

fn append_conversion(
    log: &mut EventLog,
    body: &mut String,
    amount: f64,
    source_currency: &str,
    target_currency: &str,
    date: Option<&str>,
    online: bool,
) {
    let displayed_amount = format_money(round_money(amount));
    let Some((quote, converted)) = read_quote(source_currency, target_currency, date, online, log)
        .and_then(|quote| {
            quote
                .convert(&displayed_amount)
                .map(|result| (quote, result))
        })
    else {
        log.append(
            "calculation:currency_conversion:error",
            format!("{source_currency}->{target_currency}"),
        );
        body.push_str("\n\n");
        body.push_str(&response(
            "compound_interest_rate_unavailable",
            &[("source", source_currency), ("target", target_currency)],
        ));
        return;
    };
    log.append(
        "calculation:currency_conversion",
        format!(
            "{displayed_amount} {source_currency} to {target_currency} at {}",
            quote.rate,
        ),
    );
    let expression = policy("rate-expression")
        .replace("{source}", source_currency)
        .replace("{target}", target_currency);
    body.push_str("\n\n");
    body.push_str(&response(
        "compound_interest_conversion",
        &[
            ("source", source_currency),
            ("target", target_currency),
            ("expression", &expression),
            ("rate", &quote.rate),
            ("amount", &displayed_amount),
            ("converted", &converted),
        ],
    ));
    body.push('\n');
    body.push_str(&response(
        "compound_interest_rate_detail",
        &[
            ("source", source_currency),
            ("target", target_currency),
            ("rate", &quote.rate),
            ("publisher", &quote.source),
            ("date", &quote.date),
        ],
    ));
}

pub fn parse_compound_interest_request(
    prompt: &str,
    normalized: &str,
) -> Option<CompoundInterestRequest> {
    // The investment / interest / compounding cues are language-independent
    // meanings carried by the finance lexicon, matched as raw substrings of the
    // normalized prompt.
    let lexicon = seed::lexicon();
    if !lexicon.mentions_role_raw(seed::ROLE_INVESTMENT_CUE, normalized)
        || !lexicon.mentions_role_raw(seed::ROLE_INTEREST_CUE, normalized)
        || !lexicon.mentions_role_raw(seed::ROLE_COMPOUNDING_ACTION_CUE, normalized)
    {
        return None;
    }
    let (principal, principal_currency) = parse_currency_amount(prompt)?;
    let target_currency = target_currency(prompt, Some(&principal_currency));
    let annual_rate_percent = parse_percent_before_symbol(prompt)?;
    let compounds_per_year = parse_compounds_per_year(normalized)?;
    let years = years_in_prompt(normalized)?;
    Some(CompoundInterestRequest {
        principal,
        principal_currency,
        annual_rate_percent,
        compounds_per_year,
        years,
        target_currency,
    })
}

fn parse_final_amount_conversion_request(
    prompt: &str,
    normalized: &str,
    log: &EventLog,
) -> Option<(f64, String, String)> {
    // "convert" and "final amount" are themselves meanings: a conversion
    // action applied to the final-amount reference produced by a prior turn.
    let lexicon = seed::lexicon();
    if !lexicon.mentions_role_raw(seed::ROLE_CONVERSION_ACTION_CUE, normalized)
        || !lexicon.mentions_role_raw(seed::ROLE_FINAL_AMOUNT_REFERENCE, normalized)
    {
        return None;
    }
    let (amount, source) = prior_final_amount(log)?;
    let target = target_currency(prompt, Some(&source))?;
    Some((amount, source, target))
}

fn prior_final_amount(log: &EventLog) -> Option<(f64, String)> {
    log.events()
        .iter()
        .rev()
        .filter(|event| event.kind == "prior_turn:assistant")
        .find_map(|event| parse_final_amount_from_text(&event.payload))
}

/// The amount and currency an earlier report states after the seeded
/// final-amount marker.
fn parse_final_amount_from_text(text: &str) -> Option<(f64, String)> {
    let marker = policy("final-amount-marker");
    if marker.is_empty() {
        return None;
    }
    let lower = text.to_lowercase();
    let rest = text.get(lower.find(&marker)? + marker.len()..)?;
    let start = rest.len() - rest.trim_start_matches(is_ascii_space).len();
    let digits = rest[start..]
        .find(|ch: char| !is_number_char(ch))
        .map_or(rest.len(), |offset| start + offset);
    let amount = parse_number_slice(&rest[start..digits])?;
    let word: String = rest[digits..]
        .trim_start_matches(is_ascii_space)
        .chars()
        .take_while(|ch| ch.is_alphabetic())
        .collect();
    Some((amount, currency_code(&word)?))
}

fn parse_currency_amount(prompt: &str) -> Option<(f64, String)> {
    currency_mentions(prompt).find_map(|(at, token, code)| {
        let amount = if token.starts_with(char::is_alphabetic) {
            parse_number_left(prompt, at)
        } else {
            parse_number_right(prompt, at + token.len()).or_else(|| parse_number_left(prompt, at))
        };
        Some((amount?, code))
    })
}

fn parse_percent_before_symbol(prompt: &str) -> Option<f64> {
    prompt
        .find('%')
        .and_then(|index| parse_number_left(prompt, index))
}

fn years_in_prompt(normalized: &str) -> Option<f64> {
    // The duration unit is a meaning (year_unit_cue); we read the number to
    // the left of the earliest of its surface forms that has one. An
    // occurrence with no number before it — the `year` inside "compounded
    // yearly" — is a frequency word, not the term, so the scan moves on.
    let mut positions: Vec<usize> = seed::lexicon()
        .words_for_role(seed::ROLE_YEAR_UNIT_CUE)
        .into_iter()
        .flat_map(|word| {
            normalized
                .match_indices(word.as_str())
                .map(|(at, _)| at)
                .collect::<Vec<_>>()
        })
        .collect();
    positions.sort_unstable();
    positions.dedup();
    positions
        .into_iter()
        .find_map(|at| parse_number_left(normalized, at))
}

fn parse_compounds_per_year(normalized: &str) -> Option<u32> {
    // The compounding frequency is a cluster of meanings listed in priority
    // order in the finance lexicon; the first whose surface appears in the
    // prompt names its periods per year in the seeded policy.
    seed::lexicon()
        .meanings_with_role(seed::ROLE_COMPOUNDING_FREQUENCY_CUE)
        .find(|meaning| meaning.words().any(|word| normalized.contains(word)))
        .and_then(|meaning| policy(&format!("periods_{}", meaning.slug)).parse().ok())
}

/// First currency named in the request, excluding the principal currency.
pub fn target_currency(prompt: &str, source: Option<&str>) -> Option<String> {
    currency_mentions(prompt)
        .map(|(_, _, code)| code)
        .find(|code| Some(code.as_str()) != source)
}

fn compounding_label(compounds_per_year: u32) -> String {
    crate::rule_interpreter::handler_policy(POLICY, &format!("label-{compounds_per_year}"))
        .unwrap_or_else(|| policy("label-other"))
}

const fn is_ascii_space(ch: char) -> bool {
    ch.is_ascii_whitespace()
}

const fn is_number_char(ch: char) -> bool {
    ch.is_ascii_digit() || matches!(ch, '.' | ',')
}

fn parse_number_left(text: &str, end: usize) -> Option<f64> {
    let before = text
        .get(..end.min(text.len()))?
        .trim_end_matches(is_ascii_space);
    let start = before
        .rfind(|ch: char| !is_number_char(ch))
        .map_or(0, |at| {
            at + before[at..].chars().next().map_or(1, char::len_utf8)
        });
    parse_number_slice(&before[start..])
}

fn parse_number_right(text: &str, start: usize) -> Option<f64> {
    let after = text
        .get(start.min(text.len())..)?
        .trim_start_matches(is_ascii_space);
    let end = after
        .find(|ch: char| !is_number_char(ch))
        .unwrap_or(after.len());
    parse_number_slice(&after[..end])
}

/// Read a run of digits, dots and commas. A lone comma followed by at most
/// two digits is a decimal comma (`8,5`); any other comma groups thousands.
fn parse_number_slice(value: &str) -> Option<f64> {
    if !value.chars().any(|ch| ch.is_ascii_digit()) {
        return None;
    }
    let cleaned = match value.split_once(',') {
        Some((whole, fraction))
            if !value.contains('.') && !fraction.contains(',') && fraction.len() <= 2 =>
        {
            format!("{whole}.{fraction}")
        }
        _ => value.replace(',', ""),
    };
    cleaned
        .parse::<f64>()
        .ok()
        .filter(|number| number.is_finite())
}

fn format_number(value: f64) -> String {
    if (value.fract()).abs() < 1e-10 {
        format!("{value:.0}")
    } else {
        trim_decimal(&format!("{value:.10}"))
    }
}

fn format_money(value: f64) -> String {
    format!("{value:.2}")
}

fn round_money(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

fn format_rate(value: f64) -> String {
    trim_decimal(&format!("{value:.15}"))
}

fn trim_decimal(value: &str) -> String {
    value.trim_end_matches('0').trim_end_matches('.').to_owned()
}
