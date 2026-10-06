//! Resolve the third-party versions generated code pins, at generation time.
//!
//! Issue #1168 (E133): the CI workflows Formal AI generates for other
//! repositories carried memorized action SHAs and toolchain versions, and a
//! memorized string goes stale. A generated workflow is owed the same
//! provenance as any other answer: the version it pins was *resolved* from
//! the publisher at generation time -- GitHub's releases API for actions and
//! for Kotlin, Adoptium's own statement of the current LTS for Java -- and
//! the resolution is recorded in the derivation as `SourceCapture`s.
//!
//! Resolution has three origins, in order: a live capture, the most recent
//! cached capture (`CachedSourceClient` replays valid captures offline), and
//! the shipped baseline measured in `data/seed/toolchains.lino`. Whichever
//! origin supplied a pin is said out loud, in a comment on the generated
//! workflow, so a reader never mistakes a baseline for a lookup.

use crate::seed::parser::{LinoNode, parse_lino};
use crate::source_fetch::{
    CachedSourceClient, CurlSourceTransport, FetchError, SourceCapture, SourceTransport,
};

/// The seed carrying the measured offline baseline.
const TOOLCHAINS: &str = include_str!("../embedded/data/seed/toolchains.lino");

/// The environment override for the source cache, matching the convention of
/// `solver_handler_how_synthesis`.
const CACHE_DIR_ENV: &str = "FORMAL_AI_SOURCE_CACHE_DIR";

/// Where a resolved version came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Origin {
    /// Fetched from the publisher during this generation.
    Live,
    /// Replayed from the most recent cached capture.
    Cache,
    /// The shipped `data/seed/toolchains.lino` baseline, measured by hand.
    Baseline,
}

impl Origin {
    const fn label(self) -> &'static str {
        match self {
            Self::Live => "live",
            Self::Cache => "cache",
            Self::Baseline => "baseline",
        }
    }
}

/// A resolved version and its provenance (issue #1168 R1-R4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedVersion {
    /// The release tag, e.g. `v7.0.1` or the LTS number `25`.
    pub tag: String,
    /// The 40-hex commit SHA the tag points at; empty where the publisher's
    /// latest is a version number and not a git ref (Java LTS, Kotlin).
    pub sha: String,
    /// How this pin was resolved.
    pub origin: Origin,
    /// The URL the resolution was made from.
    pub source_url: String,
    /// When the underlying capture was taken.
    pub fetched_at: String,
    /// The sha256 of the response the resolution was read from.
    pub response_sha256: String,
}

impl ResolvedVersion {
    /// The `uses:` ref to pin: the commit SHA, with the tag when there is no
    /// SHA to pin (R1 pins the SHA of the latest release).
    #[must_use]
    pub fn pinned_ref(&self) -> &str {
        if self.sha.is_empty() {
            &self.tag
        } else {
            &self.sha
        }
    }

    fn from_baseline(node: &LinoNode) -> Self {
        Self {
            tag: node.find_child_value("latest_tag").to_owned(),
            sha: node.find_child_value("latest_sha").to_owned(),
            origin: Origin::Baseline,
            source_url: node.find_child_value("releases_url").to_owned(),
            fetched_at: node.find_child_value("measured_at").to_owned(),
            response_sha256: String::new(),
        }
    }
}

/// Every pin a generated workflow needs, resolved together.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionSet {
    pub checkout: ResolvedVersion,
    pub setup_java: ResolvedVersion,
    pub setup_kotlin: ResolvedVersion,
    pub setup_python: ResolvedVersion,
    pub python_interpreter: ResolvedVersion,
    pub kotlin: ResolvedVersion,
    pub java_lts: ResolvedVersion,
    /// The captures the resolutions were read from, for the derivation.
    pub captures: Vec<SourceCapture>,
}

