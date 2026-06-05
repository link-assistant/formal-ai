//! Compound-interest finance word problems.
//!
//! The generic calculator can evaluate symbolic expressions once extracted,
//! but prompts such as "invest $1000 at 8% annual interest compounded monthly
//! for 5 years" need domain-specific slot extraction before there is an
//! arithmetic expression to delegate.

use crate::calculation::{evaluate_calculation, CalculationEvaluation};
use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::seed;

use super::finalize_simple;

const USD_EUR_FALLBACK_RATE: f64 = 0.92;

#[derive(Debug, Clone, Copy)]
struct CompoundInterestRequest {
    principal: f64,
    annual_rate_percent: f64,
    compounds_per_year: u32,
    years: f64,
    target_currency: Option<&'static str>,
    asks_for_web_rate: bool,
}

#[derive(Debug, Clone)]
struct CurrencyRate {
    rate: f64,
    expression: String,
    formatted: String,
    source_detail: Option<String>,
}

pub fn try_compound_interest(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    if let Some(request) = parse_compound_interest_request(prompt, normalized) {
        return Some(answer_compound_interest(prompt, log, request));
    }
    if let Some((amount, source_currency, target_currency)) =
        parse_final_amount_conversion_request(normalized, log)
    {
        return Some(answer_final_amount_conversion(
            prompt,
            log,
            amount,
            source_currency,
            target_currency,
            asks_for_web_rate(normalized),
        ));
    }
    None
}

