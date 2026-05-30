//! Calculator-backed answers about rates used for currency calculations.

use crate::calculation::{evaluate_calculation, CalculationEvaluation};
use crate::engine::{normalize_prompt, SymbolicAnswer};
use crate::event_log::EventLog;
use crate::language::detect as detect_language;

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
        "zh" => format!(
            "货币计算时我使用 link-calculator。USD/RUB 返回: {calculation_body}."
        ),
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

fn asks_for_usd_rate_basis(normalized: &str) -> bool {
    mentions_rate(normalized) && mentions_usd(normalized) && mentions_calculation_basis(normalized)
}

fn mentions_rate(normalized: &str) -> bool {
    normalized.contains("курс")
        || normalized.contains("exchange rate")
        || normalized.contains("currency rate")
        || normalized.contains("विनिमय दर")
        || normalized.contains("汇率")
}

fn mentions_usd(normalized: &str) -> bool {
    normalized.contains("usd")
        || normalized.contains("dollar")
        || normalized.contains("доллар")
        || normalized.contains("долар")
        || normalized.contains("долор")
        || normalized.contains("डॉलर")
        || normalized.contains("美元")
}

fn mentions_calculation_basis(normalized: &str) -> bool {
    normalized.contains("при расчет")
        || normalized.contains("при расчёт")
        || normalized.contains("в расчет")
        || normalized.contains("в расчёт")
        || normalized.contains("для расчет")
        || normalized.contains("для расчёт")
        || normalized.contains("у тебя")
        || normalized.contains("использ")
        || normalized.contains("берешь")
        || normalized.contains("берёшь")
        || normalized.contains("примен")
        || normalized.contains("calculation")
        || normalized.contains("calculations")
        || normalized.contains("do you use")
        || normalized.contains("used for")
        || normalized.contains("your rate")
        || normalized.contains("गणना")
        || normalized.contains("उपयोग")
        || normalized.contains("计算")
        || normalized.contains("使用")
}
