use std::fs;
use std::process::Command;

fn release_workflow() -> String {
    fs::read_to_string(format!(
        "{}/.github/workflows/release.yml",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap()
    .replace("\r\n", "\n")
}

fn desktop_release_workflow() -> String {
    fs::read_to_string(format!(
        "{}/.github/workflows/desktop-release.yml",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap()
    .replace("\r\n", "\n")
}

fn job_block<'a>(workflow: &'a str, job_name: &str) -> &'a str {
    let marker = format!("  {job_name}:\n");
    let start = workflow.find(&marker).unwrap();
    let body_start = start + marker.len();
    let rest = &workflow[body_start..];

    let next_job = rest
        .lines()
        .scan(0usize, |offset, line| {
            let current_offset = *offset;
            *offset += line.len() + 1;
            Some((current_offset, line))
        })
        .find_map(|(offset, line)| {
            let starts_at_job_indent = line.starts_with("  ") && !line.starts_with("    ");
            (starts_at_job_indent && line.trim_end().ends_with(':')).then_some(offset)
        });

    next_job.map_or_else(
        || &workflow[start..],
        |end| &workflow[start..body_start + end],
    )
}

fn workflow_step_block<'a>(job: &'a str, step_name: &str) -> &'a str {
    let marker = format!("      - name: {step_name}\n");
    let start = job.find(&marker).unwrap();
    let body_start = start + marker.len();
    let rest = &job[body_start..];

    let next_step = rest
        .lines()
        .scan(0usize, |offset, line| {
            let current_offset = *offset;
            *offset += line.len() + 1;
            Some((current_offset, line))
        })
        .find_map(|(offset, line)| line.starts_with("      - ").then_some(offset));

    next_step.map_or_else(|| &job[start..], |end| &job[start..body_start + end])
}

fn workflow_job_names(workflow: &str) -> Vec<&str> {
    let marker = "jobs:\n";
    let start = workflow.find(marker).unwrap() + marker.len();

    workflow[start..]
        .lines()
        .filter_map(|line| {
            let starts_at_job_indent = line.starts_with("  ") && !line.starts_with("    ");
            (starts_at_job_indent && line.trim_end().ends_with(':'))
                .then(|| line.trim().trim_end_matches(':'))
        })
        .collect()
}

#[test]
fn demo_deploy_is_independent_from_release_publication() {
    let workflow = release_workflow();
    let deploy_demo = job_block(&workflow, "deploy-demo");

    assert!(deploy_demo.contains("needs: [build]"));
    assert!(deploy_demo.contains("needs.build.result == 'success'"));
    assert!(deploy_demo.contains("github.ref == 'refs/heads/main'"));
    assert!(!deploy_demo.contains("needs: [auto-release, manual-release]"));
    assert!(!deploy_demo.contains("needs.auto-release.result"));
    assert!(!deploy_demo.contains("needs.manual-release.result"));
}

#[test]
fn demo_deploy_uses_github_pages_workflow_artifact() {
    let workflow = release_workflow();
    let deploy_demo = job_block(&workflow, "deploy-demo");

    assert!(deploy_demo.contains("pages: write"));
    assert!(deploy_demo.contains("id-token: write"));
    assert!(deploy_demo.contains("environment:\n      name: github-pages"));
    assert!(deploy_demo.contains("url: ${{ steps.deployment.outputs.page_url }}"));
    assert!(deploy_demo.contains("actions/configure-pages@v6"));
    assert!(deploy_demo.contains("actions/upload-pages-artifact@v5"));
    assert!(deploy_demo.contains("path: src/web"));
    assert!(deploy_demo.contains("id: deployment"));
    assert!(deploy_demo.contains("actions/deploy-pages@v5"));
    assert!(!deploy_demo.contains("peaceiris/actions-gh-pages"));
    assert!(!deploy_demo.contains("publish_dir: src/web"));
    assert!(!deploy_demo.contains("publish_branch: gh-pages"));
}

#[test]
fn pages_e2e_uses_deployment_output_url() {
    let workflow = release_workflow();
    let deploy_demo = job_block(&workflow, "deploy-demo");
    let pages_e2e = job_block(&workflow, "test-e2e-pages");

    assert!(deploy_demo.contains("page_url: ${{ steps.deployment.outputs.page_url }}"));
    assert!(pages_e2e.contains("needs.deploy-demo.outputs.page_url"));
    assert!(!pages_e2e.contains("PAGES_URL=https://link-assistant.github.io/formal-ai"));
}

