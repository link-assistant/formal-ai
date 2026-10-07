#!/usr/bin/env rust-script
//! Every dependency at its latest release (issue #1169, E134).
//!
//! A dependency that is merely *compatible* is still stale: `^1.2` with 1.3
//! published ships 1.3 through the lockfile but `1.2` through every fresh
//! checkout that resolves loosely, and nothing in the tree ever moves the
//! manifest forward. This gate reads what the repository actually resolves
//! (Cargo.lock, bun.lock, package-lock.json — the manifest range when a
//! directory ships no lock) and compares it against the publisher's own
//! registry, never a memorized list:
//!
//!   crates.io    /api/v1/crates/{name}            `max_stable_version`
//!   npm          registry.npmjs.org/{name}         dist-tags.latest
//!   GitHub       /repos/{owner}/{repo}/releases/latest   `tag_name`
//!   Docker Hub   /v2/repositories/{image}/tags     highest numeric tag
//!   rustc        static.rust-lang.org channel-rust-stable.toml `[pkg.rust]`
//!
//! A `uses:` ref that names a moving branch or tool selector (`@stable`,
//! `@nextest`) has no release to fall behind and is not compared; a floating
//! major (`@v7`) is current while the latest release is a `7.x.y`.
//!
//! The one sanctioned escape hatch (issue #1169 R3) is a same-line blocked
//! annotation naming the issue that tracks the hold-back. In Cargo.toml:
//!
//!   links-notation = "0.16.1" # blocked: <https://github.com/link-foundation/lino-objects-codec/issues/60>
//!
//! The reference is an issue URL, or a GitHub security advisory
//! (`…/advisories/GHSA-…`) when the hold-back waits on an unpatched fix.
//!
//! In package.json a `"<name>//"` note key in the same manifest:
//!
//!   "electron": "^44.1.0",
//!   "electron//": "blocked: <https://github.com/link-assistant/formal-ai/issues/1234>",
//!
//! A blocked finding is reported but does not fail the gate; a drift without
//! an annotation does. Live registry reads can also be skipped entirely:
//! `--offline` answers from `data/meta/dependency-registry-snapshot.lino`,
//! which `--refresh-snapshot` writes from the live registries, so CI never
//! goes red because a registry was slow on a Tuesday.
//!
//! Usage:
//!   rust-script scripts/check-dependencies-latest.rs [--repo <path>]
//!        [--offline] [--refresh-snapshot] [--apply] [--format lino]
//!
//! Exit codes: 0 clean, 1 findings, 2 usage, 3 a registry was unreachable
//! (retry, or run --refresh-snapshot when the network is back).
//!
//! ```cargo
//! [package]
//! edition = "2024"
//!
//! [dependencies]
//! ```

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
#[cfg(not(test))]
use std::process::Command;

/// Where `--refresh-snapshot` writes and `--offline` reads.
#[cfg_attr(test, allow(dead_code))]
pub const SNAPSHOT_PATH: &str = "data/meta/dependency-registry-snapshot.lino";

#[cfg_attr(test, allow(dead_code))]
const USER_AGENT: &str = "formal-ai-dependency-gate (https://github.com/link-assistant/formal-ai)";

/// One dependency the repository declares, with where it was declared.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dependency {
    pub ecosystem: Ecosystem,
    pub name: String,
    /// The manifest requirement, when the tree declares one (`"1.1"`, `^4.0.0`).
    pub declared: Option<String>,
    /// What the tree actually resolves to (lockfile entry, `uses:` ref, tag).
    pub resolved: String,
    pub path: PathBuf,
    pub line: usize,
    /// The issue URL of a same-line `# blocked:` annotation, when present.
    pub blocked: Option<String>,
}

/// Which publisher owns a dependency's versions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Ecosystem {
    CratesIo,
    Npm,
    GitHubAction,
    DockerImage,
    RustToolchain,
}

