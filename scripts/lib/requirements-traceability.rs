//! Indexed traceability reader; JavaScript twin: requirements-traceability.mjs.
//! Source partition only (issue #1580): no delivery or manual verdict changes.
use std::fs;
use std::path::Path;

const TRACEABILITY: &str = "docs/requirements-traceability.md";
const CONTINUATION: &str = "docs/requirements-traceability-continuation.md";
const LINK: &str = "(requirements-traceability-continuation.md)";

pub fn read_traceability(root: &Path) -> Result<String, String> {
    let head = fs::read_to_string(root.join(TRACEABILITY)).unwrap_or_default();
    if !head.contains(LINK) {
        return Ok(head);
    }
    let file = root.join(CONTINUATION);
    let tail = fs::read_to_string(&file)
        .map_err(|error| format!("cannot read {}: {error}", file.display()))?;
    Ok(format!("{head}\n{tail}"))
}
