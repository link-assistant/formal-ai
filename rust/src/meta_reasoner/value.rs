//! Runtime values of the instruction set, with the JavaScript semantics the
//! seed's `code` fields are written in: `JSON.stringify` equality, number
//! formatting, `<` ordering, `Set` identity and the coercions the operators
//! apply.
//!
//! Equality and ordering here are exact on purpose: the JavaScript verifier
//! compares serialised values, so a float compared with `==` is the semantics,
//! not an approximation.
#![allow(clippy::float_cmp, clippy::cast_precision_loss)]

use core::cmp::Ordering;

use serde_json::Value as Json;

use super::text::js_trim;

/// One runtime value. `Path` is a string the request wrote as a path; in
/// JavaScript it is a plain string, so it serialises, compares and types as
/// text.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// A string.
    Text(String),
    /// A number (JavaScript numbers are doubles).
    Number(f64),
    /// An array.
    List(Vec<Self>),
    /// A file-system path (a string in JavaScript).
    Path(String),
    /// A boolean (only from parsed JSON).
    Bool(bool),
    /// `null` (only from parsed JSON).
    Null,
}

impl Value {
    /// Read a parsed JSON value. Objects have no counterpart in the
    /// instruction set and yield `None`.
    ///
    /// Mirrors `JSON.parse` as used by `metaValueLiterals` in
    /// js/worker/formal_ai_worker_meta_reasoner.js.
    #[must_use]
    pub fn from_json(json: &Json) -> Option<Self> {
        match json {
            Json::Null => Some(Self::Null),
            Json::Bool(flag) => Some(Self::Bool(*flag)),
            Json::Number(number) => number.as_f64().map(Self::Number),
            Json::String(text) => Some(Self::Text(text.clone())),
            Json::Array(items) => items
                .iter()
                .map(Self::from_json)
                .collect::<Option<Vec<_>>>()
                .map(Self::List),
            Json::Object(_) => None,
        }
    }

    /// Parse JSON text into a value.
    ///
    /// Mirrors `JSON.parse` in `metaValueLiterals` and the probe samples of
    /// `metaReasonCore` in js/worker/formal_ai_worker_meta_reasoner.js.
    #[must_use]
    pub fn parse_json(text: &str) -> Option<Self> {
        serde_json::from_str::<Json>(text)
            .ok()
            .and_then(|json| Self::from_json(&json))
    }

    /// The value serialised exactly as `JSON.stringify` does.
    ///
    /// Mirrors `JSON.stringify` as used throughout
    /// js/worker/formal_ai_worker_meta_synthesis.js.
    #[must_use]
    pub fn to_json(&self) -> String {
        match self {
            Self::Text(text) | Self::Path(text) => json_string(text),
            Self::Number(number) => {
                if number.is_finite() {
                    js_number(*number)
                } else {
                    String::from("null")
                }
            }
            Self::List(items) => {
                let inner = items.iter().map(Self::to_json).collect::<Vec<_>>();
                ["[", &inner.join(","), "]"].concat()
            }
            Self::Bool(flag) => flag.to_string(),
            Self::Null => String::from("null"),
        }
    }