#[test]
fn pages_deploy_is_pinned_and_live_e2e_waits_for_matching_deployment() {
    let workflow = release_workflow();
    let deploy_demo = job_block(&workflow, "deploy-demo");
    let pages_e2e = job_block(&workflow, "test-e2e-pages");

    assert!(
        deploy_demo.contains("ref: ${{ github.sha }}"),
        "Pages deployment should use the exact commit that passed CI"
    );
    assert!(
        deploy_demo.contains("Stamp GitHub Pages artifact"),
        "Pages deployment should stamp a per-commit asset marker before upload"
    );
    assert!(
        deploy_demo.contains("scripts/stamp-pages-artifact.sh src/web \"${{ github.sha }}\""),
        "Pages deployment should stamp src/web with the workflow commit SHA"
    );
    assert!(
        pages_e2e.contains("scripts/wait-for-pages-deployment.sh"),
        "live Pages e2e should poll for the deployed commit before Playwright starts"
    );
    assert!(
        pages_e2e.contains("needs.deploy-demo.outputs.page_url"),
        "live Pages e2e should probe the resolved Pages URL"
    );
    assert!(
        pages_e2e.contains("\"${{ github.sha }}\""),
        "live Pages e2e should wait for the current workflow commit"
    );
    assert!(
        !pages_e2e.contains("run: sleep 30"),
        "a fixed sleep can still test stale GitHub Pages assets"
    );
}

#[test]
fn wait_for_pages_deployment_is_marker_authoritative() {
    // Issue #479 (root cause, take 2): the live-Pages freshness probe used to
    // require the raw deploy SHA to appear in the served index BODY
    // (`grep -Fq "$expected_sha" "$index_file"`). That coupled the probe to every
    // root page embedding the commit SHA verbatim. When the issue #479 landing
    // page shipped at `/` WITHOUT cache-busted asset refs, the stamped index
    // never contained the SHA, so the probe ran the full 300s and timed out --
    // failing the whole pipeline and (via the desktop-release gate) suppressing
    // every desktop build. The probe is now marker-authoritative: GitHub Pages
    // deploys atomically, so deployment.json advertising "sha":"<expected_sha>"
    // is sufficient proof the matching stamped index is live.
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let wait_script = fs::read_to_string(format!(
        "{manifest_dir}/scripts/wait-for-pages-deployment.sh"
    ))
    .unwrap();

    // The marker SHA is the authoritative freshness signal.
    assert!(
        wait_script.contains("${expected_sha}") && wait_script.contains("\"$marker_file\""),
        "wait script should require the deployment.json marker to advertise the expected SHA"
    );
    // The brittle "index body must contain the SHA" requirement must be gone.
    assert!(
        !wait_script.contains("-Fq \"$expected_sha\" \"$index_file\""),
        "issue #479 regression: the probe must NOT hard-require the raw SHA in the index body -- \
         a valid root page without cache-busted asset refs would hang the probe for the full timeout"
    );
    // But the defensive placeholder guards (catch a half-run stamp step) remain.
    assert!(
        wait_script.contains("__FORMAL_AI_ASSET_VERSION__")
            && wait_script.contains("__FORMAL_AI_VERSION__"),
        "wait script should still reject an index that retains un-stamped placeholders"
    );
}

#[test]
fn github_pages_artifact_advertises_crate_version_from_cargo_toml() {
    // Issue #72: the deployed Pages site advertised `0.16.0` long after the
    // crate moved past it. The fix replaces a hardcoded literal with the
    // `__FORMAL_AI_VERSION__` placeholder and a stamp step that reads the
    // current `Cargo.toml` version during the Pages deploy. Without these
    // pieces the deploy can drift again.
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let index_html = fs::read_to_string(format!("{manifest_dir}/src/web/index.html")).unwrap();
    let app_js = fs::read_to_string(format!("{manifest_dir}/src/web/app.js")).unwrap();
    let stamp_script =
        fs::read_to_string(format!("{manifest_dir}/scripts/stamp-pages-artifact.sh")).unwrap();
    let workflow = release_workflow();
    let deploy_demo = job_block(&workflow, "deploy-demo");

    assert!(
        index_html.contains("__FORMAL_AI_VERSION__"),
        "index.html should advertise the formal-ai version via a placeholder, not a hardcoded literal"
    );
    assert!(
        !index_html.contains("content=\"0.16.0\""),
        "index.html should not pin the stale 0.16.0 version literal"
    );
    assert!(
        app_js.contains("formal-ai-version"),
        "app.js should still read the formal-ai-version meta tag"
    );
    assert!(
        !app_js.contains("\"0.16.0\"") && !app_js.contains("'0.16.0'"),
        "app.js should not hardcode the stale 0.16.0 version as a string literal"
    );
    assert!(
        stamp_script.contains("__FORMAL_AI_VERSION__"),
        "stamp script should substitute the formal-ai version placeholder"
    );
    assert!(
        stamp_script.contains("formal-ai-version"),
        "stamp script should validate the rendered meta tag content"
    );
    assert!(
        stamp_script.contains("formal_ai_version"),
        "stamp deployment.json should carry the formal-ai version for the e2e wait script"
    );
    assert!(
        deploy_demo.contains("Read formal-ai version from Cargo.toml"),
        "deploy-demo should detect the crate version before stamping the artifact"
    );
    assert!(
        deploy_demo.contains(
            "scripts/stamp-pages-artifact.sh src/web \"${{ github.sha }}\" \"${{ github.sha }}\" \"${{ steps.formal_ai_version.outputs.version }}\""
        ),
        "deploy-demo should forward the resolved crate version to the stamp script"
    );

    let wait_script = fs::read_to_string(format!(
        "{manifest_dir}/scripts/wait-for-pages-deployment.sh"
    ))
    .unwrap();
    assert!(
        wait_script.contains("__FORMAL_AI_VERSION__"),
        "pages-deployment wait script should reject lingering version placeholders"
    );
}

