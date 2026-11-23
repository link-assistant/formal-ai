#!/usr/bin/env rust-script
//! Check the generated crates.io package archive against the crates.io limit.
//!
//! Usage: rust-script scripts/check-crate-package-size.rs
//!
//! ```cargo
//! [package]
//! edition = "2024"
//!
//! [dependencies]
//! toml_edit = "0.25.15"
//! regex = "1"
//! serde_json = "1"
//! ```

#[cfg(not(test))]
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
#[cfg(not(test))]
use std::process::exit;

#[path = "rust-paths.rs"]
mod rust_paths;

const CRATES_IO_MAX_PACKAGE_BYTES: u64 = 10 * 1024 * 1024;
const PACKAGE_WARN_BYTES: u64 = 8 * 1024 * 1024;
const MIB_BYTES: u64 = 1024 * 1024;

#[derive(Debug, PartialEq, Eq)]
enum PackageSizeStatus {
    WithinLimit,
    Warning,
    TooLarge,
}

const fn classify_package_size(bytes: u64) -> PackageSizeStatus {
    if bytes > CRATES_IO_MAX_PACKAGE_BYTES {
        PackageSizeStatus::TooLarge
    } else if bytes > PACKAGE_WARN_BYTES {
        PackageSizeStatus::Warning
    } else {
        PackageSizeStatus::WithinLimit
    }
}

fn format_bytes(bytes: u64) -> String {
    let hundredths = bytes.saturating_mul(100) / MIB_BYTES;
    let whole = hundredths / 100;
    let fractional = hundredths % 100;

    format!("{whole}.{fractional:02} MiB ({bytes} bytes)")
}

fn crate_archive_name(name: &str, version: &str) -> String {
    format!("{name}-{version}.crate")
}

// Cargo resolves workspace roots, environment overrides and configuration for us.
fn cargo_metadata_command(rust_root: &str) -> Command {
    let mut command = Command::new("cargo");
    command.args(["metadata", "--no-deps", "--format-version", "1"]);
    command.current_dir(rust_root);
    command
}

