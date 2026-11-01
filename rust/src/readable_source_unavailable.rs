//! Explicit unavailable source grammar for minimal-feature builds.
//! No parser, source reconstruction or verification is fabricated.
use serde_json::Value;
fn unavailable<T>() -> Result<T, String> {
    Err("SourceGrammarFeatureUnavailable".to_owned())
}
pub fn owned_source_links(_source_path: &str, _document: &Value) -> Result<Value, String> {
    unavailable()
}
pub fn source_owner(_document: &Value) -> Result<Option<&str>, String> {
    unavailable()
}
pub fn source_from_links(_document: &Value) -> Result<String, String> {
    unavailable()
}
pub fn source_to_links(_source: &str, _language: &str) -> Result<Value, String> {
    unavailable()
}
pub fn read_source_links(_notation: &str) -> Result<Value, String> {
    unavailable()
}
pub fn write_source_links(_document: &Value) -> Result<String, String> {
    unavailable()
}
pub fn verify_source_links(_document: &Value) -> Result<(), String> {
    unavailable()
}

#[cfg(test)]
mod tests {
    #[test]
    fn minimal_profile_never_claims_source_authority() {
        assert_eq!(
            super::source_to_links("export const ready = true;", "JavaScript").unwrap_err(),
            "SourceGrammarFeatureUnavailable"
        );
        assert!(super::verify_source_links(&serde_json::json!(["source"])).is_err());
    }
}
