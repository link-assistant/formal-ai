//! Print the issue-#701 adoption ledger: one before/after capability pair per
//! recorded frontier item, plus the corpus-level unknown-rate ratchet.
//!
//! ```sh
//! cargo run --release --example issue_701_adoption_ledger -- FRONTIER.lino FEED.xml
//! ```

use formal_ai::google_trends_adoption_ledger;

fn main() {
    let mut paths = std::env::args_os().skip(1);
    let frontier =
        std::fs::read_to_string(paths.next().expect("provide the frontier document path"))
            .expect("read frontier");
    let rss = std::fs::read_to_string(paths.next().expect("provide the RSS capture path"))
        .expect("read feed");
    let feed = formal_ai::parse_google_trends_rss(&rss, "US", "en").expect("parse feed");
    let ledger = google_trends_adoption_ledger(&frontier, &feed);
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
