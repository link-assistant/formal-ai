//! Precedence-climbing arithmetic evaluator used by the universal solver's
//! `try_arithmetic` handler. The evaluator is pure (no I/O, no allocations
//! beyond the token vector) and shared across every interface so "what is
//! 2 + 2?" produces the same trace from CLI, HTTP, Telegram and the demo.
//!
//! When every literal in the expression is an integer (no decimal point),
//! the evaluator uses arbitrary-precision integer arithmetic so results like
//! `123123980921093128 * 2348023048230429324` are exact rather than
//! overflowing to `inf`. Expressions that mix integers and decimals (e.g.
//! `1.5 + 2`) fall back to `f64`.

/// Errors produced when a calculation evaluator cannot produce a numeric value
/// for the requested expression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArithmeticError {
    Empty,
    Unparseable,
    DivisionByZero,
    Overflow,
    UnbalancedParens,
    Calculator(String),
}

impl std::fmt::Display for ArithmeticError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Empty => "no expression provided",
            Self::Unparseable => "expression could not be parsed",
            Self::DivisionByZero => "division by zero",
            Self::Overflow => "numeric overflow",
            Self::UnbalancedParens => "unbalanced parentheses",
            Self::Calculator(error) => error,
        })
    }
}

// ---------------------------------------------------------------------------
// Arbitrary-precision non-negative integer (base 10^9, little-endian limbs).
// ---------------------------------------------------------------------------

const BASE: u64 = 1_000_000_000;

/// A non-negative arbitrary-precision integer stored as base-10^9 limbs in
/// little-endian order (least-significant limb first).
#[derive(Debug, Clone, PartialEq, Eq)]
struct BigUint(Vec<u64>);

impl BigUint {
    fn zero() -> Self {
        Self(vec![0])
    }

    fn from_u64(value: u64) -> Self {
        if value < BASE {
            Self(vec![value])
        } else {
            Self(vec![value % BASE, value / BASE])
        }
    }

    fn is_zero(&self) -> bool {
        self.0.iter().all(|&limb| limb == 0)
    }

    fn add(&self, other: &Self) -> Self {
        let len = self.0.len().max(other.0.len());
        let mut result = Vec::with_capacity(len + 1);
        let mut carry: u64 = 0;
        for i in 0..len {
            let a = if i < self.0.len() { self.0[i] } else { 0 };
            let b = if i < other.0.len() { other.0[i] } else { 0 };
            let sum = a + b + carry;
            result.push(sum % BASE);
            carry = sum / BASE;
        }
        if carry > 0 {
            result.push(carry);
        }
        Self(result)
    }

    fn mul(&self, other: &Self) -> Self {
        let mut result = vec![0u64; self.0.len() + other.0.len()];
        for (i, &a) in self.0.iter().enumerate() {
            let mut carry: u64 = 0;
            for (j, &b) in other.0.iter().enumerate() {
                let prod =
                    u128::from(a) * u128::from(b) + u128::from(result[i + j]) + u128::from(carry);
                // Remainder and quotient both fit in u64: BASE = 10^9 < 2^30.
                #[allow(clippy::cast_possible_truncation)]
                {
                    result[i + j] = (prod % u128::from(BASE)) as u64;
                    carry = (prod / u128::from(BASE)) as u64;
                }
            }
            if carry > 0 {
                result[i + other.0.len()] += carry;
            }
        }
        while result.len() > 1 && *result.last().unwrap() == 0 {
            result.pop();
        }
        Self(result)
    }

    fn to_decimal_string(&self) -> String {
        if self.is_zero() {
            return String::from("0");
        }
        let mut parts = Vec::with_capacity(self.0.len());
        let last = self.0.len() - 1;
        for (i, &limb) in self.0.iter().enumerate().rev() {
            if i == last {
                parts.push(format!("{limb}"));
            } else {
                parts.push(format!("{limb:09}"));
            }
        }
        parts.concat()
    }

    #[allow(clippy::cast_precision_loss)]
    fn to_f64(&self) -> f64 {
        let base_f = BASE as f64;
        let mut result = 0.0_f64;
        for &limb in self.0.iter().rev() {
            result = result.mul_add(base_f, limb as f64);
        }
        result
    }
}