#[test]
fn stamp_pages_artifact_replaces_formal_ai_version_placeholder() {
    // Issue #72: end-to-end smoke test for the stamp script. Copy
    // `src/web/index.html` into a scratch directory, run the script the
    // same way CI does, and assert the rendered file advertises the
    // requested formal-ai version. Catches regressions in either the
    // placeholder or the substitution logic.
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let script = format!("{manifest_dir}/scripts/stamp-pages-artifact.sh");
    if !std::path::Path::new("/bin/bash").exists() {
        eprintln!("skipping: /bin/bash not available");
        return;
    }
    let tmp = std::env::temp_dir().join(format!(
        "formal-ai-stamp-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |duration| duration.as_nanos())
    ));
    let web_dir = tmp.join("web");
    fs::create_dir_all(&web_dir).expect("create scratch web dir");
    fs::create_dir_all(web_dir.join("tests")).expect("create scratch tests dir");
    let source_index = fs::read_to_string(format!("{manifest_dir}/src/web/index.html")).unwrap();
    fs::write(web_dir.join("index.html"), &source_index).expect("seed index.html");
    fs::write(
        web_dir.join("tests/index.html"),
        r#"<meta name="formal-ai-version" content="__FORMAL_AI_VERSION__"><script src="connectivity.js?v=__FORMAL_AI_ASSET_VERSION__"></script>"#,
    )
    .expect("seed tests/index.html");

    let output = Command::new("/bin/bash")
        .arg(&script)
        .arg(web_dir.to_str().unwrap())
        .arg("deadbeef")
        .arg("deadbeef")
        .arg("9.9.9")
        .output()
        .expect("run stamp script");
    let status_ok = output.status.success();
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    let rendered = fs::read_to_string(web_dir.join("index.html")).unwrap_or_default();
    let rendered_tests = fs::read_to_string(web_dir.join("tests/index.html")).unwrap_or_default();
    let marker = fs::read_to_string(web_dir.join("deployment.json")).unwrap_or_default();
    let _ = fs::remove_dir_all(&tmp);

    assert!(
        status_ok,
        "stamp script exited with {status:?}\nstdout: {stdout}\nstderr: {stderr}",
        status = output.status
    );
    assert!(
        rendered.contains("content=\"9.9.9\""),
        "stamped index.html should advertise the supplied formal-ai version, got:\n{rendered}"
    );
    assert!(
        !rendered.contains("__FORMAL_AI_VERSION__"),
        "stamped index.html should not retain the formal-ai version placeholder"
    );
    assert!(
        !rendered.contains("__FORMAL_AI_ASSET_VERSION__"),
        "stamped index.html should not retain the asset version placeholder"
    );
    assert!(
        rendered.contains("?v=deadbeef"),
        "issue #479: the stamped landing index must embed the deploy asset version (SHA) on its \
         asset refs so the Pages freshness probe and CDN cache-busting both see it, got:\n{rendered}"
    );
    assert!(
        rendered_tests.contains("content=\"9.9.9\""),
        "stamped tests/index.html should advertise the supplied formal-ai version, got:\n{rendered_tests}"
    );
    assert!(
        rendered_tests.contains("connectivity.js?v=deadbeef"),
        "stamp script should cache-bust nested test page assets, got:\n{rendered_tests}"
    );
    assert!(
        !rendered_tests.contains("__FORMAL_AI_VERSION__")
            && !rendered_tests.contains("__FORMAL_AI_ASSET_VERSION__"),
        "stamped tests/index.html should not retain placeholders, got:\n{rendered_tests}"
    );
    assert!(
        marker.contains("\"formal_ai_version\": \"9.9.9\""),
        "deployment.json should record the formal-ai version, got:\n{marker}"
    );
}

