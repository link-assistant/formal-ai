//! Print the issue-#701 adoption ledger: one before/after capability pair per
//! recorded frontier item, plus the corpus-level unknown-rate ratchet.
//!
//! ```sh
//! cargo run --release --example issue_701_adoption_ledger
//! cargo run --release --example issue_701_adoption_ledger -- --write
//! ```

use formal_ai::google_trends_adoption_ledger;

fn main() {
    let ledger = google_trends_adoption_ledger();
    let rendered = ledger.links_notation();
    if std::env::args().any(|argument| argument == "--write") {
        let path = "data/meta/learning-adoption-ledger.lino";
        let committed = std::fs::read_to_string(path).expect("read committed adoption ledger");
        let suffix = committed
            .find("  behavior_delta_schema\n")
            .map(|start| &committed[start..])
            .expect("preserve the generalized behavior-delta suffix");
        std::fs::write(path, format!("{rendered}\n{suffix}"))
            .expect("write regenerated adoption ledger");
    } else {
        println!("{rendered}");
    }
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
