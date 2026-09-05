use std::fs;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use formal_ai::{CachedSourceClient, FetchError, SourceTransport, execute_duckduckgo_search};

static TEMP_IDS: AtomicUsize = AtomicUsize::new(0);

const DUCKDUCKGO_HTML: &[u8] = br#"
<html><body>
  <div class="result__body">
    <a class="result__a" href="//duckduckgo.com/l/?uddg=https%3A%2F%2Fcomponent.invalid%2Fone">Component result one</a>
    <a class="result__snippet">First exact component capture.</a>
  </div>
  <div class="result__body">
    <a class="result__a" href="https://component.invalid/two">Component result two</a>
    <a class="result__snippet">Second exact component capture.</a>
  </div>
</body></html>
"#;

#[derive(Clone, Default)]
struct ComponentFixtureTransport {
    urls: Arc<Mutex<Vec<String>>>,
}

#[derive(Clone, Default)]
struct FallbackFixtureTransport {
    urls: Arc<Mutex<Vec<String>>>,
}

#[derive(Clone, Default)]
struct FailedBothFixtureTransport;

impl SourceTransport for FailedBothFixtureTransport {
    fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
        let boundary = if url.starts_with("https://html.duckduckgo.com/html/") {
            "web-capture component unavailable"
        } else {
            "Instant Answer fallback unavailable"
        };
        Err(FetchError::Transport(String::from(boundary)))
    }
}

impl SourceTransport for FallbackFixtureTransport {
    fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
        self.urls
            .lock()
            .expect("fixture URL lock")
            .push(url.to_owned());
        if url.starts_with("https://html.duckduckgo.com/html/") {
            return Err(FetchError::Transport(String::from(
                "web-capture component unavailable",
            )));
        }
        if url.starts_with("https://api.duckduckgo.com/") {
            return Ok(
                br#"{"AbstractURL":"https://fallback.invalid/result","AbstractText":"Bounded fallback result","RelatedTopics":[]}"#
                    .to_vec(),
            );
        }
        Err(FetchError::Transport(format!("unexpected URL: {url}")))
    }
}

impl SourceTransport for ComponentFixtureTransport {
    fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
        self.urls
            .lock()
            .expect("fixture URL lock")
            .push(url.to_owned());
        if url == "https://html.duckduckgo.com/html/?q=formal+ai" {
            return Ok(DUCKDUCKGO_HTML.to_vec());
        }
        Err(FetchError::Transport(format!("unexpected URL: {url}")))
    }
}

const fn fixed_time() -> u64 {
    1_753_444_800
}

fn temp_cache() -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!(
        "formal-ai-issue-896-{}-{}",
        std::process::id(),
        TEMP_IDS.fetch_add(1, Ordering::SeqCst)
    ));
    let _ = fs::remove_dir_all(&path);
    path
}

#[test]
fn native_search_executes_both_published_component_boundaries() {
    let cache = temp_cache();
    let transport = ComponentFixtureTransport::default();
    let urls = Arc::clone(&transport.urls);
    let online = CachedSourceClient::new(&cache, transport.clone())
        .with_online(true)
        .with_clock(fixed_time);

    let live = execute_duckduckgo_search(&online, "formal ai")
        .expect("web-capture HTML should be normalized and fused");
    assert_eq!(
        urls.lock().expect("fixture URL lock").as_slice(),
        ["https://html.duckduckgo.com/html/?q=formal+ai"]
    );
    assert_eq!(live.captures.len(), 1);
    assert_eq!(live.captures[0].bytes(), DUCKDUCKGO_HTML);
    assert!(!live.captures[0].cached());
    assert_eq!(live.rankings.len(), 2);
    assert_eq!(live.rankings[0].title, "Component result one");
    assert_eq!(live.fused.len(), 2);
    assert_eq!(live.fused[0].providers, [(String::from("duckduckgo"), 1)]);
    assert_eq!(
        live.component_boundaries,
        ["web-capture:search", "web-search:merger"]
    );
    assert!(live.component_diagnostics.is_empty());

    let offline = CachedSourceClient::new(&cache, transport);
    let replay = execute_duckduckgo_search(&offline, "formal ai")
        .expect("component capture should replay without transport");
    assert!(replay.captures[0].cached());
    assert_eq!(replay.rankings, live.rankings);
    assert_eq!(replay.fused, live.fused);
    assert_eq!(replay.component_boundaries, live.component_boundaries);
    assert_eq!(replay.component_diagnostics, live.component_diagnostics);
    assert_eq!(urls.lock().expect("fixture URL lock").len(), 1);

    fs::remove_dir_all(cache).expect("remove fixture cache");
}

