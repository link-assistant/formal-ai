#!/usr/bin/env rust-script
//! Test the Cargo.lock update regex used by scripts/version-and-commit.rs.
//!
//! ```cargo
//! [dependencies]
//! regex = "1"
//! ```

use regex::Regex;

fn replace_lock_version(content: &str, crate_name: &str, new_version: &str) -> String {
    let pattern = format!(
        r#"(?m)(\[\[package\]\]\s*\nname\s*=\s*"{}"\s*\nversion\s*=\s*")[^"]+(")"#,
        regex::escape(crate_name),
    );
    let re = Regex::new(&pattern).unwrap();
    re.replace(content, format!("${{1}}{}${{2}}", new_version).as_str())
        .into_owned()
}

fn main() {
    let sample = "\
[[package]]
name = \"foo\"
version = \"1.2.3\"
checksum = \"abc\"

[[package]]
name = \"formal-ai\"
version = \"0.29.0\"
dependencies = [
 \"clap\",
]

[[package]]
name = \"bar\"
version = \"4.5.6\"
";
    let out = replace_lock_version(sample, "formal-ai", "0.31.0");
    assert!(out.contains("name = \"formal-ai\"\nversion = \"0.31.0\""),
        "expected updated formal-ai version, got:\n{}", out);
    assert!(out.contains("name = \"foo\"\nversion = \"1.2.3\""),
        "must not change other crates");
    assert!(out.contains("name = \"bar\"\nversion = \"4.5.6\""),
        "must not change other crates");
    // Idempotent run
    let out2 = replace_lock_version(&out, "formal-ai", "0.31.0");
    assert_eq!(out, out2, "second run should be a no-op");
    // Non-existent crate name produces same content (no panic, replace just returns unchanged).
    let unchanged = replace_lock_version(&out, "nonexistent", "9.9.9");
    assert_eq!(out, unchanged);
    println!("all assertions passed");
}
