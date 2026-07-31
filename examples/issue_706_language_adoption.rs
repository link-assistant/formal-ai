//! Print the issue-#706 language adoption ledger: what the engine did with
//! every recorded Spanish frontier prompt before the learning cycle's proposals
//! were adopted, and what it does now.
//!
//! ```sh
//! cargo run --example issue_706_language_adoption > data/meta/language-adoption-ledger.lino
//! ```

fn main() {
    print!(
        "{}",
        formal_ai::language_adoption::language_adoption_ledger().links_notation()
    );
}