fn answer_compound_interest(
    prompt: &str,
    log: &mut EventLog,
    request: CompoundInterestRequest,
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

    let mut lines = vec![
        String::from("Compound interest calculation"),
        String::new(),
        String::from("Formula: A = P(1 + r/n)^(n*t)"),
        format!("P = {} USD", format_number(principal)),
        format!(
            "r = {} ({}% annual)",
            format_rate(annual_rate),
            format_number(request.annual_rate_percent),
        ),
        format!(
            "n = {} ({})",
            request.compounds_per_year,
            compounding_label(request.compounds_per_year),
        ),
        format!("t = {} years", format_number(request.years)),
        String::new(),
        format!(
            "Step 1: periodic rate = r/n = {}/{} = {}",
            format_rate(annual_rate),
            request.compounds_per_year,
            format_rate(periodic_rate),
        ),
        format!(
            "Step 2: number of periods = n*t = {}*{} = {}",
            request.compounds_per_year,
            format_number(request.years),
            format_number(periods),
        ),
        format!(
            "Step 3: A = {} * (1 + {})^{}",
            format_number(principal),
            format_rate(periodic_rate),
            format_number(periods),
        ),
        format!("Final amount: {} USD", format_money(final_amount)),
    ];

    if let Some(target_currency) = request.target_currency {
        append_conversion_lines(
            log,
            &mut lines,
            final_amount,
            "USD",
            target_currency,
            request.asks_for_web_rate,
        );
    }

    let body = lines.join("\n");
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
    source_currency: &'static str,
    target_currency: &'static str,
    asks_for_web_rate: bool,
) -> SymbolicAnswer {
    let mut lines = vec![
        String::from("Final amount conversion"),
        format!(
            "Source amount: {} {}",
            format_money(amount),
            source_currency
        ),
    ];
    append_conversion_lines(
        log,
        &mut lines,
        amount,
        source_currency,
        target_currency,
        asks_for_web_rate,
    );
    let body = lines.join("\n");
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

fn append_conversion_lines(
    log: &mut EventLog,
    lines: &mut Vec<String>,
    amount: f64,
    source_currency: &'static str,
    target_currency: &'static str,
    asks_for_web_rate: bool,
) {
    if let Some(rate) = currency_rate(source_currency, target_currency, log) {
        let displayed_amount = round_money(amount);
        let converted = displayed_amount * rate.rate;
        log.append(
            "calculation:currency_conversion",
            format!(
                "{} {} to {} at {}",
                format_money(displayed_amount),
                source_currency,
                target_currency,
                format_rate(rate.rate),
            ),
        );
        lines.push(String::new());
        lines.push(format!(
            "Conversion: {source_currency} -> {target_currency}"
        ));
        lines.push(format!("{} = {}", rate.expression, rate.formatted));
        lines.push(format!(
            "{} {} * {} = {} {}",
            format_money(displayed_amount),
            source_currency,
            format_rate(rate.rate),
            format_money(converted),
            target_currency,
        ));
        if let Some(detail) = rate.source_detail {
            lines.push(format!("Rate detail: {detail}"));
        }
        if asks_for_web_rate {
            lines.push(String::from(
                "Live web freshness is not independently verified here; this uses the exchange-rate source available through the local calculator.",
            ));
        }
    } else {
        log.append(
            "calculation:currency_conversion:error",
            format!("{source_currency}->{target_currency}"),
        );
        lines.push(String::new());
        lines.push(format!(
            "I calculated the USD amount, but no {source_currency}->{target_currency} exchange rate is available locally."
        ));
    }
}

fn parse_compound_interest_request(
    prompt: &str,
    normalized: &str,
) -> Option<CompoundInterestRequest> {
    // The investment / interest / compounding cues are language-independent
    // meanings carried by the finance lexicon; we test the raw substring of
    // the already-normalized prompt against every surface form (English
    // forms reproduce the original `invest`/`interest`/`compound` markers,
    // while the additional languages broaden coverage for free).
    let lexicon = seed::lexicon();
    if !lexicon.mentions_role_raw(seed::ROLE_INVESTMENT_CUE, normalized)
        || !lexicon.mentions_role_raw(seed::ROLE_INTEREST_CUE, normalized)
        || !lexicon.mentions_role_raw(seed::ROLE_COMPOUNDING_ACTION_CUE, normalized)
    {
        return None;
    }
    let principal = parse_currency_amount(prompt)?;
    let annual_rate_percent = parse_percent_before_symbol(prompt)?;
    let compounds_per_year = parse_compounds_per_year(normalized)?;
    let years = years_in_prompt(normalized)?;
    Some(CompoundInterestRequest {
        principal,
        annual_rate_percent,
        compounds_per_year,
        years,
        target_currency: target_currency(normalized),
        asks_for_web_rate: asks_for_web_rate(normalized),
    })
}

fn parse_final_amount_conversion_request(
    normalized: &str,
    log: &EventLog,
) -> Option<(f64, &'static str, &'static str)> {
    // "convert" and "final amount" are themselves meanings: a conversion
    // action applied to the final-amount reference produced by a prior turn.
    let lexicon = seed::lexicon();
    if !lexicon.mentions_role_raw(seed::ROLE_CONVERSION_ACTION_CUE, normalized)
        || !lexicon.mentions_role_raw(seed::ROLE_FINAL_AMOUNT_REFERENCE, normalized)
    {
        return None;
    }
    let target = target_currency(normalized)?;
    let (amount, source) = prior_final_amount(log)?;
    Some((amount, source, target))
}

fn prior_final_amount(log: &EventLog) -> Option<(f64, &'static str)> {
    log.events()
        .iter()
        .rev()
        .filter(|event| event.kind == "prior_turn:assistant")
        .find_map(|event| parse_final_amount_from_text(&event.payload))
}

fn parse_final_amount_from_text(text: &str) -> Option<(f64, &'static str)> {
    let lower = text.to_lowercase();
    let marker = lower.find("final amount:")?;
    let after_marker = marker + "final amount:".len();
    let amount_text = &text[after_marker..];
    let (amount, end) = parse_first_number(amount_text)?;
    let currency = currency_after(&amount_text[end..])?;
    Some((amount, currency))
}

fn parse_currency_amount(prompt: &str) -> Option<f64> {
    if let Some(dollar) = prompt.find('$') {
        return parse_number_right(prompt, dollar + '$'.len_utf8());
    }
    // The `$` glyph is a typographic symbol that stays in code; the spelled-out
    // US-dollar markers, however, are language data. We reconstruct each as a
    // space-prefixed token from the currency_usd_reference English surface forms
    // (usd, dollar, dollars) and scan for the amount immediately to their left.
    let lower = prompt.to_lowercase();
    for word in
        seed::lexicon().words_for_role_in_languages(seed::ROLE_CURRENCY_USD_REFERENCE, &["en"])
    {
        let marker = format!(" {word}");
        if let Some(index) = lower.find(&marker) {
            if let Some(amount) = parse_number_left(&lower, index) {
                return Some(amount);
            }
        }
    }
    None
}

fn parse_percent_before_symbol(prompt: &str) -> Option<f64> {
    prompt
        .find('%')
        .and_then(|index| parse_number_left(prompt, index))
}

fn years_in_prompt(normalized: &str) -> Option<f64> {
    // The duration unit is a meaning (year_unit_cue); we locate the earliest
    // of its surface forms (English `year`, plus the other languages) and read
    // the number to its left, reproducing the original `find("year")` scan.
    let earliest = seed::lexicon()
        .words_for_role(seed::ROLE_YEAR_UNIT_CUE)
        .into_iter()
        .filter_map(|word| normalized.find(&word))
        .min()?;
    parse_number_left(normalized, earliest)
}

fn parse_compounds_per_year(normalized: &str) -> Option<u32> {
    // The compounding frequency is a cluster of meanings (monthly, quarterly,
    // weekly, daily, annual), each carrying its surface forms and listed in
    // priority order in the finance lexicon. We pick the first whose surface
    // appears in the prompt and map its slug to the periods-per-year count.
    seed::lexicon()
        .meanings_with_role(seed::ROLE_COMPOUNDING_FREQUENCY_CUE)
        .find(|meaning| meaning.words().any(|word| normalized.contains(word)))
        .and_then(|meaning| compounds_per_year_for_slug(&meaning.slug))
}

fn compounds_per_year_for_slug(slug: &str) -> Option<u32> {
    match slug {
        "compounding_monthly" => Some(12),
        "compounding_quarterly" => Some(4),
        "compounding_weekly" => Some(52),
        "compounding_daily" => Some(365),
        "compounding_annual" => Some(1),
        _ => None,
    }
}

fn target_currency(normalized: &str) -> Option<&'static str> {
    // The euro target is a meaning (currency_eur_reference) matched as a
    // token-bounded word, reproducing the original padded " eur "/" euro "/
    // " euros " test; the `€` glyph is a typographic symbol that stays in code.
    if seed::lexicon().mentions_role(seed::ROLE_CURRENCY_EUR_REFERENCE, normalized)
        || normalized.contains('€')
    {
        Some("EUR")
    } else {
        None
    }
}