impl Ecosystem {
    pub const fn key(self) -> &'static str {
        match self {
            Self::CratesIo => "crates_io",
            Self::Npm => "npm",
            Self::GitHubAction => "github_action",
            Self::DockerImage => "docker_image",
            Self::RustToolchain => "rust_toolchain",
        }
    }
}

/// A dependency that is not at its latest release.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    pub dependency: Dependency,
    pub latest: String,
}

/// The registry answers one run needs: `(ecosystem, name) -> (latest, url)`.
pub struct Snapshot {
    pub refreshed_at: String,
    pub entries: BTreeMap<(Ecosystem, String), (String, String)>,
}

/// `rust-script --test` runs the embedded suite; inside the unit crate
/// (`rust/tests/unit/issue_1169_dependency_currency.rs`) the same file is a
/// module, and the live-network half stays out of both test builds.
#[cfg(not(test))]
fn main() {
    let mut repo = PathBuf::from(".");
    let mut offline = false;
    let mut refresh = false;
    let mut apply = false;
    let mut lino = false;
    let mut arguments = std::env::args().skip(1);
    while let Some(flag) = arguments.next() {
        match flag.as_str() {
            "--repo" => {
                repo = arguments
                    .next()
                    .unwrap_or_else(|| usage("--repo needs a path"))
                    .into()
            }
            "--offline" => offline = true,
            "--refresh-snapshot" => refresh = true,
            "--apply" => apply = true,
            "--format" => {
                let format = arguments
                    .next()
                    .unwrap_or_else(|| usage("--format needs lino"));
                if format != "lino" {
                    usage(&format!("unknown format {format}"));
                }
                lino = true;
            }
            other => usage(&format!("unknown flag {other}")),
        }
    }
    if !repo.join("rust/Cargo.toml").is_file() {
        usage("the repository root must contain rust/Cargo.toml");
    }

    let dependencies = collect(&repo);
    let snapshot = if refresh || !offline {
        match live_snapshot(&dependencies) {
            Ok(snapshot) => snapshot,
            Err(error) => {
                eprintln!("check-dependencies-latest: {error}");
                eprintln!(
                    "check-dependencies-latest: a registry was unreachable; retry, or --offline against the last snapshot"
                );
                std::process::exit(3);
            }
        }
    } else {
        match offline_snapshot(&repo) {
            Ok(snapshot) => snapshot,
            Err(error) => {
                eprintln!("check-dependencies-latest: {error}");
                std::process::exit(EXIT_UNVERIFIABLE);
            }
        }
    };

    if refresh {
        write_snapshot(&repo.join(SNAPSHOT_PATH), &snapshot);
        if !lino {
            println!(
                "dependency registry snapshot refreshed: {} entries ({})",
                snapshot.entries.len(),
                snapshot.refreshed_at
            );
        }
        return;
    }

    let (findings, unverifiable) = compare(&dependencies, &snapshot);
    if apply && !findings.is_empty() {
        apply_latest(&findings);
    }

    if lino {
        print!("{}", render_lino(&findings, &unverifiable));
    } else {
        for finding in &findings {
            println!(
                "{}:{} {} {} resolves {} but the latest release is {}{}",
                finding.dependency.path.display(),
                finding.dependency.line,
                finding.dependency.ecosystem.key(),
                finding.dependency.name,
                finding.dependency.resolved,
                finding.latest,
                status_note(&finding.dependency)
            );
        }
        for name in &unverifiable {
            println!("dependency currency: {name} absent from the snapshot; not checked");
        }
        println!(
            "dependency currency: {} finding(s), {} unverifiable from this snapshot",
            findings.len(),
            unverifiable.len()
        );
    }

    let code = exit_code(&findings);
    if code == 0 {
        println!("dependency currency: every finding carries a blocked annotation");
        return;
    }
    println!(
        "dependency currency: FAIL - bump the manifest or add a `# blocked: <issue-url>` annotation"
    );
    std::process::exit(code);
}

/// Exit status when a registry answer is unavailable: a live registry was
/// unreachable, or `--offline` found no snapshot. Distinct from 1 so a
/// flaky registry never reads as drift (R1169-6).
pub const EXIT_UNVERIFIABLE: i32 = 3;

