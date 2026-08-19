//! Issue #1021: Spanish paraphrases of the reported prompts, held out from every
//! wording the fix was written against.
//!
//! Spanish is registered in `data/seed/languages.lino`, but the listing
//! vocabulary in `data/seed/shell-intents.lino` carried no `language es` block
//! at all, so a Spanish listing request fell through to web search exactly the
//! way the reported English one did in issue #865. The detector combines parts
//! and is never told which language it is reading, so supplying the parts is the
//! whole fix -- no Rust changed for this.
//!
//! Run with `cargo run --example issue_1021_spanish_probe`.

use formal_ai::agentic_coding::{plan_chat_step, AgenticPlan};
use formal_ai::ChatMessage;

/// The shell command the agentic planner resolves for `prompt`, or `None` when
/// it routes somewhere other than a command execution tool.
fn shell_command(prompt: &str) -> Option<String> {
    let plan = plan_chat_step(&[ChatMessage::user(prompt)], &["exec_command"])?;
    let AgenticPlan::ToolCalls(calls) = plan else {
        return None;
    };
    let arguments: serde_json::Value = serde_json::from_str(&calls[0].arguments).ok()?;
    arguments["command"].as_str().map(str::to_owned)
}

fn main() {
    for prompt in [
        "lista los archivos aquí",
        "muéstrame los archivos de la carpeta actual",
        "aquí, enseña los ficheros",
        "¿cuáles archivos hay en este directorio?",
        "enumera el contenido del directorio actual",
        // The English original, for comparison: the same rule answers both.
        "List me files here",
        // Still not a listing request in Spanish either -- the parts have to
        // combine, so a verb without a local scope keeps its own route.
        "lista los procesos en ejecución",
    ] {
        println!("{prompt:50} -> {:?}", shell_command(prompt));
    }
}