    /// The type of a runtime value in the instruction set's vocabulary.
    ///
    /// Mirrors `metaTypeOf` in js/worker/formal_ai_worker_meta_synthesis.js.
    #[must_use]
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::Text(_) | Self::Path(_) => "text",
            Self::Number(_) => "number",
            Self::List(items) => {
                if !items.is_empty() && items.iter().all(|item| matches!(item, Self::Number(_))) {
                    "list_number"
                } else if !items.is_empty() && items.iter().all(Self::is_string) {
                    "list_text"
                } else {
                    "list_any"
                }
            }
            Self::Bool(_) | Self::Null => "unknown",
        }
    }

    /// True for a JavaScript string (text or path).
    ///
    /// Mirrors `typeof value === "string"` in `metaTypeOf`
    /// (js/worker/formal_ai_worker_meta_synthesis.js).
    #[must_use]
    pub const fn is_string(&self) -> bool {
        matches!(self, Self::Text(_) | Self::Path(_))
    }

    /// The string of a text or path value.
    ///
    /// Mirrors a JavaScript string operand in the seed's `code`
    /// (js/worker/formal_ai_worker_meta_synthesis.js `metaCompile`).
    #[must_use]
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::Text(text) | Self::Path(text) => Some(text),
            _ => None,
        }
    }

    /// The items of a list value.
    ///
    /// Mirrors a JavaScript array operand in the seed's `code`
    /// (js/worker/formal_ai_worker_meta_synthesis.js `metaCompile`).
    #[must_use]
    pub fn as_list(&self) -> Option<&[Self]> {
        match self {
            Self::List(items) => Some(items),
            _ => None,
        }
    }

    /// `String(value)`.
    ///
    /// Mirrors JavaScript `ToString`, used by `Array.prototype.join` in the
    /// seed's `code` (js/worker/formal_ai_worker_meta_synthesis.js).
    #[must_use]
    pub fn to_js_string(&self) -> String {
        match self {
            Self::Text(text) | Self::Path(text) => text.clone(),
            Self::Number(number) => js_number(*number),
            Self::List(items) => js_join(items, ","),
            Self::Bool(flag) => flag.to_string(),
            Self::Null => String::from("null"),
        }
    }

    /// `Number(value)`.
    ///
    /// Mirrors JavaScript `ToNumber`, applied by the arithmetic operators in
    /// the seed's `code` and `infer` (js/worker/formal_ai_worker_meta_synthesis.js).
    #[must_use]
    pub fn to_js_number(&self) -> f64 {
        match self {
            Self::Number(number) => *number,
            Self::Bool(flag) => {
                if *flag {
                    1.0
                } else {
                    0.0
                }
            }
            Self::Null => 0.0,
            Self::Text(text) | Self::Path(text) => string_to_number(text),
            Self::List(_) => string_to_number(&self.to_js_string()),
        }
    }
}

/// `Array.prototype.join(separator)`: `null` items become empty strings.
///
/// Mirrors the `join` calls in the seed's `code`
/// (js/worker/formal_ai_worker_meta_synthesis.js `metaCompile`).
#[must_use]
pub fn js_join(items: &[Value], separator: &str) -> String {
    items
        .iter()
        .map(|item| {
            if matches!(item, Value::Null) {
                String::new()
            } else {
                item.to_js_string()
            }
        })
        .collect::<Vec<_>>()
        .join(separator)
}

/// A string as a JSON string literal.
///
/// Mirrors `JSON.stringify(string)` in js/worker/formal_ai_worker_meta_reasoner.js.
#[must_use]
pub fn json_string(text: &str) -> String {
    serde_json::to_string(text).unwrap_or_default()
}

/// `String(number)`: the shortest round-trip digits laid out by the
/// ECMAScript `Number::toString` rules.
///
/// Mirrors `String(parameter)` in `metaRender`
/// (js/worker/formal_ai_worker_meta_synthesis.js).
#[must_use]
pub fn js_number(value: f64) -> String {
    if value.is_nan() {
        return String::from("NaN");
    }
    if value.is_infinite() {
        return String::from(if value > 0.0 { "Infinity" } else { "-Infinity" });
    }
    if value == 0.0 {
        return String::from("0");
    }
    let sign = if value < 0.0 { "-" } else { "" };
    let magnitude = value.abs();
    let scientific = format!("{magnitude:e}");
    let (mantissa, exponent) = scientific.split_once('e').unwrap_or((&scientific, "0"));
    let exponent = exponent.parse::<i64>().unwrap_or(0);
    let digits: String = mantissa.chars().filter(char::is_ascii_digit).collect();
    let count = i64::try_from(digits.len()).unwrap_or(0);
    let point = exponent + 1;
    let body = if count <= point && point <= 21 {
        let zeros = usize::try_from(point - count).unwrap_or(0);
        [digits.as_str(), &"0".repeat(zeros)].concat()
    } else if 0 < point && point <= 21 {
        let split = usize::try_from(point).unwrap_or(0);
        [&digits[..split], ".", &digits[split..]].concat()
    } else if -6 < point && point <= 0 {
        let zeros = usize::try_from(-point).unwrap_or(0);
        ["0.", &"0".repeat(zeros), &digits].concat()
    } else {
        let shown = point - 1;
        let sign_of_exponent = if shown >= 0 { "+" } else { "-" };
        let rest = &digits[1..];
        let fraction = if rest.is_empty() {
            String::new()
        } else {
            [".", rest].concat()
        };
        [
            &digits[..1],
            &fraction,
            "e",
            sign_of_exponent,
            &shown.abs().to_string(),
        ]
        .concat()
    };
    [sign, &body].concat()
}

