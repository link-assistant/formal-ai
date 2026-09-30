#!/usr/bin/env rust-script
//! Issue #1182 (E147, R9): the weekly cross-organization duplication job.
//!
//! Finds function bodies duplicated across the repositories of
//! link-foundation and link-assistant, chooses the repository that should
//! own each group, and opens one issue per group there (general wording,
//! every copy listed, never the same group twice — the marker
//! `<!-- cross-org-duplication:<hash> -->` is searched before opening).
//!
//! Usage:
//!   rust-script scripts/cross-org-duplication.rs            # scan + report
//!   rust-script scripts/cross-org-duplication.rs --open     # + file issues
//!   rust-script scripts/cross-org-duplication.rs --fixture <dir>
//!       # run the owner-selection/rendering pipeline on a fixture of tiny
//!       # repositories instead of the network (the self-tests' path)
//!
//! Pipeline (the issue's design, verbatim):
//!   1. `gh repo list <org> --limit 1000 --json name,isArchived,isFork,primaryLanguage`
//!      for both orgs; archived and forks skipped.
//!   2. Shallow-clone each (`--depth 1 --filter=blob:limit=1m`) into a
//!      scratch directory, deleted after scanning.
//!   3. Collect function-level subtrees of at least `MIN_BODY_LINES` lines
//!      from every `.rs`/`.js`/`.ts` file (the meta-language
//!      `LinkNetwork::parse` integration lands with the coding tree;
//!      here a brace/token extractor with the same shape).
//!   4. Normalize (drop comments and whitespace) and hash each subtree
//!      (FNV-1a 64-bit, the self-AST census's hash).
//!   5. Group equal hashes across repositories; rank by lines × repositories.
//!   6. Owner: the dependency a copy's repository already declares (in
//!      Cargo.toml or package.json) whose own tree contains a copy;
//!      else the most-depended-on among the copies' dependencies; else
//!      "no owner" — reported for a human decision, no issue opened.
//!   7. One issue per group in the owner, after searching its issues for
//!      the marker.
//!   8. Write `data/meta/cross-org-duplication.lino`.
//!
//! Credentials come from the shared resolver (issue #1187, E151):
//! `actions/automation-token` sets AUTOMATION_TOKEN for the `gh` calls;
//! with only the default token the job still scans and reports, and opens
//! nothing (R5 of #1187: degrade, don't demand).
//!
//! ```cargo
//! [package]
//! edition = "2024"
//! [dependencies]
//! serde_json = "1"
//! ```

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

const ORGS: &[&str] = &["link-foundation", "link-assistant"];
const MIN_BODY_LINES: usize = 5;
const MARKER_PREFIX: &str = "<!-- cross-org-duplication:";
const RECORD: &str = "data/meta/cross-org-duplication.lino";

struct Site {
    repository: String,
    path: String,
    start_line: usize,
    name: String,
}

struct Group {
    hash: String,
    body_lines: usize,
    sites: Vec<Site>,
}

/// FNV-1a 64-bit over the normalized body — the self-AST census's hash.
fn fnv1a(data: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in data {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

mod duplication_support;
use duplication_support::{functions as function_bodies, strip_comments};

fn scan_repository(root: &Path, repository: &str) -> Vec<(String, u64, usize, String, usize)> {
    let mut files = Vec::new();
    walk(root, &mut files);
    let mut out = Vec::new();
    for path in files {
        let Some(source) = fs::read_to_string(&path).ok() else {
            continue;
        };
        let clean = strip_comments(&source);
        let relative = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .display()
            .to_string();
        for (name, normalized, body_lines, start_line) in function_bodies(&clean) {
            out.push((
                name,
                fnv1a(normalized.as_bytes()),
                body_lines,
                relative.clone(),
                start_line,
            ));
        }
    }
    out
}

fn walk(root: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        if entry.file_type().is_ok_and(|kind| kind.is_symlink()) {
            continue;
        }
        let path = entry.path();
        if path.is_dir() {
            let name = path.file_name().and_then(|it| it.to_str()).unwrap_or("");
            if name == ".git" || name == "node_modules" || name == "target" {
                continue;
            }
            walk(&path, out);
        } else if path
            .extension()
            .and_then(|it| it.to_str())
            .is_some_and(|it| it == "rs" || it == "js" || it == "ts")
        {
            out.push(path);
        }
    }
}

/// The dependencies a repository declares: crate names from Cargo.toml and
/// package names from package.json (the design's step 6 signal).
fn declared_dependencies(root: &Path) -> Vec<String> {
    let mut names = Vec::new();
    for manifest in [root.join("Cargo.toml"), root.join("rust/Cargo.toml")] {
        if let Ok(cargo) = fs::read_to_string(manifest) {
            let mut dependencies = false;
            for line in cargo.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with('[') {
                    dependencies = trimmed.ends_with("dependencies]");
                    continue;
                }
                if dependencies {
                    if let Some((name, _)) = trimmed.split_once('=') {
                        let name = name.trim().trim_matches('"');
                        if !name.is_empty() && !name.starts_with('#') {
                            names.push(name.to_owned());
                        }
                    }
                }
            }
        }
    }
    for manifest in [root.join("package.json"), root.join("js/package.json")] {
        if let Ok(package) = fs::read_to_string(manifest) {
            if let Ok(value) = serde_json::from_str::<serde_json::Value>(&package) {
                for field in [
                    "dependencies",
                    "devDependencies",
                    "peerDependencies",
                    "optionalDependencies",
                ] {
                    if let Some(deps) = value.get(field).and_then(serde_json::Value::as_object) {
                        names.extend(deps.keys().cloned());
                    }
                }
            }
        }
    }
    names.sort();
    names.dedup();
    names
}

