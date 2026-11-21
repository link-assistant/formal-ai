//! Dated publisher quotes shared by currency conversion and finance handlers.

use crate::calculation::currency::{
    conversion_operands, currency_mentions, decimal, stated_quote_date,
};
use crate::engine::{SymbolicAnswer, normalize_prompt};
use crate::event_log::EventLog;
use crate::language::detect as detect_language;
use crate::rule_interpreter::handler_table_value;
use crate::seed;
use crate::source_fetch::{CachedSourceClient, CurlSourceTransport};

use super::calendar::{CalendarDate, current_utc_date};
use super::finalize_simple;

const MAX_QUOTE_AGE_DAYS: i64 = 7;

pub fn asks_for_rate_basis(normalized: &str) -> bool {
    let lexicon = seed::lexicon();
    lexicon.mentions_role_raw(seed::ROLE_EXCHANGE_RATE_REFERENCE, normalized)
        && lexicon.mentions_role_raw(seed::ROLE_CALCULATION_BASIS_REFERENCE, normalized)
}

pub(super) fn try_calculator_rate_basis(
    prompt: &str,
    log: &mut EventLog,
    online: bool,
) -> Option<SymbolicAnswer> {
    if !asks_for_rate_basis(&normalize_prompt(prompt)) {
        return None;
    }
    let mut currencies = currency_mentions(prompt).map(|(_, _, code)| code);
    let pair = currencies.next().and_then(|from| {
        let to = currencies.find(|candidate| candidate != &from)?;
        Some((from, to))
    });
    let Some((from, to)) = pair else {
        let body = seed::render_response(
            "currency-quote-pair-needed",
            detect_language(prompt).slug(),
            &[],
        )?;
        return Some(finalize_simple(
            prompt,
            log,
            "clarification",
            "response:clarification",
            &body,
            0.8,
        ));
    };
    answer_quote(
        prompt,
        "1",
        &from,
        &to,
        stated_quote_date(prompt),
        online,
        log,
    )
}

pub(super) struct CurrencyQuote {
    pub rate: String,
    pub date: String,
    pub source: String,
}

impl CurrencyQuote {
    pub(super) fn convert(&self, amount: &str) -> Option<String> {
        let (amount, amount_divisor) = decimal(amount, 6)?;
        let (rate, rate_divisor) = decimal(&self.rate, 12)?;
        let numerator = amount.checked_mul(rate)?.checked_mul(100)?;
        let denominator = amount_divisor.checked_mul(rate_divisor)?;
        let absolute = numerator.checked_abs()?;
        let cents =
            absolute / denominator + i128::from(absolute % denominator >= (denominator + 1) / 2);
        let sign = if numerator < 0 && cents != 0 { "-" } else { "" };
        Some(format!("{sign}{}.{:02}", cents / 100, cents % 100))
    }
}

#[derive(serde::Deserialize)]
struct PublisherQuote {
    base: String,
    quote: String,
    date: String,
    rate: serde_json::Number,
}