#[test]
fn static_demo_runtime_assets_are_cache_busted_by_deployment_version() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    // The interactive app moved to /app/ (issue #479); its cache-busted runtime
    // assets now live in src/web/app/index.html. The site root index.html is the
    // landing-page chooser; it ALSO cache-busts its own assets with the deploy
    // asset version (so the stamped index embeds the SHA the Pages probe waits
    // for) but, unlike /app/, its strict CSP forbids an inline asset-version
    // script, so the SHA rides on ?v= refs rather than window.FORMAL_AI_ASSET_VERSION.
    let landing_html = fs::read_to_string(format!("{manifest_dir}/src/web/index.html")).unwrap();
    let app_index_html =
        fs::read_to_string(format!("{manifest_dir}/src/web/app/index.html")).unwrap();
    let tests_index =
        fs::read_to_string(format!("{manifest_dir}/src/web/tests/index.html")).unwrap();
    let app_js = fs::read_to_string(format!("{manifest_dir}/src/web/app.js")).unwrap();
    let seed_loader_js =
        fs::read_to_string(format!("{manifest_dir}/src/web/seed_loader.js")).unwrap();
    let worker_js =
        fs::read_to_string(format!("{manifest_dir}/src/web/formal_ai_worker.js")).unwrap();
    let stamp_script =
        fs::read_to_string(format!("{manifest_dir}/scripts/stamp-pages-artifact.sh")).unwrap();
    let wait_script = fs::read_to_string(format!(
        "{manifest_dir}/scripts/wait-for-pages-deployment.sh"
    ))
    .unwrap();

    for asset in [
        "styles.css?v=__FORMAL_AI_ASSET_VERSION__",
        "seed_loader.js?v=__FORMAL_AI_ASSET_VERSION__",
        "preferences.js?v=__FORMAL_AI_ASSET_VERSION__",
        "i18n.js?v=__FORMAL_AI_ASSET_VERSION__",
        "memory.js?v=__FORMAL_AI_ASSET_VERSION__",
        "app.js?v=__FORMAL_AI_ASSET_VERSION__",
    ] {
        assert!(
            app_index_html.contains(asset),
            "app/index.html should version local asset {asset}"
        );
    }
    assert!(app_index_html.contains("window.FORMAL_AI_ASSET_VERSION"));
    // The landing page (site root) advertises the stamped formal-ai version AND
    // cache-busts every one of its own assets with the deploy asset version, so
    // the stamped index embeds the SHA the Pages freshness probe waits for
    // (issue #479 regression guard).
    assert!(landing_html.contains("__FORMAL_AI_VERSION__"));
    for asset in [
        "landing.css?v=__FORMAL_AI_ASSET_VERSION__",
        "preferences.js?v=__FORMAL_AI_ASSET_VERSION__",
        "site-chrome.js?v=__FORMAL_AI_ASSET_VERSION__",
        "landing.js?v=__FORMAL_AI_ASSET_VERSION__",
    ] {
        assert!(
            landing_html.contains(asset),
            "landing index.html should version local asset {asset} so the stamped page carries the deploy SHA"
        );
    }
    assert!(app_js.contains("withAssetVersion(\"formal_ai_worker.js\")"));
    assert!(seed_loader_js.contains("fetchText(withAssetVersion(file))"));
    assert!(worker_js.contains("importScripts(withAssetVersion(\"seed_loader.js\"))"));
    assert!(worker_js.contains("fetch(withAssetVersion(\"formal_ai_worker.wasm\"))"));
    for asset in [
        "connectivity.css?v=__FORMAL_AI_ASSET_VERSION__",
        "connectivity.js?v=__FORMAL_AI_ASSET_VERSION__",
    ] {
        assert!(
            tests_index.contains(asset),
            "tests/index.html should version local asset {asset}"
        );
    }
    assert!(tests_index.contains("__FORMAL_AI_VERSION__"));
    assert!(stamp_script.contains("__FORMAL_AI_ASSET_VERSION__"));
    assert!(stamp_script.contains("find \"$artifact_dir\" -type f -name '*.html'"));
    assert!(stamp_script.contains("deployment.json"));
    assert!(wait_script.contains("deployment.json"));
    assert!(wait_script.contains("expected_sha"));
}

#[test]
fn pages_e2e_navigation_preserves_repository_subpath() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let pages_config = fs::read_to_string(format!(
        "{manifest_dir}/tests/e2e/playwright.pages.config.js"
    ))
    .unwrap();
    let demo_spec =
        fs::read_to_string(format!("{manifest_dir}/tests/e2e/tests/demo.spec.js")).unwrap();
    let multilingual_spec = fs::read_to_string(format!(
        "{manifest_dir}/tests/e2e/tests/multilingual.spec.js"
    ))
    .unwrap();
    let connectivity_spec = fs::read_to_string(format!(
        "{manifest_dir}/tests/e2e/tests/connectivity.spec.js"
    ))
    .unwrap();

    assert!(
        pages_config.contains("normalizeBaseUrl"),
        "Pages e2e should normalize PAGES_URL with a trailing slash so ./ resolves inside /formal-ai/"
    );
    assert!(
        pages_config.contains("https://link-assistant.github.io/formal-ai/"),
        "default Pages URL should include the repository subpath and trailing slash"
    );

    // The app moved to /app/ (issue #479), so the Pages baseURL targets /app/.
    // The app specs navigate with a relative './' (→ /app/), while connectivity
    // reaches its sibling harness with a relative '../tests/' (→ /tests/). Both
    // are relative, so the /formal-ai/ repository subpath is always preserved;
    // an absolute '/…' would drop it.
    for (path, spec, expected_nav) in [
        (
            "tests/e2e/tests/demo.spec.js",
            demo_spec.as_str(),
            "page.goto('./')",
        ),
        (
            "tests/e2e/tests/multilingual.spec.js",
            multilingual_spec.as_str(),
            "page.goto('./')",
        ),
        (
            "tests/e2e/tests/connectivity.spec.js",
            connectivity_spec.as_str(),
            "page.goto('../tests/')",
        ),
    ] {
        assert!(
            !spec.contains("page.goto('/');"),
            "{path} should not navigate to / because URL resolution drops the /formal-ai/ subpath"
        );
        assert!(
            spec.contains(expected_nav),
            "{path} should navigate with a relative {expected_nav} so Pages tests stay under the repository subpath"
        );
    }
}

