//! Print the Google Trends learning-frontier report as Links Notation
//! (issues #498 + #558). Capture a current report with
//! `cargo run --example dump_google_trends_learning > /tmp/google-trends-learning.lino`.

fn main() -> Result<(), &'static str> {
    let document = formal_ai::agentic_coding::google_trends_learning::render_document();
    if document.is_empty() {
        return Err("Google Trends source unavailable");
    }
    println!("{}", document.trim_end());
    Ok(())
}
