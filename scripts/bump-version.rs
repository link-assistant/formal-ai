#!/usr/bin/env rust-script
//! Bump version in Cargo.toml
//!
//! Usage: rust-script scripts/bump-version.rs --bump-type <major|minor|patch> [--dry-run] [--rust-root <path>]
//!
//! Supports both single-language and multi-language repository structures:
//! - Single-language: Cargo.toml in repository root
//! - Multi-language: Cargo.toml in rust/ subfolder
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
use std::path::Path;
use std::process::exit;

#[path = "rust-paths.rs"]
mod rust_paths;

#[derive(Debug, Clone, Copy, PartialEq)]
enum BumpType {
    Major,
    Minor,
    Patch,
}

impl BumpType {
    fn from_str(s: &str) -> Option<BumpType> {
        match s.to_lowercase().as_str() {
            "major" => Some(BumpType::Major),
            "minor" => Some(BumpType::Minor),
            "patch" => Some(BumpType::Patch),
            _ => None,
        }
    }
}

struct Version {
    major: u32,
    minor: u32,
    patch: u32,
}

impl Version {
    fn bump(&self, bump_type: BumpType) -> String {
        match bump_type {
            BumpType::Major => format!("{}.0.0", self.major + 1),
            BumpType::Minor => format!("{}.{}.0", self.major, self.minor + 1),
            BumpType::Patch => format!("{}.{}.{}", self.major, self.minor, self.patch + 1),
        }
    }

    fn to_string(&self) -> String {
        format!("{}.{}.{}", self.major, self.minor, self.patch)
    }
}

fn get_arg(name: &str) -> Option<String> {
    let args: Vec<String> = env::args().collect();
    let flag = format!("--{}", name);

    if let Some(idx) = args.iter().position(|a| a == &flag) {
        return args.get(idx + 1).cloned();
    }

    // Check environment variable (convert dashes to underscores)
    let env_name = name.to_uppercase().replace('-', "_");
    env::var(&env_name).ok().filter(|s| !s.is_empty())
}

fn has_flag(name: &str) -> bool {
    let args: Vec<String> = env::args().collect();
    let flag = format!("--{}", name);
    args.contains(&flag)
}

fn get_current_version(cargo_toml_path: &str) -> Result<Version, String> {
    let info = rust_paths::read_package_info(Path::new(cargo_toml_path))?;
    let re = Regex::new(r"^(\d+)\.(\d+)\.(\d+)$").unwrap();
    let captures = re
        .captures(&info.version)
        .ok_or_else(|| format!("Could not parse package version in {cargo_toml_path}"))?;
    let component = |index: usize| {
        captures[index]
            .parse::<u32>()
            .map_err(|error| format!("Invalid package version in {cargo_toml_path}: {error}"))
    };
    Ok(Version {
        major: component(1)?,
        minor: component(2)?,
        patch: component(3)?,
    })
}

fn update_cargo_toml(cargo_toml_path: &str, new_version: &str) -> Result<(), String> {
    rust_paths::update_package_version(Path::new(cargo_toml_path), new_version)
}

fn main() {
    let bump_type_str = match get_arg("bump-type") {
        Some(s) => s,
        None => {
            eprintln!(
                "Usage: rust-script scripts/bump-version.rs --bump-type <major|minor|patch> [--dry-run] [--rust-root <path>]"
            );
            exit(1);
        }
    };

    let bump_type = match BumpType::from_str(&bump_type_str) {
        Some(bt) => bt,
        None => {
            eprintln!(
                "Invalid bump type: {}. Must be major, minor, or patch.",
                bump_type_str
            );
            exit(1);
        }
    };

    let dry_run = has_flag("dry-run");
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

    let current = match get_current_version(package_manifest.to_string_lossy().as_ref()) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error: {}", e);
            exit(1);
        }
    };

    let new_version = current.bump(bump_type);

    println!("Current version: {}", current.to_string());
    println!("New version: {}", new_version);

    if dry_run {
        println!("Dry run - no changes made");
    } else {
        if let Err(e) = update_cargo_toml(package_manifest.to_string_lossy().as_ref(), &new_version)
        {
            eprintln!("Error: {}", e);
            exit(1);
        }
        println!("Updated {}", package_manifest.display());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn manual_bump_reads_and_writes_only_the_package_version() {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory =
            std::env::temp_dir().join(format!("bump-package-{}-{nanos}", std::process::id()));
        std::fs::create_dir(&directory).unwrap();
        let manifest = directory.join("Cargo.toml");
        let content = "[dependencies.decoy]\nversion = \"9.9.9\"\n\
                       [package]\nname = 'actual'\nversion = '1.2.3'\n";
        std::fs::write(&manifest, content).unwrap();
        let current = get_current_version(manifest.to_str().unwrap()).unwrap();
        assert_eq!(current.to_string(), "1.2.3");
        update_cargo_toml(manifest.to_str().unwrap(), &current.bump(BumpType::Patch)).unwrap();
        assert_eq!(
            std::fs::read_to_string(manifest).unwrap(),
            content.replace("'1.2.3'", "\"1.2.4\"")
        );
        std::fs::remove_dir_all(directory).unwrap();
    }
}
