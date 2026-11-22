//! Print the issue-#706 language adoption ledger: what the engine did with
//! every recorded Spanish frontier prompt before the learning cycle's proposals
//! were adopted, and what it does now.
//!
//! ```sh
//! cargo run --manifest-path rust/Cargo.toml --example issue_706_language_adoption -- data/meta/learning-frontier-language-gap.lino > data/meta/language-adoption-ledger.lino
//! ```

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args_os()
        .nth(1)
        .ok_or("missing frontier report path")?;
    let report = std::fs::read_to_string(path)?;
    print!(
        "{}",
        formal_ai::language_adoption::language_adoption_ledger(&report).links_notation()
    );
    Ok(())
}
