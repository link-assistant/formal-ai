use formal_ai::FormalAiEngine;

#[test]
fn navigation_describes_frame_policy_check_before_iframe_preview() {
    // Regression test for issue #169: navigation must not use a host-specific
    // blocklist or blindly render an iframe that may show a blocked-frame page.
    // The browser checks frame-policy metadata first, then chooses an iframe or
    // a direct external link.
    let response = FormalAiEngine.answer("Navigate to github.com");

    assert_eq!(response.intent, "url_navigate");
    assert!(response.answer.contains("https://github.com"));
    assert!(
        response
            .answer
            .contains("I suggest opening this in a new tab"),
        "Navigation should be phrased as a polite suggestion, got: {}",
        response.answer
    );
    assert!(
        !response.answer.contains("Open this"),
        "Navigation copy should not command the user, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("browser web app"),
        "Navigation copy should describe the browser web app behavior, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("frame-policy metadata"),
        "Navigation copy should mention frame-policy metadata, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("X-Frame-Options"),
        "Navigation copy should mention X-Frame-Options, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("CSP frame-ancestors"),
        "Navigation copy should mention CSP frame-ancestors, got: {}",
        response.answer
    );
    assert!(
        !response.answer.contains("cannot reliably confirm"),
        "Navigation copy should not give up before checking headers, got: {}",
        response.answer
    );
    assert!(
        !response.answer.to_lowercase().contains("demo"),
        "Navigation copy should not call the product a demo, got: {}",
        response.answer
    );
    assert!(
        !response.answer.contains("URL requested for"),
        "GitHub navigation copy should be natural, got: {}",
        response.answer
    );
    assert!(
        !response.answer.to_lowercase().contains("preview below"),
        "GitHub navigation must not blindly promise a preview below, got: {}",
        response.answer
    );
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("url_preview:frame_policy_check:")),
        "Navigation should record the frame-policy check path: {:?}",
        response.evidence_links
    );
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("url_preview:external_link:")),
        "Navigation should record the direct external-link preview path: {:?}",
        response.evidence_links
    );
}

#[test]
fn generic_navigation_uses_same_frame_policy_path_as_github() {
    let response = FormalAiEngine.answer("Navigate to example.com");

    assert_eq!(response.intent, "url_navigate");
    assert!(response.answer.contains("https://example.com"));
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("url_preview:frame_policy_check:")),
        "Generic navigation should not depend on a hardcoded frame-blocked host table: {:?}",
        response.evidence_links
    );
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("url_preview:external_link:")),
        "The offline Rust answer should still expose a direct-link fallback: {:?}",
        response.evidence_links
    );
    assert!(
        !response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("url_preview:blocked:")),
        "Generic navigation should not hardcode a blocked host verdict: {:?}",
        response.evidence_links
    );
}
