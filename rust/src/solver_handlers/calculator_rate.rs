//! Calculator-backed answers about rates used for currency calculations.

use crate::calculation::{CalculationEvaluation, evaluate_calculation};
use crate::engine::{SymbolicAnswer, normalize_prompt};
use crate::event_log::EventLog;
use crate::language::detect as detect_language;
use crate::seed;

use super::finalize_simple;

const USD_RUB_RATE_EXPRESSION: &str = "1 USD in RUB";

pub fn try_calculator_rate_basis(prompt: &str, log: &mut EventLog) -> Option<SymbolicAnswer> {
    let normalized = normalize_prompt(prompt);
    if !asks_for_usd_rate_basis(&normalized) {
        return None;
    }

    log.append("calculation:request", USD_RUB_RATE_EXPRESSION.to_owned());
    match evaluate_calculation(USD_RUB_RATE_EXPRESSION) {
        Ok(evaluation) => Some(rate_basis_answer(prompt, log, &evaluation)),
        Err(error) => {
            let language = detect_language(prompt).slug();
            let error = error.to_string();
            log.append("calculation:error", error.clone());
            let body = match language {
                "ru" => format!(
                    "Я распознал вопрос о курсе USD/RUB для расчетов, но link-calculator не смог его вычислить: {error}."
                ),
                _ => format!(
                    "I recognized this as a question about the USD/RUB rate used for calculations, but link-calculator could not evaluate it: {error}."
                ),
            };
            Some(finalize_simple(
                prompt,
                log,
                "calculation_error",
                "response:calculation_error",
                &body,
                0.3,
            ))
        }
    }
}

