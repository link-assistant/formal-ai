//! Parse and evaluate the existing single-variable linear equation fallback.

use super::{ArithmeticError, CalculationEngine, CalculationEvaluation};

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
        if self
            .peek()
            .is_some_and(|character| character.is_alphabetic() || is_unknown_placeholder(character))
        {
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
        if let Some(character) = self.peek() {
            if is_unknown_placeholder(character) {
                self.advance(character);
            } else {
                while let Some(character) = self.peek() {
                    if character.is_alphabetic() || character == '_' {
                        self.advance(character);
                    } else {
                        break;
                    }
                }
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

    const fn advance(&mut self, character: char) {
        self.position += character.len_utf8();
    }
}

fn nearly_zero(value: f64) -> bool {
    value.abs() < f64::EPSILON
}

const fn is_unknown_placeholder(character: char) -> bool {
    matches!(character, '?' | '*')
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

pub(super) fn evaluate_linear_equation(
    expression: &str,
) -> Result<CalculationEvaluation, ArithmeticError> {
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