#[test]
fn test_job_skips_non_code_changes() {
    // Issue #442: the `test` job ran whenever the `changelog` job was *skipped*.
    // `changelog` is skipped precisely when there are no code changes (docs-only
    // commits, .gitkeep edits, changelog-fragment-only commits), so the
    // `needs.changelog.result == 'skipped'` clause turned "nothing relevant
    // changed" into "run the entire test suite". This regression guard pins the
    // corrected gating: `test` keys off the detect-changes outputs, exactly like
    // `lint` and `coverage`, and never resurrects the changelog-skip escape.
    let workflow = release_workflow();
    let test = job_block(&workflow, "test");

    assert!(
        !test.contains("needs.changelog.result == 'skipped'"),
        "test job must not run merely because the changelog check was skipped; \
         a skipped changelog means there were no code changes (issue #442)"
    );
    assert!(
        !test.contains("needs.changelog.result == 'success'"),
        "test job should be decoupled from the changelog check and gate on the \
         change detector instead (issue #442)"
    );
    assert!(
        test.contains("needs: [detect-changes]"),
        "test job should depend on detect-changes so it can gate on its outputs"
    );
    assert!(
        test.contains("needs.detect-changes.outputs.any-code-changed == 'true'"),
        "test job should run when code files changed"
    );
    assert!(
        test.contains("needs.detect-changes.outputs.rs-changed == 'true'"),
        "test job should run when Rust sources changed"
    );
    assert!(
        test.contains("needs.detect-changes.outputs.toml-changed == 'true'"),
        "test job should run when Cargo manifests changed"
    );
    assert!(
        test.contains("needs.detect-changes.outputs.workflow-changed == 'true'"),
        "test job should run when the CI workflow itself changed"
    );
    assert!(
        test.contains("github.event_name == 'push'")
            && test.contains("github.event_name == 'workflow_dispatch'"),
        "test job should still always run on push and manual dispatch"
    );
    assert!(
        test.contains("always() && !cancelled()"),
        "test job needs always() so the skipped detect-changes dependency does \
         not cascade on workflow_dispatch"
    );
}

#[test]
fn change_gated_jobs_never_depend_on_a_skipped_changelog() {
    // Generalises issue #442 across every change-gated job: none of them should
    // treat a *skipped* changelog/version check as a signal to run. A skipped
    // upstream check means "no code changed", which must never widen coverage.
    let workflow = release_workflow();
    for job_name in ["lint", "test", "coverage", "test-e2e-local"] {
        let job = job_block(&workflow, job_name);
        // Inspect only effective YAML (skip `#` comment lines) so the rationale
        // comments that quote the old buggy clause don't trip the guard.
        let has_skip_dependency = job
            .lines()
            .filter(|line| !line.trim_start().starts_with('#'))
            .any(|line| line.contains("result == 'skipped'"));
        assert!(
            !has_skip_dependency,
            "{job_name} job must not run because an upstream check was skipped (issue #442)"
        );
    }
}

#[test]
fn release_workflow_jobs_have_explicit_timeouts() {
    let workflow = release_workflow();
    let expected_timeouts = [
        ("detect-changes", 5),
        ("changelog", 10),
        ("version-check", 5),
        ("lint", 10),
        ("test", 10),
        ("coverage", 15),
        ("build", 10),
        ("auto-release", 30),
        ("manual-release", 30),
        ("changelog-pr", 10),
        ("test-e2e-local", 15),
        // deploy-demo also runs `cargo doc` for the /docs/api reference (issue
        // #479), which compiles the dependency tree on a cold cargo cache.
        ("deploy-demo", 20),
        ("test-e2e-pages", 15),
    ];

    let actual_jobs = workflow_job_names(&workflow);
    let expected_jobs = expected_timeouts
        .iter()
        .map(|(job_name, _)| *job_name)
        .collect::<Vec<_>>();
    assert_eq!(actual_jobs, expected_jobs);

    for (job_name, timeout_minutes) in expected_timeouts {
        let job = job_block(&workflow, job_name);
        let expected = format!("    timeout-minutes: {timeout_minutes}\n");
        assert!(
            job.contains(&expected),
            "{job_name} should declare {expected:?}"
        );
    }
}

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

