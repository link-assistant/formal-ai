//! The computed result of a list transformation asked without code (R1017).
//!
//! "Sort the numbers 5, 2, 9, 1" names an operation and a list but no
//! programming language and no code request: it asks for the sorted list, not
//! for a sorting program. The coding path declines it for want of a language,
//! so this reading answers with the result of the same seed-classified
//! transformation (`sort`, `reverse_sort`, `reverse` from
//! `data/seed/operation-vocabulary.lino`), rendered through the
//! `numeric_list_result_*` templates of
//! `data/seed/multilingual-responses-quantities.lino`.
//!
//! The values must form one stated list: nothing but separators (no letters)
//! between the first and the last item, so numbers scattered through prose or
//! code ("nums[0], nums[1]") are not taken for a list. Reductions stay with
//! the coding path, whose `code_request` gate keeps "in total" prose out.

use super::{Operation, ParsedListItem, compute, detect_operation, parse_list_items};
use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::language::detect as detect_language;
use crate::seed::localized_response;
use crate::solver_handlers::finalize_simple;

/// Answer a list transformation with its computed result, or decline.
pub(super) fn try_stated_result(prompt: &str, log: &mut EventLog) -> Option<SymbolicAnswer> {
    let normalized = crate::web_engine_core::normalize_prompt(prompt);
    let vocabulary = crate::seed::operation_vocabulary();
    let asks_for_code = ["code_request", "function", "implement"]
        .into_iter()
        .any(|slug| vocabulary.matches(slug, &normalized));
    if asks_for_code || crate::coding::composition_language(&normalized, None).is_some() {
        return None;
    }
    let operation = detect_operation(&vocabulary, &normalized)?;
    if !matches!(operation, Operation::Transform(_)) {
        return None;
    }
    let items = parse_list_items(prompt, operation);
    if items.len() < 2 || !forms_one_list(prompt, &items) {
        return None;
    }
    let is_float = items
        .iter()
        .filter_map(ParsedListItem::numeric_value)
        .any(|value| value.fract() != 0.0);
    let given = items
        .iter()
        .map(|item| item.text.clone())
        .collect::<Vec<_>>()
        .join(", ");
    let result = compute(operation, &items, is_float).join(", ");
    let canonical = operation.canonical();
    let language = detect_language(prompt).slug();
    log.append("numeric_list:operation", canonical);
    log.append("numeric_list:given", given.clone());
    log.append("execution_status", "computed deterministically".to_owned());
    log.append("execution_result", result.clone());
    log.append("language", language.to_owned());
    let template = localized_response(&format!("numeric_list_result_{canonical}"), language)?;
    let body = template
        .replace(concat!("{", "given}"), &given)
        .replace(concat!("{", "result}"), &result);
    Some(finalize_simple(
        prompt,
        log,
        "numeric_list_result",
        "response:numeric_list_result",
        &body,
        1.0,
    ))
}

/// Do the items stand together as one list: found in order in the prompt with
/// no letter in any gap between consecutive items?
fn forms_one_list(prompt: &str, items: &[ParsedListItem]) -> bool {
    let mut cursor = 0;
    let mut previous_end: Option<usize> = None;
    for item in items {
        let Some(offset) = prompt[cursor..].find(item.text.as_str()) else {
            return false;
        };
        let start = cursor + offset;
        if let Some(end) = previous_end
            && prompt[end..start].chars().any(char::is_alphabetic)
        {
            return false;
        }
        cursor = start + item.text.len();
        previous_end = Some(cursor);
    }
    true
}
