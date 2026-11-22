//! Read a self-healing task with its trace and source_module from stdin.

use std::io::Read as _;

use formal_ai::agentic_coding::self_heal::{case_from_task, render_document};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut task = String::new();
    std::io::stdin()
        .take(129 * 1024)
        .read_to_string(&mut task)?;
    let case = case_from_task(&task)?;
    print!("{}", render_document(&case));
    Ok(())
}
