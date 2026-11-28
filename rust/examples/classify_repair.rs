// Read a task with fenced JSON traces from stdin and print its repair strategies.

use std::io::Read as _;

use formal_ai::agentic_coding::repair_strategy::{render_document, strategies_from_task};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut task = String::new();
    std::io::stdin()
        .take(129 * 1024)
        .read_to_string(&mut task)?;
    let strategies =
        strategies_from_task(&task).ok_or("one or more failure traces are required")?;
    print!("{}", render_document(&strategies));
    Ok(())
}
