//! Issue #479 landing/app/docs/download site-layout assertions, split out of
//! `workflow_release` when that file crossed the 1000-line cap. These check the
//! static web sources the Pages deploy publishes, not the workflow itself; the
//! deploy job's own steps stay in `workflow_release`.

use std::fs;

/// Issue #479 restructured the site into a landing chooser at `/`, the app at
/// `/app/`, the docs hub at `/docs/`, and the download page at `/download/`.
/// These are the static invariants that keep the relocated app working and the
/// chooser wired up — things the e2e suite cannot easily assert (the app's
/// `<base href>`, the desktop wrapper target).
#[test]
fn issue_479_site_is_restructured_into_landing_app_docs_download() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let read = |rel: &str| {
        fs::read_to_string(format!("{manifest_dir}/{rel}"))
            .unwrap_or_else(|_| panic!("{rel} should exist after the issue #479 restructure"))
    };

    // The interactive app moved off the site root to /app/. Its document MUST
    // carry <base href="../"> so every relative asset/worker/seed URL still
    // resolves to the shared site root (under Pages' /formal-ai/ prefix and the
    // desktop static server alike).
    let app_index = read("src/web/app/index.html");
    assert!(
        app_index.contains("<base href=\"../\" />") || app_index.contains("<base href=\"../\">"),
        "src/web/app/index.html must declare <base href=\"../\"> so its relative assets resolve to the site root"
    );
    assert!(
        app_index.contains("app.js?v=__FORMAL_AI_ASSET_VERSION__"),
        "the relocated app should still load the cache-busted app bundle"
    );

    // The site root is now the landing-page chooser, wired to the shared
    // preference store + chrome and offering the three in-site routes. Every
    // script is cache-busted with ?v=__FORMAL_AI_ASSET_VERSION__ so the stamped
    // index embeds the deploy SHA (issue #479: without this the Pages freshness
    // probe never saw the SHA in the landing page and timed out the pipeline).
    let landing = read("src/web/index.html");
    for script in ["preferences.js", "site-chrome.js", "landing.js"] {
        assert!(
            landing.contains(&format!("src=\"{script}?v=__FORMAL_AI_ASSET_VERSION__\"")),
            "the landing page should load {script} cache-busted with the deploy asset version"
        );
    }
    assert!(
        landing.contains("landing.css?v=__FORMAL_AI_ASSET_VERSION__"),
        "the landing page stylesheet should be cache-busted with the deploy asset version"
    );
    for route in ["app/", "docs/", "download/"] {
        assert!(
            landing.contains(&format!("href=\"{route}\"")),
            "the landing page <noscript> fallback should link to {route}"
        );
    }

    // The documentation hub is a sibling page rendered by docs.js.
    let docs_index = read("src/web/docs/index.html");
    assert!(
        docs_index.contains("docs.js"),
        "the docs hub should render via docs.js"
    );

    // The desktop wrapper opens the app at its new /app/ location.
    let desktop_main = read("desktop/main.cjs");
    assert!(
        desktop_main.contains("/app/index.html?desktop=1"),
        "the desktop wrapper should load the app from /app/"
    );
}

/// Issue #479 (maintainer follow-up): "make sure the source code on the landing
/// is a big button". The shared chooser (site-chrome.js, used by / and /docs/)
/// must render the repository link as a prominent hero call-to-action mirroring
/// the /download page's `.primary-download` button — NOT as the small footer
/// text link it used to be. We assert against the rendering source so the
/// guarantee holds even when the e2e suite is not run.
#[test]
fn issue_479_landing_surfaces_source_code_as_a_big_button() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let chrome = fs::read_to_string(format!("{manifest_dir}/src/web/site-chrome.js"))
        .expect("src/web/site-chrome.js should exist");

    // The big button: an anchor with the dedicated class + test id, opening the
    // repository in a new tab with a safe rel.
    assert!(
        chrome.contains("class: \"source-cta\""),
        "site-chrome.js should render the source link as a .source-cta big button"
    );
    assert!(
        chrome.contains("\"data-testid\": \"source-cta\""),
        "the source-cta button needs a stable data-testid for e2e/regression coverage"
    );
    assert!(
        chrome.contains("href: config.repoUrl"),
        "the source-cta button should point at the configured repository URL"
    );
    // The .primary-download-style structure: an action eyebrow above a strong
    // label, both localized.
    for needle in [
        "class: \"source-cta-eyebrow\"",
        "class: \"source-cta-label\"",
        "text: text(locale, \"sourceEyebrow\")",
        "text: text(locale, \"footerSource\")",
    ] {
        assert!(
            chrome.contains(needle),
            "site-chrome.js source-cta should contain `{needle}`"
        );
    }

    // The button replaces — not supplements — the old small footer link. The
    // footer no longer renders a `support-links` source link.
    assert!(
        !chrome.contains("class: \"support-links\""),
        "the small footer support-links source link should be gone now the big button exists"
    );

    // The new action eyebrow is translated for every supported locale.
    for eyebrow in ["Open source", "Открытый код", "开源", "ओपन सोर्स"]
    {
        assert!(
            chrome.contains(eyebrow),
            "site-chrome.js LABELS should define the sourceEyebrow translation `{eyebrow}`"
        );
    }

    // The big-button styles exist in the landing stylesheet (loaded by / and
    // /docs/), emulating the /download page's .primary-download.
    let landing_css = fs::read_to_string(format!("{manifest_dir}/src/web/landing.css"))
        .expect("src/web/landing.css should exist");
    assert!(
        landing_css.contains(".source-cta {"),
        "landing.css should style the .source-cta big button"
    );
    assert!(
        landing_css.contains(".source-cta:hover"),
        "landing.css should give the .source-cta button a hover state like .primary-download"
    );
}
