//! Registry captures are replayable evidence, never a claim of a live fetch.
use super::{FetchError, SourceCapture, sha256_hex};
use crate::seed::parser::parse_lino;

pub(super) fn read_seed_capture(url: &str, raw: &str) -> Result<Option<SourceCapture>, FetchError> {
    let root = parse_lino(raw);
    let matches: Vec<_> = root
        .children
        .iter()
        .filter(|node| node.name == "source-captures")
        .flat_map(|node| node.children.iter())
        .filter(|node| node.name == "capture" && node.find_child_value("url") == url)
        .collect();
    let Some(node) = matches.first() else {
        return Ok(None);
    };
    if matches.len() != 1 {
        return Err(FetchError::Cache(String::from("ambiguous_capture")));
    }
    let bytes = node.find_child_value("body").as_bytes().to_vec();
    let fetched_at = node.find_child_value("fetched-at");
    let recorded = node.find_child_value("sha256");
    let sha256 = sha256_hex(&bytes);
    if fetched_at.is_empty()
        || !fetched_at.bytes().all(|byte| byte.is_ascii_digit())
        || fetched_at.parse::<u64>().ok().is_none_or(|time| time == 0)
        || sha256 != recorded
        || node.id != recorded
    {
        return Err(FetchError::Cache(String::from("invalid_capture")));
    }
    Ok(Some(SourceCapture {
        source_url: url.to_owned(),
        fetched_at: fetched_at.to_owned(),
        sha256,
        cached: true,
        bytes,
    }))
}

#[cfg(test)]
#[path = "../../tests/fixtures/source-cache-seed-contract.rs"]
mod tests;