/// The snapshot `--offline` answers from, or why there is none. An absent
/// snapshot is unverifiable (exit 3), never "no findings".
pub fn offline_snapshot(repo: &Path) -> Result<Snapshot, String> {
    read_snapshot(&repo.join(SNAPSHOT_PATH)).ok_or_else(|| {
        format!("no snapshot at {SNAPSHOT_PATH}; run --refresh-snapshot with the network up")
    })
}

/// 0 when every finding carries a blocked annotation (or there are none),
/// 1 when any drift is unannotated (R1169-3).
pub fn exit_code(findings: &[Finding]) -> i32 {
    i32::from(findings.iter().any(|f| f.dependency.blocked.is_none()))
}

#[cfg_attr(test, allow(dead_code))]
const fn status_note(dependency: &Dependency) -> &'static str {
    if dependency.blocked.is_some() {
        " (blocked)"
    } else {
        ""
    }
}

#[cfg_attr(test, allow(dead_code))]
fn usage(message: &str) -> ! {
    eprintln!("check-dependencies-latest: {message}");
    std::process::exit(2);
}

/// Every dependency the repository declares, across all five ecosystems.
pub fn collect(repo: &Path) -> Vec<Dependency> {
    let mut dependencies = Vec::new();
    collect_cargo(repo, &mut dependencies);
    collect_npm(repo, &mut dependencies);
    collect_workflows(repo, &mut dependencies);
    collect_dockerfile(repo, &mut dependencies);
    collect_rust_version(repo, &mut dependencies);
    dependencies
}

fn collect_cargo(repo: &Path, dependencies: &mut Vec<Dependency>) {
    let manifest_path = repo.join("rust/Cargo.toml");
    let Ok(text) = fs::read_to_string(&manifest_path) else {
        return;
    };
    let lock_versions = fs::read_to_string(repo.join("rust/Cargo.lock"))
        .map(|text| parse_cargo_lock(&text))
        .unwrap_or_default();
    for (name, declared, line, blocked) in parse_cargo_manifest(&text) {
        // A registry version is the currency this gate audits; path, git and
        // workspace-inherited requirements have no registry answer.
        let Some(resolved) = lock_versions.get(&name) else {
            continue;
        };
        dependencies.push(Dependency {
            ecosystem: Ecosystem::CratesIo,
            name,
            declared: Some(declared),
            resolved: resolved.clone(),
            path: manifest_path.clone(),
            line,
            blocked,
        });
    }
}

fn collect_npm(repo: &Path, dependencies: &mut Vec<Dependency>) {
    let mut manifests = Vec::new();
    walk_for(repo, "package.json", &mut manifests);
    manifests.sort();
    for manifest_path in manifests {
        let Ok(text) = fs::read_to_string(&manifest_path) else {
            continue;
        };
        let lock_versions = npm_lock_versions(&manifest_path);
        for (name, declared, line, blocked) in parse_package_json(&text) {
            let resolved = lock_versions
                .get(&name)
                .cloned()
                .unwrap_or_else(|| base_version(&declared).to_string());
            dependencies.push(Dependency {
                ecosystem: Ecosystem::Npm,
                name,
                declared: Some(declared),
                resolved,
                path: manifest_path.clone(),
                line,
                blocked,
            });
        }
    }
}

/// The exact versions a directory's lockfile resolves, preferring whichever
/// lock sits next to the manifest.
fn npm_lock_versions(manifest_path: &Path) -> BTreeMap<String, String> {
    let directory = manifest_path.parent().unwrap_or(manifest_path);
    for (lock_name, parse) in [
        (
            "package-lock.json",
            parse_package_lock as fn(&str) -> BTreeMap<String, String>,
        ),
        ("bun.lock", parse_bun_lock),
    ] {
        if let Ok(text) = fs::read_to_string(directory.join(lock_name)) {
            return parse(&text);
        }
    }
    BTreeMap::new()
}

