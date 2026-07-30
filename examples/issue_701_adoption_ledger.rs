//! Print the issue-#701 adoption ledger: one before/after capability pair per
//! recorded frontier item, plus the corpus-level unknown-rate ratchet.
//!
//! ```sh
//! cargo run --release --example issue_701_adoption_ledger
//! ```

use formal_ai::google_trends_adoption_ledger;

fn main() {
    let ledger = google_trends_adoption_ledger();
    println!("{}", ledger.links_notation());
    eprintln!(
        "pairs={} adopted={} topics={} languages={} unknown_before={} unknown_after={}",
        ledger.pairs.len(),
        ledger.adopted().len(),
        ledger.adopted_topics().len(),
        ledger.adopted_languages().len(),
        ledger.corpus_unknown_before,
        ledger.corpus_unknown_after,
    );
}
