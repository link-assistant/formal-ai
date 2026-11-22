#!/usr/bin/env rust-script
//! Wait for a crates.io package version to become visible.
//!
//! The Docker release step runs after `cargo publish`, but crates.io indexing can
//! lag briefly. Waiting here makes Docker Hub tags and GitHub releases point at a
//! crate version that users can already resolve.
//!
//! Usage:
//!   rust-script scripts/wait-for-crate.rs --release-version <version>
//!
//! Optional arguments:
//!   --crate-name <name>       Crate name. Defaults to Cargo.toml package name.
//!   --rust-root <path>        Root containing Cargo.toml. Defaults to auto-detect.
//!   --max-attempts <count>    Defaults to 30.
//!   --sleep-seconds <count>   Defaults to 10.
//!
//! Outputs (written to GITHUB_OUTPUT):
//!   - crate_available: 'true' when the version is visible, or 'skipped' for template defaults
//!
//! ```cargo
//! [package]
//! edition = "2024"
//!
//! [dependencies]
//! toml_edit = "0.25.15"
//! regex = "1"
//! ureq = "3.5.2"
//! serde_json = "1"
//! ```

use std::env;
use std::process::exit;
use std::thread;
use std::time::Duration;

#[path = "github-output.rs"]
mod github_output;
use github_output::set_output;

#[path = "rust-paths.rs"]
mod rust_paths;

#[path = "registry.rs"]
mod registry;

fn get_arg(name: &str) -> Option<String> {
    let args: Vec<String> = env::args().collect();
    let flag = format!("--{}", name);

    if let Some(idx) = args.iter().position(|a| a == &flag) {
        return args.get(idx + 1).cloned();
    }

    let env_name = name.to_uppercase().replace('-', "_");
    env::var(&env_name).ok().filter(|s| !s.is_empty())
}

fn parse_count_arg(name: &str, default: u64) -> u64 {
    get_arg(name)
        .and_then(|value| {
            value.parse::<u64>().map_or_else(
                |_| {
                    eprintln!(
                        "Warning: Invalid {} value '{}'; using default {}",
                        name, value, default
                    );
                    None
                },
                Some,
            )
        })
        .unwrap_or(default)
}

fn should_skip_crate_wait(crate_name: &str) -> bool {
    crate_name == "example-sum-package-name"
}

fn main() {
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
    let package_info = match rust_paths::read_package_info(&package_manifest) {
        Ok(info) => info,
        Err(e) => {
            eprintln!("Error: {}", e);
            exit(1);
        }
    };

    let crate_name = get_arg("crate-name").unwrap_or(package_info.name);
    let version = get_arg("release-version").unwrap_or(package_info.version);
    let max_attempts = parse_count_arg("max-attempts", 30);
    let sleep_seconds = parse_count_arg("sleep-seconds", 10);

    if should_skip_crate_wait(&crate_name) {
        println!(
            "Skipping crates.io availability wait: package name is the template default '{}'",
            crate_name
        );
        set_output("crate_available", "skipped");
        return;
    }

    for attempt in 1..=max_attempts {
        if registry::or_exit(
            "Crates.io availability",
            registry::version_exists(&crate_name, &version),
        ) {
            println!(
                "{}@{} is visible on crates.io after attempt {}",
                crate_name, version, attempt
            );
            set_output("crate_available", "true");
            return;
        }

        if attempt < max_attempts {
            println!(
                "{}@{} is not visible on crates.io yet (attempt {}/{}); waiting {}s",
                crate_name, version, attempt, max_attempts, sleep_seconds
            );
            thread::sleep(Duration::from_secs(sleep_seconds));
        }
    }

    eprintln!(
        "Error: {}@{} was not visible on crates.io after {} attempts",
        crate_name, version, max_attempts
    );
    exit(1);
}

#[cfg(test)]
mod tests {
    use super::should_skip_crate_wait;

    #[test]
    fn skips_template_default_package_name() {
        assert!(should_skip_crate_wait("example-sum-package-name"));
    }

    #[test]
    fn waits_for_real_package_names() {
        assert!(!should_skip_crate_wait("real-package-name"));
    }
}
