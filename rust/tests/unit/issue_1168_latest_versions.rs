//! Issue #1168: generated code must use the latest versions discovered at
//! generation time -- the resolution machinery lives in
//! `rust/src/version_resolution.rs` and is exercised below through its
//! public API against fixture transports; this file also pins the data-side
//! half: the `data/` gate that fails on any memorized
//! third-party action SHA or toolchain version.
//!
//! Both template files (`data/meta/stdout-program-contracts.lino`,
//! `data/meta/work-item-steps.lino`) carry `{placeholders}` only (R6), so
//! the gate scans the repository's `data/` clean.

#[path = "../../../scripts/check-generated-version-literals.rs"]
mod check_generated_version_literals;

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use check_generated_version_literals::scan_data;
use formal_ai::event_log::EventLog;
use formal_ai::source_fetch::{CachedSourceClient, FetchError, SourceTransport};
use formal_ai::version_resolution::{Origin, VersionSet, fill_workflow_versions};

fn repository_data() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the repository root sits one level above the crate")
        .join("data")
}

fn fixture_tree(tag: &str, files: &[(&str, &str)]) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "issue-1168-gate-{tag}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos())
    ));
    for (name, body) in files {
        let path = root.join("data").join(name);
        fs::create_dir_all(path.parent().expect("a directory")).expect("fixture dirs");
        fs::write(path, body).expect("fixture file");
    }
    root.join("data")
}

#[test]
fn the_gate_fails_on_a_memorized_action_sha() {
    let data = fixture_tree(
        "sha",
        &[(
            "meta/template.lino",
            "  workflow_template \"      - uses: actions/checkout@11d5960a326750d5838078e36cf38b85af677262\n\"\n",
        )],
    );
    let findings = scan_data(&data);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings[0].path.ends_with("data/meta/template.lino"));
    assert_eq!(findings[0].line, 1);
}

#[test]
fn the_gate_fails_on_a_memorized_toolchain_version() {
    let data = fixture_tree(
        "version",
        &[(
            "meta/contract.lino",
            "    ci_setup \"        with:\n          version: '2.3.10'\n\"\n",
        )],
    );
    let findings = scan_data(&data);
    assert!(
        findings.iter().any(|f| f.detail.contains("version")),
        "{findings:?}"
    );
}

#[test]
fn placeholder_templates_and_the_baseline_seed_scan_clean() {
    let data = fixture_tree(
        "clean",
        &[
            (
                "meta/template.lino",
                "  workflow_template \"      - uses: actions/checkout@{checkout_ref} # {checkout_tag}\n\"\n",
            ),
            (
                "seed/toolchains.lino",
                "generated_version actions_checkout\n  latest_tag \"v7.0.1\"\n  latest_sha \"3d3c42e5aac5ba805825da76410c181273ba90b1\"\n",
            ),
        ],
    );
    assert!(scan_data(&data).is_empty());
}

#[test]
fn captured_evidence_under_cache_is_not_generated_code() {
    let data = fixture_tree(
        "cache",
        &[(
            "cache/wiktionary/en/example.lino",
            "  etymology \"pinned in a captured page: something@0123456789abcdef0123456789abcdef01234567\"\n",
        )],
    );
    assert!(scan_data(&data).is_empty());
}

