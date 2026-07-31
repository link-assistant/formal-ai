//! Record the issue-#706 language learning frontier: every prompt a registered
//! or candidate language supplied in `data/language-additions/` that the live
//! engine still cannot answer in that language.
//!
//! ```sh
//! cargo run --example issue_706_language_frontier > data/meta/learning-frontier-language-gap.lino
//! ```
//!
//! The committed record is deliberately **frozen at the pre-adoption state**:
//! it is the "before" half of `data/meta/language-adoption-ledger.lino`. Re-run
//! this example to record a *new* frontier (a language whose corpus the engine
//! still fails), not to refresh the committed one — after the issue-#706
//! adoption every Spanish prompt routes, so a re-run prints an empty frontier
//! with an explicit `frontier_gap`, which is exactly the proof the cycle worked.

use std::path::Path;

fn main() {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/language-additions");
    match formal_ai::language_frontier::record_language_gap_frontier(&directory) {
        Ok(document) => print!("{document}"),
        Err(error) => {
            eprintln!("cannot record the language frontier: {error}");
            std::process::exit(1);
        }
    }
}
