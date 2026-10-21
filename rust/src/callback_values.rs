//! Typed receiver adapters for ECMAScript callback operations.
//!
//! Property/method receivers own dispatch, including getter effects and returned errors.
//! String trimming uses the existing ECMAScript whitespace definition, never Rust's trim.
use crate::seed::parser::LinoNode;

mod nonempty;
pub use nonempty::{CallbackNonemptyReceiver, callback_nonempty};

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

/// A typed view of one named property; the receiver owns absence, effects and errors.
pub trait CallbackPropertyReceiver {
    type Output;
    fn property(self, name: &str) -> Self::Output;
}
/// Capture one property name and dispatch exactly once per receiver invocation.
/// This typed interface does not claim the complete ECMAScript object model.
pub fn callback_property<T: CallbackPropertyReceiver>(name: &str) -> impl Fn(T) -> T::Output + '_ {
    move |value| value.property(name)
}
