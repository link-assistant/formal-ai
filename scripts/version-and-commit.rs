#!/usr/bin/env rust-script
//! Bump version in Cargo.toml and commit changes
//! Used by the CI/CD pipeline for releases
//!
//! IMPORTANT: This script checks crates.io (the source of truth for Rust packages),
//! NOT git tags. This is critical because:
//! - Git tags can exist without the package being published
//! - GitHub releases create tags but don't publish to crates.io
//! - Only crates.io publication means users can actually install the package
//!
//! Supports both single-language and multi-language repository structures:
//! - Single-language: Cargo.toml and changelog.d/ in repository root
//! - Multi-language: Cargo.toml and changelog.d/ in rust/ subfolder
//!
//! Usage: rust-script scripts/version-and-commit.rs --bump-type <major|minor|patch> [--description <desc>] [--rust-root <path>] [--tag-prefix <prefix>] [--release-label <label>]
//!
//! ```cargo
//! [dependencies]
//! regex = "1"
//! chrono = "0.4"
//! ureq = "2"
//! serde = { version = "1", features = ["derive"] }
//! serde_json = "1"
//! ```

use chrono::Utc;
use regex::Regex;
use serde::Deserialize;
use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{exit, Command};

#[path = "rust-paths.rs"]
mod rust_paths;
#[path = "self-hosting-metric.rs"]
pub mod self_hosting_metric;

const CHANGELOG_REBUILD_SCRIPT: &str = "experiments/issue_711_rebuild_changelog.mjs";
const FRAGMENT_RELEASE_MAP: &str = "docs/case-studies/issue-711/fragment-release-map.tsv";

fn get_arg(name: &str) -> Option<String> {
    let args: Vec<String> = env::args().collect();
    let flag = format!("--{}", name);

    if let Some(idx) = args.iter().position(|a| a == &flag) {
        return args.get(idx + 1).cloned();
    }

    let env_name = name.to_uppercase().replace('-', "_");
    env::var(&env_name).ok().filter(|s| !s.is_empty())
}

fn get_changelog_dir(rust_root: &str) -> String {
    if rust_root == "." {
        "./changelog.d".to_string()
    } else {
        format!("{}/changelog.d", rust_root)
    }
}

fn get_changelog_path(rust_root: &str) -> String {
    if rust_root == "." {
        "./CHANGELOG.md".to_string()
    } else {
        format!("{}/CHANGELOG.md", rust_root)
    }
}

fn set_output(key: &str, value: &str) {
    if let Ok(output_file) = env::var("GITHUB_OUTPUT") {
        if let Err(e) = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&output_file)
            .and_then(|mut f| writeln!(f, "{}={}", key, value))
        {
            eprintln!("Warning: Could not write to GITHUB_OUTPUT: {}", e);
        }
    }
    println!("Output: {}={}", key, value);
}

fn exec(command: &str, args: &[&str]) -> Result<String, String> {
    match Command::new(command).args(args).output() {
        Ok(output) => {
            if output.status.success() {
                Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Err(format!("Command failed: {}", stderr))
            }
        }
        Err(e) => Err(format!("Failed to execute: {}", e)),
    }
}

