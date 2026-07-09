//! Print the Google Trends learning-frontier report as Links Notation
//! (issues #498 + #558). Regenerates the committed
//! `data/meta/google-trends-learning.lino` artifact:
//! `cargo run --example dump_google_trends_learning > data/meta/google-trends-learning.lino`.

fn main() {
    println!(
        "{}",
        formal_ai::google_trends_learning::trending_learning_report().links_notation()
    );
}
