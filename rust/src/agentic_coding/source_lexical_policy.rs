//! Shared protocol grammar data; missing rules grant no predicate verdict.
use serde_json::Value;
use std::sync::OnceLock;
pub(super) fn source_lexical_rule(text: &str, name: &str) -> Option<bool> {
    static POLICY: OnceLock<Value> = OnceLock::new();
    let policy = POLICY.get_or_init(|| {
        serde_json::from_str(include_str!(
            "../../embedded/data/meta/source-lexical-rules.json"
        ))
        .expect("StaticSourceLexicalRules")
    });
    if text.len() > 262_144 || policy["schema"] != "source-lexical-rules/v1" {
        return None;
    }
    let rule = policy["rules"].get(name)?;
    let operator = rule["operator"].as_str()?;
    let patterns = rule["patterns"].as_array()?;
    if !matches!(operator, "contains" | "prefix")
        || patterns.is_empty()
        || patterns.len() > 8
        || patterns.iter().any(|value| {
            value
                .as_str()
                .is_none_or(|value| value.is_empty() || value.len() > 64)
        })
    {
        return None;
    }
    Some(patterns.iter().any(|value| {
        let value = value.as_str().expect("CheckedLexicalPattern");
        match operator {
            "contains" => text.contains(value),
            "prefix" => text.starts_with(value),
            _ => false,
        }
    }))
}
