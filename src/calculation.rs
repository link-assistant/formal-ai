//! Calculator delegation boundary for the universal solver.
//!
//! This module keeps natural-language prompt processing in formal-ai, delegates
//! calculator-shaped expressions to `link-calculator` first, and preserves the
//! local arithmetic evaluator for syntax the upstream crate does not support yet.

use crate::arithmetic::{evaluate_fallback_formatted, ArithmeticError};
use crate::calculation_word_problem::normalize_word_problem_detailed;
use crate::fuzzy::is_close_token_typo;
use crate::seed;

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
    pub interpretations: Vec<PromptInterpretation>,
    pub reasoning_steps: Vec<String>,
    pub result_label: Option<String>,
}

/// A visible interpretation applied while normalizing a user prompt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptInterpretation {
    pub original: String,
    pub corrected: String,
}

/// Issue #334: `link-calculator` (≤ 0.17.2) attempts a multi-gigabyte
/// allocation and aborts the whole process when handed an expression that
/// contains a *bare* period — a `.` that is not a decimal point flanked by
/// digits on both sides. The prompt "What is 2+2. What is 3+3." reduces to the
/// expression `2+2. 3+3` after the "what is" wrapper is stripped, and the
/// minimal trigger is as small as `2. 3`. A Rust allocation failure aborts via
/// `SIGABRT`/`SIGKILL` and cannot be caught with `catch_unwind`, so the only
/// safe option is to never hand such an expression to the upstream parser.
///
/// Genuine decimals (`3.14`), thousands/date dot groups (`1.000.000`,
/// `2024.01.01`) keep every `.` between two digits and stay on the fast path;
/// anything else falls through to the local fallback evaluator, which rejects
/// it with a clean `Unparseable` error instead of crashing.
///
/// Reported upstream: <https://github.com/link-assistant/calculator/issues/168>.
fn has_bare_dot(expression: &str) -> bool {
    let bytes = expression.as_bytes();
    bytes.iter().enumerate().any(|(index, &byte)| {
        if byte != b'.' {
            return false;
        }
        let prev_is_digit = index
            .checked_sub(1)
            .and_then(|i| bytes.get(i))
            .is_some_and(u8::is_ascii_digit);
        let next_is_digit = bytes.get(index + 1).is_some_and(u8::is_ascii_digit);
        !(prev_is_digit && next_is_digit)
    })
}

fn evaluate_with_link_calculator(
    expression: &str,
) -> Result<CalculationEvaluation, ArithmeticError> {
    if has_bare_dot(expression) {
        return Err(ArithmeticError::Unparseable);
    }
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
    // Issue #386: the spelled operator vocabulary lives in the
    // `arithmetic_operation` meanings (addition, subtraction, multiplication,
    // division, modulo), not a literal array. Each operator surface is matched
    // as a whole token — CJK surfaces as a substring, since those scripts have
    // no inter-word spaces — the same boundary contract the original
    // space-padded `.contains` check enforced for the English and Russian
    // operators, now extended to every language the meanings lexicalise.
    //
    // `mentions_role_spelled` skips each operator's script-independent value
    // surface (the symbol "+", "-", "*", "/", "%" the normalizer reads): those
    // are matched by the operator-*symbol* scan in `has_calculation_signal`, so
    // counting them here as a spelled "word operator" would double-count them.
    seed::lexicon().mentions_role_spelled(
        seed::ROLE_ARITHMETIC_OPERATOR_WORD,
        &expression.to_lowercase(),
    )
}

