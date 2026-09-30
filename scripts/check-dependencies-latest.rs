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
//!   crates.io    /api/v1/crates/{name}            max_stable_version
//!   npm          registry.npmjs.org/{name}         dist-tags.latest
//!   GitHub       /repos/{owner}/{repo}/releases/latest   tag_name
//!   Docker Hub   /v2/repositories/{image}/tags     newest pushed tag
//!   rustc        static.rust-lang.org channel-rust-stable.toml
//!
//! The one sanctioned escape hatch (issue #1169 R3) is a same-line blocked
//! annotation naming the issue that tracks the hold-back. In Cargo.toml:
//!
//!   links-notation = "0.16.1" # blocked: https://github.com/link-foundation/lino-objects-codec/issues/60
//!
//! In package.json a sibling key inside the same dependency object:
//!
//!   "electron": "^44.1.0",
//!   "electron//": "blocked: https://github.com/link-assistant/formal-ai/issues/1234",
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
    pub fn key(self) -> &'static str {
        match self {
            Ecosystem::CratesIo => "crates_io",
            Ecosystem::Npm => "npm",
            Ecosystem::GitHubAction => "github_action",
            Ecosystem::DockerImage => "docker_image",
            Ecosystem::RustToolchain => "rust_toolchain",
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
                let format = arguments.next().unwrap_or_else(|| usage("--format needs lino"));
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
                eprintln!("check-dependencies-latest: a registry was unreachable; retry, or --offline against the last snapshot");
                std::process::exit(3);
            }
        }
    } else {
        match read_snapshot(&repo.join(SNAPSHOT_PATH)) {
            Some(snapshot) => snapshot,
            None => {
                eprintln!(
                    "check-dependencies-latest: no snapshot at {SNAPSHOT_PATH}; run --refresh-snapshot with the network up"
                );
                std::process::exit(3);
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
        apply_latest(&repo, &findings);
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

    if findings.iter().all(|f| f.dependency.blocked.is_some()) {
        println!("dependency currency: every finding carries a blocked annotation");
        return;
    }
    println!("dependency currency: FAIL - bump the manifest or add a `# blocked: <issue-url>` annotation");
    std::process::exit(1);
}

#[cfg_attr(test, allow(dead_code))]
fn status_note(dependency: &Dependency) -> &'static str {
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
        ("package-lock.json", parse_package_lock as fn(&str) -> BTreeMap<String, String>),
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
            if matches!(name, "node_modules" | "target" | ".git" | ".claude" | "cache") {
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
        let Some((latest, _url)) = snapshot.entries.get(&(dependency.ecosystem, dependency.name.clone()))
        else {
            unverifiable.push(format!("{}/{}", dependency.ecosystem.key(), dependency.name));
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
        (a.dependency.path.display().to_string(), a.dependency.line).cmp(&(
            b.dependency.path.display().to_string(),
            b.dependency.line,
        ))
    });
    (findings, unverifiable)
}

/// Rewrite manifest requirements (and pin lines) to the fetched latest.
/// Lockfiles are deliberately untouched: `cargo update` / `bun install`
/// re-derive them from the rewritten manifests.
pub fn apply_latest(repo: &Path, findings: &[Finding]) {
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
        let mut rewritten = text;
        for finding in file_findings {
            let dependency = &finding.dependency;
            let ecosystem = dependency.ecosystem;
            let bare = without_v(&finding.latest);
            let updated = match ecosystem {
                Ecosystem::CratesIo => rewrite_cargo_line(&rewritten, dependency, &bare),
                Ecosystem::Npm => rewrite_npm_line(&rewritten, dependency, &bare),
                Ecosystem::GitHubAction => rewrite_uses_line(&rewritten, dependency, &finding.latest),
                Ecosystem::DockerImage => rewrite_from_line(&rewritten, dependency, &bare),
                Ecosystem::RustToolchain => rewrite_rust_version(&rewritten, dependency, &minor_of(&finding.latest)),
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
        let Some(declared) = dependency.declared.as_deref() else {
            return None;
        };
        let operator = if declared.starts_with('=') { "=" } else { "" };
        let updated = format!("{operator}{latest}");
        Some(line.replacen(&format!("\"{declared}\""), &format!("\"{updated}\""), 1))
    })
}

fn rewrite_npm_line(text: &str, dependency: &Dependency, latest: &str) -> String {
    rewrite_line_at(text, dependency.line, |line| {
        let Some(declared) = dependency.declared.as_deref() else {
            return None;
        };
        let prefix = ['^', '~', '>', '=']
            .iter()
            .find(|c| declared.starts(**c))
            .map(|c| c.to_string())
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
        Some(line.replacen(
            &format!("@{}", dependency.resolved),
            &format!("@{latest}"),
            1,
        ))
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

fn rewrite_line_at(text: &str, line_number: usize, rewrite: impl Fn(&str) -> Option<String>) -> String {
    let lines: Vec<&str> = text.lines().collect();
    if line_number == 0 || line_number > lines.len() {
        return text.to_string();
    }
    let Some(updated) = rewrite(lines[line_number - 1]) else {
        return text.to_string();
    };
    let mut owned: Vec<String> = lines.iter().map(|line| line.to_string()).collect();
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

/// Parse the manifest's dependency tables: `name = "req"`,
/// `name = { version = "req", .. }`; path/git/workspace entries are skipped.
pub fn parse_cargo_manifest(text: &str) -> Vec<(String, String, usize, Option<String>)> {
    let mut out = Vec::new();
    let mut section = Section::Outside;
    for (index, raw) in text.lines().enumerate() {
        let line = raw.trim_start();
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        if line.starts_with('[') {
            let header = line.trim_start_matches('[').trim_end_matches(']');
            section = if matches!(header, "dependencies" | "dev-dependencies" | "build-dependencies") {
                Section::Table
            } else if header.starts_with("dependencies.")
                || header.starts_with("dev-dependencies.")
                || header == "workspace.dependencies"
            {
                // The dependency's name is in the header; member lines are
                // its options, not dependencies themselves.
                Section::SubTable
            } else {
                Section::Outside
            };
            continue;
        }
        if section != Section::Table {
            continue;
        }
        let Some((name, value)) = split_once(line, '=') else {
            continue;
        };
        let name = name.trim().trim_matches('"').to_string();
        let value = value.trim();
        if value.contains("path =") || value.contains("git =") || value.contains("workspace = true") {
            continue;
        }
        let requirement = if let Some(version) = version_key_of_inline_table(value) {
            version
        } else if let Some(quoted) = quoted_value(value) {
            quoted
        } else {
            continue;
        };
        out.push((name, requirement, index + 1, blocked_annotation(raw)));
    }
    out
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Section {
    Outside,
    Table,
    SubTable,
}

/// The `version = "…"` member of `{ version = "…", features = […] }`.
fn version_key_of_inline_table(value: &str) -> Option<String> {
    if !value.starts_with('{') {
        return None;
    }
    for part in split_top_level_commas(&value[1..value.len().saturating_sub(1)]) {
        if let Some((key, item)) = split_once(&part, '=') {
            if key.trim() == "version" {
                return quoted_value(item.trim());
            }
        }
    }
    None
}

fn split_top_level_commas(text: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut depth = 0;
    let mut current = String::new();
    for character in text.chars() {
        match character {
            '(' | '[' | '{' => {
                depth += 1;
                current.push(character);
            }
            ')' | ']' | '}' => {
                depth = depth.saturating_sub(1);
                current.push(character);
            }
            ',' if depth == 0 => {
                parts.push(current.clone());
                current.clear();
            }
            _ => current.push(character),
        }
    }
    if !current.trim().is_empty() {
        parts.push(current);
    }
    parts
}

/// `[[package]] name/version` pairs from a Cargo.lock.
pub fn parse_cargo_lock(text: &str) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    let mut name = None;
    for line in text.lines() {
        let line = line.trim();
        if let Some(value) = line.strip_prefix("name = ") {
            name = quoted_value(value).filter(|_| !line.starts_with("#"));
        } else if let Some(value) = line.strip_prefix("version = ") {
            if let (Some(name), Some(version)) = (name.take(), quoted_value(value)) {
                out.insert(name, version);
            }
        }
    }
    out
}

/// Dependency-object members of a package.json, with `"name//"` blocked notes.
pub fn parse_package_json(text: &str) -> Vec<(String, String, usize, Option<String>)> {
    let mut blocked: BTreeMap<String, String> = BTreeMap::new();
    for line in text.lines() {
        let Some((key, value)) = json_member(line) else {
            continue;
        };
        if let Some(name) = key.strip_suffix("//") {
            if let Some(url) = json_string(value) {
                blocked.insert(name.to_string(), url.trim_start_matches("blocked:").trim().to_string());
            }
        }
    }
    let mut out = Vec::new();
    let mut in_dependencies = false;
    for (index, line) in text.lines().enumerate() {
        let trimmed = line.trim();
        // A section header opens an object on the same line: `"key": {`.
        if trimmed.starts_with('"') && trimmed.ends_with("{") {
            if let Some((key, _)) = json_member(trim.trim_end_matches('{').trim_end()) {
                in_dependencies = matches!(
                    key.as_str(),
                    "dependencies" | "devDependencies" | "optionalDependencies"
                );
            }
            continue;
        }
        if !in_dependencies {
            continue;
        }
        let Some((key, value)) = json_member(line) else {
            continue;
        };
        if key.ends_with("//") {
            continue;
        }
        let Some(range) = json_string(value) else {
            continue;
        };
        out.push((
            key.clone(),
            range,
            index + 1,
            blocked.get(&key).cloned(),
        ));
    }
    out
}

/// A `"key": value` member on one line, when present.
fn json_member(line: &str) -> Option<(String, &str)> {
    let trimmed = line.trim_start();
    let rest = trimmed.strip_prefix('"')?;
    let end = rest.find('"')?;
    let value = &rest[end + 1..];
    let colon = value.find(':')?;
    Some((
        rest[..end].to_string(),
        value[colon + 1..].trim(),
    ))
}

fn json_string(value: &str) -> Option<String> {
    quoted_value(value.trim_end_matches(',').trim())
}

/// The hoisted resolutions of a package-lock v3 `packages` map.
pub fn parse_package_lock(text: &str) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    let mut current: Option<String> = None;
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(version) = trimmed.strip_prefix("\"version\": ") {
            if let (Some(name), Some(version)) = (current.take(), quoted_value(version.trim_end_matches(','))) {
                out.insert(name, version);
            }
        } else if trimmed.starts_with("\"node_modules/") && trimmed.ends_with("\": {") {
            let path = trimmed
                .trim_start_matches("\"node_modules/")
                .trim_end_matches("\": {");
            // Hoisted entries only: nested duplicates belong to their parent.
            if !path.contains("/node_modules/") {
                current = Some(path.to_string());
            }
        }
    }
    out
}

/// The `packages` entries of a bun.lock: `"name": ["name@version", …`.
pub fn parse_bun_lock(text: &str) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for line in text.lines() {
        let trimmed = line.trim();
        let Some(rest) = trimmed.strip_prefix('"') else {
            continue;
        };
        let Some(key_end) = rest.find("\": ") else {
            continue;
        };
        let name = &rest[..key_end];
        let after = rest[key_end + 3..].trim_start();
        let Some(array_first) = after.strip_prefix("[\"") else {
            continue;
        };
        let Some(spec_end) = array_first.find('"') else {
            continue;
        };
        let spec = &array_first[..spec_end];
        // The first array element repeats the name with its version attached.
        if let Some(version) = spec.strip_prefix(&format!("{name}@")) {
            out.insert(name.to_string(), version.to_string());
        }
    }
    out
}

/// `uses: owner/repo@ref` pins, local (`./…`) and docker refs excluded.
pub fn parse_uses_pins(text: &str) -> Vec<(String, String, usize)> {
    let mut out = Vec::new();
    for (index, line) in text.lines().enumerate() {
        let Some(rest) = line.trim_start().strip_prefix("uses:") else {
            continue;
        };
        let reference = rest.trim();
        if reference.starts_with("./") || reference.starts_with("docker://") {
            continue;
        }
        let Some((action, pin)) = split_once(reference, '@') else {
            continue;
        };
        if action.split('/').count() < 2 || blocked_annotation(line).is_some() {
            continue;
        }
        out.push((action.to_string(), pin.to_string(), index + 1));
    }
    out
}

/// `FROM image:tag` bases, `${VAR}` stages and `scratch` excluded.
pub fn parse_dockerfile_from(text: &str) -> Vec<(String, String, usize)> {
    let mut out = Vec::new();
    for (index, line) in text.lines().enumerate() {
        let trimmed = line.trim_start();
        let lower = trimmed.to_ascii_lowercase();
        let Some(rest) = lower.strip_prefix("from ") else {
            continue;
        };
        let first = rest.split_whitespace().next().unwrap_or_default();
        if first.is_empty() || first.starts_with("${") || first == "scratch" {
            continue;
        }
        let (image, tag) = match split_once(first, ':') {
            Some((image, tag)) => (image.to_string(), tag.to_string()),
            None => (first.to_string(), "latest".to_string()),
        };
        if blocked_annotation(line).is_some() {
            continue;
        }
        out.push((image, tag, index + 1));
    }
    out
}

/// The issue URL of a `# blocked: <url>` annotation on this line.
pub fn blocked_annotation(line: &str) -> Option<String> {
    let marker = "# blocked: ";
    let start = line.find(marker)?;
    let url = line[start + marker.len()..].trim();
    if url.starts_with("https://") && url.contains("/issues/") {
        Some(url.to_string())
    } else {
        None
    }
}

fn quoted_value(text: &str) -> Option<String> {
    let start = text.find('"')?;
    let rest = &text[start + 1..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

fn split_once(text: &str, separator: char) -> Option<(&str, &str)> {
    let index = text.find(separator)?;
    Some((&text[..index], &text[index + separator.len_utf8()..]))
}

/// `^1.2.3` -> `1.2.3`, `>=2.0.0` -> `2.0.0`, `=0.3.0` -> `0.3.0`.
pub fn base_version(range: &str) -> &str {
    let mut base = range;
    for operator in ["^", "~", ">=", "<=", ">", "=", "=="] {
        if let Some(stripped) = base.strip_prefix(operator) {
            base = stripped;
            break;
        }
    }
    base.trim()
}

fn without_v(version: &str) -> &str {
    version.strip_prefix('v').unwrap_or(version)
}

fn minor_of(version: &str) -> String {
    let parts: Vec<&str> = without_v(version).split('.').collect();
    parts.iter().take(2).copied().collect::<Vec<_>>().join(".")
}

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
        if let Some(rest) = trimmed.strip_prefix("latest ") {
            if let (Some(key), Some(latest)) = (&current, quoted_value(rest)) {
                entries.insert(key.clone(), (latest, String::new()));
            }
        }
    }
    Some(Snapshot { refreshed_at, entries })
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
                let latest = latest_github(&name)?;
                (latest, format!("https://api.github.com/repos/{name}/releases/latest"))
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

#[cfg(not(test))]
fn latest_crates_io(name: &str) -> Result<String, String> {
    let body = fetch(&format!("https://crates.io/api/v1/crates/{name}"))?;
    json_field(&body, "max_stable_version").ok_or_else(|| format!("crates.io answer for {name} had no max_stable_version"))
}

#[cfg(not(test))]
fn latest_npm(name: &str) -> Result<String, String> {
    let body = fetch(&format!("https://registry.npmjs.org/{name}"))?;
    let tags = body
        .find("\"dist-tags\"")
        .ok_or_else(|| format!("npm answer for {name} had no dist-tags"))?;
    json_field(&body[tags..], "latest").ok_or_else(|| format!("npm dist-tags for {name} had no latest"))
}

#[cfg(not(test))]
fn latest_github(repository: &str) -> Result<String, String> {
    let body = fetch(&format!("https://api.github.com/repos/{repository}/releases/latest"))?;
    json_field(&body, "tag_name").ok_or_else(|| format!("github answer for {repository} had no tag_name"))
}

#[cfg(not(test))]
fn latest_docker_hub(image: &str) -> Result<String, String> {
    let body = fetch(&format!("https://hub.docker.com/v2/repositories/{image}/tags?page_size=100"))?;
    let results = body
        .find("\"results\"")
        .ok_or_else(|| format!("docker hub answer for {image} had no results"))?;
    let mut cursor = &body[results..];
    while let Some(name_start) = cursor.find("\"name\":\"") {
        let after = &cursor[name_start + "\"name\":\"".len()..];
        let end = after
            .find('"')
            .ok_or_else(|| format!("docker hub tag list for {image} ended mid-string"))?;
        let tag = &after[..end];
        if tag != "latest" && tag.chars().next().is_some_and(|c| c.is_ascii_digit()) {
            return Ok(tag.to_string());
        }
        cursor = after;
    }
    Err(format!("docker hub tag list for {image} had no versioned tag"))
}

#[cfg(not(test))]
fn latest_rustc() -> Result<String, String> {
    let body = fetch("https://static.rust-lang.org/dist/channel-rust-stable.toml")?;
    let mut section = String::new();
    for line in body.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            section = trimmed.to_string();
        } else if section == "[rust]" {
            if let Some(rest) = trimmed.strip_prefix("version = ") {
                return quoted_value(rest).ok_or_else(|| "rust channel had no quoted version".to_string());
            }
        }
    }
    Err("rust channel had no [rust] version".to_string())
}

/// The value of `"key":"value"` (no spaces) anywhere in `body`.
#[cfg(not(test))]
fn json_field(body: &str, key: &str) -> Option<String> {
    let needle = format!("\"{key}\":\"");
    let start = body.find(&needle)? + needle.len();
    let end = body[start..].find('"')? + start;
    Some(body[start..end].to_string())
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
mod tests {
    use super::*;

    #[test]
    fn cargo_manifest_forms() {
        let text = "\
[package]
name = \"formal-ai\"

[dependencies]
base64 = \"0.23\"
clap = { version = \"4.6\", features = [\"derive\"] }
local-crate = { path = \"../local\" }
vendored = { git = \"https://example.com/vendored\" }
inherited = { workspace = true }
links-notation = \"0.16.1\" # blocked: https://github.com/link-foundation/lino-objects-codec/issues/60

[dev-dependencies]
pretty_assertions = \"1.4\"
";
        let parsed = parse_cargo_manifest(text);
        let names: Vec<&str> = parsed.iter().map(|(n, _, _, _)| n.as_str()).collect();
        assert_eq!(names, ["base64", "clap", "links-notation", "pretty_assertions"]);
        assert_eq!(parsed[0].1, "0.23");
        assert_eq!(parsed[0].2, 6);
        assert_eq!(parsed[1].1, "4.6");
        assert_eq!(
            parsed[2].3.as_deref(),
            Some("https://github.com/link-foundation/lino-objects-codec/issues/60")
        );
    }

    #[test]
    fn cargo_lock_pairs() {
        let text = "\
[[package]]
name = \"base64\"
version = \"0.23.0\"

[[package]]
name = \"clap\"
version = \"4.6.7\"
";
        let versions = parse_cargo_lock(text);
        assert_eq!(versions.get("clap").map(String::as_str), Some("4.6.7"));
    }

    #[test]
    fn package_json_members_and_blocked_notes() {
        let text = "\
{
  \"name\": \"fixture\",
  \"dependencies\": {
    \"react\": \"19.3.0\",
    \"electron\": \"^44.1.0\",
    \"electron//\": \"blocked: https://github.com/link-assistant/formal-ai/issues/1169\"
  },
  \"devDependencies\": {
    \"playwright\": \"^1.63.0\"
  }
}
";
        let parsed = parse_package_json(text);
        let names: Vec<&str> = parsed.iter().map(|(n, _, _, _)| n.as_str()).collect();
        assert_eq!(names, ["react", "electron", "playwright"]);
        assert_eq!(parsed[0].1, "19.3.0");
        assert_eq!(parsed[1].2, 5);
        assert_eq!(
            parsed[1].3.as_deref(),
            Some("https://github.com/link-assistant/formal-ai/issues/1169")
        );
    }

    #[test]
    fn package_lock_and_bun_lock_resolutions() {
        let npm_lock = "\
{
  \"packages\": {
    \"\": {},
    \"node_modules/react\": {
      \"version\": \"19.3.0\",
      \"resolved\": \"https://registry.npmjs.org/react/-/react-19.3.0.tgz\"
    },
    \"node_modules/react/node_modules/scheduler\": {
      \"version\": \"0.28.0\"
    }
  }
}
";
        let versions = parse_package_lock(npm_lock);
        assert_eq!(versions.get("react").map(String::as_str), Some("19.3.0"));
        assert!(!versions.contains_key("react@scheduler"));

        let bun_lock = "\
{
  \"workspaces\": { \"\": { \"dependencies\": { \"react\": \"19.3.0\" } } },
  \"packages\": {
    \"react\": [\"react@19.3.0\", \"\", {}, \"sha512-…\"],
    \"@scope/tool\": [\"@scope/tool@2.1.0\", \"\", {}, \"sha512-…\"]
  }
}
";
        let versions = parse_bun_lock(bun_lock);
        assert_eq!(versions.get("react").map(String::as_str), Some("19.3.0"));
        assert_eq!(versions.get("@scope/tool").map(String::as_str), Some("2.1.0"));
    }

    #[test]
    fn uses_and_from_pins() {
        let workflow = "\
steps:
  - uses: actions/checkout@v4
  - uses: ./local/action
  - uses: zizmorcore/zizmor-action@v0.6.4
";
        let pins = parse_uses_pins(workflow);
        assert_eq!(pins.len(), 2);
        assert_eq!(pins[1], ("zizmorcore/zizmor-action".to_string(), "v0.6.4".to_string(), 4));

        let dockerfile = "\
FROM ubuntu:24.04 AS build
FROM ${BINARY_SOURCE}-binary AS selected-binary
FROM konrad/box-dind:2.10.2
";
        let bases = parse_dockerfile_from(dockerfile);
        assert_eq!(bases.len(), 2);
        assert_eq!(bases[1], ("konrad/box-dind".to_string(), "2.10.2".to_string(), 3));
    }

    #[test]
    fn blocked_annotations_must_name_an_issue() {
        assert_eq!(
            blocked_annotation("x = \"1\" # blocked: https://github.com/o/r/issues/7"),
            Some("https://github.com/o/r/issues/7".to_string())
        );
        assert_eq!(blocked_annotation("x = \"1\" # blocked: not-a-url"), None);
        assert_eq!(blocked_annotation("x = \"1\""), None);
    }

    #[test]
    fn ranges_and_minors() {
        assert_eq!(base_version("^1.63.0"), "1.63.0");
        assert_eq!(base_version("=0.3.0"), "0.3.0");
        assert_eq!(base_version("~2.4"), "2.4");
        assert_eq!(minor_of("1.98.1"), "1.98");
        assert_eq!(minor_of("v7.0.1"), "7.0");
    }

    #[test]
    fn comparison_rules() {
        let snapshot = Snapshot {
            refreshed_at: String::new(),
            entries: [
                ((Ecosystem::CratesIo, "clap".to_string()), ("4.6.7".to_string(), String::new())),
                ((Ecosystem::CratesIo, "egg".to_string()), ("0.11.0".to_string(), String::new())),
                ((Ecosystem::Npm, "react".to_string()), ("19.3.0".to_string(), String::new())),
                ((Ecosystem::RustToolchain, "rustc".to_string()), ("1.98.1".to_string(), String::new())),
            ]
            .into_iter()
            .collect(),
        };
        let dependency = |ecosystem, name, resolved| Dependency {
            ecosystem,
            name: name.to_string(),
            declared: Some("0".to_string()),
            resolved: resolved.to_string(),
            path: PathBuf::from("manifest"),
            line: 1,
            blocked: None,
        };
        let dependencies = vec![
            dependency(Ecosystem::CratesIo, "clap", "4.6.7"),
            dependency(Ecosystem::CratesIo, "egg", "0.10.0"),
            dependency(Ecosystem::Npm, "react", "19.3.0"),
            dependency(Ecosystem::Npm, "absent", "1.0.0"),
            dependency(Ecosystem::RustToolchain, "rustc", "1.98"),
        ];
        let (findings, unverifiable) = compare(&dependencies, &snapshot);
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings[0].dependency.name, "egg");
        assert_eq!(unverifiable, ["npm/absent"]);
    }

    #[test]
    fn snapshot_round_trip() {
        let directory = std::env::temp_dir().join(format!(
            "issue-1169-snapshot-{}",
            std::process::id()
        ));
        fs::create_dir_all(&directory).expect("temp dir");
        let path = directory.join("snapshot.lino");
        let snapshot = Snapshot {
            refreshed_at: "2026-09-30T15:04:05Z".to_string(),
            entries: [
                ((Ecosystem::CratesIo, "clap".to_string()), ("4.6.7".to_string(), "https://crates.io/api/v1/crates/clap".to_string())),
                ((Ecosystem::Npm, "@scope/tool".to_string()), ("2.1.0".to_string(), "https://registry.npmjs.org/@scope/tool".to_string())),
            ]
            .into_iter()
            .collect(),
        };
        write_snapshot(&path, &snapshot);
        let read = read_snapshot(&path).expect("read back");
        assert_eq!(read.entries.get(&(Ecosystem::Npm, "@scope/tool".to_string())).map(|(v, _)| v.clone()), Some("2.1.0".to_string()));
        assert_eq!(read.entries.get(&(Ecosystem::CratesIo, "clap".to_string())).map(|(v, _)| v.clone()), Some("4.6.7".to_string()));
        let _ = fs::remove_dir_all(&directory);
    }

    #[test]
    fn apply_rewrites_manifests_not_locks() {
        let text = "\
[dependencies]
clap = { version = \"4.6\", features = [\"derive\"] }
egg = \"0.10.0\"
";
        let dependency = |name: &str, declared: &str, line: usize| Dependency {
            ecosystem: Ecosystem::CratesIo,
            name: name.to_string(),
            declared: Some(declared.to_string()),
            resolved: "0.10.0".to_string(),
            path: PathBuf::from("Cargo.toml"),
            line,
            blocked: None,
        };
        let rewritten = rewrite_cargo_line(text, &dependency("clap", "4.6", 3), "4.6.7");
        assert!(rewritten.contains("clap = { version = \"4.6.7\","), "{rewritten}");
        let rewritten = rewrite_cargo_line(text, &dependency("egg", "0.10.0", 4), "0.11.0");
        assert!(rewritten.contains("egg = \"0.11.0\""), "{rewritten}");

        let json = "{\n  \"dependencies\": {\n    \"marked\": \"^18.0.11\"\n  }\n}\n";
        let dependency = Dependency {
            ecosystem: Ecosystem::Npm,
            name: "marked".to_string(),
            declared: Some("^18.0.11".to_string()),
            resolved: "18.0.11".to_string(),
            path: PathBuf::from("package.json"),
            line: 3,
            blocked: None,
        };
        let rewritten = rewrite_npm_line(json, &dependency, "18.0.14");
        assert!(rewritten.contains("\"marked\": \"^18.0.14\""), "{rewritten}");
    }

    #[test]
    fn apply_leaves_blocked_lines_alone() {
        let blocked = Dependency {
            ecosystem: Ecosystem::CratesIo,
            name: "links-notation".to_string(),
            declared: Some("0.16.1".to_string()),
            resolved: "0.16.1".to_string(),
            path: PathBuf::from("Cargo.toml"),
            line: 3,
            blocked: Some("https://github.com/link-foundation/lino-objects-codec/issues/60".to_string()),
        };
        let text = "[dependencies]\nlinks-notation = \"0.16.1\" # blocked: https://github.com/link-foundation/lino-objects-codec/issues/60\n";
        let rewritten = rewrite_cargo_line(text, &blocked, "0.22.0");
        assert_eq!(rewritten, text);
    }
}