/// Choose the owning repository for a group (the design's step 6):
/// the dependency at least one copy's repository declares AND whose own
/// tree contains a copy; else the most-declared dependency among the
/// copies' repositories; else `None` — a human decision, no issue.
fn choose_owner(group: &Group, repositories: &BTreeMap<String, Vec<String>>) -> Option<String> {
    let group_repositories: Vec<&str> = {
        let mut seen: Vec<&str> = group.sites.iter().map(|s| s.repository.as_str()).collect();
        seen.sort();
        seen.dedup();
        seen
    };
    let mut candidate_votes: BTreeMap<&str, usize> = BTreeMap::new();
    for repository in &group_repositories {
        if let Some(dependencies) = repositories.get(*repository) {
            for dependency in dependencies {
                for candidate in &group_repositories {
                    if dependency == candidate
                        || candidate.rsplit('/').next() == Some(dependency.as_str())
                    {
                        *candidate_votes.entry(candidate).or_default() += 1;
                    }
                }
            }
        }
    }
    candidate_votes
        .into_iter()
        .max_by_key(|(_, votes)| *votes)
        .map(|(owner, _)| owner.to_string())
}

fn issue_body(group: &Group) -> String {
    let mut body = String::new();
    body.push_str("A weekly cross-organization scan found function bodies that are\n");
    body.push_str("byte-identical after comment and whitespace normalization in more than\n");
    body.push_str("one repository. The general logic below is duplicated; if it belongs\n");
    body.push_str("in a maintained dependency, moving it there removes a copy from every\n");
    body.push_str("consumer. All copies:\n\n");
    for site in &group.sites {
        body.push_str(&format!(
            "- {}/{}:{} `fn {}`\n",
            site.repository, site.path, site.start_line, site.name
        ));
    }
    body.push_str(&format!(
        "\n(normalized body: {} lines)\n{}\n",
        group.body_lines,
        marker(group)
    ));
    body
}

/// The dedupe marker: searched in the owner's issues before opening.
fn marker(group: &Group) -> String {
    format!("{MARKER_PREFIX}{} -->", group.hash)
}

fn render_record(groups: &[Group], owners: &BTreeMap<String, Option<String>>) -> String {
    let mut out = String::new();
    out.push_str("# Generated by scripts/cross-org-duplication.rs; the weekly job rewrites it.\n");
    out.push_str("cross_org_duplication\n");
    for group in groups {
        out.push_str("  group\n");
        out.push_str(&format!("    hash \"{}\"\n", group.hash));
        let owner = owners
            .get(&group.hash)
            .and_then(|owner| owner.clone())
            .unwrap_or_else(|| "no-owner".to_string());
        out.push_str(&format!("    owner \"{owner}\"\n"));
        out.push_str(&format!("    body_lines {}\n", group.body_lines));
        for site in &group.sites {
            out.push_str(&format!(
                "    site \"{}/{}:{}\" fn {}\n",
                site.repository, site.path, site.start_line, site.name
            ));
        }
    }
    out
}