fn asks_for_web_rate(normalized: &str) -> bool {
    // "fetch a live rate" is a meaning (live_rate_freshness_cue) whose surface
    // forms (web, current exchange, current rate, exchange rate) are matched as
    // raw substrings, exactly as the original recognizer did.
    seed::lexicon().mentions_role_raw(seed::ROLE_LIVE_RATE_FRESHNESS_CUE, normalized)
}

fn currency_rate(
    source_currency: &'static str,
    target_currency: &'static str,
    log: &mut EventLog,
) -> Option<CurrencyRate> {
    if source_currency == target_currency {
        return Some(CurrencyRate {
            rate: 1.0,
            expression: format!("1 {source_currency} in {target_currency}"),
            formatted: format!("1 {target_currency}"),
            source_detail: None,
        });
    }
    let expression = format!("1 {source_currency} in {target_currency}");
    log.append("calculation:request", expression.clone());
    match evaluate_calculation(&expression) {
        Ok(evaluation) => rate_from_evaluation(expression, evaluation, log),
        Err(error) if source_currency == "USD" && target_currency == "EUR" => {
            log.append("calculation:error", error.to_string());
            Some(CurrencyRate {
                rate: USD_EUR_FALLBACK_RATE,
                expression,
                formatted: format!("{} EUR", format_rate(USD_EUR_FALLBACK_RATE)),
                source_detail: Some(String::from("fallback default rate")),
            })
        }
        Err(error) => {
            log.append("calculation:error", error.to_string());
            None
        }
    }
}

