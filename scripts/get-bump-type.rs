#!/usr/bin/env rust-script
//! Parse changelog fragments and determine version bump type
//!
//! This script reads changeset fragments from changelog.d/ and determines
//! the version bump type based on the frontmatter in each fragment.
//!
//! The fragment directory is discovered via rust-paths.rs — repository-root
//! changelog.d/ first, then {rust-root}/changelog.d/ — so both single- and
//! multi-language layouts resolve to the directory that actually holds the
//! fragments.
//!
//! Fragment format:
//! ---
//! bump: patch|minor|major
//! ---
//!
//! ### Added
//! - Your changes here
//!
//! Usage: rust-script scripts/get-bump-type.rs [--default <patch|minor|major>] [--rust-root <path>]
//!
//! ```cargo
//! [package]
//! edition = "2024"
//!
//! [dependencies]
//! toml_edit = "0.25.15"
//! regex = "1"
//! ```

use regex::Regex;
use std::env;
#[cfg(test)]
use std::fs;
use std::path::Path;
use std::process::exit;

#[path = "github-output.rs"]
mod github_output;
use github_output::set_output;

#[path = "rust-paths.rs"]
mod rust_paths;

#[allow(dead_code)]
#[path = "changelog-files.rs"]
mod changelog_files;

fn get_arg(name: &str) -> Option<String> {
    let args: Vec<String> = env::args().collect();
    let flag = format!("--{}", name);

    if let Some(idx) = args.iter().position(|a| a == &flag) {
        return args.get(idx + 1).cloned();
    }

    let env_name = name.to_uppercase().replace('-', "_");
    env::var(&env_name).ok().filter(|s| !s.is_empty())
}

fn get_rust_root() -> String {
    if let Some(root) = get_arg("rust-root") {
        eprintln!("Using explicitly configured Rust root: {}", root);
        return root;
    }

    if Path::new("./Cargo.toml").exists() {
        eprintln!("Detected single-language repository (Cargo.toml in root)");
        return ".".to_string();
    }

    if Path::new("./rust/Cargo.toml").exists() {
        eprintln!("Detected multi-language repository (Cargo.toml in rust/)");
        return "rust".to_string();
    }

    eprintln!("Error: Could not find Cargo.toml in expected locations");
    exit(1);
}

fn get_changelog_dir(rust_root: &str) -> String {
    rust_paths::get_changelog_dir(rust_root)
        .to_string_lossy()
        .to_string()
}

fn bump_priority(bump_type: &str) -> u8 {
    match bump_type {
        "patch" => 1,
        "minor" => 2,
        "major" => 3,
        _ => 0,
    }
}

fn parse_frontmatter(content: &str) -> Option<String> {
    let re = Regex::new(r"(?s)^---\s*\n(.*?)\n---").unwrap();

    if let Some(caps) = re.captures(content) {
        let frontmatter = caps.get(1).unwrap().as_str();

        // Parse YAML-like frontmatter (simple key: value format)
        for line in frontmatter.lines() {
            let bump_re = Regex::new(r"^\s*bump\s*:\s*(.+?)\s*$").unwrap();
            if let Some(bump_caps) = bump_re.captures(line) {
                return Some(bump_caps.get(1).unwrap().as_str().to_string());
            }
        }
    }

    None
}

fn determine_bump_type(
    changelog_dir: &str,
    default_bump: &str,
) -> std::io::Result<(String, usize)> {
    let files = changelog_files::read_fragments(Path::new(changelog_dir))?;
    if files.is_empty() {
        println!("No changelog fragments found");
        return Ok((default_bump.to_string(), 0));
    }
    let mut highest_priority: u8 = 0;
    let mut highest_bump_type = default_bump.to_string();
    for file in &files {
        if let Some(bump) = parse_frontmatter(&file.content) {
            let priority = bump_priority(&bump);
            if priority > highest_priority {
                highest_priority = priority;
                highest_bump_type = bump.clone();
            }
            println!(
                "Fragment {}: bump={}",
                file.path.file_name().unwrap().to_string_lossy(),
                bump
            );
        } else {
            println!(
                "Fragment {}: no bump specified, using default",
                file.path.file_name().unwrap().to_string_lossy()
            );
        }
    }
    Ok((highest_bump_type, files.len()))
}

fn main() {
    let default_bump = get_arg("default").unwrap_or_else(|| "patch".to_string());
    let rust_root = get_rust_root();
    let changelog_dir = get_changelog_dir(&rust_root);

    let (bump_type, fragment_count) = match determine_bump_type(&changelog_dir, &default_bump) {
        Ok(result) => result,
        Err(error) => {
            eprintln!("Error: {error}");
            exit(1);
        }
    };

    println!(
        "\nDetermined bump type: {} (from {} fragment(s))",
        bump_type, fragment_count
    );

    set_output("bump_type", &bump_type);
    set_output("fragment_count", &fragment_count.to_string());
    set_output(
        "has_fragments",
        if fragment_count > 0 { "true" } else { "false" },
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bump_selection_refuses_incomplete_inventory_and_unreadable_fragments() {
        let root = env::temp_dir().join(format!(
            "bump-fragment-io-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&root).unwrap();
        struct Cleanup(std::path::PathBuf);
        impl Drop for Cleanup {
            fn drop(&mut self) {
                let _ = fs::remove_dir_all(&self.0);
            }
        }
        let _cleanup = Cleanup(root.clone());
        let directory = root.join("changelog.d");
        let dir = directory.to_str().unwrap();
        assert_eq!(
            determine_bump_type(dir, "minor").unwrap(),
            ("minor".to_string(), 0)
        );
        fs::write(&directory, "not a directory").unwrap();
        assert!(
            determine_bump_type(dir, "patch")
                .unwrap_err()
                .to_string()
                .contains("changelog.d")
        );
        fs::remove_file(&directory).unwrap();
        fs::create_dir(&directory).unwrap();
        let major = directory.join("a.md");
        let invalid = directory.join("b.md");
        fs::write(&major, "---\nbump: major\n---\nA major change").unwrap();
        fs::write(&invalid, [0xff]).unwrap();
        assert!(
            determine_bump_type(dir, "patch")
                .unwrap_err()
                .to_string()
                .contains("b.md")
        );
        assert_eq!(fs::read(&invalid).unwrap(), [0xff]);
        fs::write(&invalid, "---\nbump: patch\n---\nA patch change").unwrap();
        fs::write(directory.join("README.md"), "instructions").unwrap();
        assert_eq!(
            determine_bump_type(dir, "patch").unwrap(),
            ("major".to_string(), 2)
        );
        assert!(major.exists() && invalid.exists());
    }
}
