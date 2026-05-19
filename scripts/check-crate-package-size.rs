#!/usr/bin/env rust-script
//! Check the generated crates.io package archive against the crates.io limit.
//!
//! Usage: rust-script scripts/check-crate-package-size.rs
//!
//! ```cargo
//! [dependencies]
//! regex = "1"
//! ```

#[cfg(not(test))]
use std::fs;
use std::path::{Path, PathBuf};
#[cfg(not(test))]
use std::process::{exit, Command};

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

fn crate_archive_path(rust_root: &str, name: &str, version: &str) -> PathBuf {
    let base = if rust_root == "." {
        PathBuf::from(".")
    } else {
        Path::new(rust_root).to_path_buf()
    };

    base.join("target")
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

    let archive_path = crate_archive_path(&rust_root, &package_info.name, &package_info.version);
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
        classify_package_size, crate_archive_name, crate_archive_path, format_bytes,
        PackageSizeStatus, CRATES_IO_MAX_PACKAGE_BYTES, PACKAGE_WARN_BYTES,
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
            crate_archive_path(".", "formal-ai", "0.61.0"),
            std::path::PathBuf::from("./target/package/formal-ai-0.61.0.crate")
        );
        assert_eq!(
            crate_archive_path("rust", "formal-ai", "0.61.0"),
            std::path::PathBuf::from("rust/target/package/formal-ai-0.61.0.crate")
        );
    }
}