// ---------------------------------------------------------------------------
// Value type: either an exact big integer or a floating-point approximation.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
enum ArithValue {
    Integer { negative: bool, magnitude: BigUint },
    Float(f64),
}

impl ArithValue {
    const fn from_f64(value: f64) -> Self {
        Self::Float(value)
    }

    fn negate(self) -> Self {
        match self {
            Self::Integer {
                negative,
                magnitude,
            } => {
                if magnitude.is_zero() {
                    Self::Integer {
                        negative: false,
                        magnitude,
                    }
                } else {
                    Self::Integer {
                        negative: !negative,
                        magnitude,
                    }
                }
            }
            Self::Float(f) => Self::Float(-f),
        }
    }

    fn to_f64(&self) -> f64 {
        match self {
            Self::Integer {
                negative,
                magnitude,
            } => {
                let f = magnitude.to_f64();
                if *negative {
                    -f
                } else {
                    f
                }
            }
            Self::Float(f) => *f,
        }
    }
}

// ---------------------------------------------------------------------------
// Arithmetic operations on ArithValue.
// ---------------------------------------------------------------------------

fn arith_add(left: ArithValue, right: ArithValue) -> Result<ArithValue, ArithmeticError> {
    match (left, right) {
        (
            ArithValue::Integer {
                negative: neg_l,
                magnitude: mag_l,
            },
            ArithValue::Integer {
                negative: neg_r,
                magnitude: mag_r,
            },
        ) => {
            if neg_l == neg_r {
                Ok(ArithValue::Integer {
                    negative: neg_l,
                    magnitude: mag_l.add(&mag_r),
                })
            } else {
                // Different signs: result = |larger| - |smaller|, sign from larger.
                let (larger, smaller, result_neg) = if big_gte(&mag_l, &mag_r) {
                    (mag_l, mag_r, neg_l)
                } else {
                    (mag_r, mag_l, neg_r)
                };
                let magnitude = big_sub(&larger, &smaller);
                Ok(ArithValue::Integer {
                    negative: if magnitude.is_zero() {
                        false
                    } else {
                        result_neg
                    },
                    magnitude,
                })
            }
        }
        (l, r) => {
            let result = l.to_f64() + r.to_f64();
            if result.is_finite() {
                Ok(ArithValue::Float(result))
            } else {
                Err(ArithmeticError::Overflow)
            }
        }
    }
}

fn arith_sub(left: ArithValue, right: ArithValue) -> Result<ArithValue, ArithmeticError> {
    arith_add(left, right.negate())
}

fn arith_mul(left: ArithValue, right: ArithValue) -> Result<ArithValue, ArithmeticError> {
    match (left, right) {
        (
            ArithValue::Integer {
                negative: neg_l,
                magnitude: mag_l,
            },
            ArithValue::Integer {
                negative: neg_r,
                magnitude: mag_r,
            },
        ) => Ok(ArithValue::Integer {
            negative: neg_l != neg_r && !(mag_l.is_zero() || mag_r.is_zero()),
            magnitude: mag_l.mul(&mag_r),
        }),
        (l, r) => {
            let result = l.to_f64() * r.to_f64();
            if result.is_finite() {
                Ok(ArithValue::Float(result))
            } else {
                Err(ArithmeticError::Overflow)
            }
        }
    }
}

fn arith_div(left: &ArithValue, right: &ArithValue) -> Result<ArithValue, ArithmeticError> {
    if right.to_f64() == 0.0 {
        return Err(ArithmeticError::DivisionByZero);
    }
    let result = left.to_f64() / right.to_f64();
    if result.is_finite() {
        Ok(ArithValue::Float(result))
    } else {
        Err(ArithmeticError::Overflow)
    }
}

fn arith_rem(left: &ArithValue, right: &ArithValue) -> Result<ArithValue, ArithmeticError> {
    if right.to_f64() == 0.0 {
        return Err(ArithmeticError::DivisionByZero);
    }
    let result = left.to_f64() % right.to_f64();
    if result.is_finite() {
        Ok(ArithValue::Float(result))
    } else {
        Err(ArithmeticError::Overflow)
    }
}

