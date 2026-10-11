use formal_ai::source_fetch::{SourceCapture, sha256_hex};
#[test]
fn arbitrary_capture_is_verified_before_replay() {
    let body = "arbitrary captured bytes";
    let digest = sha256_hex(body.as_bytes());
    let raw = format!(
        "source-captures\n  capture {digest}\n    url https://example.test/entry\n    fetched-at 1720000000\n    sha256 {digest}\n    body \"{body}\"\n"
    );
    let capture = SourceCapture::from_seed_registry("https://example.test/entry", &raw)
        .unwrap()
        .unwrap();
    assert_eq!(capture.bytes(), body.as_bytes());
    assert_eq!(capture.fetched_at(), "1720000000");
    assert!(capture.cached());
    assert!(
        SourceCapture::from_seed_registry("https://example.test/other", &raw)
            .unwrap()
            .is_none()
    );
    assert!(
        SourceCapture::from_seed_registry(
            "https://example.test/entry",
            &raw.replace(body, "tampered bytes")
        )
        .is_err()
    );
    assert!(
        SourceCapture::from_seed_registry(
            "https://example.test/entry",
            &raw.replace("1720000000", "0")
        )
        .is_err()
    );
}

#[test]
fn duplicate_and_non_unsigned_capture_times_are_rejected() {
    let body = "capture";
    let digest = sha256_hex(body.as_bytes());
    let raw = format!(
        "source-captures\n  capture {digest}\n    url https://example.test/entry\n    fetched-at 1720000000\n    sha256 {digest}\n    body \"{body}\"\n"
    );
    for time in ["0", "+1720000000", "18446744073709551616"] {
        assert!(
            SourceCapture::from_seed_registry(
                "https://example.test/entry",
                &raw.replace("1720000000", time)
            )
            .is_err()
        );
    }
    assert!(
        SourceCapture::from_seed_registry(
            "https://example.test/entry",
            &[raw.as_str(), raw.as_str()].concat()
        )
        .is_err()
    );
}