/// Issue #479: the docs hub links to a Rust API reference at `/docs/api/`. The
/// deploy-demo job generates it with `cargo doc` and copies it into the Pages
/// artifact — and the copy must run *after* stamping (rustdoc HTML carries no
/// version placeholders, so copying post-stamp keeps the large generated tree
/// out of the placeholder scan).
#[test]
fn deploy_demo_generates_api_docs_and_copies_them_after_stamping() {
    let workflow = release_workflow();
    let deploy = job_block(&workflow, "deploy-demo");

    assert!(
        deploy.contains("cargo doc --no-deps --lib"),
        "deploy-demo should build the Rust API docs with cargo doc"
    );
    assert!(
        deploy.contains("cp -R target/doc/. src/web/docs/api/"),
        "deploy-demo should copy the generated docs into src/web/docs/api/"
    );

    let stamp_pos = deploy
        .find("Stamp GitHub Pages artifact")
        .expect("deploy-demo should stamp the Pages artifact");
    let copy_pos = deploy
        .find("Copy Rust API docs into the Pages artifact")
        .expect("deploy-demo should copy the API docs");
    assert!(
        stamp_pos < copy_pos,
        "the API-docs copy must run after the stamp step so rustdoc HTML is not scanned for placeholders"
    );

    // cargo doc emits the crate root under target/doc/formal_ai/ (lib name
    // formal_ai); a redirect at the doc root keeps /docs/api/ from 404ing.
    assert!(
        deploy.contains("url=formal_ai/index.html"),
        "a redirect should point /docs/api/ at the crate root"
    );
}

#[test]
fn crate_package_manifest_uses_publish_allowlist() {
    let manifest = fs::read_to_string(format!("{}/Cargo.toml", env!("CARGO_MANIFEST_DIR")))
        .unwrap()
        .replace("\r\n", "\n");

    assert!(
        manifest.contains("include = ["),
        "Cargo.toml should explicitly allowlist files published to crates.io"
    );

    for required in [
        "\"/Cargo.lock\"",
        "\"/Cargo.toml\"",
        "\"/LICENSE\"",
        "\"/README.md\"",
        "\"/data/**\"",
        "\"/src/**\"",
    ] {
        assert!(
            manifest.contains(required),
            "Cargo.toml package include list should contain {required}"
        );
    }

    for excluded in [
        "\"/docs/**\"",
        "\"/tests/**\"",
        "\"/scripts/**\"",
        "\"/examples/**\"",
        "\"/experiments/**\"",
        "\"/.github/**\"",
    ] {
        assert!(
            !manifest.contains(excluded),
            "Cargo.toml should use an include allowlist instead of carrying explicit excluded repository artifacts"
        );
    }
}

#[test]
fn build_job_checks_generated_crate_archive_size() {
    let workflow = release_workflow();
    let build = job_block(&workflow, "build");
    let package_list = build
        .find("- name: Check package")
        .expect("build job should list package contents");
    let package_size = build
        .find("- name: Check crate package size")
        .expect("build job should check the generated crate archive size");
    let install_rust_script = build
        .find("- name: Install rust-script")
        .expect("build job should install rust-script before running script guards");

    assert!(
        package_list < package_size,
        "build job should list package contents before checking archive size"
    );
    assert!(
        install_rust_script < package_size,
        "build job should install rust-script before checking archive size"
    );
    assert!(
        build.contains("rust-script scripts/check-crate-package-size.rs"),
        "build job should run the crate size guard"
    );
}

