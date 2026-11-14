//! Keep coding consumers explicit when the optional source grammar is absent.
#[cfg(feature = "meta-language")]
pub(super) use crate::readable_source::{
    owned_source_links, source_from_links, source_owner, source_to_links, verify_source_links,
};

#[cfg(not(feature = "meta-language"))]
mod unavailable {
    use serde_json::Value;
    fn unavailable<T>() -> Result<T, String> {
        Err("SourceGrammarFeatureUnavailable".to_owned())
    }
    pub(in crate::agentic_coding) fn owned_source_links(
        _owner: &str,
        _document: &Value,
    ) -> Result<Value, String> {
        unavailable()
    }
    pub(in crate::agentic_coding) fn source_from_links(
        _document: &Value,
    ) -> Result<String, String> {
        unavailable()
    }
    pub(in crate::agentic_coding) fn source_owner(
        _document: &Value,
    ) -> Result<Option<&str>, String> {
        unavailable()
    }
    pub(in crate::agentic_coding) fn source_to_links(
        _source: &str,
        _language: &str,
    ) -> Result<Value, String> {
        unavailable()
    }
    pub(in crate::agentic_coding) fn verify_source_links(_document: &Value) -> Result<(), String> {
        unavailable()
    }
}
#[cfg(not(feature = "meta-language"))]
pub(super) use unavailable::{
    owned_source_links, source_from_links, source_owner, source_to_links, verify_source_links,
};