/// R1168-6/R1168-7: both templates now carry `{placeholders}` only, so the
/// repository's own `data/` scans clean.
#[test]
fn repository_data_carries_no_memorized_pin() {
    let findings = scan_data(&repository_data());
    let described = findings
        .iter()
        .map(|finding| {
            format!(
                "{}:{} {}",
                finding.path.display(),
                finding.line,
                finding.detail
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(described, Vec::<String>::new());
}

// --- version resolution (moved from the in-crate test module) ---

const CHECKOUT_RELEASE: &str = r#"{"tag_name":"v7.0.1"}"#;
const CHECKOUT_COMMIT: &str = r#"{"sha":"3d3c42e5aac5ba805825da76410c181273ba90b1"}"#;
const SETUP_JAVA_RELEASE: &str = r#"{"tag_name":"v6.0.1"}"#;
const SETUP_JAVA_COMMIT: &str = r#"{"sha":"de7274f081f381c8f8158605e0321c36c376e2e6"}"#;
const SETUP_KOTLIN_RELEASE: &str = r#"{"tag_name":"v2.0"}"#;
const SETUP_KOTLIN_COMMIT: &str = r#"{"sha":"ee9692514da313706b193d808526812102a344e4"}"#;
const SETUP_PYTHON_RELEASE: &str = r#"{"tag_name":"v7.0.0"}"#;
const SETUP_PYTHON_COMMIT: &str = r#"{"sha":"5fda3b95a4ea91299a34e894583c3862153e4b97"}"#;
const CPYTHON_RELEASE: &str = r#"{"tag_name":"v3.14.7"}"#;
const CPYTHON_COMMIT: &str = r#"{"sha":"0000000000000000000000000000000000000000"}"#;
const KOTLIN_RELEASE: &str = r#"{"tag_name":"v2.4.20"}"#;
const KOTLIN_COMMIT: &str = r#"{"sha":"0000000000000000000000000000000000000000"}"#;
const ADOPTIUM: &str = r#"{"most_recent_lts":25}"#;
const SETUP_COURSIER_RELEASE: &str = r#"{"tag_name":"v3.0.4"}"#;
const SETUP_COURSIER_COMMIT: &str = r#"{"sha":"648df969f41ef15fda2baba8b37f9fa3d16390a3"}"#;
const SCALA_RELEASE: &str = r#"{"tag_name":"v2.13.18"}"#;
const SCALA_COMMIT: &str = r#"{"sha":"0000000000000000000000000000000000000000"}"#;

#[derive(Default)]
struct MockTransport {
    responses: HashMap<String, &'static str>,
}

impl MockTransport {
    fn seeded() -> Self {
        let mut responses = HashMap::new();
        for (url, body) in [
            (
                "https://api.github.com/repos/actions/checkout/releases/latest",
                CHECKOUT_RELEASE,
            ),
            (
                "https://api.github.com/repos/actions/checkout/commits/v7.0.1",
                CHECKOUT_COMMIT,
            ),
            (
                "https://api.github.com/repos/actions/setup-java/releases/latest",
                SETUP_JAVA_RELEASE,
            ),
            (
                "https://api.github.com/repos/actions/setup-java/commits/v6.0.1",
                SETUP_JAVA_COMMIT,
            ),
            (
                "https://api.github.com/repos/fwilhe2/setup-kotlin/releases/latest",
                SETUP_KOTLIN_RELEASE,
            ),
            (
                "https://api.github.com/repos/fwilhe2/setup-kotlin/commits/v2.0",
                SETUP_KOTLIN_COMMIT,
            ),
            (
                "https://api.github.com/repos/actions/setup-python/releases/latest",
                SETUP_PYTHON_RELEASE,
            ),
            (
                "https://api.github.com/repos/actions/setup-python/commits/v7.0.0",
                SETUP_PYTHON_COMMIT,
            ),
            (
                "https://api.github.com/repos/python/cpython/releases/latest",
                CPYTHON_RELEASE,
            ),
            (
                "https://api.github.com/repos/python/cpython/commits/v3.14.7",
                CPYTHON_COMMIT,
            ),
            (
                "https://api.github.com/repos/JetBrains/kotlin/releases/latest",
                KOTLIN_RELEASE,
            ),
            (
                "https://api.github.com/repos/JetBrains/kotlin/commits/v2.4.20",
                KOTLIN_COMMIT,
            ),
            (
                "https://api.adoptium.net/v3/info/available_releases",
                ADOPTIUM,
            ),
            (
                "https://api.github.com/repos/coursier/setup-action/releases/latest",
                SETUP_COURSIER_RELEASE,
            ),
            (
                "https://api.github.com/repos/coursier/setup-action/commits/v3.0.4",
                SETUP_COURSIER_COMMIT,
            ),
            (
                "https://api.github.com/repos/scala/scala/releases/latest",
                SCALA_RELEASE,
            ),
            (
                "https://api.github.com/repos/scala/scala/commits/v2.13.18",
                SCALA_COMMIT,
            ),
        ] {
            responses.insert(url.to_owned(), body);
        }
        Self { responses }
    }

    fn online(cache_dir: PathBuf) -> CachedSourceClient<Self> {
        CachedSourceClient::new(cache_dir, Self::seeded())
            .with_online(true)
            .with_clock(fixed_clock)
    }
}

impl SourceTransport for MockTransport {
    fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
        self.responses
            .get(url)
            .map(|body| body.as_bytes().to_vec())
            .ok_or_else(|| FetchError::Transport(format!("no fixture for {url}")))
    }
}

const fn fixed_clock() -> u64 {
    1_800_000_000
}

fn temp_dir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "issue-1168-{tag}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos())
    ));
    fs::create_dir_all(&dir).expect("temp dir");
    dir
}