fn collect_workflows(repo: &Path, dependencies: &mut Vec<Dependency>) {
    let mut files = Vec::new();
    walk_for(&repo.join(".github/workflows"), "*.yml", &mut files);
    walk_for(&repo.join(".github/actions"), "action.yml", &mut files);
    files.sort();
    for path in files {
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        for (action, reference, line) in parse_uses_pins(&text) {
            if is_moving_ref(&reference) {
                continue;
            }
            dependencies.push(Dependency {
                ecosystem: Ecosystem::GitHubAction,
                name: action,
                declared: Some(reference.clone()),
                resolved: without_v(&reference).to_string(),
                path: path.clone(),
                line,
                blocked: None,
            });
        }
    }
}

fn collect_dockerfile(repo: &Path, dependencies: &mut Vec<Dependency>) {
    let path = repo.join("Dockerfile");
    let Ok(text) = fs::read_to_string(&path) else {
        return;
    };
    for (image, tag, line) in parse_dockerfile_from(&text) {
        dependencies.push(Dependency {
            ecosystem: Ecosystem::DockerImage,
            name: image.clone(),
            declared: Some(tag.clone()),
            resolved: tag,
            path: path.clone(),
            line,
            blocked: None,
        });
    }
}

fn collect_rust_version(repo: &Path, dependencies: &mut Vec<Dependency>) {
    let manifest_path = repo.join("rust/Cargo.toml");
    let Ok(text) = fs::read_to_string(&manifest_path) else {
        return;
    };
    for (index, line) in text.lines().enumerate() {
        let Some(rest) = line.trim_start().strip_prefix("rust-version") else {
            continue;
        };
        let Some(value) = quoted_value(rest) else {
            continue;
        };
        dependencies.push(Dependency {
            ecosystem: Ecosystem::RustToolchain,
            name: "rustc".to_string(),
            declared: Some(value.clone()),
            resolved: value,
            path: manifest_path.clone(),
            line: index + 1,
            blocked: blocked_annotation(line),
        });
    }
}

fn walk_for(root: &Path, needle: &str, found: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if path.is_dir() {
            if matches!(
                name,
                "node_modules" | "target" | ".git" | ".claude" | "cache"
            ) {
                continue;
            }
            walk_for(&path, needle, found);
        } else if needle == name || (needle.starts_with('*') && name.ends_with(&needle[1..])) {
            found.push(path);
        }
    }
}

/// Split collected dependencies into findings and snapshot-absent names.
pub fn compare(dependencies: &[Dependency], snapshot: &Snapshot) -> (Vec<Finding>, Vec<String>) {
    let mut findings = Vec::new();
    let mut unverifiable = Vec::new();
    for dependency in dependencies {
        let Some((latest, _url)) = snapshot
            .entries
            .get(&(dependency.ecosystem, dependency.name.clone()))
        else {
            unverifiable.push(format!(
                "{}/{}",
                dependency.ecosystem.key(),
                dependency.name
            ));
            continue;
        };
        let latest = latest.clone();
        if dependency.ecosystem == Ecosystem::RustToolchain {
            // `rust-version` is a floor ("1.98" means any 1.98.x); the
            // currency question is whether the floor's minor is current.
            if minor_of(&dependency.resolved) != minor_of(&latest) {
                findings.push(Finding {
                    dependency: dependency.clone(),
                    latest,
                });
            }
            continue;
        }
        if dependency.ecosystem == Ecosystem::GitHubAction {
            // A commit SHA cannot be ordered against a release tag without
            // another API call; it is reported as unverifiable, not current.
            if is_commit_sha(&dependency.resolved) {
                unverifiable.push(format!(
                    "{}/{}",
                    dependency.ecosystem.key(),
                    dependency.name
                ));
            } else if !selects_latest(&dependency.resolved, &latest) {
                findings.push(Finding {
                    dependency: dependency.clone(),
                    latest,
                });
            }
            continue;
        }
        let resolved = without_v(&dependency.resolved);
        let latest_bare = without_v(&latest);
        if resolved != latest_bare {
            findings.push(Finding {
                dependency: dependency.clone(),
                latest,
            });
        }
    }
    findings.sort_by(|a, b| {
        (a.dependency.path.display().to_string(), a.dependency.line)
            .cmp(&(b.dependency.path.display().to_string(), b.dependency.line))
    });
    (findings, unverifiable)
}

