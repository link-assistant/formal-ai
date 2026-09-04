#!/usr/bin/env rust-script
//! Print the release target now in force, and the share this cycle projects
//! (issue #1069).
//!
//! `check-self-development-release.rs` refuses on the *first* unmet condition,
//! and the missing session-backed pull request is checked before the share is,
//! so its output says nothing about where the bar sits. This reads the two
//! numbers directly off the real ledger and the real range, which is the only
//! way to see that a reviewed `target_override_basis_points` took effect.
//!
//! ```
//! rust-script experiments/issue_1069_release_target/show-target.rs
//! ```
//!
//! ```cargo
//! [package]
//! edition = "2024"
//! ```

#[path = "../../scripts/self-hosting-metric.rs"]
mod metric;

use std::path::Path;

fn main() -> Result<(), String> {
    let repo = Path::new(".");
    let ledger = Path::new("data/meta/self-hosting-ledger.lino");
    let rows = metric::read_release_rows(ledger)?;
    let newest = rows.last().ok_or("ledger has no rows")?;
    println!("newest row:            {}", newest.tag);
    println!(
        "  trailing share:      {}",
        metric::format_percentage(newest.trailing_percentage_basis_points)
    );
    println!(
        "  recorded target:     {}",
        newest
            .target_percentage_basis_points
            .map_or_else(|| "(none)".to_owned(), metric::format_percentage)
    );
    println!(
        "  reviewed override:   {}",
        newest
            .target_override_basis_points
            .map_or_else(|| "(none)".to_owned(), metric::format_percentage)
    );

    let since = &newest.tag;
    let projected = metric::project_trailing_share(repo, ledger, since, "HEAD", 3, None)?;
    println!("\nnext release ({since}..HEAD)");
    println!("  projected share:     {}", metric::format_percentage(projected));
    Ok(())
}
