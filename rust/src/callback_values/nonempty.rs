//! Core-only strict-empty comparison shared by native and `no_std` parsers.

/// Strict comparison with the empty string, without string coercion.
pub trait CallbackNonemptyReceiver {
    fn not_empty_string(self) -> bool;
}

/// Compare to the empty string without coercing other values.
#[must_use]
pub fn callback_nonempty<T: CallbackNonemptyReceiver>(value: T) -> bool {
    value.not_empty_string()
}

impl CallbackNonemptyReceiver for &str {
    fn not_empty_string(self) -> bool {
        !self.is_empty()
    }
}