impl VersionSet {
    /// Resolve every pin through `client` (live or cached), falling back per
    /// pin to the shipped baseline when the publisher cannot be reached (R5).
    #[must_use]
    pub fn resolve<T: SourceTransport>(client: &CachedSourceClient<T>) -> Self {
        let mut captures = Vec::new();
        let checkout = Self::resolve_github(client, "actions", "checkout", &mut captures)
            .or_else(|| baseline("actions_checkout"));
        let setup_java = Self::resolve_github(client, "actions", "setup-java", &mut captures)
            .or_else(|| baseline("actions_setup_java"));
        let setup_kotlin = Self::resolve_github(client, "fwilhe2", "setup-kotlin", &mut captures)
            .or_else(|| baseline("setup_kotlin_action"));
        let setup_python = Self::resolve_github(client, "actions", "setup-python", &mut captures)
            .or_else(|| baseline("actions_setup_python"));
        // The interpreter is resolved from CPython's own releases; the tag
        // carries the leading `v` GitHub requires and the workflow wants the
        // bare version.
        let python_interpreter = Self::resolve_github(client, "python", "cpython", &mut captures)
            .map(|mut resolved| {
                resolved.tag = resolved.tag.trim_start_matches('v').to_owned();
                resolved
            })
            .or_else(|| baseline("python_interpreter"));
        let kotlin = Self::resolve_github(client, "JetBrains", "kotlin", &mut captures)
            .or_else(|| baseline("kotlin_compiler"));
        let java_lts = resolve_java_lts(client, &mut captures).or_else(|| baseline("java_lts"));
        Self {
            checkout: checkout.expect("the toolchains seed carries the checkout baseline"),
            setup_java: setup_java.expect("the toolchains seed carries the setup-java baseline"),
            setup_kotlin: setup_kotlin
                .expect("the toolchains seed carries the setup-kotlin baseline"),
            setup_python: setup_python
                .expect("the toolchains seed carries the setup-python baseline"),
            python_interpreter: python_interpreter
                .expect("the toolchains seed carries the CPython baseline"),
            kotlin: kotlin.expect("the toolchains seed carries the Kotlin baseline"),
            java_lts: java_lts.expect("the toolchains seed carries the Java LTS baseline"),
            captures,
        }
    }

    /// What generation uses when no client is handed in: cached captures if
    /// the source cache has them, otherwise the shipped baseline.
    #[must_use]
    pub fn for_generation() -> Self {
        let cache_dir = std::env::var(CACHE_DIR_ENV).unwrap_or_else(|_| String::from("data"));
        let client = CachedSourceClient::new(cache_dir, CurlSourceTransport);
        Self::resolve(&client)
    }

    /// The shipped baseline alone, with no cache and no network.
    #[must_use]
    pub fn baseline() -> Option<Self> {
        Some(Self {
            checkout: baseline("actions_checkout")?,
            setup_java: baseline("actions_setup_java")?,
            setup_kotlin: baseline("setup_kotlin_action")?,
            setup_python: baseline("actions_setup_python")?,
            python_interpreter: baseline("python_interpreter")?,
            kotlin: baseline("kotlin_compiler")?,
            java_lts: baseline("java_lts")?,
            captures: Vec::new(),
        })
    }

    fn resolve_github<T: SourceTransport>(
        client: &CachedSourceClient<T>,
        publisher: &str,
        repository: &str,
        captures: &mut Vec<SourceCapture>,
    ) -> Option<ResolvedVersion> {
        let releases_url =
            format!("https://api.github.com/repos/{publisher}/{repository}/releases/latest");
        let release = client.fetch(&releases_url).ok()?;
        let origin = if release.cached() {
            Origin::Cache
        } else {
            Origin::Live
        };
        let tag = json_string(&release, "tag_name")?;
        let commit_url =
            format!("https://api.github.com/repos/{publisher}/{repository}/commits/{tag}");
        let commit = client.fetch(&commit_url).ok()?;
        let sha = json_string(&commit, "sha")?;
        captures.push(commit);
        Some(ResolvedVersion {
            tag,
            sha,
            origin,
            source_url: releases_url,
            fetched_at: String::new(),
            response_sha256: String::new(),
        })
        .map(|mut resolved| {
            let commit = captures.last().expect("just pushed");
            resolved.fetched_at = commit.fetched_at().to_owned();
            resolved.response_sha256 = commit.sha256().to_owned();
            resolved
        })
    }

