//! The handler half of an intent's `relevants` list: which handlers a prompt
//! promotes ahead of the seed-declared precedence table.
//!
//! Split out of `intent_formalization.rs` (issue #932, following the earlier
//! `write_program_request` split) so the promotion table can carry the reason
//! each entry exists without pushing the parent module past its reviewed size.

use super::push_unique;

/// Append a `handler:<name>` relevant for every handler the prompt promotes.
///
/// [`crate::method_registry::MethodRegistry::ordered_method_names_for_relevants`]
/// hoists a promoted handler ahead of the *whole* `handler-precedence.lino`
/// table, so a gate here outranks the declared order. Entries are therefore
/// listed in the order the seed declares: promoting one reading of a prompt
/// without promoting the higher-ranked reading it competes with would silently
/// invert the table.
pub(super) fn append_prompt_relevants(
    prompt: &str,
    _normalized: &str,
    relevants: &mut Vec<String>,
) {
    for relevant in crate::handler_promotion::promoted_relevants(
        &crate::handler_promotion::promotions(),
        prompt,
    ) {
        push_unique(relevants, relevant);
    }
}
