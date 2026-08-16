//! Export a verified program-plan rule from a natural-language follow-up and
//! execute the generated standalone Rust artifact.
//!
//! ```bash
//! cargo run --example issue_936_export_and_execute
//! ```

use std::error::Error;
use std::fs;
use std::io::Write as _;
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

use formal_ai::{ConversationTurn, UniversalSolver};

fn main() -> Result<(), Box<dyn Error>> {
    let solver = UniversalSolver::default();
    let initial_prompt = "Write me a Rust program that lists the files in the current directory";
    let initial = solver.solve(initial_prompt);
    let history = [
        ConversationTurn::user(initial_prompt),
        ConversationTurn::assistant(initial.answer),
    ];
    let exported = solver.solve_with_history(
        "Sort the results in reverse order and export the substitution rule to Rust",
        &history,
    );
    let recipe = exported
        .execution_recipe
        .ok_or("the solver did not return an executable export")?;

    let nonce = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let directory = std::env::temp_dir().join(format!("formal-ai-936-example-{nonce}"));
    fs::create_dir_all(&directory)?;
    let source = directory.join(&recipe.path);
    let binary = directory.join("substitution_program");
    fs::write(&source, &recipe.source)?;

    let compilation = Command::new("rustc")
        .arg("--edition=2021")
        .args(["-D", "warnings"])
        .arg(&source)
        .arg("-o")
        .arg(&binary)
        .output()?;
    if !compilation.status.success() {
        return Err(String::from_utf8_lossy(&compilation.stderr)
            .into_owned()
            .into());
    }

    let mut child = Command::new(&binary)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or("missing stdin")?
        .write_all(b"request:modifier\treverse_sort\nrequest:task\tlist_files\n")?;
    let output = child.wait_with_output()?;
    if !output.status.success() {
        return Err("generated substitution program failed".into());
    }
    let transformed = String::from_utf8(output.stdout)?;
    let expected = concat!(
        "request:modifier\treverse_sort\n",
        "request:task\tlist_files_reverse_sort\n",
    );
    if transformed != expected {
        return Err(format!("unexpected generated output: {transformed:?}").into());
    }

    println!("intent: {}", exported.intent);
    println!("artifact: {}", recipe.path);
    print!("{transformed}");
    fs::remove_dir_all(directory)?;
    Ok(())
}
