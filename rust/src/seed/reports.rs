//! English report templates: engine diagnostics, evidence labels and refusal
//! reasons rendered from the response seed rather than typed into Rust
//! (R379).
//!
//! The records live in `data/seed/multilingual-responses-engine-reports.lino`
//! beside every other response shard, so a wording is fixed in one place.

use std::collections::BTreeMap;
use std::sync::OnceLock;

/// Render the English template registered under `intent`, filling each
/// `{name}` slot from `values`.
///
/// Slots are filled in one pass, so a value that itself carries braces is
/// never re-read as a slot. A missing record degrades to the intent id -- a
/// meaning, never invented prose -- so the gap stays visible.
#[must_use]
pub fn report_text(intent: &str, values: &[(&str, &str)]) -> String {
    static REPORTS: OnceLock<BTreeMap<String, String>> = OnceLock::new();
    let reports = REPORTS.get_or_init(|| {
        let mut reports = BTreeMap::new();
        for record in super::multilingual_responses() {
            if record.language == "en" {
                reports.entry(record.intent).or_insert(record.text);
            }
        }
        reports
    });
    reports.get(intent).map_or_else(
        || intent.to_owned(),
        |template| fill_slots(template, values),
    )
}

/// Replace every `{name}` slot `values` names; any other brace is kept.
fn fill_slots(template: &str, values: &[(&str, &str)]) -> String {
    let mut out = String::with_capacity(template.len());
    let mut rest = template;
    while let Some(open) = rest.find('{') {
        out.push_str(&rest[..open]);
        let after = &rest[open + 1..];
        // A named slot is replaced whole; any other brace is copied as is.
        let (text, consumed) = after
            .find('}')
            .and_then(|close| {
                values
                    .iter()
                    .find(|(slot, _)| *slot == &after[..close])
                    .map(|(_, value)| (*value, close + 1))
            })
            .unwrap_or(("{", 0));
        out.push_str(text);
        rest = &after[consumed..];
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests {
    use super::fill_slots;

    #[test]
    fn slots_fill_in_one_pass_and_unknown_braces_survive() {
        let template = concat!("{", "a}+{", "b}={", "c}");
        let braced_b = concat!("{", "b}");
        assert_eq!(
            fill_slots(template, &[("a", braced_b), ("b", "2")]),
            concat!("{", "b}+2={", "c}")
        );
    }
}
