use formal_ai::FormalAiEngine;

#[test]
fn github_navigation_returns_new_tab_link_without_iframe_preview() {
    // Regression test for issue #169: GitHub blocks embedding, and the
    // browser-only demo cannot inspect GitHub page headers directly from the
    // GitHub Pages origin. In that case navigation should give a natural
    // new-tab link instead of rendering an iframe that shows Chrome's blocked
    // frame placeholder.
    let response = FormalAiEngine.answer("Navigate to github.com");

    assert_eq!(response.intent, "url_navigate");
    assert!(response.answer.contains("https://github.com"));
    assert!(
        response.answer.to_lowercase().contains("new tab"),
        "GitHub navigation should tell the user to open a new tab, got: {}",
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
            .any(|link| link.starts_with("url_preview:blocked:")),
        "GitHub navigation should record why iframe preview was skipped: {:?}",
        response.evidence_links
    );
}
