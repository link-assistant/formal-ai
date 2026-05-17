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
    FormalAiEquationFallback,
}

impl CalculationEngine {
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::LinkCalculator => "link-calculator",
            Self::FormalAiFallback => "formal-ai-fallback",
            Self::FormalAiEquationFallback => "formal-ai-equation-fallback",
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

#[derive(Debug, Clone, Copy, PartialEq)]
struct LinearValue {
    coefficient: f64,
    constant: f64,
}

impl LinearValue {
    const fn constant(value: f64) -> Self {
        Self {
            coefficient: 0.0,
            constant: value,
        }
    }

    const fn variable() -> Self {
        Self {
            coefficient: 1.0,
            constant: 0.0,
        }
    }

    fn add(self, other: Self) -> Self {
        Self {
            coefficient: self.coefficient + other.coefficient,
            constant: self.constant + other.constant,
        }
    }

    fn subtract(self, other: Self) -> Self {
        Self {
            coefficient: self.coefficient - other.coefficient,
            constant: self.constant - other.constant,
        }
    }

    fn negate(self) -> Self {
        Self {
            coefficient: -self.coefficient,
            constant: -self.constant,
        }
    }

    fn multiply(self, other: Self) -> Result<Self, ArithmeticError> {
        if self.has_variable() && other.has_variable() {
            return Err(ArithmeticError::Unparseable);
        }
        if self.has_variable() {
            Ok(Self {
                coefficient: self.coefficient * other.constant,
                constant: self.constant * other.constant,
            })
        } else if other.has_variable() {
            Ok(Self {
                coefficient: other.coefficient * self.constant,
                constant: other.constant * self.constant,
            })
        } else {
            Ok(Self::constant(self.constant * other.constant))
        }
    }

    fn divide(self, other: Self) -> Result<Self, ArithmeticError> {
        if other.has_variable() {
            return Err(ArithmeticError::Unparseable);
        }
        if nearly_zero(other.constant) {
            return Err(ArithmeticError::DivisionByZero);
        }
        Ok(Self {
            coefficient: self.coefficient / other.constant,
            constant: self.constant / other.constant,
        })
    }

    fn has_variable(self) -> bool {
        !nearly_zero(self.coefficient)
    }
}

struct LinearParser<'a> {
    input: &'a str,
    position: usize,
    variable: Option<String>,
}

impl<'a> LinearParser<'a> {
    const fn new(input: &'a str) -> Self {
        Self {
            input,
            position: 0,
            variable: None,
        }
    }

    fn parse(mut self) -> Result<(LinearValue, Option<String>), ArithmeticError> {
        let value = self.parse_expression()?;
        self.skip_whitespace();
        if self.position == self.input.len() {
            Ok((value, self.variable))
        } else {
            Err(ArithmeticError::Unparseable)
        }
    }

    fn parse_expression(&mut self) -> Result<LinearValue, ArithmeticError> {
        let mut value = self.parse_term()?;
        loop {
            self.skip_whitespace();
            if self.consume('+') {
                value = value.add(self.parse_term()?);
            } else if self.consume('-') || self.consume('−') {
                value = value.subtract(self.parse_term()?);
            } else {
                return Ok(value);
            }
        }
    }

    fn parse_term(&mut self) -> Result<LinearValue, ArithmeticError> {
        let mut value = self.parse_factor()?;
        loop {
            self.skip_whitespace();
            if self.consume('*') || self.consume('×') || self.consume('·') {
                value = value.multiply(self.parse_factor()?)?;
            } else if self.consume('/') || self.consume('÷') {
                value = value.divide(self.parse_factor()?)?;
            } else {
                return Ok(value);
            }
        }
    }

    fn parse_factor(&mut self) -> Result<LinearValue, ArithmeticError> {
        self.skip_whitespace();
        if self.consume('+') {
            return self.parse_factor();
        }
        if self.consume('-') || self.consume('−') {
            return Ok(self.parse_factor()?.negate());
        }
        if self.consume('(') {
            let value = self.parse_expression()?;
            self.skip_whitespace();
            if self.consume(')') {
                return Ok(value);
            }
            return Err(ArithmeticError::UnbalancedParens);
        }
        if self
            .peek()
            .is_some_and(|character| character.is_ascii_digit() || character == '.')
        {
            return self.parse_number();
        }
        if self.peek().is_some_and(char::is_alphabetic) {
            return self.parse_variable();
        }
        Err(ArithmeticError::Unparseable)
    }

