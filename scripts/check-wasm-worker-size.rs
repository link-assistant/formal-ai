#!/usr/bin/env rust-script
//! Enforce the shipped size budget for the Rust→WASM worker.
//!
//! Issue #658 (E39 / R380) absorbs the JavaScript worker logic into the
//! Rust→WASM worker (`src/web/wasm-worker` → `src/web/formal_ai_worker.wasm`).
//! As logic migrates in, the `.wasm` grows; the GitHub Pages demo fetches this
//! asset on load, so it must stay small enough for an offline-identical, fast
//! first paint. This script keeps the shipped binary under an agreed ceiling.
//!
//! Usage: rust-script scripts/check-wasm-worker-size.rs
//!
//! ```cargo
//! [dependencies]
//! ```

#[cfg(not(test))]
use std::path::{Path, PathBuf};
#[cfg(not(test))]
use std::process::exit;

#[cfg(not(test))]
const WASM_PATH: &str = "src/web/formal_ai_worker.wasm";

/// Hard ceiling for the shipped worker binary. ~5× the current size, leaving
/// room for the remaining JS→WASM migration while keeping the demo download
/// small.
const MAX_WASM_BYTES: u64 = 512 * 1024;

/// Warn threshold: within budget but worth attention before the next slice.
const WARN_WASM_BYTES: u64 = 400 * 1024;

#[derive(Debug, PartialEq, Eq)]
enum WasmSizeStatus {
    WithinBudget,
    Warning,
    TooLarge,
}

const fn classify_wasm_size(bytes: u64, warn: u64, max: u64) -> WasmSizeStatus {
    if bytes > max {
        WasmSizeStatus::TooLarge
    } else if bytes > warn {
        WasmSizeStatus::Warning
    } else {
        WasmSizeStatus::WithinBudget
    }
}

fn format_bytes(bytes: u64) -> String {
    let hundredths = bytes.saturating_mul(100) / 1024;
    let whole = hundredths / 100;
    let fractional = hundredths % 100;
    format!("{whole}.{fractional:02} KiB ({bytes} bytes)")
}

#[cfg(not(test))]
fn wasm_path(cwd: &Path) -> PathBuf {
    cwd.join(WASM_PATH)
}

#[cfg(not(test))]
fn main() {
    println!("\nChecking the Rust→WASM worker size budget...\n");

    let cwd = std::env::current_dir().expect("Failed to get current directory");
    let path = wasm_path(&cwd);

    let Ok(metadata) = std::fs::metadata(&path) else {
        println!("::error::Shipped worker binary not found at {WASM_PATH}.");
        println!(
            "Build it with `sh src/web/wasm-worker/build.sh` and commit the result.\n"
        );
        exit(1);
    };

    let bytes = metadata.len();
    println!(
        "  {WASM_PATH}: {} (warn {}, max {})\n",
        format_bytes(bytes),
        format_bytes(WARN_WASM_BYTES),
        format_bytes(MAX_WASM_BYTES),
    );

    match classify_wasm_size(bytes, WARN_WASM_BYTES, MAX_WASM_BYTES) {
        WasmSizeStatus::TooLarge => {
            println!(
                "::error::{WASM_PATH} is {} — over the {} ceiling.",
                format_bytes(bytes),
                format_bytes(MAX_WASM_BYTES)
            );
            println!(
                "Shrink the crate (opt-level=z is already set) or raise the budget\n\
                 deliberately if the migration genuinely needs the room.\n"
            );
            exit(1);
        }
        WasmSizeStatus::Warning => {
            println!(
                "::warning::{WASM_PATH} is {} — approaching the {} ceiling.",
                format_bytes(bytes),
                format_bytes(MAX_WASM_BYTES)
            );
            println!("Within budget, but keep an eye on the size before the next slice.\n");
            exit(0);
        }
        WasmSizeStatus::WithinBudget => {
            println!("Shipped worker binary is within the size budget.\n");
            exit(0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_size_bands() {
        assert_eq!(
            classify_wasm_size(WARN_WASM_BYTES, WARN_WASM_BYTES, MAX_WASM_BYTES),
            WasmSizeStatus::WithinBudget,
            "exactly at the warn threshold is still within budget"
        );
        assert_eq!(
            classify_wasm_size(WARN_WASM_BYTES + 1, WARN_WASM_BYTES, MAX_WASM_BYTES),
            WasmSizeStatus::Warning
        );
        assert_eq!(
            classify_wasm_size(MAX_WASM_BYTES, WARN_WASM_BYTES, MAX_WASM_BYTES),
            WasmSizeStatus::Warning,
            "exactly at the ceiling passes"
        );
        assert_eq!(
            classify_wasm_size(MAX_WASM_BYTES + 1, WARN_WASM_BYTES, MAX_WASM_BYTES),
            WasmSizeStatus::TooLarge
        );
    }

    #[test]
    fn formats_kib_with_two_decimals() {
        assert_eq!(format_bytes(1024), "1.00 KiB (1024 bytes)");
        assert_eq!(format_bytes(1536), "1.50 KiB (1536 bytes)");
        assert_eq!(format_bytes(95_075), "92.84 KiB (95075 bytes)");
    }

    #[test]
    fn warn_threshold_is_below_the_ceiling() {
        assert!(WARN_WASM_BYTES < MAX_WASM_BYTES);
    }
}
