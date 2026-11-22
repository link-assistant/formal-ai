use std::process::Command;

use serde_json::{Value, json};

const BODY_BYTE_LIMIT: usize = 1024 * 1024;
const DIAGNOSTIC_BYTE_LIMIT: usize = 4096;
const ORIGIN_VARIABLE: &str = "FORMAL_AI_COMPUTER_USE_HTTP_ORIGIN";

pub(super) fn configured_origin() -> Option<url::Url> {
    let value = std::env::var(ORIGIN_VARIABLE).ok()?;
    let origin = parse_http_url(&value)?;
    (origin.path() == "/" && origin.query().is_none()).then_some(origin)
}

fn parse_http_url(value: &str) -> Option<url::Url> {
    let url = url::Url::parse(value).ok()?;
    (matches!(url.scheme(), "http" | "https")
        && url.host().is_some()
        && url.username().is_empty()
        && url.password().is_none()
        && url.fragment().is_none())
    .then_some(url)
}

pub(super) fn target(origin: Option<&url::Url>, value: &str) -> Result<url::Url, String> {
    if !crate::solver_config::read_network_allowed() {
        return Err("computer_use_network_denied".to_owned());
    }
    let origin = origin.ok_or_else(|| "computer_use_http_origin_required".to_owned())?;
    parse_http_url(value)
        .filter(|target| target.origin() == origin.origin())
        .ok_or_else(|| "computer_use_http_origin_mismatch".to_owned())
}

pub(super) fn request(
    target: &url::Url,
    body: Option<&str>,
    original_url: &str,
    cache_path: &str,
) -> Result<Value, String> {
    if body.is_some_and(|text| text.len() > BODY_BYTE_LIMIT) {
        return Err("computer_use_http_body_too_large".to_owned());
    }
    let method = if body.is_some() { "POST" } else { "GET" };
    let mut curl = Command::new("curl");
    curl.args([
        "-q",
        "--silent",
        "--show-error",
        "--globoff",
        "--proto",
        "=http,https",
        "--max-time",
        "15",
        "--max-filesize",
        &BODY_BYTE_LIMIT.to_string(),
        "--write-out",
        "%{stderr}\n%{http_code}\n%{content_type}",
    ]);
    if body.is_some() {
        curl.args(["--data-binary", "@-"]);
    }
    curl.args(["--", target.as_str()]);
    let input = body.map(|text| text.as_bytes().to_vec());
    let result = crate::source_fetch::bounded_output_with_input(
        &mut curl,
        BODY_BYTE_LIMIT,
        DIAGNOSTIC_BYTE_LIMIT,
        input,
    )
    .map_err(|_| "computer_use_http_transport_failed".to_owned())?;
    if !result.status.success() {
        return Err("computer_use_http_transport_failed".to_owned());
    }
    let metadata = String::from_utf8_lossy(&result.stderr);
    let (status, content_type) = metadata
        .strip_prefix('\n')
        .and_then(|text| text.split_once('\n'))
        .ok_or_else(|| "computer_use_http_metadata_missing".to_owned())?;
    let status = status
        .parse::<u16>()
        .map_err(|_| "computer_use_http_metadata_invalid".to_owned())?;
    if !(200..300).contains(&status) {
        return Err(format!("computer_use_http_status:{status}"));
    }
    let sha256 = crate::source_fetch::sha256_hex(&result.stdout);
    let body = String::from_utf8(result.stdout)
        .map_err(|_| "computer_use_http_body_not_utf8".to_owned())?;
    Ok(json!({
        "method":method,
        "url":original_url,
        "status":status,
        "body":body,
        "headers":{"content-type":content_type},
        "sha256":sha256,
        "cache_path":cache_path,
        "cached":true,
        "provenance":{
            "method":method,
            "url":original_url,
            "status":status,
            "sha256":sha256,
            "cache_path":cache_path
        }
    }))
}