    /// Append the resolution to the derivation: every capture's provenance,
    /// then one line per pin naming what was pinned and how (R4).
    pub fn record(&self, log: &mut crate::event_log::EventLog) {
        for capture in &self.captures {
            capture.record(log);
        }
        for (name, pin) in [
            ("actions_checkout", &self.checkout),
            ("actions_setup_java", &self.setup_java),
            ("setup_kotlin_action", &self.setup_kotlin),
            ("actions_setup_python", &self.setup_python),
            ("python_interpreter", &self.python_interpreter),
            ("kotlin_compiler", &self.kotlin),
            ("java_lts", &self.java_lts),
        ] {
            log.append(
                "version_resolution",
                format!(
                    "{name} tag={} sha={} origin={}",
                    pin.tag,
                    pin.sha,
                    pin.origin.label()
                ),
            );
        }
    }

    /// The comment lines saying, per pin, where a non-live version came from
    /// (R5). Empty when every pin was resolved live.
    #[must_use]
    pub fn provenance_note(&self) -> Vec<String> {
        [
            ("actions/checkout", &self.checkout),
            ("actions/setup-java", &self.setup_java),
            ("fwilhe2/setup-kotlin", &self.setup_kotlin),
            ("actions/setup-python", &self.setup_python),
            ("python", &self.python_interpreter),
            ("kotlin", &self.kotlin),
            ("java", &self.java_lts),
        ]
        .into_iter()
        .filter(|(_, pin)| pin.origin != Origin::Live)
        .map(|(name, pin)| {
            if pin.origin == Origin::Cache {
                format!(
                    "{name} {tag}: (version resolved from cache; fetched_at={fetched_at})",
                    tag = pin.tag,
                    fetched_at = pin.fetched_at,
                )
            } else {
                format!(
                    "{name} {tag}: (version resolved from the shipped baseline measured {measured})",
                    tag = pin.tag,
                    measured = pin.fetched_at,
                )
            }
        })
        .collect()
    }
}

/// The Adoptium statement of the current Java LTS (R3): a version number, not
/// a git ref, so `sha` stays empty.
fn resolve_java_lts<T: SourceTransport>(
    client: &CachedSourceClient<T>,
    captures: &mut Vec<SourceCapture>,
) -> Option<ResolvedVersion> {
    const URL: &str = "https://api.adoptium.net/v3/info/available_releases";
    let capture = client.fetch(URL).ok()?;
    let origin = if capture.cached() {
        Origin::Cache
    } else {
        Origin::Live
    };
    let lts = serde_json::from_slice::<serde_json::Value>(capture.bytes())
        .ok()?
        .get("most_recent_lts")?
        .as_u64()?;
    let resolved = ResolvedVersion {
        tag: lts.to_string(),
        sha: String::new(),
        origin,
        source_url: URL.to_owned(),
        fetched_at: capture.fetched_at().to_owned(),
        response_sha256: capture.sha256().to_owned(),
    };
    captures.push(capture);
    Some(resolved)
}

/// One top-level JSON string field of a capture, when the body parses.
fn json_string(capture: &SourceCapture, key: &str) -> Option<String> {
    serde_json::from_slice::<serde_json::Value>(capture.bytes())
        .ok()?
        .get(key)?
        .as_str()
        .map(str::to_owned)
}

/// The shipped baseline for `id`, from the toolchains seed.
fn baseline(id: &str) -> Option<ResolvedVersion> {
    let root = parse_lino(TOOLCHAINS);
    let mut nodes = Vec::new();
    descend(&root, &mut nodes);
    nodes
        .iter()
        .find(|node| node.name == "generated_version" && node.id == id)
        .map(ResolvedVersion::from_baseline)
}