fn cargo_target_directory(command: &mut Command) -> Result<PathBuf, String> {
    let output = command
        .output()
        .map_err(|error| format!("Failed to run cargo metadata: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "cargo metadata failed with {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    target_directory_from_metadata(&output.stdout)
}

fn target_directory_from_metadata(bytes: &[u8]) -> Result<PathBuf, String> {
    let metadata: serde_json::Value = serde_json::from_slice(bytes)
        .map_err(|error| format!("Invalid cargo metadata: {error}"))?;
    let target = metadata
        .get("target_directory")
        .and_then(serde_json::Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "cargo metadata omitted target_directory".to_owned())?;
    let path = PathBuf::from(target);
    if !path.is_absolute() {
        return Err("cargo metadata target_directory must be absolute".to_owned());
    }
    Ok(path)
}

fn crate_archive_path(target_directory: impl AsRef<Path>, name: &str, version: &str) -> PathBuf {
    target_directory
        .as_ref()
        .join("package")
        .join(crate_archive_name(name, version))
}

#[cfg(not(test))]
fn print_command_output(output: &std::process::Output) {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !stdout.trim().is_empty() {
        println!("{stdout}");
    }

    if !stderr.trim().is_empty() {
        eprintln!("{stderr}");
    }
}

#[cfg(not(test))]
fn main() {
    let rust_root = match rust_paths::get_rust_root(None, true) {
        Ok(root) => root,
        Err(error) => {
            eprintln!("Error: {error}");
            exit(1);
        }
    };
    let cargo_toml = rust_paths::get_cargo_toml_path(&rust_root);
    let package_manifest = match rust_paths::get_package_manifest_path(&cargo_toml) {
        Ok(path) => path,
        Err(error) => {
            eprintln!("Error: {error}");
            exit(1);
        }
    };
    let package_info = match rust_paths::read_package_info(&package_manifest) {
        Ok(info) => info,
        Err(error) => {
            eprintln!("Error: {error}");
            exit(1);
        }
    };

    println!(
        "Checking crates.io package size for {}@{}",
        package_info.name, package_info.version
    );
    println!(
        "crates.io package size limit: {}",
        format_bytes(CRATES_IO_MAX_PACKAGE_BYTES)
    );

    let target_directory = match cargo_target_directory(&mut cargo_metadata_command(&rust_root)) {
        Ok(path) => path,
        Err(error) => {
            eprintln!("Error: {error}");
            exit(1);
        }
    };
    if std::env::var("FORMAL_AI_CI_VERBOSE").as_deref() == Ok("1") {
        eprintln!(
            "Cargo package target directory: {}",
            target_directory.display()
        );
    }
    let mut command = Command::new("cargo");
    command
        .arg("package")
        .arg("--allow-dirty")
        .arg("--no-verify")
        .arg("-p")
        .arg(&package_info.name);

    if rust_paths::needs_cd(&rust_root) {
        command.current_dir(&rust_root);
    }

    let output = match command.output() {
        Ok(output) => output,
        Err(error) => {
            eprintln!("Error: failed to run cargo package: {error}");
            exit(1);
        }
    };
    print_command_output(&output);

    if !output.status.success() {
        eprintln!("Error: cargo package failed with status {}", output.status);
        exit(1);
    }

    let archive_path =
        crate_archive_path(&target_directory, &package_info.name, &package_info.version);
    let archive_size = match fs::metadata(&archive_path) {
        Ok(metadata) => metadata.len(),
        Err(error) => {
            eprintln!(
                "Error: could not inspect generated crate archive {}: {error}",
                archive_path.display()
            );
            exit(1);
        }
    };

    let formatted_size = format_bytes(archive_size);
    match classify_package_size(archive_size) {
        PackageSizeStatus::WithinLimit => {
            println!(
                "Crate archive {} is within the crates.io limit: {}",
                archive_path.display(),
                formatted_size
            );
        }
        PackageSizeStatus::Warning => {
            println!(
                "::warning file={}::Crate archive is approaching the crates.io limit: {} of {}",
                package_manifest.display(),
                formatted_size,
                format_bytes(CRATES_IO_MAX_PACKAGE_BYTES)
            );
        }
        PackageSizeStatus::TooLarge => {
            eprintln!(
                "::error file={}::Crate archive exceeds the crates.io limit: {} of {}",
                package_manifest.display(),
                formatted_size,
                format_bytes(CRATES_IO_MAX_PACKAGE_BYTES)
            );
            eprintln!(
                "Reduce package contents with Cargo.toml include/exclude patterns before publishing."
            );
            exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CRATES_IO_MAX_PACKAGE_BYTES, PACKAGE_WARN_BYTES, PackageSizeStatus, cargo_metadata_command,
        cargo_target_directory, classify_package_size, crate_archive_name, crate_archive_path,
        format_bytes, target_directory_from_metadata,
    };

    #[test]
    fn classifies_package_sizes_against_crates_io_limit() {
        assert_eq!(
            classify_package_size(PACKAGE_WARN_BYTES - 1),
            PackageSizeStatus::WithinLimit
        );
        assert_eq!(
            classify_package_size(PACKAGE_WARN_BYTES + 1),
            PackageSizeStatus::Warning
        );
        assert_eq!(
            classify_package_size(CRATES_IO_MAX_PACKAGE_BYTES + 1),
            PackageSizeStatus::TooLarge
        );
    }

    #[test]
    fn formats_bytes_with_raw_count_for_ci_logs() {
        assert_eq!(format_bytes(10 * 1024 * 1024), "10.00 MiB (10485760 bytes)");
    }

    #[test]
    fn derives_crate_archive_path_from_package_metadata() {
        assert_eq!(
            crate_archive_name("formal-ai", "0.61.0"),
            "formal-ai-0.61.0.crate"
        );
        assert_eq!(
            crate_archive_path("./target", "formal-ai", "0.61.0"),
            std::path::PathBuf::from("./target/package/formal-ai-0.61.0.crate")
        );
        assert_eq!(
            crate_archive_path("rust/target", "formal-ai", "0.61.0"),
            std::path::PathBuf::from("rust/target/package/formal-ai-0.61.0.crate")
        );
    }

    #[test]
    fn rejects_missing_malformed_or_relative_cargo_target_observations() {
        for bytes in [
            &b"not json"[..],
            &b"{}"[..],
            &br#"{"target_directory":false}"#[..],
            &br#"{"target_directory":""}"#[..],
            &br#"{"target_directory":"relative/target"}"#[..],
        ] {
            assert!(target_directory_from_metadata(bytes).is_err());
        }
    }

    #[test]
    fn rejects_failed_metadata_processes_without_an_archive_fallback() {
        let mut missing = cargo_metadata_command(".");
        missing.current_dir(std::env::temp_dir().join(format!(
            "missing-cargo-metadata-directory-{}",
            std::process::id()
        )));
        assert!(
            cargo_target_directory(&mut missing)
                .unwrap_err()
                .starts_with("Failed to run cargo metadata:")
        );
        let mut rejected = std::process::Command::new("cargo");
        rejected.arg("--invalid-archive-metadata-regression-option");
        let error = cargo_target_directory(&mut rejected).unwrap_err();
        assert!(error.starts_with("cargo metadata failed with "));
        assert!(error.contains("invalid-archive-metadata-regression-option"));
    }

    #[test]
    fn package_archive_follows_actual_cargo_targets_without_compiling() {
        use std::fs;
        use std::time::{SystemTime, UNIX_EPOCH};
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory = std::env::temp_dir().join(format!("package-target-{nonce}"));
        struct Cleanup(std::path::PathBuf);
        impl Drop for Cleanup {
            fn drop(&mut self) {
                fs::remove_dir_all(&self.0).unwrap();
            }
        }
        fs::create_dir_all(&directory).unwrap();
        let _cleanup = Cleanup(directory.clone());
        for (label, workspace, mode) in [
            ("default", false, "default"),
            ("absolute", false, "absolute"),
            ("relative", false, "relative"),
            ("configured", false, "configured"),
            ("workspace", true, "default"),
        ] {
            let root = directory.join(label);
            let package = root.join("rust");
            fs::create_dir_all(package.join("src")).unwrap();
            fs::write(package.join("src/lib.rs"), "pub fn fixture() {}\n").unwrap();
            fs::write(
                package.join("Cargo.toml"),
                r#"[package]
name="archive-target-fixture"
version="0.1.0"
edition="2021"
license="MIT"
description="Offline archive regression"
"#,
            )
            .unwrap();
            if workspace {
                fs::write(
                    root.join("Cargo.toml"),
                    "[workspace]\nmembers=[\"rust\"]\nresolver=\"2\"\n",
                )
                .unwrap();
            }
            if mode == "configured" {
                fs::create_dir_all(root.join(".cargo")).unwrap();
                fs::write(
                    root.join(".cargo/config.toml"),
                    "[build]\ntarget-dir=\"configured-target\"\n",
                )
                .unwrap();
            }
            let configure = |command: &mut std::process::Command| {
                command
                    .env_remove("CARGO_TARGET_DIR")
                    .env_remove("CARGO_BUILD_TARGET_DIR")
                    .env_remove("CARGO_ENCODED_RUSTFLAGS")
                    .env_remove("RUSTFLAGS")
                    .env_remove("RUSTC_WRAPPER")
                    .env_remove("RUSTC_WORKSPACE_WRAPPER")
                    .env("CARGO_HOME", directory.join("cargo-home"))
                    .env("CARGO_NET_OFFLINE", "true")
                    .env("CARGO_BUILD_JOBS", "1")
                    .env("CARGO_INCREMENTAL", "0");
                if mode == "absolute" {
                    command.env("CARGO_TARGET_DIR", root.join("shared-target"));
                }
                if mode == "relative" {
                    command.env("CARGO_TARGET_DIR", "relative-target");
                }
            };
            let mut metadata = cargo_metadata_command(package.to_str().unwrap());
            configure(&mut metadata);
            let target = cargo_target_directory(&mut metadata).unwrap();
            let archive = crate_archive_path(&target, "archive-target-fixture", "0.1.0");
            let mut command = std::process::Command::new("cargo");
            command.current_dir(&package).args([
                "package",
                "--offline",
                "--allow-dirty",
                "--no-verify",
                "-p",
                "archive-target-fixture",
            ]);
            configure(&mut command);
            let output = command.output().unwrap();
            assert!(
                output.status.success(),
                "{label}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            assert!(archive.is_file(), "{label}: {}", archive.display());
            // An old default-target archive must never replace the new Cargo output.
            let old = package.join("target/package/archive-target-fixture-0.1.0.crate");
            if mode != "default" || workspace {
                assert_ne!(archive, old, "{label}");
                fs::create_dir_all(old.parent().unwrap()).unwrap();
                fs::write(&old, b"stale archive").unwrap();
                assert_ne!(
                    fs::read(&archive).unwrap(),
                    fs::read(old).unwrap(),
                    "{label}"
                );
            }
        }
    }
}