/// Evaluate an expression, delegating calculator-supported syntax to
/// `link-calculator` and preserving the in-repo evaluator as a fallback for
/// syntax the upstream crate does not support yet.
pub fn evaluate_calculation(expression: &str) -> Result<CalculationEvaluation, ArithmeticError> {
    if let Ok(evaluation) = evaluate_with_link_calculator(expression) {
        return Ok(evaluation);
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

fn leading_word_spans(value: &str, limit: usize) -> Vec<(usize, usize, &str)> {
    let mut spans = Vec::new();
    let mut start = None;
    for (index, character) in value.char_indices() {
        if character.is_whitespace() {
            if let Some(word_start) = start.take() {
                spans.push((word_start, index, &value[word_start..index]));
                if spans.len() == limit {
                    return spans;
                }
            }
        } else if start.is_none() {
            start = Some(index);
        }
    }
    if let Some(word_start) = start {
        spans.push((word_start, value.len(), &value[word_start..]));
    }
    spans
}

fn fuzzy_prefix_match(value: &str, prefix: &str) -> Option<(usize, usize, PromptInterpretation)> {
    let prefix_words: Vec<&str> = prefix.split_whitespace().collect();
    if prefix_words.is_empty() {
        return None;
    }
    let spans = leading_word_spans(value, prefix_words.len());
    if spans.len() != prefix_words.len() {
        return None;
    }
    let mut typo_count = 0;
    for ((_, _, actual), expected) in spans.iter().zip(prefix_words.iter()) {
        if actual.eq_ignore_ascii_case(expected) {
            continue;
        }
        if !is_close_token_typo(actual, expected) {
            return None;
        }
        typo_count += 1;
    }
    if typo_count != 1 {
        return None;
    }
    let matched_end = spans.last()?.1;
    Some((
        typo_count,
        matched_end,
        PromptInterpretation {
            original: value[..matched_end].to_owned(),
            corrected: prefix.trim().to_owned(),
        },
    ))
}

fn strip_fuzzy_prefix_case_insensitive<'a>(
    value: &'a str,
    prefixes: &[&str],
) -> Option<(&'a str, PromptInterpretation)> {
    let mut matches = prefixes
        .iter()
        .filter_map(|prefix| fuzzy_prefix_match(value, prefix))
        .collect::<Vec<_>>();
    matches.sort_by(|left, right| left.0.cmp(&right.0).then_with(|| right.1.cmp(&left.1)));
    let (typos, matched_end, interpretation) = matches.first()?.clone();
    if matches
        .get(1)
        .is_some_and(|candidate| candidate.0 == typos && candidate.1 == matched_end)
    {
        return None;
    }
    Some((value[matched_end..].trim_start(), interpretation))
}

#[must_use]
pub fn interpretation_statements(interpretations: &[PromptInterpretation]) -> String {
    interpretations
        .iter()
        .map(|interpretation| {
            format!(
                "Interpreted \"{}\" as \"{}\".",
                interpretation.original, interpretation.corrected
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Build the trailing strip suffixes for [`strip_calculation_wrappers`] from the
/// seed.
///
/// Issue #386: the trailing cues a calculation prompt may carry — a result query
/// ("equals", "=", "是多少", "की गणना करें") and a politeness marker ("please",
/// "пожалуйста", "请") — live in the `calculation_result_query` and `politeness`
/// meanings, not a literal array. Each surface is rebuilt into a strip suffix
/// following its script: CJK surfaces strip as-is because those scripts have no
/// inter-word spaces; a pure-symbol surface like the equals sign strips both bare
/// and on a word boundary (so a compact `2*2+2=` is recognised); every other
/// surface gains a leading space so the cue strips only on a word boundary. The
/// suffix loop in [`strip_calculation_wrappers`] runs to a fixpoint, so the set —
/// not its order — determines the result.
fn calculation_wrapper_suffixes() -> Vec<String> {
    let lexicon = seed::lexicon();
    let mut suffixes = Vec::new();
    for role in [
        seed::ROLE_CALCULATION_RESULT_QUERY_CUE,
        seed::ROLE_POLITENESS_CUE,
    ] {
        for surface in lexicon.words_for_role(role) {
            if crate::coding::contains_cjk(&surface) {
                suffixes.push(surface);
            } else if !surface.chars().any(char::is_alphabetic) {
                suffixes.push(format!(" {surface}"));
                suffixes.push(surface);
            } else {
                suffixes.push(format!(" {surface}"));
            }
        }
    }
    suffixes
}

fn strip_calculation_wrappers(prompt: &str) -> (String, bool, Vec<PromptInterpretation>) {
    // Issue #386: the calculation request cues (imperatives like "calculate" /
    // "посчитай" and question openers like "what is" / "сколько будет") live in
    // the `calculation_request` meaning, not a literal array. Each surface is
    // rebuilt into a strip prefix following its script: space-delimited scripts
    // gain a trailing space so the cue strips only on a word boundary
    // ("calculate " never eats the start of "calculated"), while CJK surfaces
    // strip as-is because those scripts have no inter-word spaces. The seed
    // lists the Chinese cues longest first, and `words_for_role` preserves that
    // order, so a more specific cue strips before a shorter one it contains.
    let cue_prefixes: Vec<String> = seed::lexicon()
        .words_for_role(seed::ROLE_CALCULATION_REQUEST_CUE)
        .into_iter()
        .map(|surface| {
            if crate::coding::contains_cjk(&surface) {
                surface
            } else {
                format!("{surface} ")
            }
        })
        .collect();
    let prefixes: Vec<&str> = cue_prefixes.iter().map(String::as_str).collect();
    let suffixes = calculation_wrapper_suffixes();

    let mut working = trim_prompt_punctuation(prompt).to_owned();
    let mut explicit = false;
    let mut interpretations = Vec::new();
    loop {
        let mut changed = false;
        for prefix in &prefixes {
            if let Some(stripped) = strip_prefix_case_insensitive(&working, prefix) {
                working = stripped.to_owned();
                explicit = true;
                changed = true;
                break;
            }
        }
        if !changed {
            if let Some((stripped, interpretation)) =
                strip_fuzzy_prefix_case_insensitive(&working, &prefixes)
            {
                working = stripped.to_owned();
                explicit = true;
                interpretations.push(interpretation);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    loop {
        working = trim_prompt_punctuation(&working).to_owned();
        let mut changed = false;
        for suffix in &suffixes {
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
    (working.trim().to_owned(), explicit, interpretations)
}

/// Surfaces whose presence beside a number marks a prompt as a calculation.
///
/// Issue #386: the former 62-entry signal array is rebuilt from three seed
/// roles, so the calculator router reasons over the same self-describing
/// lexicon every other handler reads instead of a private word list. Each
/// surface is shaped into a match pattern by its script and role:
///
/// - `math_function_name` ("sqrt", "sin", "логарифм", "对数", …) — an ASCII name
///   gains only a leading space so it still fires when glued to its argument's
///   parenthesis ("sqrt(16)"); a non-ASCII name matches as a raw substring
///   because those scripts have no inter-word spaces.
/// - `calculation_domain_term` (currencies and measurement units: "usd", "kg",
///   "ms", "доллар", "公斤", "месяцев", …) — an ASCII surface matches as a whole
///   token (leading and trailing space) so a short code like "ms" never fires
///   inside "items" nor "mb" inside "number"; a non-ASCII surface matches as a
///   raw substring.
/// - `quantity_conversion_cue`, CJK members only ("转换", "兑换", "换成") — the
///   Chinese conversion verbs match as raw substrings, reproducing the former
///   "换成"/"兑换成"/"转换为" entries (the seed's shorter "转换"/"兑换" are faithful
///   substrings of those). The Latin cues ("to", "into") are deliberately
///   excluded here: they are far too common to mark a calculation on their own.
///
/// The caller pads the lowercased expression with surrounding spaces and tests
/// `contains` against each signal, so the set — not its order — decides.
fn calculator_domain_signals() -> Vec<String> {
    let lexicon = seed::lexicon();
    let mut signals = Vec::new();
    for surface in lexicon.words_for_role(seed::ROLE_MATH_FUNCTION_NAME) {
        if surface.is_ascii() {
            signals.push(format!(" {surface}"));
        } else {
            signals.push(surface);
        }
    }
    for surface in lexicon.words_for_role(seed::ROLE_CALCULATION_DOMAIN_TERM) {
        if surface.is_ascii() {
            signals.push(format!(" {surface} "));
        } else {
            signals.push(surface);
        }
    }
    for surface in lexicon.words_for_role(seed::ROLE_QUANTITY_CONVERSION_CUE) {
        if crate::coding::contains_cjk(&surface) {
            signals.push(surface);
        }
    }
    signals
}

fn has_calculation_signal(expression: &str, explicit: bool) -> bool {
    let lower = format!(" {} ", expression.to_lowercase());
    let has_digit = expression.chars().any(|c| c.is_ascii_digit());
    let has_spelled_arithmetic = contains_spelled_arithmetic(expression);
    if !has_digit && !has_spelled_arithmetic {
        return false;
    }
    let has_letter = expression.chars().any(char::is_alphabetic);
    let has_operator_symbol = expression
        .contains(['+', '*', '/', '%', '^', '=', '×', '·', '÷', '−'])
        || (!has_letter && expression.contains('-'));
    let has_word_operator = contains_word_operator(expression);
    if has_operator_symbol || has_word_operator {
        return true;
    }
    let has_currency_symbol = expression.contains(['$', '€', '¥', '₹', '₽']);
    // A quantity_conversion_cue (currency/unit conversion marker or verb) exempts
    // a currency-plus-letters prompt from the prose-rejection guard below, because
    // a conversion is itself a calculation. Matched whole-token through the
    // lexicon, so the bare target markers "to"/"into" only count on a word
    // boundary — exactly what the former hardcoded `lower.contains(" to ")` did.
    let looks_like_conversion = seed::lexicon().mentions_role(
        seed::ROLE_QUANTITY_CONVERSION_CUE,
        &expression.to_lowercase(),
    );
    if has_currency_symbol && has_letter && !explicit && !looks_like_conversion {
        return false;
    }
    let has_known_calculator_word = calculator_domain_signals()
        .iter()
        .any(|signal| lower.contains(signal.as_str()));
    if has_known_calculator_word {
        return true;
    }
    explicit && !has_letter
}

fn contains_spelled_arithmetic(expression: &str) -> bool {
    let lower = format!(" {} ", expression.to_lowercase());
    let forms = seed::lexicon().role_word_forms(seed::ROLE_CARDINAL_NUMBER_WORD);
    let has_number_word = forms.iter().any(|form| {
        // Skip pure-numeral surfaces (e.g. "10"): a bare digit run is handled by
        // the numeric parser, so the spelled-arithmetic path only cares about
        // word forms like "two", "два", or "三".
        !form
            .text
            .chars()
            .all(|character| character.is_ascii_digit())
            && lower.contains(&format!(" {} ", form.text))
    });
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
    let (stripped, explicit, interpretations) = strip_calculation_wrappers(trimmed);
    let mut candidates = Vec::new();
    if !stripped.is_empty() && has_calculation_signal(&stripped, explicit) {
        candidates.push(CalculationCandidate {
            expression: stripped.clone(),
            explicit,
            interpretations: interpretations.clone(),
            reasoning_steps: Vec::new(),
            result_label: None,
        });
    }
    // Issue #334 step 2: rewrite a natural-language word problem ("the 10th
    // Fibonacci number and multiply it by 8% of 500. Show me the code ...") into
    // a calculator expression and offer it as an additional candidate.
    if let Some(normalized) = normalize_word_problem_detailed(&stripped) {
        if has_calculation_signal(&normalized.expression, explicit)
            && !candidates
                .iter()
                .any(|candidate| candidate.expression == normalized.expression)
        {
            candidates.push(CalculationCandidate {
                expression: normalized.expression,
                explicit,
                interpretations,
                reasoning_steps: normalized.reasoning_steps,
                result_label: normalized.result_label,
            });
        }
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
            interpretations: Vec::new(),
            reasoning_steps: Vec::new(),
            result_label: None,
        });
    }
    candidates
}