#[test]
fn resolves_every_pin_from_live_fixtures() {
    let dir = temp_dir("live");
    let client = MockTransport::online(dir);
    let versions = VersionSet::resolve(&client);
    assert_eq!(versions.checkout.tag, "v7.0.1");
    assert_eq!(
        versions.checkout.sha,
        "3d3c42e5aac5ba805825da76410c181273ba90b1"
    );
    assert_eq!(versions.checkout.origin, Origin::Live);
    assert_eq!(versions.setup_java.tag, "v6.0.1");
    assert_eq!(versions.setup_java.origin, Origin::Live);
    assert_eq!(versions.setup_kotlin.tag, "v2.0");
    assert_eq!(versions.setup_python.tag, "v7.0.0");
    assert_eq!(versions.setup_python.origin, Origin::Live);
    assert_eq!(versions.python_interpreter.tag, "3.14.7");
    assert_eq!(versions.kotlin.tag, "v2.4.20");
    assert_eq!(versions.java_lts.tag, "25");
    assert_eq!(versions.java_lts.sha, "");
    assert_eq!(versions.java_lts.origin, Origin::Live);
    assert_eq!(versions.setup_coursier.tag, "v3.0.4");
    assert_eq!(
        versions.setup_coursier.sha,
        "648df969f41ef15fda2baba8b37f9fa3d16390a3"
    );
    assert_eq!(versions.setup_coursier.origin, Origin::Live);
    assert_eq!(versions.scala.tag, "2.13.18");
    assert_eq!(versions.scala.origin, Origin::Live);
    assert_eq!(versions.provenance_note(), [] as [std::string::String; 0]);
}

#[test]
fn records_the_resolution_in_the_derivation() {
    let dir = temp_dir("record");
    let client = MockTransport::online(dir);
    let versions = VersionSet::resolve(&client);
    let mut log = EventLog::new();
    versions.record(&mut log);
    let joined = log
        .events()
        .iter()
        .map(|event| format!("{} {}", event.kind, event.payload))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(joined.contains("version_resolution"));
    assert!(joined.contains("actions_checkout tag=v7.0.1"));
    assert!(joined.contains("java_lts tag=25 sha= origin=live"));
    assert!(joined.contains("source:http"));
}

#[test]
fn offline_replays_the_cached_capture_and_says_so() {
    let dir = temp_dir("cache");
    let online = MockTransport::online(dir.clone());
    let warmed = VersionSet::resolve(&online);
    assert_eq!(warmed.checkout.origin, Origin::Live);
    drop(online);

    let offline = CachedSourceClient::new(dir, MockTransport::default())
        .with_online(false)
        .with_clock(fixed_clock);
    let versions = VersionSet::resolve(&offline);
    assert_eq!(versions.checkout.tag, "v7.0.1");
    assert_eq!(versions.checkout.origin, Origin::Cache);
    assert_eq!(versions.kotlin.tag, "v2.4.20");
    let note = versions.provenance_note().join("\n");
    assert!(
        note.contains("(version resolved from cache; fetched_at=1800000000)"),
        "the R5 annotation must name the cache and its timestamp: {note}"
    );
}

#[test]
fn empty_cache_falls_back_to_the_shipped_baseline() {
    let dir = temp_dir("baseline");
    let offline = CachedSourceClient::new(dir, MockTransport::default())
        .with_online(false)
        .with_clock(fixed_clock);
    let versions = VersionSet::resolve(&offline);
    assert_eq!(versions.checkout.origin, Origin::Baseline);
    assert_eq!(versions.checkout.tag, "v7.0.1");
    assert_eq!(versions.setup_java.tag, "v6.0.1");
    assert_eq!(versions.setup_kotlin.tag, "v2.0");
    assert_eq!(versions.kotlin.tag, "v2.4.20");
    assert_eq!(versions.java_lts.tag, "25");
    let note = versions.provenance_note().join("\n");
    assert!(note.contains("shipped baseline"), "{note}");
}

