use formal_ai::FormalAiEngine;

#[test]
fn navigation_returns_polite_new_tab_link_without_iframe_preview() {
    // Regression test for issue #169: the browser web app cannot reliably
    // preflight arbitrary frame-policy headers before rendering a cross-origin
    // URL. Navigation should therefore use a direct external link instead of a
    // host-specific blocklist or an iframe that may show a blocked-frame page.
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
        response.answer.contains("This web app"),
        "Navigation copy should call the product a web app, got: {}",
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
        !response.answer.to_lowercase().contains("preview below")
            && !response.answer.to_lowercase().contains("iframe"),
        "GitHub navigation must not promise an iframe preview, got: {}",
        response.answer
    );
    assert!(
        !response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("url_preview:iframe:")),
        "GitHub navigation must not record iframe preview evidence: {:?}",
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
fn generic_navigation_uses_same_external_link_path_as_github() {
    let response = FormalAiEngine.answer("Navigate to example.com");

    assert_eq!(response.intent, "url_navigate");
    assert!(response.answer.contains("https://example.com"));
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("url_preview:external_link:")),
        "Generic navigation should not depend on a hardcoded frame-blocked host table: {:?}",
        response.evidence_links
    );
    assert!(
        !response.evidence_links.iter().any(|link| {
            link.starts_with("url_preview:blocked:") || link.starts_with("url_preview:iframe:")
        }),
        "Generic navigation should use the same external-link-only path: {:?}",
        response.evidence_links
    );
}