#[test]
fn native_search_reports_component_failure_before_bounded_fallback() {
    let cache = temp_cache();
    let transport = FallbackFixtureTransport::default();
    let urls = Arc::clone(&transport.urls);
    let client = CachedSourceClient::new(&cache, transport.clone()).with_online(true);

    let execution = execute_duckduckgo_search(&client, "fallback")
        .expect("Instant Answer compatibility path should remain available");
    assert_eq!(execution.rankings.len(), 1);
    assert_eq!(execution.rankings[0].title, "Bounded fallback result");
    assert_eq!(
        execution.component_boundaries,
        ["web-capture:search:fallback", "web-search:merger"]
    );
    assert_eq!(execution.component_diagnostics.len(), 1);
    assert_eq!(
        execution.component_diagnostics,
        ["web-capture:search:unavailable"]
    );
    assert_eq!(urls.lock().expect("fixture URL lock").len(), 2);

    let offline = CachedSourceClient::new(&cache, transport);
    let replay = execute_duckduckgo_search(&offline, "fallback")
        .expect("fallback capture should replay without transport");
    assert!(replay.captures[0].cached());
    assert_eq!(replay.rankings, execution.rankings);
    assert_eq!(replay.fused, execution.fused);
    assert_eq!(replay.component_boundaries, execution.component_boundaries);
    assert_eq!(
        replay.component_diagnostics,
        execution.component_diagnostics
    );
    assert_eq!(urls.lock().expect("fixture URL lock").len(), 2);

    fs::remove_dir_all(cache).expect("remove fixture cache");
}

#[test]
fn native_search_reports_both_component_and_fallback_failures() {
    let cache = temp_cache();
    let client = CachedSourceClient::new(&cache, FailedBothFixtureTransport).with_online(true);

    let error = execute_duckduckgo_search(&client, "unavailable")
        .expect_err("both unavailable paths must return an error")
        .to_string();
    assert!(error.contains("web-capture component unavailable"));
    assert!(error.contains("Instant Answer fallback unavailable"));

    let _ = fs::remove_dir_all(cache);
}

#[test]
fn published_web_search_merger_remains_available_without_the_server_feature() {
    let by_provider = std::collections::HashMap::from([(
        String::from("duckduckgo"),
        vec![web_search::SearchResult {
            title: String::from("Component result"),
            url: String::from("https://component.invalid/result"),
            snippet: String::from("Transport-independent merge input"),
            source: String::from("duckduckgo"),
            rank: 1,
            score: None,
            sources: None,
        }],
    )]);

    let merged = web_search::merger::merge_results(
        &by_provider,
        &web_search::MergeOptions::new().with_rrf_k(60.0),
    );

    assert_eq!(merged.len(), 1);
    assert_eq!(merged[0].url, "https://component.invalid/result");
    assert_eq!(merged[0].source, "duckduckgo");
}