fn rate_from_evaluation(
    expression: String,
    evaluation: CalculationEvaluation,
    log: &mut EventLog,
) -> Option<CurrencyRate> {
    log.append("calculation:engine", evaluation.engine.slug());
    if let Some(lino) = &evaluation.lino {
        log.append("calculation:lino", lino.clone());
    }
    if !evaluation.steps.is_empty() {
        log.append("calculation:steps", evaluation.steps.len().to_string());
    }
    let rate = leading_number(&evaluation.formatted)?;
    let source_detail = rate_source_step(&evaluation).map(str::to_owned);
    Some(CurrencyRate {
        rate,
        expression,
        formatted: evaluation.formatted,
        source_detail,
    })
}

fn rate_source_step(evaluation: &CalculationEvaluation) -> Option<&str> {
    evaluation
        .steps
        .iter()
        .map(String::as_str)
        .find(|step| step.contains("Exchange rate:") || step.contains("exchange rate:"))
}

fn parse_number_left(text: &str, end: usize) -> Option<f64> {
    let bytes = text.as_bytes();
    let mut cursor = end.min(bytes.len());
    while cursor > 0 && bytes[cursor - 1].is_ascii_whitespace() {
        cursor -= 1;
    }
    let number_end = cursor;
    while cursor > 0 && is_number_byte(bytes[cursor - 1]) {
        cursor -= 1;
    }
    parse_number_slice(&text[cursor..number_end])
}

fn parse_number_right(text: &str, start: usize) -> Option<f64> {
    let bytes = text.as_bytes();
    let mut cursor = start.min(bytes.len());
    while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
        cursor += 1;
    }
    let number_start = cursor;
    while cursor < bytes.len() && is_number_byte(bytes[cursor]) {
        cursor += 1;
    }
    parse_number_slice(&text[number_start..cursor])
}

fn parse_first_number(text: &str) -> Option<(f64, usize)> {
    let bytes = text.as_bytes();
    let mut start = 0usize;
    while start < bytes.len() && !bytes[start].is_ascii_digit() {
        start += 1;
    }
    if start == bytes.len() {
        return None;
    }
    let mut end = start;
    while end < bytes.len() && is_number_byte(bytes[end]) {
        end += 1;
    }
    parse_number_slice(&text[start..end]).map(|value| (value, end))
}

fn leading_number(text: &str) -> Option<f64> {
    parse_first_number(text).map(|(value, _)| value)
}

const fn is_number_byte(byte: u8) -> bool {
    byte.is_ascii_digit() || matches!(byte, b'.' | b',')
}

fn parse_number_slice(value: &str) -> Option<f64> {
    let cleaned = value.replace(',', "");
    if cleaned.chars().any(|ch| ch.is_ascii_digit()) {
        cleaned.parse::<f64>().ok()
    } else {
        None
    }
}

fn currency_after(text: &str) -> Option<&'static str> {
    // The currency word that follows a parsed amount is recognised from the
    // currency_usd_reference / currency_eur_reference English surface forms
    // (usd|dollar|dollars, eur|euro|euros); the returned ISO codes stay in code.
    let lower = text.trim_start().to_lowercase();
    let lexicon = seed::lexicon();
    if lexicon
        .words_for_role_in_languages(seed::ROLE_CURRENCY_USD_REFERENCE, &["en"])
        .iter()
        .any(|word| lower.starts_with(word.as_str()))
    {
        Some("USD")
    } else if lexicon
        .words_for_role_in_languages(seed::ROLE_CURRENCY_EUR_REFERENCE, &["en"])
        .iter()
        .any(|word| lower.starts_with(word.as_str()))
    {
        Some("EUR")
    } else {
        None
    }
}

const fn compounding_label(compounds_per_year: u32) -> &'static str {
    match compounds_per_year {
        1 => "annually",
        4 => "quarterly",
        12 => "monthly",
        52 => "weekly",
        365 => "daily",
        _ => "times per year",
    }
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
