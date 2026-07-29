//! Seed-backed vocabulary for the external benchmark harness.
//!
//! Benchmark diagnostics are part of the user-visible CLI and persisted
//! learning artifacts, so their wording belongs in the Links seed network
//! rather than in Rust literals.

use crate::seed;

/// Resolve an English benchmark message from the shared Links seed.
#[must_use]
pub fn text(intent: &str) -> String {
    seed::response_for(intent, "en").unwrap_or_else(|| intent.to_owned())
}

/// Resolve a benchmark message and substitute its named fields.
#[must_use]
pub fn render(intent: &str, values: &[(&str, &str)]) -> String {
    let mut rendered = text(intent);
    for (name, value) in values {
        rendered = rendered.replace(&format!("{{{name}}}"), value);
    }
    rendered
}