fn descend<'a>(node: &'a LinoNode, out: &mut Vec<&'a LinoNode>) {
    out.push(node);
    for child in &node.children {
        descend(child, out);
    }
}

/// Rewrite every generated-workflow pin to its resolved value.
///
/// Two shapes are rewritten (R6): the `{placeholder}` forms the templates are
/// migrating to, and the stale `@<sha>` / `version: '…'` literals the
/// migration has not reached yet -- a literal that slipped through is
/// rewritten rather than shipped. The `version:` and `java-version:` rewrites
/// are scoped to the block of the action that consumes them, so an unrelated
/// `version:` key in the same workflow is left alone.
#[must_use]
pub fn fill_workflow_versions(template: &str, versions: &VersionSet) -> String {
    let mut text = template
        .replace("{checkout_ref}", versions.checkout.pinned_ref())
        .replace("{checkout_tag}", &versions.checkout.tag)
        .replace("{setup_java_ref}", versions.setup_java.pinned_ref())
        .replace("{setup_java_tag}", &versions.setup_java.tag)
        .replace("{setup_kotlin_ref}", versions.setup_kotlin.pinned_ref())
        .replace("{setup_kotlin_tag}", &versions.setup_kotlin.tag)
        .replace("{setup_python_ref}", versions.setup_python.pinned_ref())
        .replace("{setup_python_tag}", &versions.setup_python.tag)
        .replace("{python_version}", &versions.python_interpreter.tag)
        .replace("{kotlin_version}", &versions.kotlin.tag)
        .replace("{java_lts}", &versions.java_lts.tag);

    #[derive(PartialEq)]
    enum Block {
        None,
        Kotlin,
        Java,
        Python,
    }
    let mut block = Block::None;
    let mut out = String::with_capacity(text.len());
    for line in text.lines() {
        let mut line = line.to_owned();
        if line.contains("actions/checkout@") {
            line = replace_action_ref(&line, "actions/checkout", &versions.checkout);
        }
        if line.contains("actions/setup-java@") {
            line = replace_action_ref(&line, "actions/setup-java", &versions.setup_java);
            block = Block::Java;
        } else if line.contains("fwilhe2/setup-kotlin@") {
            line = replace_action_ref(&line, "fwilhe2/setup-kotlin", &versions.setup_kotlin);
            block = Block::Kotlin;
        } else if line.contains("actions/setup-python@") {
            line = replace_action_ref(&line, "actions/setup-python", &versions.setup_python);
            block = Block::Python;
        } else if line.trim_start().starts_with("- uses:") {
            block = Block::None;
        }
        if line.contains("java-version:") && block == Block::Java {
            line = replace_quoted_value(&line, &versions.java_lts.tag);
        }
        if line.contains("python-version:") && block == Block::Python {
            line = replace_quoted_value(&line, &versions.python_interpreter.tag);
        }
        if line.contains("version:") && block == Block::Kotlin {
            line = replace_quoted_value(&line, &versions.kotlin.tag);
        }
        out.push_str(&line);
        out.push('\n');
    }
    out.trim_end_matches('\n').to_owned()
}