    fn parse_number(&mut self) -> Result<LinearValue, ArithmeticError> {
        let start = self.position;
        let mut has_digit = false;
        let mut has_dot = false;
        while let Some(character) = self.peek() {
            if character.is_ascii_digit() {
                has_digit = true;
                self.advance(character);
            } else if character == '.' && !has_dot {
                has_dot = true;
                self.advance(character);
            } else {
                break;
            }
        }
        if !has_digit {
            return Err(ArithmeticError::Unparseable);
        }
        self.input[start..self.position]
            .parse::<f64>()
            .map(LinearValue::constant)
            .map_err(|_| ArithmeticError::Unparseable)
    }

    fn parse_variable(&mut self) -> Result<LinearValue, ArithmeticError> {
        let start = self.position;
        while let Some(character) = self.peek() {
            if character.is_alphabetic() || character == '_' {
                self.advance(character);
            } else {
                break;
            }
        }
        let name = self.input[start..self.position].to_owned();
        if name.is_empty() {
            return Err(ArithmeticError::Unparseable);
        }
        if let Some(existing) = &self.variable {
            if existing != &name {
                return Err(ArithmeticError::Unparseable);
            }
        } else {
            self.variable = Some(name);
        }
        Ok(LinearValue::variable())
    }

    fn skip_whitespace(&mut self) {
        while let Some(character) = self.peek() {
            if character.is_whitespace() {
                self.advance(character);
            } else {
                break;
            }
        }
    }

    fn consume(&mut self, expected: char) -> bool {
        if self.peek() == Some(expected) {
            self.advance(expected);
            true
        } else {
            false
        }
    }

    fn peek(&self) -> Option<char> {
        self.input[self.position..].chars().next()
    }

    fn advance(&mut self, character: char) {
        self.position += character.len_utf8();
    }
}

fn nearly_zero(value: f64) -> bool {
    value.abs() < f64::EPSILON
}

fn format_equation_number(value: f64) -> Result<String, ArithmeticError> {
    if !value.is_finite() {
        return Err(ArithmeticError::Overflow);
    }
    if nearly_zero(value) {
        return Ok(String::from("0"));
    }
    if value.fract().abs() < 1e-10 {
        return Ok(format!("{value:.0}"));
    }
    let rendered = format!("{value:.10}");
    Ok(rendered
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_owned())
}

fn evaluate_linear_equation(expression: &str) -> Result<CalculationEvaluation, ArithmeticError> {
    let mut parts = expression.split('=');
    let left = parts.next().ok_or(ArithmeticError::Unparseable)?;
    let right = parts.next().ok_or(ArithmeticError::Unparseable)?;
    if parts.next().is_some() {
        return Err(ArithmeticError::Unparseable);
    }
    let (left_value, left_variable) = LinearParser::new(left).parse()?;
    let (right_value, right_variable) = LinearParser::new(right).parse()?;
    let variable = match (left_variable, right_variable) {
        (Some(left), Some(right)) if left == right => left,
        (Some(left), None) => left,
        (None, Some(right)) => right,
        _ => return Err(ArithmeticError::Unparseable),
    };
    let coefficient = left_value.coefficient - right_value.coefficient;
    if nearly_zero(coefficient) {
        return Err(ArithmeticError::Unparseable);
    }
    let value = (right_value.constant - left_value.constant) / coefficient;
    Ok(CalculationEvaluation {
        formatted: format!("{variable} = {}", format_equation_number(value)?),
        engine: CalculationEngine::FormalAiEquationFallback,
        lino: None,
        steps: Vec::new(),
    })
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
        " плюс ",
        " минус ",
        " умножить ",
        " умножь ",
        " умножить на ",
        " разделить на ",
        " делить на ",
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
    if expression.contains('=') {
        if let Ok(evaluation) = evaluate_linear_equation(expression) {
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
    let has_spelled_arithmetic = contains_spelled_arithmetic(expression);
    if !has_digit && !has_spelled_arithmetic {
        return false;
    }
    let has_letter = expression.chars().any(char::is_alphabetic);
    let has_strong_symbol = expression.contains([
        '+', '*', '/', '%', '^', '=', '×', '·', '÷', '−', '$', '€', '¥', '₹', '₽',
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

fn contains_spelled_arithmetic(expression: &str) -> bool {
    let lower = format!(" {} ", expression.to_lowercase());
    let has_number_word = [
        " zero ",
        " one ",
        " two ",
        " three ",
        " four ",
        " five ",
        " six ",
        " seven ",
        " eight ",
        " nine ",
        " ten ",
        " ноль ",
        " нуль ",
        " один ",
        " одна ",
        " одно ",
        " два ",
        " две ",
        " три ",
        " четыре ",
        " пять ",
        " шесть ",
        " семь ",
        " восемь ",
        " девять ",
        " десять ",
    ]
    .iter()
    .any(|number| lower.contains(number));
    has_number_word && contains_word_operator(expression)
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