fn exec_check(command: &str, args: &[&str]) -> bool {
    Command::new(command)
        .args(args)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Run a git command inside `repo`, so the release logic can be exercised
/// against a throwaway repository in tests instead of the process CWD.
fn git(repo: &Path, args: &[&str]) -> Result<String, String> {
    let repo_str = repo.to_string_lossy().to_string();
    let mut full: Vec<&str> = vec!["-C", &repo_str];
    full.extend_from_slice(args);
    exec("git", &full)
}

/// Number of commits `origin/<branch>` has that HEAD does not.
fn commits_behind(repo: &Path, branch: &str) -> u32 {
    git(repo, &["rev-list", "--count", &format!("HEAD..origin/{}", branch)])
        .ok()
        .and_then(|out| out.trim().parse().ok())
        .unwrap_or(0)
}

/// Rebase onto the latest `origin/<branch>` so a concurrent release that landed
/// mid-job is picked up.
///
/// This MUST run while the working tree is still clean, before the version bump
/// is written and staged: `git rebase` refuses to run against a dirty index
/// ("cannot rebase: Your index contains uncommitted changes"), which is what
/// broke the release job. Syncing first also means the bump is computed from the
/// newest state of the branch instead of a stale checkout.
fn sync_with_remote(repo: &Path, branch: &str) -> Result<(), String> {
    // A missing remote is not fatal: the push step reports the real problem.
    if let Err(e) = git(repo, &["fetch", "origin", branch]) {
        eprintln!("Warning: Could not fetch origin/{}: {}", branch, e);
        return Ok(());
    }

    let behind = commits_behind(repo, branch);
    if behind == 0 {
        return Ok(());
    }

    println!(
        "Local branch is behind origin/{} by {} commit(s), rebasing...",
        branch, behind
    );
    if let Err(e) = git(repo, &["rebase", &format!("origin/{}", branch)]) {
        let _ = git(repo, &["rebase", "--abort"]);
        return Err(format!("Error rebasing onto origin/{}: {}", branch, e));
    }
    Ok(())
}

struct Version {
    major: u32,
    minor: u32,
    patch: u32,
    #[allow(dead_code)]
    pre_release: Option<String>,
}

impl Version {
    fn parse(content: &str) -> Option<Version> {
        let re = Regex::new(r#"(?m)^version\s*=\s*"(\d+)\.(\d+)\.(\d+)(?:-([^"]+))?""#).ok()?;
        let caps = re.captures(content)?;
        Some(Version {
            major: caps.get(1)?.as_str().parse().ok()?,
            minor: caps.get(2)?.as_str().parse().ok()?,
            patch: caps.get(3)?.as_str().parse().ok()?,
            pre_release: caps.get(4).map(|m| m.as_str().to_string()),
        })
    }

    fn bump(&self, bump_type: &str) -> String {
        match bump_type {
            "major" => format!("{}.0.0", self.major + 1),
            "minor" => format!("{}.{}.0", self.major, self.minor + 1),
            _ => format!("{}.{}.{}", self.major, self.minor, self.patch + 1),
        }
    }
}

fn update_cargo_toml(cargo_toml_path: &str, new_version: &str) -> Result<(), String> {
    let content =
        fs::read_to_string(cargo_toml_path).map_err(|e| format!("Failed to read {}: {}", cargo_toml_path, e))?;

    let re = Regex::new(r#"(?m)^(version\s*=\s*")[^"]+(")"#).unwrap();
    let new_content = re.replace(&content, format!("${{1}}{}${{2}}", new_version).as_str());

    fs::write(cargo_toml_path, new_content.as_ref())
        .map_err(|e| format!("Failed to write {}: {}", cargo_toml_path, e))?;

    println!("Updated {} to version {}", cargo_toml_path, new_version);
    Ok(())
}

/// Update the workspace-package `version = "..."` entry in Cargo.lock so that
/// it stays in sync with Cargo.toml in the same commit. Without this, every
/// release leaves Cargo.lock stale and the next code PR is forced to either
/// merge a `version = "X.Y.Z"` conflict in Cargo.lock or ship a follow-up
/// "sync Cargo.lock" commit.
///
/// Returns Ok(true) if Cargo.lock was updated, Ok(false) if it does not exist
/// or no matching entry was found.
fn update_cargo_lock(cargo_lock_path: &Path, crate_name: &str, new_version: &str) -> Result<bool, String> {
    if !cargo_lock_path.exists() {
        println!(
            "No Cargo.lock at {} (skipping lock-file version sync)",
            cargo_lock_path.display()
        );
        return Ok(false);
    }

    let path_str = cargo_lock_path.to_string_lossy();
    let content = fs::read_to_string(cargo_lock_path).map_err(|e| format!("Failed to read {}: {}", path_str, e))?;

    // Match the [[package]] entry for our crate:
    //   [[package]]
    //   name = "<crate_name>"
    //   version = "..."
    let pattern = format!(
        r#"(?m)(\[\[package\]\]\s*\nname\s*=\s*"{}"\s*\nversion\s*=\s*")[^"]+(")"#,
        regex::escape(crate_name),
    );
    let re = Regex::new(&pattern).map_err(|e| format!("Failed to build Cargo.lock regex: {}", e))?;

    if !re.is_match(&content) {
        println!(
            "Warning: Could not find [[package]] entry for `{}` in {} \
             (lock file left untouched)",
            crate_name, path_str
        );
        return Ok(false);
    }

    let new_content = re.replace(&content, format!("${{1}}{}${{2}}", new_version).as_str());

    if new_content == content {
        println!("Cargo.lock already at version {}", new_version);
        return Ok(false);
    }

    fs::write(cargo_lock_path, new_content.as_ref()).map_err(|e| format!("Failed to write {}: {}", path_str, e))?;

    println!("Updated {} to version {}", path_str, new_version);
    Ok(true)
}

#[derive(Deserialize)]
struct CratesIoCrate {
    versions: Option<Vec<CratesIoVersionEntry>>,
}

#[derive(Deserialize)]
struct CratesIoVersionEntry {
    num: String,
    yanked: bool,
}

fn get_crate_name(cargo_toml_path: &str) -> Result<String, String> {
    let content =
        fs::read_to_string(cargo_toml_path).map_err(|e| format!("Failed to read {}: {}", cargo_toml_path, e))?;

    let re = Regex::new(r#"(?m)^name\s*=\s*"([^"]+)""#).unwrap();

    if let Some(caps) = re.captures(&content) {
        Ok(caps.get(1).unwrap().as_str().to_string())
    } else {
        Err(format!("Could not find name in {}", cargo_toml_path))
    }
}

fn check_tag_exists(tag_prefix: &str, version: &str) -> bool {
    exec_check("git", &["rev-parse", &format!("{}{}", tag_prefix, version)])
}

fn check_version_on_crates_io(crate_name: &str, version: &str) -> bool {
    let url = format!("https://crates.io/api/v1/crates/{}/{}", crate_name, version);
    match ureq::get(&url)
        .set("User-Agent", "rust-script-version-and-commit")
        .call()
    {
        Ok(response) => response.status() == 200,
        Err(_) => false,
    }
}

fn get_max_published_version(crate_name: &str) -> Option<(u32, u32, u32)> {
    let url = format!("https://crates.io/api/v1/crates/{}", crate_name);
    match ureq::get(&url)
        .set("User-Agent", "rust-script-version-and-commit")
        .call()
    {
        Ok(response) => {
            if response.status() == 200 {
                if let Ok(body) = response.into_string() {
                    if let Ok(data) = serde_json::from_str::<CratesIoCrate>(&body) {
                        if let Some(versions) = data.versions {
                            let mut max: Option<(u32, u32, u32)> = None;
                            for v in &versions {
                                if v.yanked {
                                    continue;
                                }
                                let base = match v.num.split('-').next() {
                                    Some(b) => b,
                                    None => continue,
                                };
                                let parts: Vec<&str> = base.split('.').collect();
                                if parts.len() == 3 {
                                    if let (Ok(a), Ok(b), Ok(c)) = (
                                        parts[0].parse::<u32>(),
                                        parts[1].parse::<u32>(),
                                        parts[2].parse::<u32>(),
                                    ) {
                                        let tuple = (a, b, c);
                                        if max.map_or(true, |m| tuple > m) {
                                            max = Some(tuple);
                                        }
                                    }
                                }
                            }
                            return max;
                        }
                    }
                }
            }
            None
        }
        Err(_) => None,
    }
}

fn ensure_version_exceeds_published(
    version_str: &str,
    crate_name: &str,
    tag_prefix: &str,
    max_published: Option<(u32, u32, u32)>,
) -> String {
    let parts: Vec<&str> = version_str
        .split('-')
        .next()
        .unwrap_or(version_str)
        .split('.')
        .collect();
    if parts.len() != 3 {
        return version_str.to_string();
    }

    let mut major: u32 = parts[0].parse().unwrap_or(0);
    let mut minor: u32 = parts[1].parse().unwrap_or(0);
    let mut patch: u32 = parts[2].parse().unwrap_or(0);

    if let Some((pub_major, pub_minor, pub_patch)) = max_published {
        if (major, minor, patch) <= (pub_major, pub_minor, pub_patch) {
            println!(
                "Version {}.{}.{} is not greater than max published {}.{}.{}, adjusting to {}.{}.{}",
                major,
                minor,
                patch,
                pub_major,
                pub_minor,
                pub_patch,
                pub_major,
                pub_minor,
                pub_patch + 1
            );
            major = pub_major;
            minor = pub_minor;
            patch = pub_patch + 1;
        }
    }

    let mut candidate = format!("{}.{}.{}", major, minor, patch);
    let mut safety_counter = 0;
    while (check_tag_exists(tag_prefix, &candidate) || check_version_on_crates_io(crate_name, &candidate))
        && safety_counter < 100
    {
        println!(
            "Version {} already has a git tag or is published on crates.io, bumping patch",
            candidate
        );
        patch += 1;
        candidate = format!("{}.{}.{}", major, minor, patch);
        safety_counter += 1;
    }

    if safety_counter >= 100 {
        eprintln!("Error: Could not find an unpublished version after 100 attempts");
        exit(1);
    }

    candidate
}

fn strip_frontmatter(content: &str) -> String {
    let re = Regex::new(r"(?s)^---\s*\n.*?\n---\s*\n(.*)$").unwrap();
    if let Some(caps) = re.captures(content) {
        caps.get(1).unwrap().as_str().trim().to_string()
    } else {
        content.trim().to_string()
    }
}

fn remove_changelog_fragments(files: &[PathBuf]) {
    for file in files {
        fs::remove_file(file).unwrap_or_else(|e| panic!("Failed to remove {}: {}", file.display(), e));
        println!("Removed changelog fragment {}", file.display());
    }
}

fn collect_changelog_with_date(changelog_dir: &str, changelog_file: &str, version: &str, date_str: &str) -> bool {
    let dir_path = Path::new(changelog_dir);
    if !dir_path.exists() {
        return false;
    }

    let mut files: Vec<_> = match fs::read_dir(dir_path) {
        Ok(entries) => entries
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| {
                p.extension().map_or(false, |ext| ext == "md")
                    && p.file_name().map_or(false, |name| name != "README.md")
            })
            .collect(),
        Err(_) => return false,
    };

    if files.is_empty() {
        return false;
    }

    files.sort();

    let fragments: Vec<String> = files
        .iter()
        .filter_map(|f| fs::read_to_string(f).ok())
        .map(|c| strip_frontmatter(&c))
        .filter(|c| !c.is_empty())
        .collect();

    if fragments.is_empty() {
        return false;
    }

    // No leading newline: `lines[..idx]` already ends with the blank line that
    // follows the insert marker, so opening with one produced a second blank
    // line that `issue_711_rebuild_changelog.mjs --check` rejects.
    let new_entry = format!("## [{}] - {}\n\n{}\n", version, date_str, fragments.join("\n\n"));

    if !Path::new(changelog_file).exists() {
        return false;
    }

    let mut content = fs::read_to_string(changelog_file).unwrap_or_default();
    let lines: Vec<&str> = content.lines().collect();
    let mut insert_index = None;

    for (i, line) in lines.iter().enumerate() {
        if line.starts_with("## [") {
            insert_index = Some(i);
            break;
        }
    }

    if let Some(idx) = insert_index {
        let mut new_lines: Vec<String> = lines[..idx].iter().map(|s| s.to_string()).collect();
        new_lines.push(new_entry.clone());
        new_lines.extend(lines[idx..].iter().map(|s| s.to_string()));
        // `lines()` drops the trailing newline and `join` never restores it, so
        // every release used to strip the file's final newline.
        content = format!("{}\n", new_lines.join("\n"));
    } else {
        content.push_str(&new_entry);
    }

    fs::write(changelog_file, content).expect("Failed to write changelog");
    remove_changelog_fragments(&files);

    println!("Collected {} changelog fragment(s)", files.len());
    true
}

fn regenerate_release_artifacts(version: &str, date: &str) -> Result<bool, String> {
    if !Path::new(CHANGELOG_REBUILD_SCRIPT).is_file() {
        return Ok(false);
    }
    exec(
        "node",
        &[
            CHANGELOG_REBUILD_SCRIPT,
            "--write",
            "--ref",
            "HEAD",
            "--pending-release",
            version,
            "--pending-date",
            date,
        ],
    )?;
    println!("Regenerated changelog and fragment release map");
    Ok(true)
}

fn record_self_hosting_release(tag_prefix: &str, new_version: &str) -> Result<PathBuf, String> {
    let repo = PathBuf::from(exec("git", &["rev-parse", "--show-toplevel"])?);
    let tag_pattern = format!("{tag_prefix}[0-9]*");
    let since = exec(
        "git",
        &[
            "describe",
            "--tags",
            "--match",
            &tag_pattern,
            "--abbrev=0",
            "HEAD",
        ],
    )?;
    let tag = format!("{tag_prefix}{new_version}");
    let ledger = repo.join("data/meta/self-hosting-ledger.lino");
    let row = self_hosting_metric::record_release(&repo, &ledger, &tag, &since, "HEAD", 3)?;
    println!(
        "Recorded self-hosting metric for {tag}: {} ({}/{} changed lines)",
        self_hosting_metric::format_percentage(row.percentage_basis_points),
        row.self_authored_lines,
        row.changed_lines,
    );
    Ok(ledger)
}

#[cfg(test)]
mod tests {
    use super::collect_changelog_with_date;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(name: &str) -> PathBuf {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let path = std::env::temp_dir().join(format!("version-and-commit-{name}-{nanos}"));
        fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn changelog_collection_consumes_fragments_once_and_keeps_readme() {
        let repo = temp_dir("changelog-cleanup");
        let changelog_dir = repo.join("changelog.d");
        fs::create_dir_all(&changelog_dir).unwrap();

        let first_fragment = changelog_dir.join("20260714_fix_release_loop.md");
        let second_fragment = changelog_dir.join("20260714_note_release_loop.md");
        fs::write(
            &first_fragment,
            r#"---
bump: patch
---

### Fixed
- Prevent already-collected changelog fragments from triggering another release.
"#,
        )
        .unwrap();
        fs::write(
            &second_fragment,
            r#"### Changed
- Keep release commits from leaving stale changelog fragments behind.
"#,
        )
        .unwrap();
        fs::write(changelog_dir.join("README.md"), "Fragment instructions\n").unwrap();

        let changelog = repo.join("CHANGELOG.md");
        fs::write(
            &changelog,
            r#"# Changelog

## [0.1.0] - 2026-01-01

### Added
- Initial release
"#,
        )
        .unwrap();

        collect_changelog_with_date(
            changelog_dir.to_str().unwrap(),
            changelog.to_str().unwrap(),
            "0.2.0",
            "2026-07-14",
        );

        let after_first_release = fs::read_to_string(&changelog).unwrap();
        assert!(after_first_release.contains("## [0.2.0] - 2026-07-14"));
        assert!(after_first_release.contains("Prevent already-collected"));
        assert!(after_first_release.contains("Keep release commits"));
        assert!(!after_first_release.contains("bump: patch"));
        assert!(!first_fragment.exists());
        assert!(!second_fragment.exists());
        assert!(changelog_dir.join("README.md").exists());

        collect_changelog_with_date(
            changelog_dir.to_str().unwrap(),
            changelog.to_str().unwrap(),
            "0.3.0",
            "2026-07-15",
        );
        assert_eq!(fs::read_to_string(&changelog).unwrap(), after_first_release);
    }

    /// The release commit must write CHANGELOG.md in exactly the shape
    /// `experiments/issue_711_rebuild_changelog.mjs --check` reconstructs from
    /// git history, byte for byte. It did not: the entry was spliced in before
    /// the first `## [` line, but `lines[..idx]` already ends with the blank
    /// line that follows the insert marker and `new_entry` opened with another
    /// `\n`, so every release left a second blank line after the marker. And
    /// `lines()` drops the trailing newline that `join("\n")` never restores,
    /// so every release also stripped the final newline. Both survived because
    /// the check only runs when the lint job's path filter fires, which a
    /// release commit does not trigger -- so `main` went red on the next
    /// unrelated PR and someone hand-fixed it ("refresh reconstructed release
    /// artifacts", repeatedly).
    #[test]
    fn release_writes_the_changelog_exactly_as_reconstruction_expects() {
        let repo = temp_dir("changelog-canonical-shape");
        let changelog_dir = repo.join("changelog.d");
        fs::create_dir_all(&changelog_dir).unwrap();
        fs::write(
            changelog_dir.join("fragment.md"),
            "---\nbump: patch\n---\n\n### Fixed\n- A representative fragment.\n",
        )
        .unwrap();

        // The canonical shape: marker, exactly one blank line, newest section,
        // one blank line between sections, single trailing newline.
        let changelog = repo.join("CHANGELOG.md");
        let before = "\
# Changelog

<!-- changelog-insert-here -->

## [0.1.0] - 2026-01-01

### Added
- Initial release
";
        fs::write(&changelog, before).unwrap();

        collect_changelog_with_date(
            changelog_dir.to_str().unwrap(),
            changelog.to_str().unwrap(),
            "0.2.0",
            "2026-07-16",
        );

        let expected = "\
# Changelog

<!-- changelog-insert-here -->

## [0.2.0] - 2026-07-16

### Fixed
- A representative fragment.

## [0.1.0] - 2026-01-01

### Added
- Initial release
";
        assert_eq!(fs::read_to_string(&changelog).unwrap(), expected);
    }

    fn git_ok(repo: &Path, args: &[&str]) {
        super::git(repo, args).unwrap_or_else(|e| panic!("git {:?} failed: {}", args, e));
    }

    /// Set up a bare origin plus a clone whose identity is configured, so the
    /// release helper can commit without inheriting the developer's git config.
    fn repo_with_origin(name: &str) -> (PathBuf, PathBuf) {
        let root = temp_dir(name);
        let origin = root.join("origin.git");
        let work = root.join("work");
        git_ok(
            &root,
            &["init", "--bare", "--initial-branch=main", origin.to_str().unwrap()],
        );
        git_ok(&root, &["clone", origin.to_str().unwrap(), work.to_str().unwrap()]);
        git_ok(&work, &["config", "user.email", "ci@example.com"]);
        git_ok(&work, &["config", "user.name", "CI"]);
        fs::write(work.join("Cargo.toml"), "version = \"0.1.0\"\n").unwrap();
        git_ok(&work, &["add", "-A"]);
        git_ok(&work, &["commit", "-m", "init"]);
        git_ok(&work, &["push", "-u", "origin", "main"]);
        (root, work)
    }

    /// Land a commit on origin/main from an independent clone, simulating a
    /// concurrent release that pushes while this job is running.
    fn push_concurrent_commit(root: &Path, subject: &str) {
        let other = root.join("other");
        git_ok(
            root,
            &[
                "clone",
                root.join("origin.git").to_str().unwrap(),
                other.to_str().unwrap(),
            ],
        );
        git_ok(&other, &["config", "user.email", "other@example.com"]);
        git_ok(&other, &["config", "user.name", "Other"]);
        fs::write(other.join("NOTES.md"), "concurrent\n").unwrap();
        git_ok(&other, &["add", "-A"]);
        git_ok(&other, &["commit", "-m", subject]);
        git_ok(&other, &["push", "origin", "main"]);
    }

    /// Regression test for the release failure in CI run 29484631709:
    /// "Error rebasing onto origin/main: Command failed: error: cannot rebase:
    /// Your index contains uncommitted changes." A concurrent release pushed to
    /// origin/main while this job had already staged its version bump, so the
    /// sync must happen before anything is written and staged.
    #[test]
    fn syncs_with_concurrent_release_then_commits_bump_on_top() {
        let (root, work) = repo_with_origin("concurrent-release");
        push_concurrent_commit(&root, "concurrent change");

        // Order under test: sync while clean, then bump, stage and commit.
        super::sync_with_remote(&work, "main").expect("sync must succeed against an advanced remote");
        fs::write(work.join("Cargo.toml"), "version = \"0.2.0\"\n").unwrap();
        git_ok(&work, &["add", "Cargo.toml"]);
        git_ok(&work, &["commit", "-m", "chore: release v0.2.0"]);

        // The release commit sits on top of the concurrent one, and both survive.
        let log = super::git(&work, &["log", "--format=%s"]).unwrap();
        assert_eq!(
            log.lines().collect::<Vec<_>>(),
            vec!["chore: release v0.2.0", "concurrent change", "init"]
        );
        assert_eq!(
            fs::read_to_string(work.join("Cargo.toml")).unwrap(),
            "version = \"0.2.0\"\n"
        );
        assert_eq!(fs::read_to_string(work.join("NOTES.md")).unwrap(), "concurrent\n");
        assert!(super::git(&work, &["status", "--porcelain"]).unwrap().is_empty());
    }

    /// Pins the ordering itself: syncing after the bump is staged is exactly the
    /// failure CI hit, so this must stay impossible.
    #[test]
    fn syncing_after_staging_the_bump_fails() {
        let (root, work) = repo_with_origin("sync-after-staging");
        push_concurrent_commit(&root, "concurrent change");

        fs::write(work.join("Cargo.toml"), "version = \"0.2.0\"\n").unwrap();
        git_ok(&work, &["add", "Cargo.toml"]);

        let err = super::sync_with_remote(&work, "main")
            .expect_err("rebase must refuse to run against a staged version bump");
        assert!(err.contains("cannot rebase"), "unexpected error: {}", err);
    }

    /// Being *ahead* of origin is not a reason to rebase. The old
    /// `local != remote` check treated ahead and diverged as "behind".
    #[test]
    fn does_not_rebase_when_only_ahead_of_remote() {
        let (_root, work) = repo_with_origin("ahead-of-remote");

        fs::write(work.join("Cargo.toml"), "version = \"0.2.0\"\n").unwrap();
        git_ok(&work, &["add", "Cargo.toml"]);
        git_ok(&work, &["commit", "-m", "chore: release v0.2.0"]);

        assert_eq!(super::commits_behind(&work, "main"), 0);
        // Clean no-op even though HEAD != origin/main.
        super::sync_with_remote(&work, "main").expect("being ahead must not trigger a rebase");
        let log = super::git(&work, &["log", "--format=%s"]).unwrap();
        assert_eq!(log.lines().collect::<Vec<_>>(), vec!["chore: release v0.2.0", "init"]);
    }
}

fn main() {
    let bump_type = match get_arg("bump-type") {
        Some(bt) => bt,
        None => {
            eprintln!("Usage: rust-script scripts/version-and-commit.rs --bump-type <major|minor|patch> [--description <desc>] [--rust-root <path>] [--tag-prefix <prefix>] [--release-label <label>]");
            exit(1);
        }
    };

    if !["major", "minor", "patch"].contains(&bump_type.as_str()) {
        eprintln!("Invalid bump type: {}. Must be major, minor, or patch.", bump_type);
        exit(1);
    }

    let description = get_arg("description");
    let tag_prefix = get_arg("tag-prefix").unwrap_or_else(|| "v".to_string());
    let release_label = get_arg("release-label");
    let rust_root = match rust_paths::get_rust_root(None, true) {
        Ok(root) => root,
        Err(e) => {
            eprintln!("Error: {}", e);
            exit(1);
        }
    };
    let cargo_toml = rust_paths::get_cargo_toml_path(&rust_root);
    let package_manifest = match rust_paths::get_package_manifest_path(&cargo_toml) {
        Ok(path) => path,
        Err(e) => {
            eprintln!("Error: {}", e);
            exit(1);
        }
    };
    let changelog_dir = get_changelog_dir(&rust_root);
    let changelog_file = get_changelog_path(&rust_root);

    // Configure git
    let _ = exec("git", &["config", "user.name", "github-actions[bot]"]);
    let _ = exec(
        "git",
        &["config", "user.email", "github-actions[bot]@users.noreply.github.com"],
    );

    // Sync with the remote while the tree is still clean, so a concurrent
    // release is picked up before the bump is computed and staged.
    let current_branch = exec("git", &["rev-parse", "--abbrev-ref", "HEAD"]).unwrap_or_else(|_| "main".to_string());
    if let Err(e) = sync_with_remote(Path::new("."), &current_branch) {
        eprintln!("{}", e);
        exit(1);
    }

    // Get current version
    let content = match fs::read_to_string(&package_manifest) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading {}: {}", package_manifest.display(), e);
            exit(1);
        }
    };

    let current = match Version::parse(&content) {
        Some(v) => v,
        None => {
            eprintln!("Error: Could not parse version from {}", package_manifest.display());
            exit(1);
        }
    };

    let initial_bump = current.bump(&bump_type);

    let crate_name = match get_crate_name(package_manifest.to_string_lossy().as_ref()) {
        Ok(name) => name,
        Err(e) => {
            eprintln!("Error: {}", e);
            exit(1);
        }
    };

    let max_published = get_max_published_version(&crate_name);
    if let Some((ma, mi, pa)) = max_published {
        println!("Max published version on crates.io: {}.{}.{}", ma, mi, pa);
    } else {
        println!("No versions published on crates.io yet (or crate not found)");
    }

    println!(
        "Initial bump ({}) from {}.{}.{}: {}",
        bump_type, current.major, current.minor, current.patch, initial_bump
    );

    let new_version = ensure_version_exceeds_published(&initial_bump, &crate_name, &tag_prefix, max_published);

    if new_version != initial_bump {
        println!(
            "Adjusted version from {} to {} to exceed published versions",
            initial_bump, new_version
        );
    }

    println!("Final release version: {}", new_version);

    let self_hosting_ledger = match record_self_hosting_release(&tag_prefix, &new_version) {
        Ok(path) => path,
        Err(e) => {
            eprintln!("Error recording self-hosting release metric: {}", e);
            exit(1);
        }
    };

    // Update version in Cargo.toml
    if let Err(e) = update_cargo_toml(package_manifest.to_string_lossy().as_ref(), &new_version) {
        eprintln!("Error: {}", e);
        exit(1);
    }

    // Update the workspace-package entry in Cargo.lock so it stays in sync.
    let cargo_lock_path = rust_paths::get_cargo_lock_path(&rust_root);
    let lock_updated = match update_cargo_lock(&cargo_lock_path, &crate_name, &new_version) {
        Ok(updated) => updated,
        Err(e) => {
            eprintln!("Error updating Cargo.lock: {}", e);
            exit(1);
        }
    };

    // Collect changelog fragments, then reconstruct the release artifacts while
    // the deletions are still visible against HEAD. The map records only
    // fragment -> release, so it can be committed atomically with the release
    // instead of depending on that commit's not-yet-existing SHA.
    let release_date = Utc::now().format("%Y-%m-%d").to_string();
    let collected = collect_changelog_with_date(&changelog_dir, &changelog_file, &new_version, &release_date);
    let reconstructed = if collected {
        match regenerate_release_artifacts(&new_version, &release_date) {
            Ok(reconstructed) => reconstructed,
            Err(e) => {
                eprintln!("Error regenerating release artifacts: {}", e);
                exit(1);
            }
        }
    } else {
        false
    };

    // Stage Cargo.toml, Cargo.lock (when bumped), CHANGELOG.md, the release
    // metric ledger, and consumed fragments.
    let package_manifest_str = package_manifest.to_string_lossy().to_string();
    let cargo_lock_str = cargo_lock_path.to_string_lossy().to_string();
    let self_hosting_ledger_str = self_hosting_ledger.to_string_lossy().to_string();
    let mut add_args: Vec<&str> = vec![
        "add",
        &package_manifest_str,
        &changelog_file,
        &self_hosting_ledger_str,
    ];
    if lock_updated {
        add_args.push(&cargo_lock_str);
    }
    if let Err(e) = exec("git", &add_args) {
        eprintln!("Error staging release files: {}", e);
        exit(1);
    }
    if Path::new(&changelog_dir).exists() {
        if let Err(e) = exec("git", &["add", "-A", &changelog_dir]) {
            eprintln!("Error staging consumed changelog fragments: {}", e);
            exit(1);
        }
    }
    if reconstructed {
        if let Err(e) = exec("git", &["add", FRAGMENT_RELEASE_MAP]) {
            eprintln!("Error staging fragment release map: {}", e);
            exit(1);
        }
    }

    // Check if there are changes to commit
    if exec_check("git", &["diff", "--cached", "--quiet"]) {
        println!("No changes to commit");
        set_output("version_committed", "false");
        set_output("new_version", &new_version);
        return;
    }

    // Commit the staged release files (the rebase already happened, while clean).
    let label_suffix = release_label.as_ref().map(|l| format!(" ({})", l)).unwrap_or_default();
    let commit_msg = match &description {
        Some(desc) => format!(
            "chore: release {}{}{}\n\n{}",
            tag_prefix, new_version, label_suffix, desc
        ),
        None => format!("chore: release {}{}{}", tag_prefix, new_version, label_suffix),
    };

    if let Err(e) = exec("git", &["commit", "-m", &commit_msg]) {
        eprintln!("Error committing: {}", e);
        exit(1);
    }
    println!("Committed version {}", new_version);

    // Push changes with retry (handles concurrent pushes in multi-workflow repos)
    let max_push_attempts = 3;
    for attempt in 1..=max_push_attempts {
        match exec("git", &["push"]) {
            Ok(_) => break,
            Err(e) => {
                if attempt < max_push_attempts {
                    eprintln!("Push failed (attempt {}/{}): {}", attempt, max_push_attempts, e);
                    eprintln!("Pulling with rebase and retrying...");
                    if let Err(rebase_err) = exec("git", &["pull", "--rebase", "origin", &current_branch]) {
                        eprintln!("Error during pull --rebase: {}", rebase_err);
                        let _ = exec("git", &["rebase", "--abort"]);
                        exit(1);
                    }
                } else {
                    eprintln!("Error pushing after {} attempts: {}", max_push_attempts, e);
                    exit(1);
                }
            }
        }
    }

    // Tag only once the release commit is on the remote, so a `pull --rebase`
    // retry above can never leave the tag on an orphaned pre-rebase commit.
    let tag_name = format!("{}{}", tag_prefix, new_version);
    let tag_msg = match &description {
        Some(desc) => format!("Release {}{}\n\n{}", tag_name, label_suffix, desc),
        None => format!("Release {}{}", tag_name, label_suffix),
    };

    if let Err(e) = exec("git", &["tag", "-a", &tag_name, "-m", &tag_msg]) {
        eprintln!("Error creating tag: {}", e);
        exit(1);
    }
    println!("Created tag {}", tag_name);

    if let Err(e) = exec("git", &["push", "--tags"]) {
        eprintln!("Error pushing tags: {}", e);
        exit(1);
    }
    println!("Pushed changes and tags");

    set_output("version_committed", "true");
    set_output("new_version", &new_version);
}