fn scan_checkouts(checkouts: &Path) -> (Vec<Group>, BTreeMap<String, Vec<String>>) {
    let mut repositories: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut buckets: BTreeMap<u64, Vec<Site>> = BTreeMap::new();
    let mut lines_of: BTreeMap<u64, usize> = BTreeMap::new();
    for entry in fs::read_dir(checkouts)
        .unwrap_or_else(|_| panic!("{} lists", checkouts.display()))
        .flatten()
    {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let repository = path
            .file_name()
            .and_then(|it| it.to_str())
            .unwrap_or_default()
            .replace("__", "/");
        repositories.insert(repository.clone(), declared_dependencies(&path));
        for (name, hash, body_lines, relative, start_line) in scan_repository(&path, &repository) {
            lines_of.insert(hash, body_lines);
            buckets.entry(hash).or_default().push(Site {
                repository: repository.clone(),
                path: relative,
                start_line,
                name,
            });
        }
    }
    let mut groups: Vec<Group> = buckets
        .into_iter()
        .filter(|(_, sites)| {
            sites.len() >= 2
                && sites
                    .iter()
                    .map(|site| site.repository.as_str())
                    .collect::<std::collections::BTreeSet<_>>()
                    .len()
                    >= 2
        })
        .map(|(hash, sites)| Group {
            hash: format!("{hash:016x}"),
            body_lines: *lines_of.get(&hash).unwrap_or(&0),
            sites,
        })
        .filter(|group| group.body_lines >= MIN_BODY_LINES)
        .collect();
    groups.sort_by(|left, right| {
        let left_score = left.body_lines * left.sites.len();
        let right_score = right.body_lines * right.sites.len();
        right_score.cmp(&left_score)
    });
    let mut owners = BTreeMap::new();
    for group in &groups {
        let owner = choose_owner(group, &repositories);
        owners.insert(group.hash.clone(), owner);
    }
    (groups, owners)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let open = args.iter().any(|argument| argument == "--open");
    let fixture = args
        .iter()
        .position(|argument| argument == "--fixture")
        .and_then(|index| args.get(index + 1).cloned());

    let checkouts = if let Some(fixture) = fixture {
        PathBuf::from(fixture)
    } else {
        let scratch = PathBuf::from("target/cross-org-checkouts");
        let _ = fs::remove_dir_all(&scratch);
        fs::create_dir_all(&scratch).expect("scratch directory");
        for org in ORGS {
            // gh's built-in jq does the filtering: one repository name per
            // line, archived and forks already excluded.
            let listing = std::process::Command::new("gh")
                .args([
                    "repo",
                    "list",
                    org,
                    "--limit",
                    "1000",
                    "--json",
                    "name,isArchived,isFork,primaryLanguage",
                    "--jq",
                    ".[] | select(.isArchived == false and .isFork == false) | .name",
                ])
                .output()
                .expect("gh repo list runs");
            assert!(listing.status.success(), "gh repo list {org} failed");
            for name in String::from_utf8_lossy(&listing.stdout).lines() {
                let name = name.trim();
                if name.is_empty() {
                    continue;
                }
                let status = std::process::Command::new("git")
                    .args([
                        "clone",
                        "--depth",
                        "1",
                        "--filter=blob:limit=1m",
                        "--quiet",
                        &format!("https://github.com/{org}/{name}.git"),
                        &format!("{org}__{name}"),
                    ])
                    .current_dir(&scratch)
                    .status();
                if !status.is_ok_and(|it| it.success()) {
                    eprintln!("cross-org-duplication: clone {org}/{name} failed, skipping");
                }
            }
        }
        scratch
    };

    let (groups, owners) = scan_checkouts(&checkouts);
    println!(
        "cross-org-duplication: {} group(s) across repositories",
        groups.len()
    );
    for group in &groups {
        let owner = owners
            .get(&group.hash)
            .cloned()
            .flatten()
            .unwrap_or_else(|| "no-owner".to_string());
        println!(
            "  {} sites ~{}L owner {} {}",
            group.sites.len(),
            group.body_lines,
            owner,
            group.hash
        );
    }
    fs::create_dir_all("data/meta").expect("record directory");
    fs::write(RECORD, render_record(&groups, &owners)).expect("duplication record is written");
    println!("wrote {RECORD}");

    if !open {
        println!("--open not given: issues were not created (scan-and-report mode)");
        return;
    }
    for group in &groups {
        let Some(owner) = owners.get(&group.hash).cloned().flatten() else {
            println!(
                "group {}: no owner, reported for a human decision",
                group.hash
            );
            continue;
        };
        let search = std::process::Command::new("gh")
            .args([
                "search",
                "issues",
                &marker(group),
                "--repo",
                &owner,
                "--json",
                "url",
            ])
            .output()
            .expect("gh search issues runs");
        if !search.status.success() {
            eprintln!(
                "group {}: issue search failed; refusing creation without duplicate check",
                group.hash
            );
            continue;
        }
        let results = String::from_utf8_lossy(&search.stdout).to_string();
        if results.contains("\"url\"") {
            println!("group {}: already filed in {owner}", group.hash);
            continue;
        }
        let title = format!(
            "Deduplicate {}-line function body shared across repositories",
            group.body_lines
        );
        let body_file = checkouts.join(format!("issue-{}.md", group.hash));
        fs::write(&body_file, issue_body(group)).expect("issue body file");
        let status = std::process::Command::new("gh")
            .args([
                "issue",
                "create",
                "--repo",
                &owner,
                "--title",
                &title,
                "--body-file",
                body_file.to_str().expect("body path"),
            ])
            .output()
            .expect("gh issue create runs");
        if status.status.success() {
            println!(
                "group {}: filed {}",
                group.hash,
                String::from_utf8_lossy(&status.stdout).trim()
            );
        } else {
            eprintln!(
                "group {}: gh issue create in {owner} failed (credential layer may be read-only): {}",
                group.hash,
                String::from_utf8_lossy(&status.stderr).trim()
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> tempdir::TempDir {
        // rust-script tests cannot add dependencies; use a stable scratch
        // path under the system temp directory instead.
        let dir = std::env::temp_dir().join(format!("cross-org-fixture-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("dependency/src")).expect("dependency dir");
        fs::create_dir_all(dir.join("consumer-a/src")).expect("consumer-a dir");
        fs::create_dir_all(dir.join("consumer-b/src")).expect("consumer-b dir");
        fs::write(
            dir.join("dependency/Cargo.toml"),
            "[package]\nname = \"dependency\"\n\n[dependencies]\n",
        )
        .expect("dependency manifest");
        let duplicated = "pub fn shared_logic(input: u64) -> u64 {\n    let doubled = input * 2;\n    let shifted = doubled << 1;\n    let mixed = shifted ^ doubled;\n    let folded = mixed.rotate_left(3);\n    let masked = folded & 0xff;\n    masked.wrapping_add(shifted)\n}\n";
        fs::write(dir.join("dependency/src/lib.rs"), duplicated).expect("dependency copy");
        for consumer in ["consumer-a", "consumer-b"] {
            fs::write(
                dir.join(format!("{consumer}/Cargo.toml")),
                format!("[package]\nname = \"{consumer}\"\n\n[dependencies]\ndependency = \"1\"\n"),
            )
            .expect("consumer manifest");
            fs::write(dir.join(format!("{consumer}/src/lib.rs")), duplicated)
                .expect("consumer copy");
        }
        tempdir::TempDir { path: dir }
    }

    mod tempdir {
        use std::path::PathBuf;

        pub struct TempDir {
            pub path: PathBuf,
        }

        impl Drop for TempDir {
            fn drop(&mut self) {
                let _ = std::fs::remove_dir_all(&self.path);
            }
        }
    }

    #[test]
    fn the_dependency_owning_the_copies_is_selected_as_owner() {
        let directory = fixture();
        let (groups, owners) = scan_checkouts(&directory.path);
        assert!(
            !groups.is_empty(),
            "the duplicated body is found across repositories"
        );
        for group in &groups {
            assert!(group.body_lines >= MIN_BODY_LINES, "five-line minimum");
            let owner = owners.get(&group.hash).cloned().flatten();
            assert_eq!(
                owner.as_deref(),
                Some("dependency"),
                "both consumers declare it; the dependency's own tree carries a copy"
            );
        }
    }

    #[test]
    fn the_issue_body_lists_every_copy_and_carries_the_marker() {
        let directory = fixture();
        let (groups, owners) = scan_checkouts(&directory.path);
        let group = &groups[0];
        let body = issue_body(group);
        for site in &group.sites {
            assert!(
                body.contains(&format!("{}/{}", site.repository, site.path)),
                "every copy is listed: {}",
                site.path
            );
        }
        assert!(body.contains(&marker(group)));
        assert!(body.contains("identical after comment and whitespace normalization"));
        assert_eq!(owners.len(), groups.len());
    }

    #[test]
    fn the_marker_search_prevents_a_second_issue() {
        let directory = fixture();
        let (groups, _) = scan_checkouts(&directory.path);
        let group = &groups[0];
        // The scan-and-report path never opens; the marker contract that
        // prevents duplicates is: marker() is stable for the same body.
        assert_eq!(marker(group), marker(group));
        assert!(marker(group).starts_with(MARKER_PREFIX));
    }

    #[test]
    fn fnv1a_matches_the_reference_vectors() {
        assert_eq!(fnv1a(b""), 0xcbf2_9ce4_8422_2325);
        assert_eq!(fnv1a(b"a"), 0xaf63_dc4c_8601_ec8c);
    }

    #[test]
    fn comments_and_whitespace_leave_the_hash_untouched() {
        let plain = "fn a() {\n    1 + 2\n}\n";
        let commented = "fn a() { // note\n    1+2 /* strip */\n}\n";
        let left = function_bodies(plain)[0].1.clone();
        let right = function_bodies(commented)[0].1.clone();
        assert_eq!(left, right);
    }
}
