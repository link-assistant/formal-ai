//! Calculator delegation boundary for the universal solver.
//!
//! This module keeps natural-language prompt processing in formal-ai, delegates
//! calculator-shaped expressions to `link-calculator`, and preserves the local
//! arithmetic evaluator for syntax the upstream crate does not support yet.

use crate::arithmetic::{evaluate_fallback_formatted, ArithmeticError};

/// Engine that produced a calculation result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CalculationEngine {
    LinkCalculator,
    FormalAiFallback,
}

impl CalculationEngine {
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::LinkCalculator => "link-calculator",
            Self::FormalAiFallback => "formal-ai-fallback",
        }
    }
}

/// Structured calculation result used by the solver trace.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CalculationEvaluation {
    pub formatted: String,
    pub engine: CalculationEngine,
    pub lino: Option<String>,
    pub steps: Vec<String>,
}

/// A candidate calculator expression extracted from a user prompt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CalculationCandidate {
    pub expression: String,
    pub explicit: bool,
}

fn evaluate_with_link_calculator(
    expression: &str,
) -> Result<CalculationEvaluation, ArithmeticError> {
    let mut calculator = link_calculator::Calculator::new();
    let (_expression, value, steps, lino) = calculator
        .calculate_with_value(expression)
        .map_err(|error| ArithmeticError::Calculator(error.to_string()))?;
    Ok(CalculationEvaluation {
        formatted: value.to_display_string(),
        engine: CalculationEngine::LinkCalculator,
        lino: Some(lino),
        steps,
    })
}

fn should_use_fallback_before_calculator(expression: &str) -> bool {
    contains_word_operator(expression) || contains_binary_percent_remainder(expression)
}

fn contains_word_operator(expression: &str) -> bool {
    let lower = format!(" {} ", expression.to_lowercase());
    [
        " plus ",
        " minus ",
        " times ",
        " multiplied by ",
        " divided by ",
        " modulo ",
        " mod ",
    ]
    .iter()
    .any(|operator| lower.contains(operator))
}

fn contains_binary_percent_remainder(expression: &str) -> bool {
    let mut chars = expression.char_indices();
    while let Some((_, character)) = chars.next() {
        if character != '%' {
            continue;
        }
        let after = chars
            .clone()
            .map(|(_, c)| c)
            .collect::<String>()
            .trim_start()
            .to_lowercase();
        if after.starts_with("of") {
            continue;
        }
        if after.starts_with('*')
            || after.starts_with('/')
            || after.starts_with('+')
            || after.starts_with('-')
            || after.is_empty()
        {
            continue;
        }
        if after
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_digit() || c == '(')
        {
            return true;
        }
    }
    false
}

/// Evaluate an expression, delegating calculator-supported syntax to
/// `link-calculator` and preserving the in-repo evaluator as a fallback for
/// syntax the upstream crate does not support yet.
pub fn evaluate_calculation(expression: &str) -> Result<CalculationEvaluation, ArithmeticError> {
    if !should_use_fallback_before_calculator(expression) {
        if let Ok(evaluation) = evaluate_with_link_calculator(expression) {
            return Ok(evaluation);
        }
    }
    let formatted = evaluate_fallback_formatted(expression)?;
    Ok(CalculationEvaluation {
        formatted,
        engine: CalculationEngine::FormalAiFallback,
        lino: None,
        steps: Vec::new(),
    })
}

fn trim_prompt_punctuation(value: &str) -> &str {
    value
        .trim()
        .trim_matches(|c| matches!(c, '?' | '!' | '。' | '？' | '！'))
        .trim()
        .trim_end_matches('.')
        .trim()
}

fn strip_prefix_case_insensitive<'a>(value: &'a str, prefix: &str) -> Option<&'a str> {
    let lower = value.to_lowercase();
    lower
        .starts_with(prefix)
        .then(|| value[prefix.len()..].trim_start())
}

fn strip_suffix_case_insensitive<'a>(value: &'a str, suffix: &str) -> Option<&'a str> {
    let lower = value.to_lowercase();
    lower
        .ends_with(suffix)
        .then(|| value[..value.len() - suffix.len()].trim_end())
}

