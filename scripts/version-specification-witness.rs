use std::io::Write;
use std::path::Path;
use std::process::Command;

pub(super) const WITNESS: &str = "experiments/formal_ai_subagent/evidence/specification-delivery-1188/unicode-identifier-identity/governed-baseline.v8";
const SPECIFICATION_REFRESH_SCRIPT: &str = "scripts/refresh-release-specification-witness.mjs";

pub(super) fn refresh(previous_manifest: &str) -> Result<bool, String> {
    if !Path::new(WITNESS).is_file() {
        return Ok(false);
    }
    let mut child = Command::new("node")
        .args([
            SPECIFICATION_REFRESH_SCRIPT,
            "--previous-cargo-stdin",
            "--write",
        ])
        .stdin(std::process::Stdio::piped())
        .spawn()
        .map_err(|error| format!("Could not launch specification witness refresh: {error}"))?;
    child
        .stdin
        .take()
        .ok_or("Witness refresh input unavailable")?
        .write_all(previous_manifest.as_bytes())
        .map_err(|error| format!("Could not send preceding Cargo manifest: {error}"))?;
    let status = child
        .wait()
        .map_err(|error| format!("Witness refresh failed: {error}"))?;
    if !status.success() {
        return Err("Specification witness version-only conservation failed".to_string());
    }
    Ok(true)
}
