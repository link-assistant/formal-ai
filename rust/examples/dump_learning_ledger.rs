//! Print the configured human-approved learning ledger as Links Notation.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ledger = formal_ai::learning_ledger::configured_ledger()?;
    println!("{}", ledger.links_notation());
    Ok(())
}
