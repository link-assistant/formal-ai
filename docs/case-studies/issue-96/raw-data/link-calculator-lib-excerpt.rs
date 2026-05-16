//! Link Calculator - A grammar-based expression parser and calculator.
//!
//! This library provides a WebAssembly-compatible calculator that supports:
//! - `DateTime` parsing and arithmetic
//! - Decimal numbers with units
//! - Currency conversions with temporal awareness
//! - Links notation for expression representation
//!
//! # Example
//!
//! ```
//! use link_calculator::Calculator;
//!
//! let mut calculator = Calculator::new();
//! let result = calculator.calculate_internal("2 + 3");
//! assert!(result.success);
//! assert_eq!(result.result, "5");
//! ```

#![allow(clippy::module_inception)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::option_if_let_else)]
#![allow(clippy::use_self)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::if_not_else)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::format_push_string)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::match_same_arms)]

pub mod crypto_api;
pub mod currency_api;
pub mod error;
pub mod grammar;
pub mod lino;
pub mod plan;
pub mod types;
pub mod utils;
pub mod wasm;

pub use plan::{CalculationPlan, RateSource};
pub use utils::{generate_issue_link, truncate};

use error::{CalculatorError, ErrorInfo};
use grammar::ExpressionParser;
use types::{DateTimeResult, Expression, Value, ValueKind};
use wasm_bindgen::prelude::*;

/// Package version (matches Cargo.toml version).
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Data for plotting a function.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PlotData {
    /// X-axis values.
    pub x_values: Vec<f64>,
    /// Y-axis values.
    pub y_values: Vec<f64>,
    /// Label for the plot (e.g., "sin(x)/x").
    pub label: String,
    /// X-axis label.
    pub x_label: String,
    /// Y-axis label.
    pub y_label: String,
}

/// A single calculation step with i18n support.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CalculationStep {
    /// The translation key for this step type.
    pub key: String,
    /// Parameters for interpolation in the translated message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<std::collections::HashMap<String, String>>,
    /// The raw (English) text for fallback.
    pub text: String,
}

impl CalculationStep {
    /// Creates a new step with a translation key, params, and fallback text.
    #[must_use]
    pub fn new(
        key: impl Into<String>,
        params: Option<std::collections::HashMap<String, String>>,
        text: impl Into<String>,
    ) -> Self {
        Self {
            key: key.into(),
            params,
            text: text.into(),
        }
    }

    /// Creates a simple step with just text (no translation key).
    #[must_use]
    pub fn text_only(text: impl Into<String>) -> Self {
        let text = text.into();
        Self {
            key: String::new(),
            params: None,
            text,
        }
    }
}

/// Repeating decimal notation formats.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RepeatingDecimalFormats {
    /// Vinculum notation with overline: 0.3̅
    pub vinculum: String,
    /// Parenthesis notation: 0.(3)
    pub parenthesis: String,
    /// Ellipsis notation: 0.333...
    pub ellipsis: String,
    /// LaTeX notation: 0.\overline{3}
    pub latex: String,
    /// Fraction representation: 1/3
    pub fraction: String,
}

/// Result of a calculation operation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CalculationResult {
    /// The computed value as a string.
    pub result: String,
    /// The input interpreted in links notation format.
    pub lino_interpretation: String,
    /// Alternative links notation interpretations the user can switch between.
    /// The first element is always the currently selected (default) interpretation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alternative_lino: Option<Vec<String>>,
    /// Step-by-step explanation of the calculation (raw text for backwards compatibility).
    pub steps: Vec<String>,
    /// Step-by-step explanation with i18n support.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub steps_i18n: Option<Vec<CalculationStep>>,
    /// Whether the calculation was successful.
    pub success: bool,
    /// Error message if calculation failed (raw text for backwards compatibility).
    pub error: Option<String>,
    /// Error information for i18n support.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_info: Option<ErrorInfo>,
    /// Link to create an issue for unrecognized input.
    pub issue_link: Option<String>,
    /// LaTeX representation of the input (for rendering mathematical formulas).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latex_input: Option<String>,
    /// LaTeX representation of the result (for rendering mathematical formulas).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latex_result: Option<String>,
    /// Whether this is a symbolic result (e.g., indefinite integral).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_symbolic: Option<bool>,
    /// Plot data points for graphing (x, y pairs).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plot_data: Option<PlotData>,
    /// Repeating decimal notations (if the result is a repeating decimal).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repeating_decimal: Option<RepeatingDecimalFormats>,
    /// Fraction representation of the result (if applicable).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fraction: Option<String>,
    /// Whether the result represents a live (auto-updating) time expression.
    /// When `true`, the frontend should periodically re-calculate the expression.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_live_time: Option<bool>,
    /// Structured datetime metadata for browser-local and UTC conversion display.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub datetime_result: Option<DateTimeResult>,
}

impl CalculationResult {
    /// Creates a successful calculation result.
    #[must_use]
    pub fn success(result: String, lino: String, steps: Vec<String>) -> Self {
        Self {
            result,
            lino_interpretation: lino,
            alternative_lino: None,
            steps,
            steps_i18n: None,
            success: true,
            error: None,
            error_info: None,
            issue_link: None,
            latex_input: None,
            latex_result: None,
            is_symbolic: None,
            plot_data: None,
            repeating_decimal: None,
            fraction: None,
            is_live_time: None,
            datetime_result: None,
        }
    }

    /// Creates a successful calculation result with rational value information.
    #[must_use]
    pub fn success_with_value(value: &Value, lino: String, steps: Vec<String>) -> Self {
        let result = value.to_display_string();

        // Extract repeating decimal and fraction info if available
        let (repeating_decimal, fraction) = if let Some(rational) = value.as_rational() {
            let fraction = if !rational.is_integer() {
                Some(rational.to_fraction_string())
            } else {
                None
            };

            let repeating =
                rational
                    .to_repeating_decimal_notation()
                    .map(|rd| RepeatingDecimalFormats {
                        vinculum: rd.to_vinculum_notation(),
                        parenthesis: rd.to_parenthesis_notation(),
                        ellipsis: rd.to_ellipsis_notation(),
