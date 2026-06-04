use super::*;

#[test]
fn split_body_and_status_parses_curl_sentinel_format() {
    let raw = "hello world\n__formal_ai_http_status__:200";
    let (body, status) = split_body_and_status(raw).expect("should parse");
    assert_eq!(body, "hello world");
    assert_eq!(status, 200);
}

#[test]
fn split_body_and_status_handles_empty_body() {
    let raw = "\n__formal_ai_http_status__:404";
    let (body, status) = split_body_and_status(raw).expect("should parse");
    assert_eq!(body, "");
    assert_eq!(status, 404);
}

#[test]
fn split_body_and_status_returns_error_for_missing_sentinel() {
    let raw = "hello world";
    assert!(split_body_and_status(raw).is_err());
}

#[test]
fn split_body_and_status_handles_body_containing_newlines() {
    let raw = "line1\nline2\nline3\n__formal_ai_http_status__:500";
    let (body, status) = split_body_and_status(raw).expect("should parse");
    assert_eq!(body, "line1\nline2\nline3");
    assert_eq!(status, 500);
}

#[test]
fn http_error_display_truncates_long_body() {
    let body = "x".repeat(1000);
    let error = HttpError::Status {
        status: 500,
        body: body.clone(),
    };
    let rendered = error.to_string();
    assert!(rendered.starts_with("http 500:"));
    assert!(
        rendered.len() < body.len() + 50,
        "long body should be truncated, got {} chars",
        rendered.len(),
    );
}
