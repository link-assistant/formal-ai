#!/usr/bin/env rust-script
//! Check the packaged `.crate` archive against the crates.io upload limit
//!
//! crates.io rejects `cargo publish` uploads larger than 10 MiB (10485760 bytes)
//! with an HTTP 413 error. This guard runs `cargo package` to build the archive
//! and fails the release before publishing when the archive is too large, so the
//! release pipeline surfaces a clear, actionable error instead of a late 413.
//!
//! Supports both single-language and multi-language repository structures:
//! - Single-language: Cargo.toml in repository root
//! - Multi-language: Cargo.toml in rust/ subfolder
//!
//! Usage: rust-script scripts/check-crate-size.rs [--rust-root <path>]
//!
//! Outputs (written to `GITHUB_OUTPUT`):
//!   - `crate_size_bytes`: size of the generated `.crate` archive in bytes
//!   - `crate_size_check`: 'pass' or 'fail'
//!
//! Reference: <https://doc.rust-lang.org/cargo/reference/publishing.html#packaging-a-crate>
//!
//! ```cargo
//! [dependencies]
//! regex = "1"
//! ```

#[cfg(not(test))]
use std::env;
use std::fs;
#[cfg(not(test))]
use std::io::Write;
use std::path::{Path, PathBuf};
#[cfg(not(test))]
use std::process::{exit, Command};

#[cfg(not(test))]
#[path = "rust-paths.rs"]
mod rust_paths;

/// crates.io hard upload limit for a `.crate` archive (10 MiB).
/// See <https://doc.rust-lang.org/cargo/reference/publishing.html#packaging-a-crate>
const MAX_CRATE_BYTES: u64 = 10 * 1024 * 1024;
/// Warn once the archive grows past 80% of the limit so projects can react
/// before a release is actually blocked.
const WARN_CRATE_BYTES: u64 = MAX_CRATE_BYTES * 8 / 10;

#[derive(Debug, PartialEq, Eq)]
enum SizeStatus {
    WithinLimit,
    Warning,
    Violation,
}

const fn classify_size(size_bytes: u64) -> SizeStatus {
    if size_bytes > MAX_CRATE_BYTES {
        SizeStatus::Violation
    } else if size_bytes > WARN_CRATE_BYTES {
        SizeStatus::Warning
    } else {
        SizeStatus::WithinLimit
    }
}

fn format_mib(size_bytes: u64) -> String {
    #[allow(clippy::cast_precision_loss)]
    let mib = size_bytes as f64 / (1024.0 * 1024.0);
    format!("{mib:.2} MiB ({size_bytes} bytes)")
}

#[cfg(not(test))]
fn set_output(key: &str, value: &str) {
    if let Ok(output_file) = env::var("GITHUB_OUTPUT") {
        if let Ok(mut file) = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&output_file)
        {
            let _ = writeln!(file, "{key}={value}");
        }
    }
    println!("Output: {key}={value}");
}

/// Locate the `.crate` archive produced by `cargo package` for the given
/// package name and version inside `<rust_root>/target/package`.
fn find_crate_archive(rust_root: &str, name: &str, version: &str) -> Option<PathBuf> {
    let package_dir = if rust_root == "." {
        PathBuf::from("target/package")
    } else {
        Path::new(rust_root).join("target/package")
    };

    let archive = package_dir.join(format!("{name}-{version}.crate"));
    archive.exists().then_some(archive)
}