pub(super) fn read_quote(
    from: &str,
    to: &str,
    stated_date: Option<&str>,
    online: bool,
    log: &mut EventLog,
) -> Option<CurrencyQuote> {
    let today = current_utc_date()?;
    let requested = match stated_date {
        Some(date) => CalendarDate::from_iso(date)?,
        None => today,
    };
    if requested.days_since_unix_epoch > today.days_since_unix_epoch {
        return None;
    }
    if from == to {
        return Some(CurrencyQuote {
            rate: "1".to_owned(),
            date: String::new(),
            source: String::new(),
        });
    }
    let provider = handler_table_value("currency-quote-provider", from)
        .or_else(|| handler_table_value("currency-quote-provider", to))
        .or_else(|| handler_table_value("currency-quote-provider", "default"))?;
    let url = format!(
        "{provider}/{}/{}?date={}",
        from.to_ascii_lowercase(),
        to.to_ascii_lowercase(),
        requested.iso()
    );
    log.append("currency:request", url.clone());
    let client = CachedSourceClient::new(
        crate::coding::synthesis_runtime::source_cache_root(),
        CurlSourceTransport,
    )
    .with_ttl_seconds(3_600)
    .with_online(online && crate::solver_config::read_network_allowed());
    let capture = match client.fetch(&url) {
        Ok(capture) => capture,
        Err(error) => {
            log.append("currency:source_error", error.to_string());
            return None;
        }
    };
    let (quote, rate) = serde_json::from_slice::<PublisherQuote>(capture.bytes())
        .ok()
        .and_then(|quote| {
            let effective = CalendarDate::from_iso(&quote.date)?;
            let age = requested.days_since_unix_epoch - effective.days_since_unix_epoch;
            // Use fixed notation when the JSON parser displays a small decimal in scientific notation.
            let mut rate = quote.rate.to_string();
            if rate.contains(['e', 'E']) {
                rate = format!("{:.12}", quote.rate.as_f64()?)
                    .trim_end_matches('0')
                    .to_owned();
            }
            let (numerator, _) = decimal(&rate, 12)?;
            (quote.base == from
                && quote.quote == to
                && (0..=MAX_QUOTE_AGE_DAYS).contains(&age)
                && numerator > 0)
                .then_some((quote, rate))
        })
        .or_else(|| {
            log.append(
                "currency:source_error",
                "publisher quote failed pair, date or rate validation",
            );
            None
        })?;
    capture.record(log);
    log.append("currency:source", url.clone());
    log.append("currency:date", quote.date.clone());
    Some(CurrencyQuote {
        rate,
        date: quote.date,
        source: url,
    })
}

/// Install verified quotes into the expression evaluator only when its parsed
/// currency arithmetic cannot run with an empty rate database.
pub(super) fn supply_expression_quotes(
    calculator: &mut link_calculator::Calculator,
    currencies: &[String],
    expression: &str,
    online: bool,
    log: &mut EventLog,
) {
    const MAX_CURRENCIES_PER_EXPRESSION: usize = 4;
    let [base, targets @ ..] = currencies else {
        return;
    };
    if currencies.len() > MAX_CURRENCIES_PER_EXPRESSION {
        return;
    }
    let date = stated_quote_date(expression);
    for target in targets {
        let Some(quote) = read_quote(base, target, date, online, log) else {
            continue;
        };
        let rates = format!(r#"{{"{target}":{}}}"#, quote.rate);
        calculator.update_rates_from_api(base, date.unwrap_or(&quote.date), &rates);
    }
}

pub(super) fn conversion(
    prompt: &str,
    expression: &str,
    log: &mut EventLog,
    online: bool,
) -> Option<SymbolicAnswer> {
    let request = conversion_operands(expression)?;
    answer_quote(
        prompt,
        &request.amount,
        &request.from,
        &request.to,
        request.date,
        online,
        log,
    )
}

fn answer_quote(
    prompt: &str,
    amount: &str,
    from: &str,
    to: &str,
    date: Option<&str>,
    online: bool,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let language = detect_language(prompt).slug();
    let pair = [("from", from), ("to", to)];
    let Some((quote, result)) = read_quote(from, to, date, online, log)
        .and_then(|quote| quote.convert(amount).map(|result| (quote, result)))
    else {
        let body = seed::render_response("currency-quote-unavailable", language, &pair)?;
        return Some(finalize_simple(
            prompt,
            log,
            "calculation_error",
            "response:calculation_error",
            &body,
            0.3,
        ));
    };
    let derivation = format!("{amount} × {} ≈ {result} {to}", quote.rate);
    let intent = if from == to {
        "currency-quote-identity"
    } else {
        "currency-conversion-publisher"
    };
    let body = seed::render_response(
        intent,
        language,
        &[
            ("amount", amount),
            ("from", from),
            ("to", to),
            ("rate", &quote.rate),
            ("result", &result),
            ("date", &quote.date),
            ("source", &quote.source),
            ("derivation", &derivation),
        ],
    )?;
    log.append("currency:rate_basis", format!("{from}:{}{to}", quote.rate));
    log.append("currency:derivation", derivation);
    log.append("currency:result", format!("{result} {to}"));
    log.append("calculation:engine", "evidenced-currency");
    log.append("calculation", body.clone());
    Some(finalize_simple(
        prompt,
        log,
        "calculation",
        "response:calculation",
        &body,
        1.0,
    ))
}