#[test]
fn release_workflow_publishes_optional_docker_hub_image_after_crate_is_visible() {
    let workflow = release_workflow();

    assert!(
        workflow.contains("DOCKERHUB_IMAGE: ${{ vars.DOCKERHUB_IMAGE }}"),
        "workflow should expose an opt-in Docker Hub image variable"
    );
    assert_eq!(
        workflow.matches("docker/login-action@v4").count(),
        2,
        "auto and manual release jobs should log in to Docker Hub when configured"
    );
    assert_eq!(
        workflow.matches("docker/metadata-action@v6").count(),
        2,
        "auto and manual release jobs should derive Docker tags for Docker Hub"
    );
    assert_eq!(
        workflow.matches("docker/build-push-action@v7").count(),
        2,
        "auto and manual release jobs should publish Docker Hub images when configured"
    );
    assert!(
        workflow.contains("password: ${{ env.DOCKERHUB_TOKEN }}"),
        "Docker Hub login should use DOCKERHUB_TOKEN"
    );

    let auto_release = job_block(&workflow, "auto-release");
    let auto_publish = auto_release
        .find("- name: Publish to Crates.io")
        .expect("auto release should publish the crate");
    let auto_wait = auto_release
        .find("- name: Wait for Crate availability on Crates.io")
        .expect("auto release should wait for the crate");
    let auto_docker = auto_release
        .find("- name: Publish Docker image to Docker Hub")
        .expect("auto release should publish the Docker image");
    let auto_github_release = auto_release
        .find("- name: Create GitHub Release")
        .expect("auto release should create a GitHub release");

    assert!(
        auto_publish < auto_wait && auto_wait < auto_docker && auto_docker < auto_github_release,
        "auto release should publish crates.io first, then Docker Hub, then GitHub release"
    );

    let manual_release = job_block(&workflow, "manual-release");
    let manual_publish = manual_release
        .find("- name: Publish to Crates.io")
        .expect("manual release should publish the crate");
    let manual_wait = manual_release
        .find("- name: Wait for Crate availability on Crates.io")
        .expect("manual release should wait for the crate");
    let manual_docker = manual_release
        .find("- name: Publish Docker image to Docker Hub")
        .expect("manual release should publish the Docker image");
    let manual_github_release = manual_release
        .find("- name: Create GitHub Release")
        .expect("manual release should create a GitHub release");

    assert!(
        manual_publish < manual_wait
            && manual_wait < manual_docker
            && manual_docker < manual_github_release,
        "manual release should publish crates.io first, then Docker Hub, then GitHub release"
    );
}

#[test]
fn release_workflow_defers_rate_limited_crates_publish_without_downstream_artifacts() {
    let workflow = release_workflow();
    let auto_release = job_block(&workflow, "auto-release");

    for step_name in [
        "Wait for Crate availability on Crates.io",
        "Configure Docker Hub publishing",
        "Create GitHub Release",
    ] {
        let step = workflow_step_block(auto_release, step_name);
        assert!(
            step.contains("steps.check.outputs.crate_published == 'true'"),
            "auto-release {step_name} should still run when the crate was already published"
        );
        assert!(
            step.contains("steps.publish-crate.outputs.publish_result == 'success'"),
            "auto-release {step_name} should wait for a successful crates.io publish before creating downstream artifacts"
        );
        assert!(
            !step.contains("steps.check.outputs.should_release == 'true'\n"),
            "auto-release {step_name} should not run solely because a release is needed"
        );
    }

    let manual_release = job_block(&workflow, "manual-release");
    for step_name in [
        "Wait for Crate availability on Crates.io",
        "Configure Docker Hub publishing",
        "Create GitHub Release",
    ] {
        let step = workflow_step_block(manual_release, step_name);
        assert!(
            step.contains("steps.publish-crate.outputs.publish_result == 'success'"),
            "manual-release {step_name} should wait for a successful crates.io publish before creating downstream artifacts"
        );
        assert!(
            !step.contains(
                "steps.version.outputs.version_committed == 'true' || steps.version.outputs.already_released == 'true'\n"
            ),
            "manual-release {step_name} should not run solely because a version step completed"
        );
    }
}

