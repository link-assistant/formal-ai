//! Precedence-climbing arithmetic evaluator used by the universal solver's
//! `try_arithmetic` handler. The evaluator is pure (no I/O, no allocations
//! beyond the token vector) and shared across every interface so "what is
//! 2 + 2?" produces the same trace from CLI, HTTP, Telegram and the demo.

/// Errors produced by [`evaluate_arithmetic`] when the evaluator cannot
/// produce a finite numeric value for the requested expression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArithmeticError {
    Empty,
    Unparseable,
    DivisionByZero,
    Overflow,
    UnbalancedParens,
}

impl std::fmt::Display for ArithmeticError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Empty => "no expression provided",
            Self::Unparseable => "expression could not be parsed",
            Self::DivisionByZero => "division by zero",
            Self::Overflow => "numeric overflow",
            Self::UnbalancedParens => "unbalanced parentheses",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ArithmeticToken {
    Number(f64),
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
                let parsed: f64 = number.parse().map_err(|_| ArithmeticError::Unparseable)?;
                tokens.push(ArithmeticToken::Number(parsed));
            }
            _ => return Err(ArithmeticError::Unparseable),
        }
    }
    Ok(tokens)
}

struct ArithmeticParser<'a> {
    tokens: &'a [ArithmeticToken],
    cursor: usize,
}

impl<'a> ArithmeticParser<'a> {
    const fn new(tokens: &'a [ArithmeticToken]) -> Self {
        Self { tokens, cursor: 0 }
    }

    fn peek(&self) -> Option<ArithmeticToken> {
        self.tokens.get(self.cursor).copied()
    }

    fn advance(&mut self) -> Option<ArithmeticToken> {
        let current = self.peek();
        if current.is_some() {
            self.cursor += 1;
        }
        current
    }

    fn parse(&mut self) -> Result<f64, ArithmeticError> {
        let value = self.parse_additive()?;
        if self.cursor != self.tokens.len() {
            return Err(ArithmeticError::Unparseable);
        }
        Ok(value)
    }

    fn parse_additive(&mut self) -> Result<f64, ArithmeticError> {
        let mut left = self.parse_multiplicative()?;
        while let Some(token) = self.peek() {
            let is_plus = match token {
                ArithmeticToken::Plus => true,
                ArithmeticToken::Minus => false,
                _ => break,
            };
            self.advance();
            let right = self.parse_multiplicative()?;
            left = if is_plus { left + right } else { left - right };
            if !left.is_finite() {
                return Err(ArithmeticError::Overflow);
            }
        }
        Ok(left)
    }

    fn parse_multiplicative(&mut self) -> Result<f64, ArithmeticError> {
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
                '*' => left * right,
                '/' => {
                    if right == 0.0 {
                        return Err(ArithmeticError::DivisionByZero);
                    }
                    left / right
                }
                _ => {
                    if right == 0.0 {
                        return Err(ArithmeticError::DivisionByZero);
                    }
                    left % right
                }
            };
            if !left.is_finite() {
                return Err(ArithmeticError::Overflow);
            }
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<f64, ArithmeticError> {
        match self.peek() {
            Some(ArithmeticToken::Minus) => {
                self.advance();
                Ok(-self.parse_unary()?)
            }
            Some(ArithmeticToken::Plus) => {
                self.advance();
                self.parse_unary()
            }
            _ => self.parse_primary(),
        }
    }

    fn parse_primary(&mut self) -> Result<f64, ArithmeticError> {
        match self.advance() {
            Some(ArithmeticToken::Number(value)) => Ok(value),
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

/// Evaluate a numeric expression that may include `+ - * / %`, parentheses,
/// integer and decimal literals, and English-word operators (`plus`, `minus`,
/// `times`, `multiplied by`, `divided by`, `modulo`).
pub fn evaluate_arithmetic(expression: &str) -> Result<f64, ArithmeticError> {
    let lower = expression.to_lowercase();
    let normalized = lower
        .replace(" multiplied by ", " * ")
        .replace(" divided by ", " / ")
        .replace(" times ", " * ")
        .replace(" plus ", " + ")
        .replace(" minus ", " - ")
        .replace(" modulo ", " % ")
        .replace(" mod ", " % ");
    let tokens = tokenize_arithmetic(&normalized)?;
    if tokens.is_empty() {
        return Err(ArithmeticError::Empty);
    }
    ArithmeticParser::new(&tokens).parse()
}

/// Render the evaluator's `f64` answer with the minimal sufficient surface
/// form: integers stay integers, decimals keep their significant digits.
#[must_use]
pub fn format_arithmetic_result(value: f64) -> String {
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

/// Pull an arithmetic expression out of a natural-language prompt. Returns
/// `None` when the prompt does not look like a calculator request, so the
/// solver can fall through to other specialized handlers.
#[must_use]
pub fn extract_arithmetic_expression(prompt: &str) -> Option<String> {
    let trimmed = prompt.trim();
    if trimmed.is_empty() {
        return None;
    }
    let lower = trimmed.to_lowercase();

    let prefixes = [
        "what is ",
        "what's ",
        "what does ",
        "calculate ",
        "compute ",
        "evaluate ",
        "how much is ",
        "solve ",
    ];

    let mut working: &str = trimmed;
    for prefix in prefixes {
        if lower.starts_with(prefix) {
            working = &trimmed[prefix.len()..];
            break;
        }
    }

    let working = working
        .trim_end_matches('?')
        .trim_end_matches('.')
        .trim_end_matches('!')
        .trim();
    let working = working
        .trim_end_matches(" equal")
        .trim_end_matches(" equals")
        .trim_end_matches(" =")
        .trim();

    if working.is_empty() {
        return None;
    }

    let working_lower = working.to_lowercase();
    let has_symbolic_operator = working.contains(['+', '-', '*', '/', '%', '×', '·', '÷', '−']);
    let has_word_operator = [
        " plus ",
        " minus ",
        " times ",
        " multiplied by ",
        " divided by ",
        " modulo ",
        " mod ",
    ]
    .iter()
    .any(|word| working_lower.contains(word));
    let has_digit = working.chars().any(|c| c.is_ascii_digit());

    if !has_digit {
        return None;
    }
    if !has_symbolic_operator && !has_word_operator {
        return None;
    }

    let only_arithmetic_chars = working.chars().all(|c| {
        c.is_ascii_digit()
            || c.is_whitespace()
            || matches!(
                c,
                '+' | '-' | '*' | '/' | '%' | '(' | ')' | '.' | '_' | '×' | '·' | '÷' | '−' | ','
            )
            || c.is_ascii_alphabetic()
    });
    if !only_arithmetic_chars {
        return None;
    }

    Some(working.to_owned())
}