#[cfg(not(test))]
fn main() {
    let rust_root = match rust_paths::get_rust_root(None, true) {
        Ok(root) => root,
        Err(e) => {
            eprintln!("Error: {e}");
            exit(1);
        }
    };
    let cargo_toml = rust_paths::get_cargo_toml_path(&rust_root);
    let package_manifest = match rust_paths::get_package_manifest_path(&cargo_toml) {
        Ok(path) => path,
        Err(e) => {
            eprintln!("Error: {e}");
            exit(1);
        }
    };
    let package_info = match rust_paths::read_package_info(&package_manifest) {
        Ok(info) => info,
        Err(e) => {
            eprintln!("Error: {e}");
            exit(1);
        }
    };
    let name = package_info.name;
    let version = package_info.version;

    println!("Package: {name}@{version}");
    println!(
        "\nBuilding `.crate` archive to verify it stays under the crates.io {} limit...\n",
        format_mib(MAX_CRATE_BYTES)
    );

    // Generate the archive. `--no-verify` skips the recompile (the lint/test/build
    // jobs already verify the package) while still writing the `.crate` archive,
    // which is all this size guard needs.
    let mut cmd = Command::new("cargo");
    cmd.arg("package")
        .arg("--allow-dirty")
        .arg("--no-verify")
        .arg("-p")
        .arg(&name);
    if rust_paths::needs_cd(&rust_root) {
        cmd.current_dir(&rust_root);
    }

    let status = cmd.status().expect("Failed to execute cargo package");
    if !status.success() {
        eprintln!("::error::cargo package failed; cannot determine crate archive size");
        exit(1);
    }

    let Some(archive) = find_crate_archive(&rust_root, &name, &version) else {
        eprintln!(
            "::error::Could not find generated archive {name}-{version}.crate in target/package"
        );
        exit(1);
    };

    let size_bytes = match fs::metadata(&archive) {
        Ok(meta) => meta.len(),
        Err(e) => {
            eprintln!("::error::Could not read size of {}: {e}", archive.display());
            exit(1);
        }
    };

    set_output("crate_size_bytes", &size_bytes.to_string());
    println!("Archive: {}", archive.display());
    println!("Size: {}", format_mib(size_bytes));
    println!("Limit: {}", format_mib(MAX_CRATE_BYTES));

    match classify_size(size_bytes) {
        SizeStatus::Violation => {
            let message = format!(
                "Packaged crate archive is {} which exceeds the crates.io upload limit of {}. \
                 Reduce the package by adding a narrow `include` allowlist in Cargo.toml (or \
                 `exclude` large files such as docs, case studies, generated artifacts, and \
                 experiments). See https://doc.rust-lang.org/cargo/reference/manifest.html#the-exclude-and-include-fields",
                format_mib(size_bytes),
                format_mib(MAX_CRATE_BYTES)
            );
            println!("::error::{message}");
            eprintln!("\nERROR: {message}\n");
            set_output("crate_size_check", "fail");
            exit(1);
        }
        SizeStatus::Warning => {
            let message = format!(
                "Packaged crate archive is {} which is approaching the crates.io upload limit of {}. \
                 Consider trimming the package with an `include`/`exclude` allowlist in Cargo.toml.",
                format_mib(size_bytes),
                format_mib(MAX_CRATE_BYTES)
            );
            println!("::warning::{message}");
            println!("\nWARNING: {message}\n");
            set_output("crate_size_check", "pass");
            exit(0);
        }
        SizeStatus::WithinLimit => {
            println!("\nCrate archive is within the crates.io upload limit\n");
            set_output("crate_size_check", "pass");
            exit(0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn limit_matches_crates_io_documented_bytes() {
        assert_eq!(MAX_CRATE_BYTES, 10_485_760);
    }

    #[test]
    fn warns_before_blocking() {
        assert_eq!(classify_size(0), SizeStatus::WithinLimit);
        assert_eq!(classify_size(WARN_CRATE_BYTES), SizeStatus::WithinLimit);
        assert_eq!(classify_size(WARN_CRATE_BYTES + 1), SizeStatus::Warning);
        assert_eq!(classify_size(MAX_CRATE_BYTES), SizeStatus::Warning);
    }

    #[test]
    fn blocks_above_hard_limit() {
        assert_eq!(classify_size(MAX_CRATE_BYTES + 1), SizeStatus::Violation);
        // The downstream formal-ai failure: 16.1 MiB compressed must be rejected.
        assert_eq!(
            classify_size(16 * 1024 * 1024 + 100 * 1024),
            SizeStatus::Violation
        );
    }

    #[test]
    fn format_mib_is_human_readable() {
        assert_eq!(format_mib(MAX_CRATE_BYTES), "10.00 MiB (10485760 bytes)");
    }

    #[test]
    fn find_crate_archive_returns_none_when_missing() {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("check-crate-size-missing-{nanos}"));
        fs::create_dir_all(&root).unwrap();
        let result = find_crate_archive(root.to_str().unwrap(), "demo", "0.1.0");
        assert_eq!(result, None);
    }

    #[test]
    fn find_crate_archive_locates_generated_archive() {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("check-crate-size-found-{nanos}"));
        let package_dir = root.join("target/package");
        fs::create_dir_all(&package_dir).unwrap();
        let archive = package_dir.join("demo-0.1.0.crate");
        fs::write(&archive, b"fake archive").unwrap();

        let result = find_crate_archive(root.to_str().unwrap(), "demo", "0.1.0");
        assert_eq!(result, Some(archive));
    }
}
