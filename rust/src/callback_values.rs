//! Typed receiver adapters for the five ECMAScript callback operations.
//! Property/method receivers own dispatch, including getter effects and returned errors.
//! String trimming uses the existing ECMAScript whitespace definition, never Rust's trim.
use crate::seed::parser::LinoNode;

/// A receiver of one text-property observation.
pub trait CallbackTextReceiver {
    type Output;
    fn text(self) -> Self::Output;
}
/// A receiver of one value-property observation.
pub trait CallbackValueReceiver {
    type Output;
    fn value(self) -> Self::Output;
}
/// A receiver of one trim-method invocation.
pub trait CallbackTrimReceiver {
    type Output;
    fn trim(self) -> Self::Output;
}
/// Strict comparison with the empty string, without string coercion.
pub trait CallbackNonemptyReceiver {
    fn not_empty_string(self) -> bool;
}
/// A receiver of the JavaScript length-greater-than-zero operation.
pub trait CallbackLengthReceiver {
    fn positive_length(self) -> bool;
}

/// Observe the text property exactly once, preserving the receiver's return type.
#[must_use]
pub fn callback_text<T: CallbackTextReceiver>(value: T) -> T::Output {
    value.text()
}
/// Observe the value property exactly once, preserving the receiver's return type.
#[must_use]
pub fn callback_value<T: CallbackValueReceiver>(value: T) -> T::Output {
    value.value()
}
/// Invoke the receiver's trim operation exactly once.
#[must_use]
pub fn callback_trim<T: CallbackTrimReceiver>(value: T) -> T::Output {
    value.trim()
}
/// Compare to the empty string without coercing other values.
#[must_use]
pub fn callback_nonempty<T: CallbackNonemptyReceiver>(value: T) -> bool {
    value.not_empty_string()
}
/// Apply the receiver's length comparison without replacing its dispatch.
#[must_use]
pub fn callback_has_length<T: CallbackLengthReceiver>(value: T) -> bool {
    value.positive_length()
}

impl<'a> CallbackValueReceiver for &'a LinoNode {
    type Output = &'a str;
    fn value(self) -> Self::Output {
        &self.id
    }
}
impl<'a> CallbackTrimReceiver for &'a str {
    type Output = &'a str;
    fn trim(self) -> Self::Output {
        self.trim_matches(crate::meta_reasoner::text::is_js_whitespace)
    }
}
impl CallbackNonemptyReceiver for &str {
    fn not_empty_string(self) -> bool {
        self != ""
    }
}
impl CallbackNonemptyReceiver for &serde_json::Value {
    fn not_empty_string(self) -> bool {
        self.as_str() != Some("")
    }
}
impl<T> CallbackLengthReceiver for &[T] {
    fn positive_length(self) -> bool {
        !self.is_empty()
    }
}
impl CallbackLengthReceiver for &str {
    fn positive_length(self) -> bool {
        self.as_bytes().positive_length()
    }
}
