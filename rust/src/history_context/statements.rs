//! Issue and pull-request bodies and lifecycles as formal statements
//! (issue #1180 R3).
//!
//! A captured body is more than text: the lines that state a numbered
//! requirement become one `requirement:` statement each, and the capture's
//! timestamp fields become the dated sequence of lifecycle states the record
//! passed through. Both read their shapes from the seed's
//! `requirement_statement` and `state_transition` rows.

use super::HistoryRules;

/// The dated lifecycle transitions of one captured issue or pull request,
/// chronological, as `<prefix><state>@<timestamp>` evidence.
///
/// Each `state_transition` row is dated by the first of its fields the
/// capture carries; a state the capture does not date is absent, never
/// inferred. Equal timestamps keep seed order.
#[must_use]
pub fn state_transitions(value: &serde_json::Value, rules: &HistoryRules) -> Vec<String> {
    let mut dated: Vec<(&str, &str)> = rules
        .state_transitions
        .iter()
        .filter_map(|rule| {
            rule.fields
                .iter()
                .find_map(|field| value.get(field).and_then(serde_json::Value::as_str))
                .filter(|stamp| !stamp.is_empty())
                .map(|stamp| (stamp, rule.state.as_str()))
        })
        .collect();
    dated.sort_by_key(|entry| entry.0);
    dated
        .into_iter()
        .map(|(stamp, state)| format!("{}{state}@{stamp}", rules.state_evidence_prefix))
        .collect()
}

/// The requirement statements of a body, in order, as
/// `<prefix><marker><number> <statement>` evidence.
///
/// A line is one statement when, after any seeded `lead`, it opens with the
/// seed's marker, a number, and a separator (end of line, space, `.`, `:`,
/// `)` or `*`); a reference such as `R1180-3` inside prose is not one.
#[must_use]
pub fn requirement_statements(body: &str, rules: &HistoryRules) -> Vec<String> {
    let rule = &rules.requirement;
    if rule.marker.is_empty() {
        return Vec::new();
    }
    let mut statements = Vec::new();
    for line in body.lines() {
        let mut rest = line.trim();
        while let Some(lead) = rule
            .leads
            .iter()
            .find(|lead| !lead.is_empty() && rest.starts_with(lead.as_str()))
        {
            rest = rest[lead.len()..].trim_start();
        }
        let Some(after_marker) = rest.strip_prefix(rule.marker.as_str()) else {
            continue;
        };
        let digits: String = after_marker
            .chars()
            .take_while(char::is_ascii_digit)
            .collect();
        if digits.is_empty() {
            continue;
        }
        let tail = &after_marker[digits.len()..];
        if !tail.is_empty() && !tail.starts_with([' ', '.', ':', ')', '*']) {
            continue;
        }
        let statement = tail
            .trim_start_matches([' ', '.', ':', ')', '*'])
            .trim_end_matches('*')
            .trim();
        if statement.is_empty() {
            continue;
        }
        statements.push(format!(
            "{}{}{digits} {statement}",
            rule.evidence_prefix, rule.marker
        ));
    }
    statements
}