/// `Number(string)` for the decimal forms a request or a probe can carry.
fn string_to_number(text: &str) -> f64 {
    let trimmed = js_trim(text);
    if trimmed.is_empty() {
        return 0.0;
    }
    match trimmed {
        "Infinity" | "+Infinity" => return f64::INFINITY,
        "-Infinity" => return f64::NEG_INFINITY,
        _ => {}
    }
    if let Some(hex) = trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"))
    {
        return u64::from_str_radix(hex, 16).map_or(f64::NAN, |number| number as f64);
    }
    if trimmed.chars().all(|character| {
        character.is_ascii_digit() || matches!(character, '.' | 'e' | 'E' | '+' | '-')
    }) {
        trimmed.parse::<f64>().unwrap_or(f64::NAN)
    } else {
        f64::NAN
    }
}

/// A primitive operand of the abstract relational comparison.
enum Primitive {
    Text(String),
    Number(f64),
}

fn to_primitive(value: &Value) -> Primitive {
    match value {
        Value::Text(text) | Value::Path(text) => Primitive::Text(text.clone()),
        Value::List(_) => Primitive::Text(value.to_js_string()),
        other => Primitive::Number(other.to_js_number()),
    }
}

/// `a < b` with JavaScript's abstract relational comparison: two strings
/// compare by UTF-16 code units, anything else numerically.
///
/// Mirrors the `sort_list` comparator `(a < b ? -1 : a > b ? 1 : 0)` in
/// data/seed/meta-reasoning.lino, run by js/worker/formal_ai_worker_meta_synthesis.js.
#[must_use]
pub fn js_less(left: &Value, right: &Value) -> bool {
    match (to_primitive(left), to_primitive(right)) {
        (Primitive::Text(a), Primitive::Text(b)) => {
            a.encode_utf16().cmp(b.encode_utf16()) == Ordering::Less
        }
        (a, b) => primitive_number(&a) < primitive_number(&b),
    }
}

fn primitive_number(value: &Primitive) -> f64 {
    match value {
        Primitive::Text(text) => string_to_number(text),
        Primitive::Number(number) => *number,
    }
}

/// `SameValueZero`, the identity a `Set` deduplicates by. Arrays are distinct
/// objects, so two lists are never the same value.
///
/// Mirrors `new Set(input)` in the `unique_items` code of
/// data/seed/meta-reasoning.lino (js/worker/formal_ai_worker_meta_synthesis.js).
#[must_use]
pub fn same_value_zero(left: &Value, right: &Value) -> bool {
    match (left, right) {
        (Value::Number(a), Value::Number(b)) => a == b || (a.is_nan() && b.is_nan()),
        (Value::Text(a) | Value::Path(a), Value::Text(b) | Value::Path(b)) => a == b,
        (Value::Bool(a), Value::Bool(b)) => a == b,
        (Value::Null, Value::Null) => true,
        _ => false,
    }
}

/// `Number.prototype.toFixed(2)`: ties round away from zero, as the
/// specification picks the larger candidate.
///
/// Mirrors `score.toFixed(2)` in `metaGround`
/// (js/worker/formal_ai_worker_meta_reasoner.js).
#[must_use]
pub fn to_fixed2(value: f64) -> String {
    let doubled = value * 200.0;
    let exact_tie = doubled.is_finite()
        && doubled.fract() == 0.0
        && (doubled / 200.0) == value
        && (doubled.abs() % 2.0) == 1.0;
    if exact_tie {
        let rounded = (doubled + doubled.signum()) / 2.0;
        return format!("{:.2}", rounded / 100.0);
    }
    format!("{value:.2}")
}
