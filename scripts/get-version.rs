#!/usr/bin/env rust-script
//! Get the current version from Cargo.toml
//!
//! This script reads the version from Cargo.toml and outputs it
//! for use in GitHub Actions.
//!
//! Supports both single-language and multi-language repository structures:
//! - Single-language: Cargo.toml in repository root
//! - Multi-language: Cargo.toml in rust/ subfolder
//!
//! Usage: rust-script scripts/get-version.rs [--rust-root <path>]
//!
//! Outputs (written to GITHUB_OUTPUT):
//!   - version: The current version from Cargo.toml
//!
//! ```cargo
//! [package]
//! edition = "2024"
//!
//! [dependencies]
//! regex = "1"
//! ```

use std::process::exit;

#[path = "github-output.rs"]
mod github_output;
use github_output::set_output;

#[path = "rust-paths.rs"]
mod rust_paths;

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

    match rust_paths::read_package_info(&package_manifest) {
        Ok(info) => {
            println!("Current version: {}", info.version);
            set_output("version", &info.version);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            exit(1);
        }
    }
}
