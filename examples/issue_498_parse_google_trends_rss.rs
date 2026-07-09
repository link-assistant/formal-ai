//! Convert a Google Trends RSS feed from stdin into the issue-#498 seed format.
//!
//! ```text
//! curl -sL 'https://trends.google.com/trending/rss?geo=US&hl=ru' \
//!   | cargo run --example issue_498_parse_google_trends_rss \
//!   > data/seed/google-trends-snapshot.lino
//! ```

use std::io::{self, Read};

fn main() {
    let mut rss = String::new();
    io::stdin()
        .read_to_string(&mut rss)
        .expect("Google Trends RSS should be readable from stdin");
    let snapshot = formal_ai::parse_google_trends_rss(&rss, "US", "ru")
        .expect("Google Trends RSS should contain item records");
    print!(
        "{}",
        formal_ai::render_google_trends_snapshot_lino(&snapshot)
    );
}