/// `action@<anything>` becomes `action@<resolved ref> # <tag>`, per R1: the
/// SHA of the latest release with the tag in a comment.
fn replace_action_ref(line: &str, action: &str, pin: &ResolvedVersion) -> String {
    let marker = format!("{action}@");
    let Some(start) = line.find(&marker) else {
        return line.to_owned();
    };
    let ref_start = start + marker.len();
    let ref_end = line[ref_start..]
        .find(|c: char| !(c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-')))
        .map_or(line.len(), |offset| ref_start + offset);
    let comment = format!("  # {}", pin.tag);
    format!(
        "{}{}{}{}",
        &line[..ref_start],
        pin.pinned_ref(),
        comment,
        &line[ref_end..]
    )
}

/// `'key: <quoted value>'` with the resolved version, quotes preserved.
fn replace_quoted_value(line: &str, value: &str) -> String {
    let Some(colon) = line.find(':') else {
        return line.to_owned();
    };
    let (head, tail) = line.split_at(colon + 1);
    let trimmed = tail.trim_start();
    let quote = trimmed.chars().next().filter(|c| *c == '\'' || *c == '"');
    match quote {
        Some(q) => format!("{head} {q}{value}{q}"),
        None => format!("{head} {value}"),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

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

    fn fixed_clock() -> u64 {
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
        assert!(versions.provenance_note().is_empty());
    }

    #[test]
    fn records_the_resolution_in_the_derivation() {
        let dir = temp_dir("record");
        let client = MockTransport::online(dir);
        let versions = VersionSet::resolve(&client);
        let mut log = crate::event_log::EventLog::new();
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
        assert!(warmed.checkout.origin == Origin::Live);
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
    }

    #[test]
    fn rewrites_the_stale_ci_setup_literals() {
        let versions = VersionSet::baseline().expect("baselines");
        let stale = "      - uses: actions/setup-java@b6effb05e454b25005698d916606bdc6ffcbf961\n        with:\n          distribution: temurin\n          java-version: '21'\n      - uses: fwilhe2/setup-kotlin@51a059ff08b95e2b83aa952b5b46b696d5b615a0\n        with:\n          version: '2.3.10'\n";
        let filled = fill_workflow_versions(stale, &versions);
        assert!(
            filled.contains("actions/setup-java@de7274f081f381c8f8158605e0321c36c376e2e6"),
            "{filled}"
        );
        assert!(filled.contains("java-version: '25'"), "{filled}");
        assert!(
            filled.contains("fwilhe2/setup-kotlin@ee9692514da313706b193d808526812102a344e4"),
            "{filled}"
        );
        assert!(filled.contains("version: '2.4.20'"), "{filled}");
        assert!(!filled.contains("'21'"), "{filled}");
        assert!(!filled.contains("2.3.10"), "{filled}");
        assert!(!filled.contains("b6effb05"), "{filled}");
        assert!(!filled.contains("51a059ff"), "{filled}");
    }

    #[test]
    fn rewrites_the_workflow_template_checkout_ref() {
        let versions = VersionSet::baseline().expect("baselines");
        let stale = "      - uses: actions/checkout@11d5960a326750d5838078e36cf38b85af677262\n";
        let filled = fill_workflow_versions(stale, &versions);
        assert!(
            filled.contains("actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1"),
            "{filled}"
        );
        assert!(filled.contains("# v7.0.1"), "{filled}");
        assert!(!filled.contains("11d5960"), "{filled}");
    }

    #[test]
    fn fills_the_placeholder_forms() {
        let versions = VersionSet::baseline().expect("baselines");
        let template = "      - uses: actions/checkout@{checkout_ref} # {checkout_tag}\n";
        let filled = fill_workflow_versions(template, &versions);
        assert!(
            filled.contains("actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1"),
            "{filled}"
        );
    }

    #[test]
    fn rewrites_the_stale_python_ci_setup_literals() {
        let versions = VersionSet::baseline().expect("baselines");
        let stale = "      - uses: actions/setup-python@ece7cb06caefa5fff74198d8649806c4678c61a1\n        with:\n          python-version: '3.14'\n";
        let filled = fill_workflow_versions(stale, &versions);
        assert!(
            filled.contains("actions/setup-python@5fda3b95a4ea91299a34e894583c3862153e4b97"),
            "{filled}"
        );
        assert!(filled.contains("python-version: '3.14.7'"), "{filled}");
        assert!(!filled.contains("ece7cb06"), "{filled}");
    }

    #[test]
    fn a_version_key_outside_the_setup_blocks_is_left_alone() {
        let versions = VersionSet::baseline().expect("baselines");
        let template =
            "      - uses: some/other-action@v1\n        with:\n          version: '9.9.9'\n";
        let filled = fill_workflow_versions(template, &versions);
        assert!(filled.contains("version: '9.9.9'"), "{filled}");
    }
}