/// Returns true if `a >= b`.
fn big_gte(a: &BigUint, b: &BigUint) -> bool {
    if a.0.len() != b.0.len() {
        return a.0.len() > b.0.len();
    }
    for (al, bl) in a.0.iter().rev().zip(b.0.iter().rev()) {
        if al != bl {
            return al > bl;
        }
    }
    true // equal
}

/// Subtract `smaller` from `larger` (assumes `larger >= smaller`).
fn big_sub(larger: &BigUint, smaller: &BigUint) -> BigUint {
    let mut result = larger.0.clone();
    let mut borrow: u64 = 0;
    for (i, limb) in result.iter_mut().enumerate() {
        let s = if i < smaller.0.len() { smaller.0[i] } else { 0 };
        if *limb >= s + borrow {
            *limb -= s + borrow;
            borrow = 0;
        } else {
            *limb = *limb + BASE - s - borrow;
            borrow = 1;
        }
    }
    while result.len() > 1 && *result.last().unwrap() == 0 {
        result.pop();
    }
    BigUint(result)
}

// ---------------------------------------------------------------------------
// Tokenizer.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
enum ArithmeticToken {
    Number(ArithValue),
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    LParen,
    RParen,
}

fn tokenize_arithmetic(input: &str) -> Result<Vec<ArithmeticToken>, ArithmeticError> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();
    while let Some(&character) = chars.peek() {
        if character.is_whitespace() {
            chars.next();
            continue;
        }
        match character {
            '+' => {
                chars.next();
                tokens.push(ArithmeticToken::Plus);
            }
            '-' | '−' => {
                chars.next();
                tokens.push(ArithmeticToken::Minus);
            }
            '*' | '×' | '·' => {
                chars.next();
                tokens.push(ArithmeticToken::Star);
            }
            '/' | '÷' => {
                chars.next();
                tokens.push(ArithmeticToken::Slash);
            }
            '%' => {
                chars.next();
                tokens.push(ArithmeticToken::Percent);
            }
            '(' => {
                chars.next();
                tokens.push(ArithmeticToken::LParen);
            }
            ')' => {
                chars.next();
                tokens.push(ArithmeticToken::RParen);
            }
            digit if digit.is_ascii_digit() || digit == '.' => {
                let mut number = String::new();
                let mut has_dot = false;
                while let Some(&next) = chars.peek() {
                    if next.is_ascii_digit() {
                        number.push(next);
                        chars.next();
                    } else if next == '.' && !has_dot {
                        has_dot = true;
                        number.push(next);
                        chars.next();
                    } else if next == '_' {
                        chars.next();
                    } else {
                        break;
                    }
                }
                let value = if has_dot {
                    let parsed: f64 = number.parse().map_err(|_| ArithmeticError::Unparseable)?;
                    ArithValue::from_f64(parsed)
                } else {
                    parse_integer_literal(&number)?
                };
                tokens.push(ArithmeticToken::Number(value));
            }
            _ => return Err(ArithmeticError::Unparseable),
        }
    }
    Ok(tokens)
}

/// Parse a decimal integer string into a `BigUint`-backed `ArithValue`.
fn parse_integer_literal(s: &str) -> Result<ArithValue, ArithmeticError> {
    if s.is_empty() {
        return Err(ArithmeticError::Unparseable);
    }
    // Build BigUint by repeated multiply-by-10 + add-digit.
    let mut magnitude = BigUint::zero();
    let ten = BigUint::from_u64(10);
    for ch in s.chars() {
        let digit = u64::from(ch.to_digit(10).ok_or(ArithmeticError::Unparseable)?);
        magnitude = magnitude.mul(&ten);
        magnitude = magnitude.add(&BigUint::from_u64(digit));
    }
    Ok(ArithValue::Integer {
        negative: false,
        magnitude,
    })
}

// ---------------------------------------------------------------------------
// Parser.
// ---------------------------------------------------------------------------

struct ArithmeticParser<'a> {
    tokens: &'a [ArithmeticToken],
    cursor: usize,
}

impl<'a> ArithmeticParser<'a> {
    const fn new(tokens: &'a [ArithmeticToken]) -> Self {
        Self { tokens, cursor: 0 }
    }

    fn peek(&self) -> Option<&ArithmeticToken> {
        self.tokens.get(self.cursor)
    }

