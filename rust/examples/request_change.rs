//! Prepare a proposed change from a task with a fenced JSON request and target_module.

use std::io::Read as _;

use formal_ai::agentic_coding::change_request::{render_document, request_from_task};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut task = String::new();
    std::io::stdin()
        .take(129 * 1024)
        .read_to_string(&mut task)?;
    let request = request_from_task(&task)
        .ok_or("an explicit request and owned target_module are required")?;
    print!("{}", render_document(&request));
    Ok(())
}