fn strip_calculation_wrappers(prompt: &str) -> (String, bool) {
    let prefixes = [
        "please calculate ",
        "please compute ",
        "can you calculate ",
        "can you compute ",
        "could you calculate ",
        "could you compute ",
        "what is ",
        "what's ",
        "what does ",
        "calculate ",
        "compute ",
        "evaluate ",
        "how much is ",
        "solve ",
        "сколько будет ",
        "посчитай ",
        "посчитайте ",
        "вычисли ",
        "вычислите ",
        "рассчитай ",
        "рассчитайте ",
        "请计算",
        "请算一下",
        "计算一下",
        "算一下",
        "计算",
        "कृपया गणना करें ",
        "गणना करें ",
    ];
    let suffixes = [
        " equal",
        " equals",
        " =",
        "=",
        " please",
        " for me",
        " пожалуйста",
        "是多少",
        "等于多少",
        "等于几",
        "कितना है",
        "क्या है",
        "की गणना करें",
    ];

    let mut working = trim_prompt_punctuation(prompt).to_owned();
    let mut explicit = false;
    loop {
        let mut changed = false;
        for prefix in prefixes {
            if let Some(stripped) = strip_prefix_case_insensitive(&working, prefix) {
                working = stripped.to_owned();
                explicit = true;
                changed = true;
                break;
            }
        }
        if !changed {
            break;
        }
    }
    loop {
        working = trim_prompt_punctuation(&working).to_owned();
        let mut changed = false;
        for suffix in suffixes {
            if let Some(stripped) = strip_suffix_case_insensitive(&working, suffix) {
                working = stripped.to_owned();
                explicit = true;
                changed = true;
                break;
            }
        }
        if !changed {
            break;
        }
    }
    (working.trim().to_owned(), explicit)
}

fn has_calculation_signal(expression: &str, explicit: bool) -> bool {
    let lower = format!(" {} ", expression.to_lowercase());
    let has_digit = expression.chars().any(|c| c.is_ascii_digit());
    if !has_digit {
        return false;
    }
    let has_letter = expression.chars().any(char::is_alphabetic);
    let has_strong_symbol = expression.contains([
        '+', '*', '/', '%', '^', '×', '·', '÷', '−', '$', '€', '¥', '₹', '₽',
    ]) || (!has_letter && expression.contains('-'));
    if has_strong_symbol || contains_word_operator(expression) {
        return true;
    }
    let has_known_calculator_word = [
        " sqrt",
        " sin",
        " cos",
        " tan",
        " log",
        " ln",
        " usd ",
        " eur ",
        " rub ",
        " dollars",
        " dollar",
        " euros",
        " euro",
        " rubles",
        " ruble",
        " kg ",
        " kb ",
        " mb ",
        " ms ",
        " seconds",
        " second",
        " minutes",
        " minute",
        " hours",
        " hour",
        " days",
        " day",
        " grams",
        " gram",
        " months",
        " month",
        " tons",
        " ton",
        "руб",
        "доллар",
        "евро",
        "тонн",
        "кг",
        "феврал",
        "январ",
        "месяц",
        "месяцев",
        "день",
        "дней",
        "换成",
        "兑换成",
        "转换为",
        "美元",
        "欧元",
        "公斤",
        "二月",
        "一月",
        "个月",
        "天",
        "ग्राम",
        "किलोग्राम",
        "डॉलर",
        "यूरो",
        "फरवरी",
        "जनवरी",
        "महीने",
        "दिन",
    ]
    .iter()
    .any(|signal| lower.contains(signal));
    if has_known_calculator_word {
        return true;
    }
    explicit && !has_letter
}

/// Pull calculation expressions out of a natural-language prompt. Returns an
/// empty vector when the prompt does not look like a calculator request, so
/// the solver can fall through to other specialized handlers.
#[must_use]
pub fn calculation_expression_candidates(prompt: &str) -> Vec<CalculationCandidate> {
    let trimmed = trim_prompt_punctuation(prompt);
    if trimmed.is_empty() {
        return Vec::new();
    }
    let (stripped, explicit) = strip_calculation_wrappers(trimmed);
    let mut candidates = Vec::new();
    if !stripped.is_empty() && has_calculation_signal(&stripped, explicit) {
        candidates.push(CalculationCandidate {
            expression: stripped,
            explicit,
        });
    }
    if trimmed
        != candidates
            .first()
            .map(|candidate| candidate.expression.as_str())
            .unwrap_or_default()
        && has_calculation_signal(trimmed, false)
    {
        candidates.push(CalculationCandidate {
            expression: trimmed.to_owned(),
            explicit: false,
        });
    }
    candidates
}
