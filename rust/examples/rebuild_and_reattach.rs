// Render a plan for a change recorded in FORMAL_AI_ACCEPTED_CHANGES.

use std::io::Read as _;

use formal_ai::agentic_coding::rebuild_plan::{plan_from_task, render_document};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut task = String::new();
    std::io::stdin()
        .take(129 * 1024)
        .read_to_string(&mut task)?;
    let plan = plan_from_task(&task).ok_or("an operator-recorded accepted change is required")?;
    print!("{}", render_document(&plan));
    Ok(())
}
