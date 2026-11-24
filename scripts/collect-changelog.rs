#!/usr/bin/env rust-script
//! Collect changelog fragments into CHANGELOG.md
//!
//! This script collects all .md files from changelog.d/ (except README.md)
//! and prepends them to CHANGELOG.md, then removes the processed fragments.
//!
//! Supports both single-language and multi-language repository structures;
//! the fragment directory and CHANGELOG.md are discovered via rust-paths.rs
//! (repository root first, then the rust root) so either layout resolves to
//! the files that actually exist.
//!
//! Usage: rust-script scripts/collect-changelog.rs [--rust-root <path>]
//!
//! ```cargo
//! [package]
//! edition = "2024"
//!
//! [dependencies]
//! toml_edit = "0.25.15"
//! regex = "1"
//! chrono = "0.4"
//! ```

#[cfg(not(test))]
use chrono::Utc;
use regex::Regex;
use std::env;
use std::fs;
use std::path::Path;
use std::process::exit;

#[path = "rust-paths.rs"]
mod rust_paths;

#[path = "changelog-files.rs"]
mod changelog_files;

#[allow(dead_code)]
#[path = "release-changelog.rs"]
mod release_changelog;

const INSERT_MARKER: &str = "<!-- changelog-insert-here -->";

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

fn get_cargo_toml_path(rust_root: &str) -> String {
    if rust_root == "." {
        "./Cargo.toml".to_string()
    } else {
        format!("{}/Cargo.toml", rust_root)
    }
}

fn get_changelog_dir(rust_root: &str) -> String {
    rust_paths::get_changelog_dir(rust_root)
        .to_string_lossy()
        .to_string()
}

fn get_changelog_path(rust_root: &str) -> String {
    rust_paths::get_changelog_path(rust_root)
        .to_string_lossy()
        .to_string()
}

fn get_version_from_cargo(cargo_toml_path: &str) -> Result<String, String> {
    rust_paths::read_package_info(Path::new(cargo_toml_path)).map(|package| package.version)
}

fn strip_frontmatter(content: &str) -> String {
    let re = Regex::new(r"(?s)^---\s*\n.*?\n---\s*\n(.*)$").unwrap();
    if let Some(caps) = re.captures(content) {
        caps.get(1).unwrap().as_str().trim().to_string()
    } else {
        content.trim().to_string()
    }
}