/// Rewrite manifest requirements (and pin lines) to the fetched latest.
/// Lockfiles are deliberately untouched: `cargo update` / `bun install`
/// re-derive them from the rewritten manifests.
pub fn apply_latest(findings: &[Finding]) {
    let mut per_file: BTreeMap<PathBuf, Vec<&Finding>> = BTreeMap::new();
    for finding in findings {
        if finding.dependency.blocked.is_some() {
            continue;
        }
        per_file
            .entry(finding.dependency.path.clone())
            .or_default()
            .push(finding);
    }
    for (path, file_findings) in per_file {
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        let mut rewritten = text.clone();
        for finding in file_findings {
            let dependency = &finding.dependency;
            let ecosystem = dependency.ecosystem;
            let bare = without_v(&finding.latest);
            let updated = match ecosystem {
                Ecosystem::CratesIo => rewrite_cargo_line(&rewritten, dependency, bare),
                Ecosystem::Npm => rewrite_npm_line(&rewritten, dependency, bare),
                Ecosystem::GitHubAction => rewrite_uses_line(
                    &rewritten,
                    dependency,
                    &same_precision(&finding.latest, &dependency.resolved),
                ),
                Ecosystem::DockerImage => rewrite_from_line(&rewritten, dependency, bare),
                Ecosystem::RustToolchain => {
                    rewrite_rust_version(&rewritten, dependency, &minor_of(&finding.latest))
                }
            };
            rewritten = updated;
        }
        if rewritten != text {
            fs::write(&path, rewritten).expect("rewrite the manifest");
        }
    }
}

fn rewrite_cargo_line(text: &str, dependency: &Dependency, latest: &str) -> String {
    rewrite_line_at(text, dependency.line, |line| {
        let declared = dependency.declared.as_deref()?;
        let operator = if declared.starts_with('=') { "=" } else { "" };
        let updated = format!("{operator}{latest}");
        Some(line.replacen(&format!("\"{declared}\""), &format!("\"{updated}\""), 1))
    })
}

fn rewrite_npm_line(text: &str, dependency: &Dependency, latest: &str) -> String {
    rewrite_line_at(text, dependency.line, |line| {
        let declared = dependency.declared.as_deref()?;
        let prefix = ['^', '~', '>', '=']
            .iter()
            .find(|c| declared.starts_with(**c))
            .map(std::string::ToString::to_string)
            .unwrap_or_default();
        Some(line.replacen(
            &format!("\"{declared}\""),
            &format!("\"{prefix}{latest}\""),
            1,
        ))
    })
}

fn rewrite_uses_line(text: &str, dependency: &Dependency, latest: &str) -> String {
    rewrite_line_at(text, dependency.line, |line| {
        let pinned = dependency
            .declared
            .as_deref()
            .unwrap_or(&dependency.resolved);
        Some(line.replacen(&format!("@{pinned}"), &format!("@{latest}"), 1))
    })
}

fn rewrite_from_line(text: &str, dependency: &Dependency, latest: &str) -> String {
    rewrite_line_at(text, dependency.line, |line| {
        Some(line.replacen(
            &format!(":{}", dependency.resolved),
            &format!(":{latest}"),
            1,
        ))
    })
}

fn rewrite_rust_version(text: &str, dependency: &Dependency, minor: &str) -> String {
    rewrite_line_at(text, dependency.line, |line| {
        Some(line.replacen(
            &format!("\"{}\"", dependency.resolved),
            &format!("\"{minor}\""),
            1,
        ))
    })
}