#[test]
fn the_toolchains_seed_carries_every_baseline() {
    let versions = VersionSet::baseline().expect("all five baselines shipped");
    assert_eq!(versions.checkout.tag, "v7.0.1");
    assert_eq!(
        versions.checkout.sha,
        "3d3c42e5aac5ba805825da76410c181273ba90b1"
    );
    assert_eq!(
        versions.setup_java.sha,
        "de7274f081f381c8f8158605e0321c36c376e2e6"
    );
    assert_eq!(
        versions.setup_kotlin.sha,
        "ee9692514da313706b193d808526812102a344e4"
    );
    assert_eq!(
        versions.setup_python.sha,
        "5fda3b95a4ea91299a34e894583c3862153e4b97"
    );
    assert_eq!(versions.python_interpreter.tag, "3.14.7");
    assert_eq!(versions.kotlin.tag, "v2.4.20");
    assert_eq!(versions.java_lts.tag, "25");
    assert_eq!(
        versions.setup_coursier.sha,
        "648df969f41ef15fda2baba8b37f9fa3d16390a3"
    );
    assert_eq!(versions.scala.tag, "2.13.18");
}

/// R1168-6: the Scala `ci_setup` row is placeholders only, so the coursier
/// ref, the JVM it installs, and the Scala version all come from the
/// resolved set.
#[test]
fn fills_the_scala_ci_setup_placeholders() {
    let versions = VersionSet::baseline().expect("baselines");
    let template = "      - uses: coursier/setup-action@{setup_coursier_ref} # {setup_coursier_tag}\n        with:\n          jvm: temurin:{java_lts}\n          apps: scala:{scala_version} scalac:{scala_version}\n";
    assert_eq!(
        fill_workflow_versions(template, &versions),
        "      - uses: coursier/setup-action@648df969f41ef15fda2baba8b37f9fa3d16390a3  # v3.0.4\n        with:\n          jvm: temurin:25\n          apps: scala:2.13.18 scalac:2.13.18"
    );
}

#[test]
fn rewrites_the_stale_ci_setup_literals() {
    let versions = VersionSet::baseline().expect("baselines");
    let stale = "      - uses: actions/setup-java@b6effb05e454b25005698d916606bdc6ffcbf961\n        with:\n          distribution: temurin\n          java-version: '21'\n      - uses: fwilhe2/setup-kotlin@51a059ff08b95e2b83aa952b5b46b696d5b615a0\n        with:\n          version: '2.3.10'\n";
    assert_eq!(
        fill_workflow_versions(stale, &versions),
        "      - uses: actions/setup-java@de7274f081f381c8f8158605e0321c36c376e2e6  # v6.0.1\n        with:\n          distribution: temurin\n          java-version: '25'\n      - uses: fwilhe2/setup-kotlin@ee9692514da313706b193d808526812102a344e4  # v2.0\n        with:\n          version: '2.4.20'"
    );
}

#[test]
fn rewrites_the_workflow_template_checkout_ref() {
    let versions = VersionSet::baseline().expect("baselines");
    let stale = "      - uses: actions/checkout@11d5960a326750d5838078e36cf38b85af677262\n";
    assert_eq!(
        fill_workflow_versions(stale, &versions),
        "      - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1  # v7.0.1"
    );
}

#[test]
fn fills_the_placeholder_forms() {
    let versions = VersionSet::baseline().expect("baselines");
    let template = "      - uses: actions/checkout@{checkout_ref} # {checkout_tag}\n";
    assert_eq!(
        fill_workflow_versions(template, &versions),
        "      - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1  # v7.0.1"
    );
}

#[test]
fn rewrites_the_stale_python_ci_setup_literals() {
    let versions = VersionSet::baseline().expect("baselines");
    let stale = "      - uses: actions/setup-python@ece7cb06caefa5fff74198d8649806c4678c61a1\n        with:\n          python-version: '3.14'\n";
    assert_eq!(
        fill_workflow_versions(stale, &versions),
        "      - uses: actions/setup-python@5fda3b95a4ea91299a34e894583c3862153e4b97  # v7.0.0\n        with:\n          python-version: '3.14.7'"
    );
}

#[test]
fn a_version_key_outside_the_setup_blocks_is_left_alone() {
    let versions = VersionSet::baseline().expect("baselines");
    let template =
        "      - uses: some/other-action@v1\n        with:\n          version: '9.9.9'\n";
    assert_eq!(
        fill_workflow_versions(template, &versions),
        template.trim_end_matches('\n')
    );
}