    fn advance(&mut self) -> Option<&ArithmeticToken> {
        let current = self.tokens.get(self.cursor);
        if current.is_some() {
            self.cursor += 1;
        }
        current
    }

    fn parse(&mut self) -> Result<ArithValue, ArithmeticError> {
        let value = self.parse_additive()?;
        if self.cursor != self.tokens.len() {
            return Err(ArithmeticError::Unparseable);
        }
        Ok(value)
    }

    fn parse_additive(&mut self) -> Result<ArithValue, ArithmeticError> {
        let mut left = self.parse_multiplicative()?;
        while let Some(token) = self.peek() {
            let is_plus = match token {
                ArithmeticToken::Plus => true,
                ArithmeticToken::Minus => false,
                _ => break,
            };
            self.advance();
            let right = self.parse_multiplicative()?;
            left = if is_plus {
                arith_add(left, right)?
            } else {
                arith_sub(left, right)?
            };
        }
        Ok(left)
    }

    fn parse_multiplicative(&mut self) -> Result<ArithValue, ArithmeticError> {
        let mut left = self.parse_unary()?;
        while let Some(token) = self.peek() {
            let op = match token {
                ArithmeticToken::Star => '*',
                ArithmeticToken::Slash => '/',
                ArithmeticToken::Percent => '%',
                _ => break,
            };
            self.advance();
            let right = self.parse_unary()?;
            left = match op {
                '*' => arith_mul(left, right)?,
                '/' => arith_div(&left, &right)?,
                _ => arith_rem(&left, &right)?,
            };
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<ArithValue, ArithmeticError> {
        match self.peek() {
            Some(ArithmeticToken::Minus) => {
                self.advance();
                Ok(self.parse_unary()?.negate())
            }
            Some(ArithmeticToken::Plus) => {
                self.advance();
                self.parse_unary()
            }
            _ => self.parse_primary(),
        }
    }

    fn parse_primary(&mut self) -> Result<ArithValue, ArithmeticError> {
        match self.advance() {
            Some(ArithmeticToken::Number(value)) => Ok(value.clone()),
            Some(ArithmeticToken::LParen) => {
                let inner = self.parse_additive()?;
                match self.advance() {
                    Some(ArithmeticToken::RParen) => Ok(inner),
                    _ => Err(ArithmeticError::UnbalancedParens),
                }
            }
            _ => Err(ArithmeticError::Unparseable),
        }
    }
}

// ---------------------------------------------------------------------------
// Public API.
// ---------------------------------------------------------------------------

fn normalize_expression(expression: &str) -> String {
    let lower = expression.to_lowercase();
    lower
        .replace(" multiplied by ", " * ")
        .replace(" divided by ", " / ")
        .replace(" times ", " * ")
        .replace(" plus ", " + ")
        .replace(" minus ", " - ")
        .replace(" modulo ", " % ")
        .replace(" mod ", " % ")
}

pub fn evaluate_fallback_formatted(expression: &str) -> Result<String, ArithmeticError> {
    let normalized = normalize_expression(expression);
    let tokens = tokenize_arithmetic(&normalized)?;
    if tokens.is_empty() {
        return Err(ArithmeticError::Empty);
    }
    let value = ArithmeticParser::new(&tokens).parse()?;
    Ok(format_arith_value(&value))
}

/// Render an `ArithValue` as a string. Exact integers are shown without
/// scientific notation; floats use the minimal sufficient surface form.
fn format_arith_value(value: &ArithValue) -> String {
    match value {
        ArithValue::Integer {
            negative,
            magnitude,
        } => {
            let s = magnitude.to_decimal_string();
            if *negative && s != "0" {
                format!("-{s}")
            } else {
                s
            }
        }
        ArithValue::Float(f) => format_f64(*f),
    }
}

fn format_f64(value: f64) -> String {
    if !value.is_finite() {
        return String::from("non-finite");
    }
    if value.fract() == 0.0 && value.abs() < 1e15 {
        format!("{value:.0}")
    } else {
        let rendered = format!("{value:.10}");
        let trimmed = rendered.trim_end_matches('0').trim_end_matches('.');
        if trimmed.is_empty() || trimmed == "-" {
            String::from("0")
        } else {
            trimmed.to_owned()
        }
    }
}
