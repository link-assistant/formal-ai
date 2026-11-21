use super::*;

/// The workflow checks a workflow-context clause asks for.
pub(super) fn workflow_checks(
    clause: &str,
    words: &str,
    literals: &[TaskLiteral],
    language: Option<&str>,
) -> Vec<Value> {
    let mut checks = Vec::new();
    let negation = first_role_match(&clause.to_lowercase(), "statement_negation_cue");
    let paths: Vec<&TaskLiteral> = literals
        .iter()
        .filter(|literal| literal.text.contains('/') && !literal.text.contains(char::is_whitespace))
        .collect();
    if !paths.is_empty() {
        let after = |literal: &TaskLiteral| negation.is_some_and(|found| literal.start >= found.0);
        let included: Vec<&str> = paths
            .iter()
            .filter(|literal| !after(literal))
            .map(|literal| relative(&literal.text))
            .collect();
        let excluded: Vec<&str> = paths
            .iter()
            .filter(|literal| after(literal))
            .map(|literal| relative(&literal.text))
            .collect();
        return vec![
            json!({ "check": "workflow_path", "included": included, "excluded": excluded }),
        ];
    }
    if negation.is_none() {
        let pairs: Vec<(String, String)> = template("workflow-trigger-events")
            .and_then(|source| serde_json::from_str(&source).ok())
            .unwrap_or_default();
        let events: Vec<&str> = pairs
            .iter()
            .filter(|(role, _)| mentions(role, words))
            .map(|(_, event)| event.as_str())
            .collect();
        if !events.is_empty() {
            checks.push(json!({ "check": "workflow_triggers", "events": events }));
        }
    }
    if mentions("workflow-execution-verb", words) {
        checks.push(json!({ "check": "workflow_runs_program" }));
    }
    if evidenced("print_stdout", words) || mentions("output_obligation_verb", words) {
        let lines: Vec<&str> = literals
            .iter()
            .filter(|literal| !literal.escaped)
            .map(|literal| literal.text.as_str())
            .collect();
        let lines = if lines.is_empty() {
            Value::Null
        } else {
            json!(lines)
        };
        checks.push(json!({ "check": "workflow_verifies_output", "lines": lines }));
    }
    if mentions("workflow-status-obligation", words) {
        checks.push(json!({ "check": "workflow_status" }));
    }
    if mentions("ci_badge_obligation", words) {
        checks.push(json!({ "check": "workflow_badge" }));
    }
    if mentions("file_naming_obligation", words) {
        let names = file_names(clause, literals);
        if !names.is_empty() {
            checks.push(json!({ "check": "workflow_file_name", "names": names }));
        }
    }
    if let Some(language) = language
        && requested_in(clause).as_deref() == Some(language)
    {
        checks.push(json!({ "check": "workflow_runtime", "language": language }));
    }
    if checks.is_empty() {
        checks.push(json!({ "check": "workflow_exists" }));
    }
    checks
}