#[test]
fn desktop_budget_bounds_the_published_component_cold_build() {
    let workflow = fs::read_to_string(format!(
        "{}/.github/workflows/desktop-release.yml",
        env!("CARGO_MANIFEST_DIR")
    ))
    .expect("desktop release workflow");

    // Issue #1017 moved the cap out of an inline expression and into the matrix
    // (`capmin`), so the macOS packaging retry guard can be derived from the same
    // number instead of a second copy of it. The guarantee issue #896 needs is
    // unchanged, so it is asserted against the values rather than against one
    // expression's spelling: the job is bounded by its matrix cap, every packaged
    // target carries one, and the three legs that pay for the published crates'
    // unconditional graph carry strictly more headroom than the rest.
    assert!(
        workflow.contains("    timeout-minutes: ${{ matrix.capmin }}\n"),
        "the desktop build job must stay bounded by a cap it declares"
    );

    let mut heavy = Vec::new();
    let mut light = Vec::new();
    for entry in workflow.lines().filter(|line| line.contains("capmin:")) {
        let label = entry
            .split("label: \"")
            .nth(1)
            .and_then(|tail| tail.split('"').next())
            .unwrap_or_else(|| panic!("matrix entry without a label: {entry}"));
        let capmin: u32 = entry
            .split("capmin:")
            .nth(1)
            .map(|tail| tail.trim_start().trim_end_matches([' ', '}']).trim())
            .and_then(|value| value.parse().ok())
            .unwrap_or_else(|| panic!("matrix entry without a numeric capmin: {entry}"));
        if label == "macos-x64" || label.starts_with("windows-") {
            heavy.push((label.to_string(), capmin));
        } else {
            light.push((label.to_string(), capmin));
        }
    }

    assert_eq!(
        heavy.len(),
        3,
        "macOS x64 and both Windows targets must each declare a cap"
    );
    assert!(!light.is_empty(), "the remaining targets must declare caps");
    let smallest_heavy = heavy.iter().map(|(_, cap)| *cap).min().expect("heavy caps");
    let largest_light = light.iter().map(|(_, cap)| *cap).max().expect("light caps");
    assert!(
        smallest_heavy > largest_light,
        "macOS x64 and both Windows targets need bounded headroom for the published crates' \
         unconditional graph, but the caps are {heavy:?} against {light:?}"
    );
}

/// The floor issue #896 measured for `Build Package`: a cold cache has to
/// compile the published crates' unconditional graph inside it. Raising the cap
/// above this is a matter for the measurement that motivates it (issue #1076
/// raised it to 20); dropping below it is the regression this test exists for.
const MIN_BUILD_PACKAGE_CAP_MINUTES: u32 = 15;

#[test]
fn build_package_budget_bounds_the_published_component_cold_build() {
    let workflow = fs::read_to_string(format!(
        "{}/.github/workflows/release.yml",
        env!("CARGO_MANIFEST_DIR")
    ))
    .expect("release workflow");

    let build = workflow
        .split("  build:\n")
        .nth(1)
        .expect("Build Package job")
        .split("\n  auto-release:")
        .next()
        .expect("Build Package job body");

    let cap: u32 = build
        .lines()
        .find_map(|line| line.strip_prefix("    timeout-minutes:"))
        .and_then(|value| value.trim().parse().ok())
        .expect("Build Package must declare a job cap in whole minutes");

    // The invariant issue #896 established is headroom, so this is a floor, not
    // an equality. Pinning the exact number made a *raise* fail: issue #1076
    // measured the job and moved the cap from 15 to 20, which gives the cold
    // build more room, not less, and this assertion still reported it as a
    // broken boundary (D20).
    assert!(
        cap >= MIN_BUILD_PACKAGE_CAP_MINUTES,
        "the release build needs headroom for the published crates' unconditional graph \
         on a cold cache: {cap}m is below the {MIN_BUILD_PACKAGE_CAP_MINUTES}m floor"
    );
}

#[test]
fn same_task_agent_cli_authorship_is_preserved() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let read = |path: &str| {
        fs::read_to_string(root.join(path)).unwrap_or_else(|error| panic!("{path}: {error}"))
    };
    let session = "ses_03b4c6698ffedcgrdsX2ni15WK";
    let generated = read(
        "docs/case-studies/issue-896/self-hosting-authorship/web-component-boundary-invariant.lino",
    );
    let canonical = read("data/meta/web-component-boundary-invariant.lino");
    assert_eq!(generated, canonical);

    let agent_log = read("docs/case-studies/issue-896/self-hosting-authorship/agent-cli.log");
    assert!(agent_log.contains(session));
    let formal_ai_log = read("docs/case-studies/issue-896/self-hosting-authorship/formal-ai.log");
    for transition in [
        "planned ToolCalls",
        "tool=write",
        "tool: \"bash\"",
        "planned Final",
        "web-component-boundary-invariant.lino",
    ] {
        assert!(
            formal_ai_log.contains(transition),
            "server trace is missing {transition}"
        );
    }

    let decomposition =
        read("docs/case-studies/issue-896/self-hosting-authorship/decomposition.lino");
    assert_eq!(decomposition.matches("issue_896_smallest_leaf_").count(), 5);
    assert_eq!(
        decomposition
            .matches("authorship formal_ai_agent_cli")
            .count(),
        1
    );
    assert!(decomposition.contains(&format!("session {session}")));
    assert!(decomposition.contains("formal_ai_authored_percent 20"));
}
