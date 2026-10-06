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
    CachedSourceClient, CurlSourceTransport, SourceCapture, SourceTransport,
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
            commit.fetched_at().clone_into(&mut resolved.fetched_at);
            commit.sha256().clone_into(&mut resolved.response_sha256);
            resolved
        })
    }

    /// Every pin beside its `generated_version` id in the toolchains seed.
    const fn pins(&self) -> [(&'static str, &ResolvedVersion); 7] {
        [
            ("actions_checkout", &self.checkout),
            ("actions_setup_java", &self.setup_java),
            ("setup_kotlin_action", &self.setup_kotlin),
            ("actions_setup_python", &self.setup_python),
            ("python_interpreter", &self.python_interpreter),
            ("kotlin_compiler", &self.kotlin),
            ("java_lts", &self.java_lts),
        ]
    }

    /// The pin the toolchains seed names by `id`.
    fn pin(&self, id: &str) -> Option<&ResolvedVersion> {
        self.pins()
            .into_iter()
            .find(|(name, _)| *name == id)
            .map(|(_, pin)| pin)
    }

    /// Append the resolution to the derivation: every capture's provenance,
    /// then one line per pin naming what was pinned and how (R4).
    pub fn record(&self, log: &mut crate::event_log::EventLog) {
        for capture in &self.captures {
            capture.record(log);
        }
        for (name, pin) in self.pins() {
            log.append_fields(
                "version_resolution",
                &[
                    ("pin", name),
                    ("tag", &pin.tag),
                    ("sha", &pin.sha),
                    ("origin", pin.origin.label()),
                ],
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
            let intent = if pin.origin == Origin::Cache {
                "version_provenance_cache"
            } else {
                "version_provenance_baseline"
            };
            crate::seed::report_text(
                intent,
                &[
                    ("name", name),
                    ("tag", &pin.tag),
                    ("fetched_at", &pin.fetched_at),
                ],
            )
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
        .map(|node| ResolvedVersion::from_baseline(node))
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
    let text = template
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

    let actions = workflow_actions(versions);
    let step_marker = workflow_step_marker();
    // The `version:`-shaped key the current action block consumes, and the
    // pin it is rewritten to; a new `uses:` step closes the block.
    let mut block: Option<(&str, &ResolvedVersion)> = None;
    let mut out = String::with_capacity(text.len());
    for line in text.lines() {
        let mut line = line.to_owned();
        let mut entered = false;
        for action in &actions {
            if line.contains(&action.marker) {
                line = replace_action_ref(&line, &action.uses, action.pin);
                if let Some((key, pin)) = &action.scoped {
                    block = Some((key.as_str(), *pin));
                    entered = true;
                }
            }
        }
        if !entered
            && step_marker
                .as_deref()
                .is_some_and(|marker| line.trim_start().starts_with(marker))
        {
            block = None;
        }
        // A `version:` key takes the bare version (`2.4.20`), not the
        // release tag (`v2.4.20`) the publisher's API names it by.
        if let Some((_, pin)) = block.filter(|(key, _)| line.contains(*key)) {
            line = replace_quoted_value(&line, pin.tag.trim_start_matches('v'));
        }
        out.push_str(&line);
        out.push('\n');
    }
    out.trim_end_matches('\n').to_owned()
}

/// One `uses:` action a generated workflow pins, as the toolchains seed
/// declares it on its `generated_version` row (`workflow_uses`, and for a
/// setup action the `workflow_version_key` it consumes and the
/// `workflow_version_pin` that key is rewritten to).
struct WorkflowAction<'a> {
    uses: String,
    marker: String,
    pin: &'a ResolvedVersion,
    scoped: Option<(String, &'a ResolvedVersion)>,
}

fn workflow_actions(versions: &VersionSet) -> Vec<WorkflowAction<'_>> {
    let root = parse_lino(TOOLCHAINS);
    let mut nodes = Vec::new();
    descend(&root, &mut nodes);
    nodes
        .iter()
        .filter(|node| node.name == "generated_version")
        .filter_map(|node| {
            let uses = node.find_child_value("workflow_uses");
            if uses.is_empty() {
                return None;
            }
            let pin = versions.pin(&node.id)?;
            let key = node.find_child_value("workflow_version_key");
            let scoped = versions
                .pin(node.find_child_value("workflow_version_pin"))
                .filter(|_| !key.is_empty())
                .map(|scoped_pin| (key.to_owned(), scoped_pin));
            Some(WorkflowAction {
                uses: uses.to_owned(),
                marker: format!("{uses}@"),
                pin,
                scoped,
            })
        })
        .collect()
}

/// The line prefix that opens a new workflow step, from the toolchains seed.
fn workflow_step_marker() -> Option<String> {
    let root = parse_lino(TOOLCHAINS);
    let mut nodes = Vec::new();
    descend(&root, &mut nodes);
    nodes
        .iter()
        .find(|node| node.name == "generated_workflow_step")
        .map(|node| node.find_child_value("marker").to_owned())
        .filter(|marker| !marker.is_empty())
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
    // A `# <tag>` comment already behind the ref (the placeholder form
    // `@{checkout_ref} # {checkout_tag}`) is replaced, not doubled.
    let rest = &line[ref_end..];
    let rest = if rest.trim_start().starts_with('#') {
        ""
    } else {
        rest
    };
    format!(
        "{}{}{}{}",
        &line[..ref_start],
        pin.pinned_ref(),
        comment,
        rest
    )
}

/// `'key: <quoted value>'` with the resolved version, quotes preserved.
fn replace_quoted_value(line: &str, value: &str) -> String {
    let Some(colon) = line.find(':') else {
        return line.to_owned();
    };
    let (head, tail) = line.split_at(colon + 1);
    let trimmed = tail.trim_start();
    // The value keeps the quote character the line already used, if any.
    let quote = match trimmed.chars().next() {
        Some(mark @ ('\'' | '"')) => mark.to_string(),
        _ => String::new(),
    };
    [head, " ", quote.as_str(), value, quote.as_str()].concat()
}
