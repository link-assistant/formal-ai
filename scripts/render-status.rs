#!/usr/bin/env rust-script
//! One generated status surface (#1138, plan 11 L2; plan 00 section 4.5).
//!
//! Status of every requirement, benchmark and gate is generated from the
//! `data/meta` ledgers and the `Evidence` records by **one** script, into
//! **one** file, `docs/status.md`, plus the two in-place regions whose existing
//! pin tests require the number to stand in the document (`docs/benchmarks.md`
//! and `README.md`). VISION, ROADMAP, ARCHITECTURE, GOALS and the REQUIREMENTS
//! shards link to it instead of restating a number.
//!
//! Modelled on `scripts/assemble-requirements.rs`: the same `--write` /
//! `--check` / bare-invocation shape, the same banner, the same total order read
//! from file names.
//!
//! The seven ledger inputs (plan 11, Option A):
//!   data/benchmarks/external-results.lino
//!   data/meta/self-hosting-ledger.lino
//!   data/meta/debt-ratchet.lino
//!   data/meta/core-boundary-ledger.lino
//!   data/meta/handler-migration-ledger.lino
//!   data/meta/ladder-ratchet.lino
//!   data/meta/worker-line-budget/*.lino
//! plus data/meta/requirement-status-ledger.lino for the per-requirement rows.
//!
//! **Wave T skeleton.** Plan 14's wave T writes every test first and observes it
//! failing; this script exists so `tests/unit/specification/status_render.rs`
//! compiles and runs, and it refuses every mode with the leaf that owns it.
//! Emitting a `docs/status.md` here would publish numbers no ledger produced,
//! which is what plan 00 section 6.2 forbids.
//!
//! Usage:
//!   rust-script scripts/render-status.rs --write
//!   rust-script scripts/render-status.rs --check
//!
//! ```cargo
//! [package]
//! edition = "2024"
//! ```

use std::env;
use std::process::exit;

/// The one generated file, relative to the repository root.
const STATUS_DOCUMENT: &str = "docs/status.md";

/// The two in-place regions retained, and the pin test that requires each.
const PINNED_REGIONS: &[(&str, &str)] = &[
    ("docs/benchmarks.md", "benchmarks"),
    ("README.md", "self-hosting"),
];

fn main() {
    let mode = env::args().nth(1).unwrap_or_default();
    let known = matches!(mode.as_str(), "" | "--write" | "--check");
    if !known {
        eprintln!("render-status: unknown mode {mode}; expected --write or --check");
        exit(2);
    }
    eprintln!(
        "render-status: not implemented. Plan 11 leaf L2 owns {STATUS_DOCUMENT} and the \
         {} pinned regions ({}); plan 11 leaf L1 owns \
         data/meta/requirement-status-ledger.lino, which this script reads. Wave T ships \
         the test, not the renderer.",
        PINNED_REGIONS.len(),
        PINNED_REGIONS
            .iter()
            .map(|(document, region)| format!("{document}:{region}"))
            .collect::<Vec<_>>()
            .join(", ")
    );
    exit(1);
}
