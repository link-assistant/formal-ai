#!/usr/bin/env rust-script
//! Check structural en/ru/hi/zh/es lexeme parity and its explicit dated debt.
//!
//! Usage:
//!   rust-script scripts/check-language-parity.rs
//!   rust-script scripts/check-language-parity.rs --count
//!   rust-script scripts/check-language-parity.rs --write --date YYYY-MM-DD
//!   rust-script --test scripts/check-language-parity.rs
//!
//! ```cargo
//! [package]
//! edition = "2024"
//! ```

#[path = "language-parity-lib.rs"]
mod language_parity;

fn main() {
    match language_parity::run(std::env::args().skip(1)) {
        Ok(message) => println!("{message}"),
        Err(error) => {
            eprintln!("language parity check failed:\n{error}");
            std::process::exit(1);
        }
    }
}