fn collect_fragments(
    changelog_dir: &str,
) -> std::io::Result<(String, Vec<changelog_files::Fragment>)> {
    let files = changelog_files::read_fragments(Path::new(changelog_dir))?;
    let content = files
        .iter()
        .map(|file| strip_frontmatter(&file.content))
        .filter(|content| !content.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n");
    Ok((content, files))
}

fn update_changelog(
    changelog_file: &str,
    version: &str,
    fragments: &str,
    date_str: &str,
) -> std::io::Result<()> {
    // One section, with no surrounding blank lines of its own. Each branch below
    // adds exactly the separators its context needs, so the result matches what
    // `experiments/issue_711_rebuild_changelog.mjs --check` reconstructs: the
    // marker followed by one blank line, one blank line between sections, and a
    // single trailing newline.
    let section = format!("## [{}] - {}\n\n{}", version, date_str, fragments);
    let new_entry = format!("{}\n", section);

    let existing = match changelog_files::read_file(Path::new(changelog_file)) {
        Ok(content) => Some(content),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => return Err(error),
    };
    if let Some(mut content) = existing {
        if content.contains(INSERT_MARKER) {
            // The marker is followed by "\n\n## [previous]", so the section
            // carries no trailing newline of its own here.
            content = content.replace(INSERT_MARKER, &format!("{}\n\n{}", INSERT_MARKER, section));
        } else {
            if let Some(offset) = release_changelog::first_heading_offset(&content) {
                // Preserve examples, old sections and their exact line endings.
                content.insert_str(offset, &format!("{section}\n\n"));
            } else {
                // Append after the main heading.
                content.push_str(&new_entry);
            }
        }

        fs::write(changelog_file, content).map_err(|error| {
            changelog_files::path_error("write changelog", Path::new(changelog_file), error)
        })?;
    } else {
        let content = format!(
            "# Changelog\n\n\
            All notable changes to this project will be documented in this file.\n\n\
            The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),\n\
            and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).\n\n\
            {}\n\n{}\n",
            INSERT_MARKER, section
        );
        fs::write(changelog_file, content).map_err(|error| {
            changelog_files::path_error("write changelog", Path::new(changelog_file), error)
        })?;
    }

    println!("Updated CHANGELOG.md with version {}", version);
    Ok(())
}

fn collect_changelog_with_date(
    changelog_dir: &str,
    changelog_file: &str,
    version: &str,
    date: &str,
) -> std::io::Result<bool> {
    let (fragments, files) = collect_fragments(changelog_dir)?;
    if fragments.is_empty() {
        return Ok(false);
    }
    update_changelog(changelog_file, version, &fragments, date)?;
    changelog_files::remove_fragments(&files)?;
    Ok(true)
}

#[cfg(not(test))]
fn main() {
    let rust_root = get_rust_root();
    let cargo_toml = get_cargo_toml_path(&rust_root);
    let changelog_dir = get_changelog_dir(&rust_root);
    let changelog_file = get_changelog_path(&rust_root);

    let version = match get_version_from_cargo(&cargo_toml) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error: {}", e);
            exit(1);
        }
    };

    println!("Collecting changelog fragments for version {}", version);

    let date = Utc::now().format("%Y-%m-%d").to_string();
    match collect_changelog_with_date(&changelog_dir, &changelog_file, &version, &date) {
        Ok(true) => {}
        Ok(false) => {
            println!("No changelog fragments found");
            return;
        }
        Err(error) => {
            eprintln!("Error: {error}");
            exit(1);
        }
    }

    println!("Changelog collection complete");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insertion_ignores_fenced_examples_and_preserves_existing_bytes() {
        let directory = env::temp_dir().join(format!(
            "collector-fenced-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&directory).unwrap();
        struct Cleanup(std::path::PathBuf);
        impl Drop for Cleanup {
            fn drop(&mut self) {
                let _ = fs::remove_dir_all(&self.0);
            }
        }
        let _cleanup = Cleanup(directory.clone());
        let changelog = directory.join("CHANGELOG.md");
        for newline in ["\n", "\r\n"] {
            for delimiter in ["```", "~~~", "   ````"] {
                let prefix = format!(
                    "# Changelog{newline}{newline}{delimiter}md{newline}## [9.9.9]{newline}{delimiter}{newline}{newline}"
                );
                let previous = format!("## [1.0.0]{newline}  Preserve these bytes.{newline}");
                fs::write(&changelog, format!("{prefix}{previous}")).unwrap();
                update_changelog(
                    changelog.to_str().unwrap(),
                    "1.1.0",
                    "### Fixed\n- New notes",
                    "2026-11-22",
                )
                .unwrap();
                let content = fs::read_to_string(&changelog).unwrap();
                assert!(content.starts_with(&prefix), "{content:?}");
                assert!(content.ends_with(&previous), "{content:?}");
                assert_eq!(
                    release_changelog::section(&content, "1.1.0").unwrap(),
                    "### Fixed\n- New notes"
                );
            }
        }
    }

    #[test]
    fn failed_fragment_or_existing_changelog_reads_preserve_every_input() {
        let directory = env::temp_dir().join(format!(
            "collector-io-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&directory).unwrap();
        struct Cleanup(std::path::PathBuf);
        impl Drop for Cleanup {
            fn drop(&mut self) {
                let _ = fs::remove_dir_all(&self.0);
            }
        }
        let _cleanup = Cleanup(directory.clone());
        let fragments = directory.join("changelog.d");
        fs::create_dir(&fragments).unwrap();
        let good = fragments.join("a.md");
        let bad = fragments.join("b.md");
        let changelog = directory.join("CHANGELOG.md");
        let original = "# Changelog\n\n<!-- changelog-insert-here -->\n";
        fs::write(&good, "A release note").unwrap();
        fs::write(&bad, [0xff]).unwrap();
        fs::write(&changelog, original).unwrap();
        let collect = || {
            collect_changelog_with_date(
                fragments.to_str().unwrap(),
                changelog.to_str().unwrap(),
                "1.2.3",
                "2026-11-20",
            )
        };
        assert!(collect().unwrap_err().to_string().contains("b.md"));
        assert_eq!(fs::read_to_string(&changelog).unwrap(), original);
        assert_eq!(fs::read_to_string(&good).unwrap(), "A release note");
        assert_eq!(fs::read(&bad).unwrap(), [0xff]);
        fs::write(&bad, "Second release note").unwrap();
        fs::write(&changelog, [0xff]).unwrap();
        assert!(collect().unwrap_err().to_string().contains("CHANGELOG.md"));
        assert_eq!(fs::read(&changelog).unwrap(), [0xff]);
        assert!(good.exists() && bad.exists());
        fs::write(&changelog, original).unwrap();
        assert!(collect().unwrap());
        assert!(!good.exists() && !bad.exists());
        let after = fs::read_to_string(&changelog).unwrap();
        assert!(after.contains("A release note\n\nSecond release note"));
        assert!(!collect().unwrap());
        assert_eq!(fs::read_to_string(&changelog).unwrap(), after);
    }

    #[test]
    fn changelog_version_uses_the_package_and_refuses_inherited_or_missing_versions() {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let id = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let directory = env::temp_dir().join(format!(
            "collect-changelog-package-{}-{}-{id}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&directory).unwrap();
        struct Cleanup(std::path::PathBuf);
        impl Drop for Cleanup {
            fn drop(&mut self) {
                let _ = fs::remove_dir_all(&self.0);
            }
        }
        let _cleanup = Cleanup(directory.clone());
        let manifest = directory.join("Cargo.toml");
        let path = manifest.to_str().unwrap();
        for (identity, expected) in [
            ("version = '1.2.3'", Some("1.2.3")),
            ("version.workspace = true", None),
            ("version = { workspace = true }", None),
            ("", None),
        ] {
            let content = format!(
                "[dependencies.decoy]\nversion = \"9.9.9\"\n[package] # actual\nname = 'actual'\n{identity}\n"
            );
            // The former global regex always selected this dependency version.
            let old = Regex::new(r#"(?m)^version\s*=\s*"([^"]+)""#).unwrap();
            assert_eq!(&old.captures(&content).unwrap()[1], "9.9.9");
            fs::write(&manifest, &content).unwrap();
            let actual = get_version_from_cargo(path);
            if let Some(version) = expected {
                assert_eq!(actual.unwrap(), version);
            } else {
                assert!(actual.unwrap_err().contains("[package].version"));
            }
            assert_eq!(fs::read_to_string(&manifest).unwrap(), content);
        }
    }
}
