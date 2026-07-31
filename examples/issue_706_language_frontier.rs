//! Record the issue-#706 language learning frontier: every prompt a registered
//! or candidate language supplied in `data/language-additions/` that the live
//! engine still cannot answer in that language.
//!
//! ```sh
//! cargo run --example issue_706_language_frontier > data/meta/learning-frontier-language-gap.lino
//! ```

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