fn rate_basis_answer(
    prompt: &str,
    log: &mut EventLog,
    evaluation: &CalculationEvaluation,
) -> SymbolicAnswer {
    log.append("calculation:engine", evaluation.engine.slug());
    if let Some(lino) = &evaluation.lino {
        log.append("calculation:lino", lino.clone());
    }
    if !evaluation.steps.is_empty() {
        log.append("calculation:steps", evaluation.steps.len().to_string());
    }
    log.append("calculation:rate_basis", "USD/RUB".to_owned());

    let calculation_body = format!("{USD_RUB_RATE_EXPRESSION} = {}", evaluation.formatted);
    let language = detect_language(prompt).slug();
    let mut body = match language {
        "ru" => format!(
            "При расчетах валюты я использую link-calculator. Для USD/RUB он возвращает: {calculation_body}."
        ),
        "hi" => format!(
            "मुद्रा गणनाओं के लिए मैं link-calculator का उपयोग करता हूं। USD/RUB के लिए वह लौटाता है: {calculation_body}."
        ),
        "zh" => format!("货币计算时我使用 link-calculator。USD/RUB 返回: {calculation_body}."),
        _ => format!(
            "For currency calculations I use link-calculator. For USD/RUB it returns: {calculation_body}."
        ),
    };
    if let Some(rate_step) = rate_source_step(evaluation) {
        let details = match language {
            "ru" => "Детали курса от калькулятора",
            "hi" => "कैलकुलेटर दर विवरण",
            "zh" => "计算器汇率详情",
            _ => "Calculator rate details",
        };
        body.push_str("\n\n");
        body.push_str(details);
        body.push_str(": ");
        body.push_str(rate_step);
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

fn rate_source_step(evaluation: &CalculationEvaluation) -> Option<&str> {
    evaluation
        .steps
        .iter()
        .map(String::as_str)
        .find(|step| step.contains("Exchange rate:") || step.contains("exchange rate:"))
}

/// Does the prompt ask which exchange rate the assistant uses for currency
/// calculations? Recognition is data-driven: the three concepts each live in
/// `data/seed/meanings-calculator.lino` and are queried by semantic *role*, not
/// by a hardcoded per-language word list.
///
/// The prompt must reference all three concepts at once — an
/// [`exchange_rate`](seed::ROLE_EXCHANGE_RATE_REFERENCE) between currencies, the
/// [`us_dollar`](seed::ROLE_CURRENCY_USD_REFERENCE) currency, and a
/// [`calculation_basis`](seed::ROLE_CALCULATION_BASIS_REFERENCE) phrase asking
/// what the assistant uses. Each role's surface forms are inflectable stems and
/// fixed phrases, so they are matched as raw substrings via
/// [`Lexicon::mentions_role_raw`](seed::Lexicon::mentions_role_raw), byte-for-byte
/// reproducing the original three `contains` disjunctions while keeping every
/// surface word in the seed rather than in code.
pub fn asks_for_usd_rate_basis(normalized: &str) -> bool {
    let lexicon = seed::lexicon();
    lexicon.mentions_role_raw(seed::ROLE_EXCHANGE_RATE_REFERENCE, normalized)
        && lexicon.mentions_role_raw(seed::ROLE_CURRENCY_USD_REFERENCE, normalized)
        && lexicon.mentions_role_raw(seed::ROLE_CALCULATION_BASIS_REFERENCE, normalized)
}

// Captured quote conversion, mirrored by the JavaScript currency quote reader.
// The source table supplies RUB per currency; exact integer ratios round once
// to cents, half away from zero. No binary floating point or network dependency.
fn quote_decimal(text: &str) -> Option<(i128, i128)> {
    let text = text.replace(',', ".");
    let negative = text.starts_with('-');
    let unsigned = text
        .strip_prefix('+')
        .or_else(|| text.strip_prefix('-'))
        .unwrap_or(&text);
    let (whole, fraction) = unsigned.split_once('.').unwrap_or((unsigned, ""));
    if whole.is_empty()
        || whole.len() > 12
        || fraction.len() > 6
        || !whole.bytes().all(|c| c.is_ascii_digit())
        || !fraction.bytes().all(|c| c.is_ascii_digit())
    {
        return None;
    }
    let mantissa = format!("{whole}{fraction}").parse::<i128>().ok()?;
    Some((
        if negative { -mantissa } else { mantissa },
        10_i128.checked_pow(fraction.len().try_into().ok()?)?,
    ))
}

fn quote_currency(word: &str) -> Option<&'static str> {
    let word = word.to_lowercase();
    for (role, code) in [
        (seed::ROLE_CURRENCY_USD_REFERENCE, "USD"),
        (seed::ROLE_CURRENCY_EUR_REFERENCE, "EUR"),
        (seed::ROLE_CURRENCY_RUB_REFERENCE, "RUB"),
    ] {
        if seed::lexicon().words_for_role(role).iter().any(|surface| {
            let cyrillic = surface
                .chars()
                .next()
                .is_some_and(|c| ('\u{0400}'..='\u{04ff}').contains(&c));
            if cyrillic {
                word.starts_with(surface.as_str())
            } else {
                word == *surface
            }
        }) {
            return Some(code);
        }
    }
    None
}

pub(super) fn conversion(
    prompt: &str,
    expression: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    use crate::rule_interpreter::handler_table_value;
    let mut expression = expression.trim();
    let lowered = expression.to_lowercase();
    for cue in seed::lexicon().words_for_role("quantity_conversion_cue") {
        if lowered
            .strip_prefix(cue.as_str())
            .is_some_and(|tail| tail.starts_with(char::is_whitespace))
        {
            expression = expression[cue.len()..].trim_start();
            break;
        }
    }
    let tokens: Vec<&str> = expression.split_whitespace().collect();
    if tokens.len() < 4 {
        return None;
    }
    let connector = tokens[2..tokens.len() - 1].join(" ").to_lowercase();
    if !crate::rule_interpreter::handler_table_rows("currency-conversion-connector")
        .iter()
        .any(|(_, word)| word == &connector)
    {
        return None;
    }
    let from = quote_currency(tokens[1])?;
    let to = quote_currency(tokens[tokens.len() - 1])?;
    let from_rate = handler_table_value("currency-reference-quote", from)?;
    let to_rate = handler_table_value("currency-reference-quote", to)?;
    let (amount, amount_divisor) = quote_decimal(tokens[0])?;
    let (source_rate, source_divisor) = quote_decimal(from_rate)?;
    let (target_rate, target_divisor) = quote_decimal(to_rate)?;
    if source_rate <= 0 || target_rate <= 0 {
        return None;
    }
    let numerator = amount
        .checked_mul(source_rate)?
        .checked_mul(target_divisor)?
        .checked_mul(100)?;
    let denominator = amount_divisor
        .checked_mul(source_divisor)?
        .checked_mul(target_rate)?;
    let absolute = numerator.checked_abs()?;
    let cents =
        absolute / denominator + i128::from(absolute % denominator >= (denominator + 1) / 2);
    let result = format!(
        "{}{}.{:02}",
        if numerator < 0 && cents != 0 { "-" } else { "" },
        cents / 100,
        cents % 100
    );
    let amount_text = tokens[0].replace(',', ".");
    let date = handler_table_value("currency-reference-provenance", "date")?;
    let source = handler_table_value("currency-reference-provenance", "source")?;
    let derivation = format!("{amount_text} × {from_rate} / {to_rate} ≈ {result} {to}");
    let language = detect_language(prompt).slug();
    let mut body = seed::localized_response("currency-conversion-captured", language)?.to_owned();
    for (key, value) in [
        ("amount", amount_text.as_str()),
        ("from", from),
        ("to", to),
        ("fromRate", from_rate),
        ("toRate", to_rate),
        ("result", result.as_str()),
        ("date", date),
        ("source", source),
        ("derivation", derivation.as_str()),
    ] {
        body = body.replace(&format!("{{{key}}}"), value);
    }
    log.append(
        "currency:rate_basis",
        format!("{from}:{from_rate}RUB;{to}:{to_rate}RUB"),
    );
    log.append("currency:source", source);
    log.append("currency:date", date);
    log.append("currency:derivation", derivation);
    log.append("currency:result", format!("{result} {to}"));
    log.append("calculation:engine", "captured-currency");
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