fn rewrite_line_at(
    text: &str,
    line_number: usize,
    rewrite: impl Fn(&str) -> Option<String>,
) -> String {
    let lines: Vec<&str> = text.lines().collect();
    if line_number == 0 || line_number > lines.len() {
        return text.to_string();
    }
    let Some(updated) = rewrite(lines[line_number - 1]) else {
        return text.to_string();
    };
    let mut owned: Vec<String> = lines.iter().map(std::string::ToString::to_string).collect();
    owned[line_number - 1] = updated;
    let mut out = owned.join("\n");
    if text.ends_with('\n') {
        out.push('\n');
    }
    out
}

/// Findings as flat lino records, one per drift.
#[cfg_attr(test, allow(dead_code))]
pub fn render_lino(findings: &[Finding], unverifiable: &[String]) -> String {
    let mut out = String::new();
    for finding in findings {
        let dependency = &finding.dependency;
        let _ = writeln!(
            out,
            "currency_finding\n  ecosystem \"{}\"\n  name \"{}\"\n  resolved \"{}\"\n  latest \"{}\"\n  path \"{}\"\n  line {}\n  blocked {}\n",
            dependency.ecosystem.key(),
            dependency.name,
            dependency.resolved,
            finding.latest,
            dependency.path.display(),
            dependency.line,
            dependency.blocked.is_some(),
        );
    }
    for name in unverifiable {
        let _ = writeln!(out, "unverifiable_dependency\n  name \"{name}\"\n");
    }
    out
}

#[path = "check-dependencies-latest-parsers.rs"]
mod parsers;
pub use parsers::*;

