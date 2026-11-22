fn main() -> Result<(), &'static str> {
    let document = formal_ai::agentic_coding::google_trends_catalog::render_document();
    if document.is_empty() {
        return Err("Google Trends source unavailable");
    }
    print!("{document}");
    Ok(())
}
