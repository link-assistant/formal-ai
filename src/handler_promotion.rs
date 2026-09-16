//! Which handlers a prompt hoists ahead of `data/seed/handler-precedence.lino`
//! (#1138 B9, plan 09 Architecture 2).
//!
//! The nineteen promotion predicates hard-coded in
//! `src/intent_formalization/prompt_relevants.rs` become rows of
//! `data/seed/handler-promotions.lino`, whose `when` conditions are written in
//! exactly the grammar `data/seed/handler-rules.lino` already uses and
//! `crate::rule_interpreter` already evaluates. One evaluator, two callers: a
//! promotion becomes a seed edit, never a Rust edit.
//!
//! Wave T skeleton: the shapes the tests name exist, the behaviour does not.

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

/// One promotion row: a handler name, the rank that orders it against the other
/// promotions that fired, the condition block in the handler-rules grammar, and
/// the written reason the row exists.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HandlerPromotion {
    /// The handler this row hoists, e.g. `web_search`.
    pub handler: String,
    /// Lower rank wins when two promotions fire.
    pub rank: u32,
    /// The `when` block verbatim, in the `data/seed/handler-rules.lino` grammar.
    pub when: String,
    /// Why the row exists, quoted from the issue that asked for it.
    pub because: String,
}

impl HandlerPromotion {
    /// The record as Links Notation, so a promotion round-trips through the seed.
    #[must_use]
    pub fn to_links_notation(&self) -> String {
        todo!("plan 09 leaf 10 -- transcribe the nineteen promotions to seed")
    }
}

/// Every promotion the shipped seed declares, in declaration order.
#[must_use]
pub fn promotions() -> Vec<HandlerPromotion> {
    todo!("plan 09 leaf 10 -- read data/seed/handler-promotions.lino")
}

/// Parse a promotions document. Used by the tests to inject a fixture row and
/// observe routing change with no Rust edit.
///
/// # Errors
/// Returns the parse failure when the document is not a promotions document.
pub fn promotions_from(text: &str) -> Result<Vec<HandlerPromotion>, String> {
    let _ = text;
    todo!("plan 09 leaf 10 -- parse data/seed/handler-promotions.lino")
}

/// The `handler:<name>` relevants a prompt promotes, in `rank` order, evaluated
/// through the one rule interpreter. Keeps no handler names.
#[must_use]
pub fn promoted_relevants(promotions: &[HandlerPromotion], prompt: &str) -> Vec<String> {
    let _ = (promotions, prompt);
    todo!("plan 09 leaf 11 -- evaluate the seed promotions in prompt_relevants")
}
