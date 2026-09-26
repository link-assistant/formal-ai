//! Print the canonical, human-approved learning ledger as Links Notation
//! (issue #558). Regenerates the committed `data/meta/learning-ledger.lino`
//! artifact: `cargo run --example dump_learning_ledger > data/meta/learning-ledger.lino`.

fn main() {
    println!(
        "{}",
        formal_ai::learning_ledger::canonical_ledger().links_notation()
    );
}
