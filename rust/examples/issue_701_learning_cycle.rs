//! Print the issue-#701 learning cycle over the recorded Google Trends
//! frontier: derived candidates, their held-out validation, the promotion
//! proposals in the issue-#656 shape, and every blocked class.
//!
//! ```sh
//! cargo run --release --example issue_701_learning_cycle -- FRONTIER.lino
//! ```

use formal_ai::google_trends_learning_cycle;
use formal_ai::promotion::render_promotion_proposals;

fn main() {
    let path = std::env::args_os()
        .nth(1)
        .expect("provide the frontier document path");
    let document = std::fs::read_to_string(path).expect("read frontier document");
    let run = google_trends_learning_cycle(&document);
    println!("{}", run.links_notation());
    println!();
    println!("{}", render_promotion_proposals(&run.proposals));
}