/// Read the snapshot written by `--refresh-snapshot`.
pub fn read_snapshot(path: &Path) -> Option<Snapshot> {
    let text = fs::read_to_string(path).ok()?;
    let mut refreshed_at = String::new();
    let mut entries = BTreeMap::new();
    let mut current: Option<(Ecosystem, String)> = None;
    for line in text.lines() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix("refreshed_at ") {
            refreshed_at = quoted_value(rest).unwrap_or_default();
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("registry_entry ") {
            let id = rest.trim().to_string();
            current = ecosystem_of(&id).map(|ecosystem| (ecosystem, bare_id(&id, ecosystem)));
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("latest ")
            && let (Some(key), Some(latest)) = (&current, quoted_value(rest))
        {
            entries.insert(key.clone(), (latest, String::new()));
        }
    }
    Some(Snapshot {
        refreshed_at,
        entries,
    })
}

/// Write the snapshot `--offline` runs against.
pub fn write_snapshot(path: &Path, snapshot: &Snapshot) {
    let mut text = String::new();
    let _ = writeln!(
        text,
        "# Written by `rust-script scripts/check-dependencies-latest.rs --refresh-snapshot`.\n# One flat record per registry answer; `--offline` reads exactly these.\ndependency_registry_snapshot\n  record_type \"dependency_registry_snapshot\"\n  refreshed_at \"{}\"",
        snapshot.refreshed_at
    );
    for ((ecosystem, name), (latest, url)) in &snapshot.entries {
        let _ = writeln!(
            text,
            "registry_entry {}/{}\n  latest \"{latest}\"\n  source_url \"{url}\"",
            ecosystem.key(),
            name
        );
    }
    if let Some(directory) = path.parent() {
        let _ = fs::create_dir_all(directory);
    }
    fs::write(path, text).expect("write the registry snapshot");
}

fn ecosystem_of(id: &str) -> Option<Ecosystem> {
    match id.split('/').next()? {
        "crates_io" => Some(Ecosystem::CratesIo),
        "npm" => Some(Ecosystem::Npm),
        "github_action" => Some(Ecosystem::GitHubAction),
        "docker_image" => Some(Ecosystem::DockerImage),
        "rust_toolchain" => Some(Ecosystem::RustToolchain),
        _ => None,
    }
}

/// Strip the ecosystem prefix (`npm/@scope/name` -> `@scope/name`), keeping
/// scoped npm names intact.
fn bare_id(id: &str, ecosystem: Ecosystem) -> String {
    let prefix_len = ecosystem.key().len() + 1;
    id[prefix_len.min(id.len())..].to_string()
}

#[cfg(not(test))]
fn live_snapshot(dependencies: &[Dependency]) -> Result<Snapshot, String> {
    let mut snapshot = Snapshot {
        refreshed_at: iso_now(),
        entries: BTreeMap::new(),
    };
    let mut wanted: Vec<(Ecosystem, String)> = dependencies
        .iter()
        .map(|d| (d.ecosystem, d.name.clone()))
        .collect();
    wanted.sort();
    wanted.dedup();
    for (ecosystem, name) in wanted {
        let (latest, url) = match ecosystem {
            Ecosystem::CratesIo => (
                latest_crates_io(&name)?,
                format!("https://crates.io/api/v1/crates/{name}"),
            ),
            Ecosystem::Npm => (
                latest_npm(&name)?,
                format!("https://registry.npmjs.org/{name}"),
            ),
            Ecosystem::GitHubAction => {
                let repository = action_repository(&name);
                (
                    latest_github(&repository)?,
                    format!("https://api.github.com/repos/{repository}/releases/latest"),
                )
            }
            Ecosystem::DockerImage => (
                latest_docker_hub(&name)?,
                format!("https://hub.docker.com/v2/repositories/{name}/tags"),
            ),
            Ecosystem::RustToolchain => (
                latest_rustc()?,
                "https://static.rust-lang.org/dist/channel-rust-stable.toml".to_string(),
            ),
        };
        snapshot.entries.insert((ecosystem, name), (latest, url));
    }
    Ok(snapshot)
}

#[cfg(not(test))]
fn fetch(url: &str) -> Result<String, String> {
    let output = Command::new("curl")
        .args(["-sfL", "--max-time", "30", "-A", USER_AGENT, url])
        .output()
        .map_err(|error| format!("curl failed to launch: {error}"))?;
    if !output.status.success() {
        return Err(format!("curl exited {} for {url}", output.status));
    }
    String::from_utf8(output.stdout).map_err(|_| format!("non-utf8 body from {url}"))
}

/// The live fetchers: one `curl` each, parsed by the pure readers in
/// `check-dependencies-latest-parsers.rs` (which the embedded suite pins
/// against real-shaped publisher answers, R1169-2).
#[cfg(not(test))]
fn latest_crates_io(name: &str) -> Result<String, String> {
    let body = fetch(&format!("https://crates.io/api/v1/crates/{name}"))?;
    crates_io_latest(&body)
        .ok_or_else(|| format!("crates.io answer for {name} had no max_stable_version"))
}

#[cfg(not(test))]
fn latest_npm(name: &str) -> Result<String, String> {
    let body = fetch(&format!("https://registry.npmjs.org/{name}"))?;
    npm_latest(&body).ok_or_else(|| format!("npm dist-tags for {name} had no latest"))
}

#[cfg(not(test))]
fn latest_github(repository: &str) -> Result<String, String> {
    let body = fetch(&format!(
        "https://api.github.com/repos/{repository}/releases/latest"
    ))?;
    github_latest(&body).ok_or_else(|| format!("github answer for {repository} had no tag_name"))
}

#[cfg(not(test))]
fn latest_docker_hub(image: &str) -> Result<String, String> {
    let body = fetch(&format!(
        "https://hub.docker.com/v2/repositories/{image}/tags?page_size=100"
    ))?;
    docker_hub_latest(&body)
        .ok_or_else(|| format!("docker hub tag list for {image} had no versioned tag"))
}

#[cfg(not(test))]
fn latest_rustc() -> Result<String, String> {
    let body = fetch("https://static.rust-lang.org/dist/channel-rust-stable.toml")?;
    rust_channel_latest(&body).ok_or_else(|| "rust channel had no [pkg.rust] version".to_string())
}

#[cfg(not(test))]
fn iso_now() -> String {
    let output = Command::new("date")
        .arg("-u")
        .arg("+%Y-%m-%dT%H:%M:%SZ")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();
    output.trim().to_string()
}

#[cfg(test)]
#[path = "check-dependencies-latest-tests.rs"]
mod tests;