#[test]
fn desktop_release_workflow_run_resolves_child_release_not_head_sha_tag() {
    // Issue #479: the automated release tags a CHILD "chore: release vX.Y.Z"
    // commit (its first parent is the completed CI head SHA) and is pushed with
    // GITHUB_TOKEN, so GitHub never starts a CI run for it and suppresses the
    // `release` event. The previous resolve logic required a tag whose commit
    // EQUALS the workflow_run head SHA -- a match that could never happen -- so
    // the build was skipped and every /download entry read "Not available in
    // latest release".
    //
    // The corrected resolve logic lives in scripts/desktop-release-resolve.sh
    // (behaviorally covered by desktop_release_resolve.rs). This guard pins the
    // workflow wiring and the absence of the old, broken skip path.
    let workflow = desktop_release_workflow();
    let resolve = job_block(&workflow, "resolve");
    let pick = workflow_step_block(resolve, "Resolve tag and whether desktop assets are needed");
    let resolve_script = fs::read_to_string(format!(
        "{}/scripts/desktop-release-resolve.sh",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap()
    .replace("\r\n", "\n");

    assert!(
        pick.contains("WORKFLOW_RUN_HEAD_SHA: ${{ github.event.workflow_run.head_sha }}"),
        "workflow_run desktop builds should pass the completed CI run head SHA to the resolve script"
    );
    assert!(
        pick.contains("bash scripts/desktop-release-resolve.sh"),
        "the resolve step should delegate to the unit-tested resolve script"
    );
    assert!(
        resolve.contains("actions/checkout@v6"),
        "the resolve job must check out the repo so the resolve script is available"
    );

    // The old, broken behavior must not come back: never skip merely because no
    // tag points at the head SHA (the auto-release tag never does).
    assert!(
        !workflow.contains("No release tag points at workflow_run head SHA"),
        "issue #479 regression: must not skip when no tag matches the head SHA"
    );
    assert!(
        !resolve_script.contains("No release tag points at workflow_run head SHA"),
        "issue #479 regression: the resolve script must not reinstate the head-SHA skip"
    );

    // The corrected script must fall back to the latest release and keep the
    // self-healing idempotency guard.
    assert!(
        resolve_script.contains("latest_release_tag()"),
        "resolve script should fall back to the latest published release"
    );
    assert!(
        resolve_script.contains(r#"select(.commit.sha == \"$WORKFLOW_RUN_HEAD_SHA\")"#),
        "resolve script should keep the defensive exact-SHA tier"
    );
    assert!(
        resolve_script.contains("formal-ai-desktop-"),
        "resolve script should count existing desktop assets for the idempotency guard"
    );
    assert!(
        resolve_script.contains("::group::"),
        "resolve script should emit grouped verbose diagnostics for future debugging"
    );
}

#[test]
fn desktop_release_runs_on_any_completed_main_pipeline_not_only_success() {
    // Issue #479 (root cause, take 2): PR #480 fixed the resolve *script* but
    // left the resolve *job* gated behind `workflow_run.conclusion == 'success'`.
    // The auto-release publishes the GitHub release in an EARLY pipeline job, so
    // any LATER job failing (e.g. the E2E Pages probe timing out) made the whole
    // pipeline conclude `failure`, the gate skipped, and no desktop assets were
    // ever built -- the fix stayed dormant and /download still read "Not
    // available in latest release". The gate must run on ANY completed main-branch
    // pipeline except cancelled/skipped, delegating the real build decision to the
    // self-healing resolve script + its idempotency guard.
    let workflow = desktop_release_workflow();
    let resolve = job_block(&workflow, "resolve");

    assert!(
        resolve.contains("github.event.workflow_run.head_branch == 'main'"),
        "desktop release should still only auto-build for main-branch pipelines"
    );
    assert!(
        resolve.contains("github.event.workflow_run.conclusion != 'cancelled'")
            && resolve.contains("github.event.workflow_run.conclusion != 'skipped'"),
        "desktop release should run on any completed main pipeline except cancelled/skipped"
    );
    assert!(
        !resolve.contains("github.event.workflow_run.conclusion == 'success'"),
        "issue #479 regression: desktop release must NOT gate on full-pipeline success -- a late \
         unrelated failure (e.g. E2E Pages timeout) would again suppress every desktop build"
    );
}

#[test]
fn desktop_release_lets_electron_builder_read_package_json_build_key() {
    let workflow = desktop_release_workflow();
    let package_json = fs::read_to_string(format!(
        "{}/desktop/package.json",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap();
    let smoke = fs::read_to_string(format!(
        "{}/desktop/scripts/smoke.mjs",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap();

    assert!(
        workflow.contains("npx --no-install electron-builder ${{ matrix.ebflag }} --publish never"),
        "desktop release workflow should invoke electron-builder without passing package.json as a config file"
    );
    assert!(
        !workflow.contains("--config package.json"),
        "package.json is an app manifest; passing it as --config makes electron-builder reject its top-level build key"
    );
    assert!(
        !package_json.contains("--config package.json"),
        "desktop npm build scripts should not pass package.json as an explicit electron-builder config file"
    );
    assert!(
        smoke.contains("--config package.json"),
        "desktop smoke checks should guard against reintroducing the invalid config flag"
    );
}

#[test]
fn release_scripts_check_configured_release_artifacts() {
    let release_check = fs::read_to_string(format!(
        "{}/scripts/check-release-needed.rs",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap();
    let wait_for_crate = fs::read_to_string(format!(
        "{}/scripts/wait-for-crate.rs",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap();
    let release_script = fs::read_to_string(format!(
        "{}/scripts/create-github-release.rs",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap();

    assert!(
        release_check.contains("check_docker_hub_tag"),
        "release-needed check should verify configured Docker Hub tags"
    );
    assert!(
        release_check.contains("check_docker_hub_tag(image, \"latest\")"),
        "release-needed check should verify Docker Hub latest tags as part of completeness"
    );
    assert!(
        release_check.contains("check_github_release"),
        "release-needed check should verify GitHub release artifacts"
    );
    assert!(
        release_check.contains("crate_published"),
        "release-needed check should output whether the crate already exists"
    );
    assert!(
        wait_for_crate.contains("crates.io/api/v1/crates"),
        "release workflow should wait for crates.io visibility before image publishing"
    );
    assert!(
        wait_for_crate.contains("example-sum-package-name")
            && wait_for_crate.contains("crate_available\", \"skipped\""),
        "crate availability wait should preserve template-safe publishing skips"
    );
    assert!(
        release_script.contains("--docker-hub-url"),
        "GitHub release creation should accept a Docker Hub URL"
    );
    assert!(
        release_script.contains("fn docker_hub_badge"),
        "GitHub release notes should include Docker Hub badge support"
    );
}
